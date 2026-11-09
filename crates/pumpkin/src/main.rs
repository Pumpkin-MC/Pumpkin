#![deny(clippy::unwrap_used)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
// Don't warn on event sending macros
#![recursion_limit = "512"]

#[cfg(target_os = "wasi")]
compile_error!("Compiling for WASI targets is not supported!");

use std::{
    backtrace::{Backtrace, BacktraceStatus},
    io,
    panic::PanicHookInfo,
    process::exit,
    sync::{OnceLock, atomic::Ordering},
    thread::{self, ThreadId},
};
#[cfg(not(unix))]
use tokio::signal::ctrl_c;
#[cfg(unix)]
use tokio::signal::unix::{SignalKind, signal};

use pumpkin_core::{
    CRASH_REPORT, SERVER_IS_STOPPING,
    crash::{CrashReport, FullBacktrace},
    stop_or_exit_server, stop_server,
};

use pumpkin::{StartupCommand, parse_startup_command};
use pumpkin_util::text::{
    TextComponent,
    color::{Color, NamedColor},
};
use tracing::warn;

mod dedicated_server;
mod game_test_server;

static MAIN_THREAD: OnceLock<ThreadId> = OnceLock::new();

// WARNING: All rayon calls from the tokio runtime must be non-blocking! This includes things
// like `par_iter`. These should be spawned in the the rayon pool and then passed to the tokio
// runtime with a channel! See `Level::fetch_chunks` as an example!
#[tokio::main]
async fn main() {
    let command = match parse_startup_command(std::env::args().skip(1)) {
        Ok(command) => command,
        Err(error) => error.exit(),
    };

    initialize_runtime();
    tokio::spawn(async {
        if let Err(err) = setup_sighandler().await {
            tracing::error!("Unable to setup signal handlers: {err}");
        }
    });

    match command {
        StartupCommand::DedicatedServer => dedicated_server::run().await,
        StartupCommand::GameTestServer(options) => {
            if let Err(error) = game_test_server::run(options).await {
                tracing::error!("{error}");
                exit(1);
            }
        }
    }
}

fn initialize_runtime() {
    let _ = MAIN_THREAD.set(thread::current().id());

    // reqwest is built with `rustls-no-provider`, so pick the ring provider (the one
    // wasmtime-wasi-http/rtc already force) before any client can be constructed.
    let _ = rustls::crypto::ring::default_provider().install_default();

    // Initialize global Rayon thread pool with named worker threads
    let _ = rayon::ThreadPoolBuilder::new()
        .thread_name(|i| format!("Rayon-Worker-{i}"))
        .build_global();

    // Set the panic handler.
    std::panic::set_hook(Box::new(handle_panic));

    #[cfg(feature = "console-subscriber")]
    console_subscriber::init();
}

fn handle_interrupt() {
    warn!(
        "{}",
        TextComponent::text("Received interrupt signal; stopping server...")
            .color_named(NamedColor::Red)
            .to_pretty_console()
    );
    stop_or_exit_server();
}

fn handle_panic(panic_info: &PanicHookInfo<'_>) {
    // Generate a crash report.
    let crash_report = {
        // We capture the backtraces here, and not in the
        // crash report, so that the backtrace doesn't show
        // the CrashReport's `new` function.
        let captured_backtrace = Backtrace::capture();
        let full_backtrace = if captured_backtrace.status() == BacktraceStatus::Captured {
            FullBacktrace::Captured
        } else {
            FullBacktrace::ForceCaptured(Backtrace::force_capture())
        };

        CrashReport::new(panic_info, captured_backtrace, full_backtrace)
    };

    let payload = panic_info.payload();

    if is_main_thread() {
        // It's the first panic;
        // We cannot gracefully shut down as the main thread
        // has panicked. However, we can still generate the crash report.

        if let Some(crash_report) = try_set_crash_report(crash_report) {
            crash_report.print_to_console();
            crash_report.save_and_log();

            tracing::error!(
                "{}",
                TextComponent::text("Aborting due to the main thread panicking.")
                    .color(Color::Named(NamedColor::Red))
                    .to_pretty_console()
            );
        } else {
            // It's a subsequent panic.
            tracing::error!(
                "{}: {}",
                TextComponent::text(
                    "The main thread panicked while stopping the server; aborting."
                )
                .color(Color::Named(NamedColor::Red))
                .bold()
                .to_pretty_console(),
                payload
                    .downcast_ref::<&str>()
                    .copied()
                    .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
                    .unwrap_or("<unknown>")
            );
        }

        exit(1);
    }

    if try_set_crash_report(crash_report).is_some() {
        // It's the first panic; let's stop the server.
        stop_server();
    } else {
        // It's a subsequent panic; let's just alert about it.
        tracing::error!(
            "{}: {}",
            TextComponent::text("Encountered panic while shutting down")
                .color(Color::Named(NamedColor::Red))
                .bold()
                .to_pretty_console(),
            payload
                .downcast_ref::<&str>()
                .copied()
                .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
                .unwrap_or("<unknown>")
        );
    }
}

fn is_main_thread() -> bool {
    Some(&thread::current().id()) == MAIN_THREAD.get()
}

/// Returns `Some` if the crash report was successfully set. That
/// means it is the first panic, and it must be logged and saved later.
///
/// Returns `None` otherwise as the panic is subsequent.
fn try_set_crash_report(crash_report: CrashReport) -> Option<&'static CrashReport> {
    if !SERVER_IS_STOPPING.load(Ordering::Acquire) && CRASH_REPORT.set(crash_report).is_ok() {
        CRASH_REPORT.get()
    } else {
        None
    }
}

// Non-UNIX Ctrl-C handling
#[cfg(not(unix))]
async fn setup_sighandler() -> io::Result<()> {
    if ctrl_c().await.is_ok() {
        handle_interrupt();
    }

    Ok(())
}

// Unix signal handling
#[cfg(unix)]
async fn setup_sighandler() -> io::Result<()> {
    let mut interrupt = signal(SignalKind::interrupt())?;
    let mut hangup = signal(SignalKind::hangup())?;
    let mut terminate = signal(SignalKind::terminate())?;

    let received = tokio::select! {
        received = interrupt.recv() => received,
        received = hangup.recv() => received,
        received = terminate.recv() => received,
    };

    if received.is_some() {
        handle_interrupt();
    }

    Ok(())
}
