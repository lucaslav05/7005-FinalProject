use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpStream, UdpSocket};

//use serde::{Deserialize, Serialize};

/*
 * Listens on udp socket and receives messages from client
 *
 * Result:
 * prints message to standard output
 * returns ack, including seq num to client
 *
 * Args:
 * --listen-ip:     ip address to bind
 * --listen-port:   UDP port to listen on
 *
 * Handles one client at a time and is not required to support concurrent connections
*/

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Args {
    #[arg(long)]
    listen_ip: String,

    #[arg(long)]
    listen_port: String,

    #[arg(long)]
    log_host: String,

    #[arg(long)]
    log_port: u16,
}

#[derive(Deserialize)]
struct Message {
    msg: String,
    seq: u64,
}

#[derive(Serialize)]
struct Ack {
    seq: u64,
}

#[derive(Serialize)]
struct LogEvent {
    ts: f64,
    component: String,
    event: String,
    seq: Option<u64>,
}

fn timestamp() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs_f64()
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let server_addr = format!("{}:{}", args.listen_ip, args.listen_port);
    let udp = UdpSocket::bind(server_addr).await?;

    println!("Server listening on {}...", args.listen_port);

    let log_addr = format!("{}:{}", args.log_host, args.log_port);
    let log_stream = TcpStream::connect(log_addr).await?;
    let log_stream = tokio::sync::Mutex::new(log_stream);

    let mut received: HashSet<u64> = HashSet::new();
    let mut buf = [0u8; 2048];

    loop {
        let (n, addr) = udp.recv_from(&mut buf).await?;

        let msg: Message = match serde_json::from_slice(&buf[..n]) {
            Ok(m) => m,
            Err(_) => continue,
        };

        send_log(
            &log_stream,
            LogEvent {
                ts: timestamp(),
                component: "server".to_string(),
                event: "recv".to_string(),
                seq: Some(msg.seq),
            },
        )
        .await;

        if received.contains(&msg.seq) {
            println!("Duplicate seq {} ignored", msg.seq);
        } else {
            println!("Got msg='{}' seq={} from {}", msg.msg, msg.seq, addr);
            received.insert(msg.seq);
        }

        let ack = Ack { seq: msg.seq };
        let encoded = serde_json::to_vec(&ack).unwrap();
        udp.send_to(&encoded, addr).await?;

        send_log(
            &log_stream,
            LogEvent {
                ts: timestamp(),
                component: "server".to_string(),
                event: "ack_send".to_string(),
                seq: Some(msg.seq),
            },
        )
        .await;
    }
}

async fn send_log(stream: &tokio::sync::Mutex<TcpStream>, log: LogEvent) {
    let data = serde_json::to_string(&log).unwrap() + "\n";
    let mut s = stream.lock().await;
    let _ = s.write_all(data.as_bytes()).await;
}
