mod bot;
mod bot_tests;
mod harness;

use std::{process::ExitCode, time::Instant};

use tokio::net::TcpListener;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

#[expect(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "CLI test runner outputs results to stdout/stderr"
)]
#[tokio::main]
async fn main() -> ExitCode {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let _ = rayon::ThreadPoolBuilder::new()
        .thread_name(|i| format!("Rayon-Tester-{i}"))
        .build_global();

    let start = Instant::now();
    println!("Starting pumpkin-tester (single server + headless bots)...");

    let temp_dir = match tempfile::tempdir() {
        Ok(dir) => dir,
        Err(err) => {
            eprintln!("Failed to create temporary world directory: {err}");
            return ExitCode::FAILURE;
        }
    };

    let server = match harness::create_test_server(&temp_dir).await {
        Ok(s) => s,
        Err(err) => {
            eprintln!("{err}");
            return ExitCode::FAILURE;
        }
    };

    harness::prepare_spawn_chunk(&server).await;

    let listener = match TcpListener::bind("127.0.0.1:0").await {
        Ok(l) => l,
        Err(err) => {
            eprintln!("Failed to bind local TCP listener: {err}");
            return ExitCode::FAILURE;
        }
    };
    let local_addr = match listener.local_addr() {
        Ok(addr) => addr,
        Err(err) => {
            eprintln!("Failed to query local TCP address: {err}");
            return ExitCode::FAILURE;
        }
    };

    let cancel = CancellationToken::new();
    let tasks = TaskTracker::new();
    harness::spawn_java_listener(listener, server.clone(), cancel.clone(), &tasks);

    let mut summary_lines = Vec::new();
    let result = bot_tests::run_bot_tests(&server, local_addr, &mut summary_lines).await;

    cancel.cancel();
    for player in server.get_all_players() {
        player.remove().await;
        server.remove_player(&player);
    }
    tasks.close();
    tasks.wait().await;
    server.shutdown().await;

    match result {
        Ok(()) => {
            for line in summary_lines {
                println!("{line}");
            }
            println!(
                "All bot integration tests passed in {}ms!",
                start.elapsed().as_millis()
            );
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("pumpkin-tester FAILED:\n{err}");
            ExitCode::FAILURE
        }
    }
}
