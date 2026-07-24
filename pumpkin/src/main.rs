// Don't warn on event sending macros
#![recursion_limit = "512"]

#[cfg(target_os = "wasi")]
compile_error!("Compiling for WASI targets is not supported!");

/// Optional mimalloc global allocator (Cargo feature `mimalloc`).
/// Default builds keep the system allocator.
#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use pumpkin_data::packet::CURRENT_MC_VERSION;
use std::{
    backtrace::{Backtrace, BacktraceStatus},
    io::{self},
    panic::PanicHookInfo,
    process::exit,
    sync::{OnceLock, atomic::Ordering},
    thread::{self, ThreadId},
};
#[cfg(not(unix))]
use tokio::signal::ctrl_c;
#[cfg(unix)]
use tokio::signal::unix::{SignalKind, signal};

use pumpkin::{
    CRASH_REPORT, SERVER_EXIT_CODE, SERVER_IS_STOPPING,
    crash::{CrashReport, FullBacktrace},
    data::VanillaData,
    stop_or_exit_server,
};
use pumpkin::{PumpkinServer, stop_server};

use pumpkin_config::{
    AllocatorBackend, CompressionBackend, LoadConfiguration, PerformanceConfig, PumpkinConfig,
};
use pumpkin_util::text::{
    TextComponent,
    color::{Color, NamedColor},
};
use pumpkin_util::translation::{bilingual_console, configure_server_locale, server_locale};
use std::time::Instant;
use tracing::{debug, info, warn};

const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

static MAIN_THREAD: OnceLock<ThreadId> = OnceLock::new();

// WARNING: All rayon calls from the tokio runtime must be non-blocking! This includes things
// like `par_iter`. These should be spawned in the the rayon pool and then passed to the tokio
// runtime with a channel! See `Level::fetch_chunks` as an example!
#[allow(clippy::too_many_lines)]
#[tokio::main]
async fn main() {
    MAIN_THREAD
        .set(thread::current().id())
        .expect("Expected to successfully set the main thread ID");

    // Set the panic handler.
    std::panic::set_hook(Box::new(handle_panic));

    #[cfg(feature = "console-subscriber")]
    console_subscriber::init();
    let time = Instant::now();

    let exec_dir = std::env::current_dir().unwrap();

    let config = PumpkinConfig::load(&exec_dir);

    let vanilla_data = VanillaData::load();

    pumpkin::init_logger(&config.advanced);

    // Console language: en_us / zh_cn / zh_en (bilingual). Players keep client locale.
    let locale_cfg = config.advanced.logging.locale.as_str();
    if !configure_server_locale(locale_cfg) {
        warn!(
            "{}",
            TextComponent::custom(
                "pumpkin",
                "server.locale.invalid",
                server_locale(),
                vec![TextComponent::text(locale_cfg.to_string())],
            )
            .to_pretty_console()
        );
    }
    if bilingual_console() {
        info!(
            "{}",
            TextComponent::custom(
                "pumpkin",
                "server.locale.bilingual",
                server_locale(),
                vec![]
            )
            .to_pretty_console()
        );
    } else {
        info!(
            "{}",
            TextComponent::custom(
                "pumpkin",
                "server.locale.active",
                server_locale(),
                vec![TextComponent::text(server_locale().as_str())],
            )
            .to_pretty_console()
        );
    }

    info!(
        "{}",
        TextComponent::custom(
            "pumpkin",
            "server.starting",
            server_locale(),
            vec![
                TextComponent::text("Pumpkin").color_named(NamedColor::Gold),
                TextComponent::text(CARGO_PKG_VERSION.to_string()).color_named(NamedColor::Green),
                TextComponent::text(CURRENT_MC_VERSION.protocol_version().to_string())
                    .color_named(NamedColor::DarkBlue),
            ],
        )
        .to_pretty_console(),
    );

    debug!(
        "Build info: FAMILY: \"{}\", OS: \"{}\", ARCH: \"{}\", BUILD: \"{}\"",
        std::env::consts::FAMILY,
        std::env::consts::OS,
        std::env::consts::ARCH,
        if cfg!(debug_assertions) {
            "Debug"
        } else {
            "Release"
        }
    );
    log_performance_backends(&config.advanced.performance);
    if cfg!(debug_assertions) {
        warn!(
            "Pumpkin is running an unoptimized debug build. Do not use this build for performance testing; run `cargo run --release` or use a release binary."
        );
    }
    print_support_links_and_warning();

    tokio::spawn(async {
        setup_sighandler()
            .await
            .expect("Unable to setup signal handlers");
    });

    let pumpkin_server = PumpkinServer::new(config.basic, config.advanced, vanilla_data).await;
    let plugin_wait_time = pumpkin_server.init_plugins().await;

    let time_elapsed = time.elapsed().saturating_sub(plugin_wait_time);

    info!(
        "{}",
        TextComponent::custom(
            "pumpkin",
            "server.started",
            server_locale(),
            vec![
                TextComponent::text(format!("{}ms", time_elapsed.as_millis()))
                    .color_named(NamedColor::Gold),
            ],
        )
        .to_pretty_console()
    );
    let advanced_config = &pumpkin_server.server.advanced_config;
    let java_part = if advanced_config.networking.java.enabled {
        format!(
            "{} {}",
            TextComponent::custom("pumpkin", "server.java_edition", server_locale(), vec![],)
                .color_named(NamedColor::Yellow)
                .to_pretty_console(),
            TextComponent::text(format!("{}", advanced_config.networking.java.address))
                .color_named(NamedColor::DarkBlue)
                .to_pretty_console()
        )
    } else {
        String::new()
    };
    let bedrock_part = if advanced_config.networking.bedrock.enabled {
        format!(
            "{} {}",
            TextComponent::custom("pumpkin", "server.bedrock_edition", server_locale(), vec![],)
                .color_named(NamedColor::Gold)
                .to_pretty_console(),
            TextComponent::text(format!("{}", advanced_config.networking.bedrock.address))
                .color_named(NamedColor::DarkBlue)
                .to_pretty_console()
        )
    } else {
        String::new()
    };
    let ports = match (
        advanced_config.networking.java.enabled,
        advanced_config.networking.bedrock.enabled,
    ) {
        (true, true) => format!("{java_part} | {bedrock_part}"),
        (true, false) => java_part,
        (false, true) => bedrock_part,
        (false, false) => String::new(),
    };
    info!(
        "{}",
        TextComponent::custom(
            "pumpkin",
            "server.running",
            server_locale(),
            vec![TextComponent::text(ports)],
        )
        .to_pretty_console()
    );

    pumpkin_server.start().await;

    info!(
        "{}",
        TextComponent::custom("pumpkin", "server.stopped", server_locale(), vec![])
            .color_named(NamedColor::Red)
            .to_pretty_console()
    );

    exit(SERVER_EXIT_CODE.load(Ordering::Acquire));
}
fn print_support_links_and_warning() {
    warn!(
        "{}",
        TextComponent::custom(
            "pumpkin",
            "server.development.warning",
            server_locale(),
            vec![],
        )
        .color_named(NamedColor::DarkRed)
        .to_pretty_console(),
    );
    info!(
        "{}",
        TextComponent::custom(
            "pumpkin",
            "server.report_issues",
            server_locale(),
            vec![
                TextComponent::text("https://github.com/Pumpkin-MC/Pumpkin/issues")
                    .color_named(NamedColor::DarkAqua),
            ],
        )
        .to_pretty_console()
    );
    info!(
        "{}",
        TextComponent::custom(
            "pumpkin",
            "server.join_discord",
            server_locale(),
            vec![
                TextComponent::custom("pumpkin", "server.discord", server_locale(), vec![])
                    .color_named(NamedColor::DarkBlue),
                TextComponent::text("https://discord.gg/wT8XjrjKkf").color_named(NamedColor::Aqua),
            ],
        )
        .to_pretty_console()
    );
}

fn handle_interrupt() {
    warn!(
        "{}",
        TextComponent::custom(
            "pumpkin",
            "server.stopping.interrupt",
            server_locale(),
            vec![],
        )
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
                TextComponent::custom(
                    "pumpkin",
                    "server.panic.main_abort",
                    server_locale(),
                    vec![],
                )
                .color(Color::Named(NamedColor::Red))
                .to_pretty_console()
            );
        } else {
            // It's a subsequent panic.
            tracing::error!(
                "{}: {}",
                TextComponent::custom(
                    "pumpkin",
                    "server.panic.main_while_stop",
                    server_locale(),
                    vec![],
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
            TextComponent::custom(
                "pumpkin",
                "server.panic.while_shutdown",
                server_locale(),
                vec![],
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
}

fn is_main_thread() -> bool {
    Some(&thread::current().id()) == MAIN_THREAD.get()
}

/// Log which performance backends are active and warn on config/feature mismatch.
///
/// Allocator and `flate2` backend are compile-time; `pumpkin.toml` records the
/// intended choice so operators know what to rebuild with.
fn log_performance_backends(perf: &PerformanceConfig) {
    let built_mimalloc = cfg!(feature = "mimalloc");
    let built_zlib_rs = cfg!(feature = "zlib-rs");

    match perf.allocator {
        AllocatorBackend::Mimalloc if !built_mimalloc => {
            warn!(
                "pumpkin.toml advanced.performance.allocator = \"mimalloc\" but this binary was not built with --features mimalloc; using system allocator"
            );
        }
        AllocatorBackend::System if built_mimalloc => {
            warn!(
                "pumpkin.toml advanced.performance.allocator = \"system\" but binary was built with --features mimalloc; mimalloc is active (compile-time only)"
            );
        }
        _ => {}
    }

    match perf.compression_backend {
        CompressionBackend::ZlibRs if !built_zlib_rs => {
            warn!(
                "pumpkin.toml advanced.performance.compression_backend = \"zlib_rs\" but this binary was not built with --features zlib-rs; using pure-Rust miniz_oxide"
            );
        }
        CompressionBackend::Rust if built_zlib_rs => {
            warn!(
                "pumpkin.toml advanced.performance.compression_backend = \"rust\" but binary was built with --features zlib-rs; zlib-rs is active (compile-time only)"
            );
        }
        _ => {}
    }

    let active_allocator = if built_mimalloc { "mimalloc" } else { "system" };
    let active_compression = if built_zlib_rs {
        "zlib-rs"
    } else {
        "rust (miniz_oxide)"
    };

    info!(
        "{}",
        TextComponent::custom(
            "pumpkin",
            "server.performance.backends",
            server_locale(),
            vec![
                TextComponent::text(active_allocator),
                TextComponent::text(perf.allocator.as_str()),
                TextComponent::text(active_compression),
                TextComponent::text(perf.compression_backend.as_str()),
            ],
        )
        .to_pretty_console()
    );
    // Keep structured fields for log scrapers (English identifiers).
    debug!(
        "Performance backends: allocator={} (config={}), compression={} (config={})",
        active_allocator,
        perf.allocator.as_str(),
        active_compression,
        perf.compression_backend.as_str(),
    );
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

// Non-UNIX Ctrl-C handling — loop so a second Ctrl+C force-exits.
#[cfg(not(unix))]
async fn setup_sighandler() -> io::Result<()> {
    loop {
        if ctrl_c().await.is_ok() {
            handle_interrupt();
        }
    }
}

// Unix signal handling — keep listening so a second SIGINT force-exits stuck shutdowns.
#[cfg(unix)]
async fn setup_sighandler() -> io::Result<()> {
    let mut sigint = signal(SignalKind::interrupt())?;
    let mut sighup = signal(SignalKind::hangup())?;
    let mut sigterm = signal(SignalKind::terminate())?;

    loop {
        tokio::select! {
            Some(()) = sigint.recv() => handle_interrupt(),
            Some(()) = sighup.recv() => handle_interrupt(),
            Some(()) = sigterm.recv() => handle_interrupt(),
        }
    }
}
