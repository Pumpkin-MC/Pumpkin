// Don't warn on event sending macros
#![recursion_limit = "512"]

#[cfg(target_os = "wasi")]
compile_error!("Compiling for WASI targets is not supported!");

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
    CRASH_REPORT, LoggerOption, PumpkinServer, SERVER_EXIT_CODE, SERVER_IS_STOPPING, SHOULD_STOP,
    STOP_INTERRUPT,
    crash::{CrashReport, FullBacktrace},
    data::VanillaData,
    localized_log, localized_log_format, localized_text, stop_or_exit_server, stop_server,
};
use pumpkin::{PumpkinServer, stop_server};

use pumpkin_config::{LoadConfiguration, PumpkinConfig};
use pumpkin_i18n::{
    self, locale_to_log_string, resolve_server_locale, set_server_command_locale,
    set_server_global_locale,
};
use pumpkin_util::text::{
    TextComponent,
    color::{Color, NamedColor},
};
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
        .expect(&localized_log("debug.expect.main_thread_id_failed"));

    // Set the panic handler.
    std::panic::set_hook(Box::new(handle_panic));

    #[cfg(feature = "console-subscriber")]
    console_subscriber::init();
    let time = Instant::now();

    let exec_dir = std::env::current_dir().unwrap();

    let config = PumpkinConfig::load(&exec_dir);

    let vanilla_data = VanillaData::load();

    pumpkin::init_logger(&config.advanced);

    // Initialize server locales from config.
    let server_global_locale = resolve_server_locale(&config.advanced.locale.server_logging);
    let server_command_locale = resolve_server_locale(&config.advanced.locale.server_command);
    set_server_global_locale(server_global_locale);
    set_server_command_locale(server_command_locale);
    info!(
        "{}",
        localized_log_format(
            "server.log.locale_info",
            &[
                format!(
                    "{} ({})",
                    locale_to_log_string(server_command_locale),
                    config.advanced.locale.server_command
                ),
                format!(
                    "{} ({})",
                    locale_to_log_string(server_global_locale),
                    config.advanced.locale.server_logging
                ),
            ],
        )
    );

    info!(
        "{}",
        TextComponent::text(localized_log_format(
            "server.log.starting_server",
            &["Starting {} {} Minecraft (Protocol {})"
                TextComponent::text("Pumpkin")
                    .color_named(NamedColor::Gold)
                    .to_pretty_console(),
                TextComponent::text(CARGO_PKG_VERSION.to_string())
                    .color_named(NamedColor::Green)
                    .to_pretty_console(),
                TextComponent::text(CURRENT_MC_VERSION.protocol_version().to_string())
                    .color_named(NamedColor::DarkBlue)
                    .to_pretty_console(),
            ],
        ))
        .to_pretty_console()
    );

    debug!(
        "{}",
        localized_log_format(
            "server.log.build_info",
            &[
                std::env::consts::FAMILY.to_owned(),
                std::env::consts::OS.to_owned(),
                std::env::consts::ARCH.to_owned(),
                localized_log(if cfg!(debug_assertions) {
                    "server.log.profile_debug"
                } else {
                    "server.log.profile_release"
                }),
            ],
        )
    );
    print_support_links_and_warning();

    tokio::spawn(async {
        setup_sighandler()
            .await
            .expect(&localized_log("debug.expect.signal_handlers_failed"));
    });

    let pumpkin_server = PumpkinServer::new(config.basic, config.advanced, vanilla_data).await;
    let plugin_wait_time = pumpkin_server.init_plugins().await;

    let time_elapsed = time.elapsed().saturating_sub(plugin_wait_time);

    info!(
        "{}",
        localized_log_format(
            "server.log.started_server",
            &[
                TextComponent::text(format!("{}ms", time_elapsed.as_millis()))
                    .color_named(NamedColor::Gold)
                    .to_pretty_console()
            ],
        )
    );
    let basic_config = &pumpkin_server.server.basic_config;
    info!(
        "{}",
        localized_log_format(
            "server.log.server_running",
            &[
                if basic_config.java_edition {
                    format!(
                        "{} {}",
                        TextComponent::text(localized_log("server.log.java_edition_label"))
                            .color_named(NamedColor::Yellow)
                            .to_pretty_console(),
                        TextComponent::text(format!("{}", basic_config.java_edition_address))
                            .color_named(NamedColor::DarkBlue)
                            .to_pretty_console()
                    )
                } else {
                    TextComponent::text(String::new()).to_pretty_console()
                },
                if basic_config.java_edition && basic_config.bedrock_edition {
                    " | ".to_owned()
                } else {
                    String::new()
                },
                if basic_config.bedrock_edition {
                    format!(
                        "{} {}",
                        TextComponent::text(localized_log("server.log.bedrock_edition_label"))
                            .color_named(NamedColor::Gold)
                            .to_pretty_console(),
                        TextComponent::text(format!("{}", basic_config.bedrock_edition_address))
                            .color_named(NamedColor::DarkBlue)
                            .to_pretty_console()
                    )
                } else {
                    TextComponent::text(String::new()).to_pretty_console()
                },
            ],
        )
    );

    pumpkin_server.start().await;

    info!(
        "{}",
        TextComponent::text(localized_log("server.log.server_stopped"))
            .color_named(NamedColor::Red)
            .to_pretty_console()
    );

    exit(SERVER_EXIT_CODE.load(Ordering::Acquire));
}

fn print_support_links_and_warning() {
    warn!(
        "{}",
        TextComponent::text(localized_log("server.log.under_development"))
            .color_named(NamedColor::DarkRed)
            .to_pretty_console(),
    );
    info!(
        "{}",
        localized_log_format(
            "server.log.report_issues",
            &[TextComponent::text(localized_log("server.issues_url"))
                .color_named(NamedColor::DarkAqua)
                .to_pretty_console()],
        )
    );
    info!(
        "{}",
        localized_log_format(
            "server.join_community_support",
            &[
                TextComponent::text(localized_log("server.discord_label"))
                    .color_named(NamedColor::DarkBlue)
                    .to_pretty_console(),
                TextComponent::text(localized_log("server.discord_url"))
                    .color_named(NamedColor::Aqua)
                    .to_pretty_console(),
            ],
        )
    );
}

fn handle_interrupt() {
    warn!(
        "{}",
        TextComponent::text(localized_log("server.log.received_interrupt"))
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
                localized_text("server.panic.main_thread_aborting", [])
                    .color(Color::Named(NamedColor::Red))
                    .to_pretty_console()
            );
        } else {
            // It's a subsequent panic.
            tracing::error!(
                "{}: {}",
                localized_text("server.panic.main_thread_panicked_while_stopping", [])
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
            localized_text("server.panic.shutdown_panic", [])
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
    if signal(SignalKind::interrupt())?.recv().await.is_some() {
        handle_interrupt();
    }

    if signal(SignalKind::hangup())?.recv().await.is_some() {
        handle_interrupt();
    }

    if signal(SignalKind::terminate())?.recv().await.is_some() {
        handle_interrupt();
    }

    Ok(())
}
