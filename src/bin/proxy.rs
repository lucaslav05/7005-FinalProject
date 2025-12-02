use clap::Parser;
use crossterm::{
    event::DisableMouseCapture,
    event::EnableMouseCapture,
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    style::{Color, Style},
    text::Line,
    widgets::{Bar, BarChart, BarGroup, Block},
};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{io::stdout, sync::Arc, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    net::{TcpListener, TcpStream, UdpSocket},
    sync::Mutex,
    time::sleep,
};

/*
 * Proxy Server:
 * listen for packets from the client on specified ip port
 * forward packets to actual server
 * listen for packets from the server and forward back to client
 * randomly drop packets based on configured drop probabilities
 * randomly delaying packets based on configured delay probabilities
 *
 * Independent Configs for both directions
 * Delay times as millisecond  range using min/max
 *
 * Args:
 * --listen-ip:     client ip
 * --listen-port:   client port
 * --target-ip:     server ip
 * --target-port:   server port
 *
 * --client-drop:   drop chance for packets from client
 * --server-drop:   drop chance for packets from server
 *
 * --client-delay:  delay chance for packets from client
 * --server delay:  delay chance for packets from server
 *
 * --client-delay-time-min: minimum delay time for client packets
 * --client-delay-time-max: maximum delay time for client packets
 * --server-delay-time-min: minimum delay time for server packets
 * --server-delay-time-max: maximum delay time for server packets
 */

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about=None)]
struct Args {
    #[arg(short, long)]
    listen_ip: String,

    #[arg(short, long)]
    listen_port: u16,

    #[arg(short, long)]
    target_ip: String,

    #[arg(short, long)]
    target_port: u16,

    #[arg(short, long)]
    client_drop: f64,

    #[arg(short, long)]
    server_drop: f64,

    #[arg(short, long)]
    client_delay: f64,

    #[arg(short, long)]
    server_delay: f64,

    #[arg(short, long)]
    client_delay_time_min: u64,

    #[arg(short, long)]
    client_delay_time_max: u64,

    #[arg(short, long)]
    server_delay_time_min: u64,

    #[arg(short, long)]
    server_delay_time_max: u64,

    #[arg(short, long)]
    log_port: u16,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    msg: String,
    seq: u64,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Ack {
    received: u64,
}

#[derive(Default)]
struct Metrics {
    packets_sent: u64,     // client send
    packets_received: u64, // server recv
    ack_sent: u64,         // server ack_send
    ack_received: u64,     // client ack_recv
}

#[derive(Deserialize, Serialize, Clone)]
struct LogEvent {
    ts: f64,
    component: String,
    event: String,
    seq: Option<u64>,
}

#[allow(dead_code)]
fn timestamp() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs_f64()
}

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let args = Args::parse();
    let metrics = Arc::new(Mutex::new(Metrics::default()));

    let log_listener = TcpListener::bind(("0.0.0.0", args.log_port)).await?;
    let metrics_clone = metrics.clone();
    tokio::spawn(async move {
        loop {
            if let Ok((stream, addr)) = log_listener.accept().await {
                println!("Log connection from {}", addr);
                let metrics_clone2 = metrics_clone.clone();
                tokio::spawn(handle_log(stream, metrics_clone2));
            }
        }
    });

    let client_sock =
        Arc::new(UdpSocket::bind(format!("{}:{}", args.listen_ip, args.listen_port)).await?);
    let server_sock = Arc::new(UdpSocket::bind("0.0.0.0:0").await?);
    let server_addr = format!("{}:{}", args.target_ip, args.target_port)
        .parse::<std::net::SocketAddr>()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    let last_client = Arc::new(Mutex::new(None::<std::net::SocketAddr>));

    // Client -> Server
    {
        let client_sock1 = client_sock.clone();
        let server_sock1 = server_sock.clone();
        let last_client1 = last_client.clone();
        let metrics1 = metrics.clone();
        let args1 = Arc::new(args.clone());

        tokio::spawn(async move {
            let mut buf = vec![0u8; 2048];
            loop {
                if let Ok((n, client_addr)) = client_sock1.recv_from(&mut buf).await {
                    // remember client
                    {
                        let mut lock = last_client1.lock().await;
                        *lock = Some(client_addr);
                    }

                    // Drop
                    let mut rng = StdRng::seed_from_u64(42);
                    if rng.random::<f64>() < (args1.client_drop) {
                        let _m = metrics1.lock().await;
                        // optional: count dropped packets in separate metric
                        continue;
                    }

                    // Delay
                    if rng.random::<f64>() < (args1.client_delay) {
                        let min = args1.client_delay_time_min;
                        let max = args1.client_delay_time_max.max(min);
                        let delay = if min == max {
                            min
                        } else {
                            rng.random_range(min..=max)
                        };
                        sleep(Duration::from_millis(delay)).await;
                    }

                    let _ = server_sock1.send_to(&buf[..n], server_addr).await;
                }
            }
        });
    }

    {
        // Server -> Client
        let client_sock2 = client_sock.clone();
        let server_sock2 = server_sock.clone();
        let last_client2 = last_client.clone();
        let args = Arc::new(args.clone());

        tokio::spawn(async move {
            let mut buf = vec![0u8; 2048];
            loop {
                if let Ok((n, _server_from)) = server_sock2.recv_from(&mut buf).await {
                    // Drop
                    let mut rng = StdRng::seed_from_u64(42);
                    if rng.random::<f64>() < (args.server_drop) {
                        continue;
                    }

                    // Delay
                    if rng.random::<f64>() < (args.server_delay) {
                        let min = args.server_delay_time_min;
                        let max = args.server_delay_time_max.max(min);
                        let delay = if min == max {
                            min
                        } else {
                            rng.random_range(min..=max)
                        };
                        sleep(Duration::from_millis(delay)).await;
                    }

                    // Forward to client
                    if let Some(client_addr) = *last_client2.lock().await {
                        let _ = client_sock2.send_to(&buf[..n], client_addr).await;
                    }
                }
            }
        });
    }

    // ratatui
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| draw_tui(f, &metrics))?;

        // Exit on q
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    Ok(())
}

async fn handle_log(stream: TcpStream, metrics: Arc<Mutex<Metrics>>) {
    let reader = BufReader::new(stream);
    let mut lines = reader.lines();

    while let Ok(Some(line)) = lines.next_line().await {
        if let Ok(log) = serde_json::from_str::<LogEvent>(&line) {
            let mut m = metrics.lock().await;
            match (log.component.as_str(), log.event.as_str()) {
                ("client", "send") => m.packets_sent += 1,
                ("client", "ack_recv") => m.ack_received += 1,
                ("server", "recv") => m.packets_received += 1,
                ("server", "ack_send") => m.ack_sent += 1,
                _ => {}
            }
        }
    }
}

fn draw_tui(f: &mut Frame, metrics: &Arc<Mutex<Metrics>>) {
    let m = metrics.blocking_lock();
    let values = [
        ("Sent", m.packets_sent.min(u64::from(u8::MAX)) as u8),
        ("Recv", m.packets_received.min(u64::from(u8::MAX)) as u8),
        ("ACK Sent", m.ack_sent.min(u64::from(u8::MAX)) as u8),
        ("ACK Recv", m.ack_received.min(u64::from(u8::MAX)) as u8),
    ];

    let bars: Vec<Bar> = values
        .iter()
        .map(|(label, val)| {
            Bar::default()
                .value(*val as u64)
                .label(Line::from(*label))
                .text_value(format!("{val}"))
                .style(Style::default().fg(Color::Cyan))
                .value_style(Style::default().fg(Color::Black).bg(Color::Cyan))
        })
        .collect();

    let barchart = BarChart::default()
        .data(BarGroup::default().bars(&bars))
        .block(Block::new().title("UDP Proxy Metrics"))
        .bar_width(10);

    f.render_widget(barchart, f.area());
}
