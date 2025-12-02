use clap::Parser;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpStream, UdpSocket},
    sync::mpsc,
    time::{Duration, timeout},
};

/*
 * Reliability Mechanism:
 * assign a seq number to each message
 * send the message to the server and wait for ack
 * retransmit if no ack within timeout period
 * after a maximum num of retries, give up on that message and error
 *
 * Args:
 * --target-ip
 * --target-port
 * --timeout
 * --max-retries
 *
 * One Server Max at a time
 * No connection/handshake logic
*/

#[derive(Serialize)]
struct Message {
    msg: String,
    seq: u64,
}

#[derive(Deserialize)]
struct Ack {
    seq: u64,
}

#[derive(Serialize)]
struct LogEvent {
    ts: f64,
    component: &'static str,
    event: &'static str,
    seq: u64,
}

fn timestamp() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs_f64()
}

async fn log_task(mut stream: TcpStream, mut rx: mpsc::Receiver<LogEvent>) {
    while let Some(ev) = rx.recv().await {
        let line = serde_json::to_string(&ev).unwrap() + "\n";
        let _ = stream.write_all(line.as_bytes()).await;
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Args {
    #[arg(long)]
    target_ip: String,

    #[arg(long)]
    target_port: u16,

    #[arg(long)]
    timeout: u64,

    #[arg(long)]
    max_retries: u32,
}

/**
Main function to act as the driver for the client
**/
#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let args = Args::parse();

    let server_addr = format!("{}:{}", args.target_ip, args.target_port); // <-- change
    let bind_addr = "0.0.0.0:3000"; // client UDP port

    let log_addr = "127.0.0.1:9100"; // UI log stream (client channel)

    let udp = UdpSocket::bind(bind_addr).await?;
    udp.connect(server_addr).await?;

    let (log_tx, log_rx) = mpsc::channel::<LogEvent>(1000);

    tokio::spawn(async move {
        let stream = TcpStream::connect(log_addr).await.unwrap();
        log_task(stream, log_rx).await;
    });

    let mut stdin = BufReader::new(tokio::io::stdin()).lines();
    let mut seq: u64 = 1;

    println!("Client ready");

    loop {
        print!("> ");
        use std::io::Write;
        std::io::stdout().flush()?;

        let line = match stdin.next_line().await? {
            Some(l) => l,
            None => break,
        };

        if line.trim().is_empty() {
            continue;
        }

        let msg = Message {
            seq,
            msg: line.clone(),
        };

        let encoded = serde_json::to_vec(&msg)?;

        // initial send
        udp.send(&encoded).await?;
        log_tx
            .send(LogEvent {
                ts: timestamp(),
                component: "client",
                event: "send",
                seq,
            })
            .await
            .ok();

        // ---- Wait for ACK ----
        let mut buf = [0u8; 256];
        let tries = 0;

        loop {
            if tries >= args.max_retries {
                eprintln!(
                    "ERROR: seq {} failed after {} retries",
                    seq, args.max_retries
                );
                break;
            }

            let recv_result = timeout(Duration::from_secs(args.timeout), udp.recv(&mut buf)).await;

            match recv_result {
                // ACK received
                Ok(Ok(n)) => {
                    let ack: Ack = serde_json::from_slice(&buf[..n])?;

                    if ack.seq == seq {
                        log_tx
                            .send(LogEvent {
                                ts: timestamp(),
                                component: "client",
                                event: "ack_recv",
                                seq,
                            })
                            .await
                            .ok();
                        println!("ACK for seq {}", seq);
                        break;
                    }
                }

                // Timeout → resend
                _ => {
                    udp.send(&encoded).await?;
                    log_tx
                        .send(LogEvent {
                            ts: timestamp(),
                            component: "client",
                            event: "resend",
                            seq,
                        })
                        .await
                        .ok();
                    println!("Timeout, resend seq {}", seq);
                }
            }
        }

        seq += 1;
    }

    Ok(())
}
