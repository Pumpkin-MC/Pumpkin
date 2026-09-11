use clap::{Parser, ValueEnum};
use colored::Colorize;
use rand::RngExt;
use std::{
    net::SocketAddr,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    time::timeout,
};

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
enum FuzzMode {
    All,
    Raw,
    Framed,
    Stateful,
    CorruptVarint,
}

#[derive(Parser, Debug)]
#[command(author, version, about = "Pumpkin Server Network Packet Fuzzer & Stress Tool", long_about = None)]
struct Args {
    /// Target server host
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Target server port
    #[arg(short, long, default_value_t = 25565)]
    port: u16,

    /// Number of concurrent async fuzz worker tasks
    #[arg(short, long, default_value_t = 8)]
    concurrency: usize,

    /// Duration to fuzz in seconds
    #[arg(short, long, default_value_t = 30)]
    duration: u64,

    /// Fuzzing mode
    #[arg(short, long, value_enum, default_value_t = FuzzMode::All)]
    mode: FuzzMode,
}

#[derive(Default)]
struct Stats {
    connections: AtomicU64,
    packets_sent: AtomicU64,
    bytes_sent: AtomicU64,
    server_closes: AtomicU64,
    errors: AtomicU64,
}

fn encode_var_int(val: i32, buf: &mut Vec<u8>) {
    let mut uval = val as u32;
    loop {
        let mut byte = (uval & 0x7F) as u8;
        uval >>= 7;
        if uval != 0 {
            byte |= 0x80;
        }
        buf.push(byte);
        if uval == 0 {
            break;
        }
    }
}

fn encode_packet(packet_id: i32, payload: &[u8]) -> Vec<u8> {
    let mut id_buf = Vec::new();
    encode_var_int(packet_id, &mut id_buf);
    let total_len = id_buf.len() + payload.len();
    let mut out = Vec::new();
    encode_var_int(total_len as i32, &mut out);
    out.extend_from_slice(&id_buf);
    out.extend_from_slice(payload);
    out
}

fn make_handshake_packet(next_state: i32) -> Vec<u8> {
    let mut payload = Vec::new();
    encode_var_int(769, &mut payload); // Protocol version 769 (1.21.4)
    let addr = b"127.0.0.1";
    encode_var_int(addr.len() as i32, &mut payload);
    payload.extend_from_slice(addr);
    payload.extend_from_slice(&25565u16.to_be_bytes());
    encode_var_int(next_state, &mut payload);
    encode_packet(0x00, &payload)
}

async fn check_server_alive(addr: SocketAddr) -> bool {
    let Ok(Ok(mut stream)) = timeout(Duration::from_millis(1500), TcpStream::connect(addr)).await
    else {
        return false;
    };

    let handshake = make_handshake_packet(1); // Status state
    if stream.write_all(&handshake).await.is_err() {
        return false;
    }

    let status_req = encode_packet(0x00, &[]);
    if stream.write_all(&status_req).await.is_err() {
        return false;
    }

    let mut buf = [0u8; 1024];
    matches!(
        timeout(Duration::from_millis(1500), stream.read(&mut buf)).await,
        Ok(Ok(n)) if n > 0
    )
}

async fn fuzz_worker(
    target_addr: SocketAddr,
    mode: FuzzMode,
    stop: Arc<AtomicBool>,
    stats: Arc<Stats>,
) {
    while !stop.load(Ordering::Relaxed) {
        let stream_result =
            timeout(Duration::from_millis(1500), TcpStream::connect(target_addr)).await;
        let mut stream = match stream_result {
            Ok(Ok(s)) => {
                stats.connections.fetch_add(1, Ordering::Relaxed);
                s
            }
            _ => {
                stats.errors.fetch_add(1, Ordering::Relaxed);
                tokio::time::sleep(Duration::from_millis(10)).await;
                continue;
            }
        };

        let selected_mode = match mode {
            FuzzMode::All => match rand::rng().random_range(0..4) {
                0 => FuzzMode::Stateful,
                1 => FuzzMode::Framed,
                2 => FuzzMode::CorruptVarint,
                _ => FuzzMode::Raw,
            },
            other => other,
        };

        let res: Result<(), tokio::io::Error> = async {
            match selected_mode {
                FuzzMode::Stateful => {
                    let next_state = match rand::rng().random_range(0..3) {
                        0 => 1, // Status
                        1 => 2, // Login
                        _ => 7, // Config
                    };
                    let handshake = make_handshake_packet(next_state);
                    stream.write_all(&handshake).await?;
                    stats.packets_sent.fetch_add(1, Ordering::Relaxed);
                    stats
                        .bytes_sent
                        .fetch_add(handshake.len() as u64, Ordering::Relaxed);

                    let count = rand::rng().random_range(10..40);
                    for _ in 0..count {
                        if stop.load(Ordering::Relaxed) {
                            break;
                        }
                        let (frame, sleep_ms) = {
                            let mut rng = rand::rng();
                            let pkt_id = rng.random_range(0..0x60);
                            let pkt_len = rng.random_range(0..512);
                            let mut payload = vec![0u8; pkt_len];
                            rng.fill(&mut payload[..]);
                            (encode_packet(pkt_id, &payload), rng.random_range(1..5))
                        };
                        stream.write_all(&frame).await?;
                        stats.packets_sent.fetch_add(1, Ordering::Relaxed);
                        stats
                            .bytes_sent
                            .fetch_add(frame.len() as u64, Ordering::Relaxed);
                        tokio::time::sleep(Duration::from_millis(sleep_ms)).await;
                    }
                }
                FuzzMode::Framed => {
                    let count = rand::rng().random_range(5..30);
                    for _ in 0..count {
                        if stop.load(Ordering::Relaxed) {
                            break;
                        }
                        let (frame, sleep_ms) = {
                            let mut rng = rand::rng();
                            let pkt_id = rng.random_range(0..255);
                            let pkt_len = rng.random_range(0..2048);
                            let mut payload = vec![0u8; pkt_len];
                            rng.fill(&mut payload[..]);
                            (encode_packet(pkt_id, &payload), rng.random_range(1..4))
                        };
                        stream.write_all(&frame).await?;
                        stats.packets_sent.fetch_add(1, Ordering::Relaxed);
                        stats
                            .bytes_sent
                            .fetch_add(frame.len() as u64, Ordering::Relaxed);
                        tokio::time::sleep(Duration::from_millis(sleep_ms)).await;
                    }
                }
                FuzzMode::CorruptVarint => {
                    let count = rand::rng().random_range(5..20);
                    for _ in 0..count {
                        if stop.load(Ordering::Relaxed) {
                            break;
                        }
                        let bad_varint = {
                            let mut rng = rand::rng();
                            let byte_count = rng.random_range(5..10);
                            let mut v = Vec::with_capacity(byte_count + 64);
                            for _ in 0..byte_count {
                                v.push(0x80 | rng.random_range(0..128));
                            }
                            let extra_len = rng.random_range(10..100);
                            let mut extra = vec![0u8; extra_len];
                            rng.fill(&mut extra[..]);
                            v.extend_from_slice(&extra);
                            v
                        };

                        stream.write_all(&bad_varint).await?;
                        stats.packets_sent.fetch_add(1, Ordering::Relaxed);
                        stats
                            .bytes_sent
                            .fetch_add(bad_varint.len() as u64, Ordering::Relaxed);
                    }
                }
                FuzzMode::Raw | FuzzMode::All => {
                    let count = rand::rng().random_range(5..20);
                    for _ in 0..count {
                        if stop.load(Ordering::Relaxed) {
                            break;
                        }
                        let (chunk, sleep_ms) = {
                            let mut rng = rand::rng();
                            let chunk_len = rng.random_range(1..4096);
                            let mut c = vec![0u8; chunk_len];
                            rng.fill(&mut c[..]);
                            (c, rng.random_range(1..5))
                        };
                        stream.write_all(&chunk).await?;
                        stats.packets_sent.fetch_add(1, Ordering::Relaxed);
                        stats
                            .bytes_sent
                            .fetch_add(chunk.len() as u64, Ordering::Relaxed);
                        tokio::time::sleep(Duration::from_millis(sleep_ms)).await;
                    }
                }
            }
            Ok(())
        }
        .await;

        if res.is_ok() {
            stats.server_closes.fetch_add(1, Ordering::Relaxed);
        } else {
            stats.errors.fetch_add(1, Ordering::Relaxed);
        }

        let sleep_ms = rand::rng().random_range(5..30);
        tokio::time::sleep(Duration::from_millis(sleep_ms)).await;
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let addr_str = format!("{}:{}", args.host, args.port);
    let target_addr: SocketAddr = match addr_str.parse() {
        Ok(a) => a,
        Err(e) => {
            eprintln!(
                "{} Failed to parse address '{}': {}",
                "Error:".red().bold(),
                addr_str,
                e
            );
            std::process::exit(1);
        }
    };

    println!(
        "{}",
        "=== Pumpkin Server Network Packet Fuzzer ===".cyan().bold()
    );
    println!("{:<15} {}", "Target:".bold(), addr_str.yellow());
    println!("{:<15} {}", "Concurrency:".bold(), args.concurrency);
    println!("{:<15} {}s", "Duration:".bold(), args.duration);
    println!("{:<15} {:?}", "Mode:".bold(), args.mode);
    println!(
        "{}",
        "-------------------------------------------------".dimmed()
    );

    print!("Probing target server... ");
    if check_server_alive(target_addr).await {
        println!("{}", "ONLINE (Responding to status pings)".green().bold());
    } else {
        println!(
            "{}",
            "UNRESPONSIVE / OFFLINE (will proceed with test)"
                .yellow()
                .bold()
        );
    }

    let stats = Arc::new(Stats::default());
    let stop = Arc::new(AtomicBool::new(false));

    let mut handles = Vec::new();
    for _ in 0..args.concurrency {
        let handle = tokio::spawn(fuzz_worker(
            target_addr,
            args.mode,
            stop.clone(),
            stats.clone(),
        ));
        handles.push(handle);
    }

    let start = Instant::now();
    let duration = Duration::from_secs(args.duration);

    while start.elapsed() < duration {
        tokio::time::sleep(Duration::from_secs(1)).await;
        let elapsed = start.elapsed().as_secs();
        let conns = stats.connections.load(Ordering::Relaxed);
        let pkts = stats.packets_sent.load(Ordering::Relaxed);
        let bytes = stats.bytes_sent.load(Ordering::Relaxed);
        let closes = stats.server_closes.load(Ordering::Relaxed);
        let errs = stats.errors.load(Ordering::Relaxed);
        let rate_kb = (bytes as f64 / elapsed.max(1) as f64) / 1024.0;

        println!(
            "[{:3}s] Conns: {:5} | Packets: {:6} | Data: {:7.1} KB ({:6.1} KB/s) | Closes: {:4} | Resets/Timeouts: {:4}",
            elapsed,
            conns,
            pkts,
            bytes as f64 / 1024.0,
            rate_kb,
            closes,
            errs
        );
    }

    stop.store(true, Ordering::Relaxed);
    for h in handles {
        let _ = h.await;
    }

    println!(
        "{}",
        "-------------------------------------------------".dimmed()
    );
    let conns = stats.connections.load(Ordering::Relaxed);
    let pkts = stats.packets_sent.load(Ordering::Relaxed);
    let bytes = stats.bytes_sent.load(Ordering::Relaxed);
    let closes = stats.server_closes.load(Ordering::Relaxed);
    let errs = stats.errors.load(Ordering::Relaxed);

    println!("{}", "Fuzzing Session Complete:".green().bold());
    println!("  Total Connections:   {}", conns);
    println!("  Total Packets Sent:  {}", pkts);
    println!("  Total Data Sent:     {:.2} KB", bytes as f64 / 1024.0);
    println!("  Server Normal Drops: {}", closes);
    println!("  Connection Resets:   {}", errs);

    print!("\nVerifying server health post-fuzzing... ");
    if check_server_alive(target_addr).await {
        println!(
            "{}",
            "SUCCESS: Server is alive and responsive!".green().bold()
        );
    } else {
        println!("{}", "WARNING: Server did not respond to post-fuzz query. Check server logs for any crash or hang.".red().bold());
    }
}
