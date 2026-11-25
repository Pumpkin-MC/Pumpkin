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
#[cfg(not(any(unix, windows)))]
use tokio::signal::ctrl_c;
#[cfg(unix)]
use tokio::signal::unix::{Signal, SignalKind, signal};
#[cfg(windows)]
use tokio::signal::windows;

use pumpkin_core::{
    CRASH_REPORT, SERVER_IS_STOPPING, SHOULD_STOP,
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

#[expect(
    clippy::print_stderr,
    reason = "startup arguments are parsed before logging is initialized"
)]
fn warn_unknown_startup_arg(arg: &str) {
    eprintln!("Warning: ignoring unknown startup argument {arg:?}");
}

// WARNING: All rayon calls from the tokio runtime must be non-blocking! This includes things
// like `par_iter`. These should be spawned in the the rayon pool and then passed to the tokio
// runtime with a channel! See `Level::fetch_chunks` as an example!
#[tokio::main]
async fn main() {
    let command = match parse_startup_command(std::env::args().skip(1), warn_unknown_startup_arg) {
        Ok(command) => command,
        Err(error) => error.exit(),
    };

    initialize_runtime();
    if let Err(err) = setup_sighandler() {
        tracing::error!("Unable to setup signal handlers: {err}");
    }

    match command {
        StartupCommand::DedicatedServer(options) => dedicated_server::run(options).await,
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

#[derive(Clone, Copy)]
enum SignalAction {
    Interrupt,
    #[cfg(any(unix, windows))]
    Shutdown,
}

fn handle_signal(action: SignalAction) {
    let already_stopping =
        SHOULD_STOP.load(Ordering::Acquire) || SERVER_IS_STOPPING.load(Ordering::Acquire);

    match action {
        SignalAction::Interrupt => {
            let message = if already_stopping {
                "Received interrupt signal while already stopping; forcing exit..."
            } else {
                "Received interrupt signal; stopping server..."
            };
            warn!(
                "{}",
                TextComponent::text(message)
                    .color_named(NamedColor::Red)
                    .to_pretty_console()
            );
            stop_or_exit_server();
        }
        #[cfg(any(unix, windows))]
        SignalAction::Shutdown => {
            if already_stopping {
                return;
            }
            warn!(
                "{}",
                TextComponent::text("Received termination signal; stopping server...")
                    .color_named(NamedColor::Red)
                    .to_pretty_console()
            );
            stop_server();
        }
    }
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

// CTRL_LOGOFF_EVENT is intentionally omitted: Windows sends it to background console services
// whenever any interactive/RDP user logs off.
#[cfg(windows)]
fn setup_sighandler() -> io::Result<()> {
    let mut ctrl_c = windows::ctrl_c()?;
    let mut ctrl_break = windows::ctrl_break()?;
    let mut ctrl_close = windows::ctrl_close()?;
    let mut ctrl_shutdown = windows::ctrl_shutdown()?;

    tokio::spawn(async move {
        loop {
            let action = tokio::select! {
                Some(()) = ctrl_c.recv() => SignalAction::Interrupt,
                Some(()) = ctrl_break.recv() => SignalAction::Interrupt,
                Some(()) = ctrl_close.recv() => SignalAction::Shutdown,
                Some(()) = ctrl_shutdown.recv() => SignalAction::Shutdown,
                else => break,
            };
            handle_signal(action);
        }
    });

    Ok(())
}

#[cfg(not(any(unix, windows)))]
#[expect(
    clippy::unnecessary_wraps,
    reason = "matches unix and windows setup_sighandler signature"
)]
fn setup_sighandler() -> io::Result<()> {
    tokio::spawn(async {
        while ctrl_c().await.is_ok() {
            handle_signal(SignalAction::Interrupt);
        }
    });

    Ok(())
}

#[cfg(unix)]
async fn recv_optional_signal(signal: Option<&mut Signal>) -> Option<()> {
    if let Some(sig) = signal {
        sig.recv().await
    } else {
        std::future::pending().await
    }
}

#[cfg(unix)]
fn setup_sighandler() -> io::Result<()> {
    let mut interrupt = signal(SignalKind::interrupt())?;
    let mut hangup = signal(SignalKind::hangup())?;
    let mut terminate = signal(SignalKind::terminate())?;
    // LXC/Proxmox (`pct stop` / `lxc-stop`) and UPS daemons send SIGPWR for graceful shutdown.
    #[cfg(any(target_os = "linux", target_os = "android"))]
    let mut power = Some(signal(SignalKind::from_raw(libc::SIGPWR))?);
    #[cfg(not(any(target_os = "linux", target_os = "android")))]
    let mut power: Option<Signal> = None;

    tokio::spawn(async move {
        loop {
            let action = tokio::select! {
                Some(()) = interrupt.recv() => SignalAction::Interrupt,
                Some(()) = hangup.recv() => SignalAction::Shutdown,
                Some(()) = terminate.recv() => SignalAction::Shutdown,
                Some(()) = recv_optional_signal(power.as_mut()) => SignalAction::Shutdown,
                else => break,
            };
            handle_signal(action);
        }
    });

    Ok(())
}
