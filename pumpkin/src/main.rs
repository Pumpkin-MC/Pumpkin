// Don't warn on event sending macros
#![recursion_limit = "512"]
#![expect(unused_labels)]

#[cfg(target_os = "wasi")]
compile_error!("Compiling for WASI targets is not supported!");

use pumpkin_data::packet::CURRENT_MC_PROTOCOL;
use std::{
    io::{self},
    sync::{Arc, LazyLock, OnceLock},
};
#[cfg(not(unix))]
use tokio::signal::ctrl_c;
#[cfg(unix)]
use tokio::signal::unix::{SignalKind, signal};

use pumpkin::data::VanillaData;
use pumpkin::{LoggerOption, PumpkinServer, SHOULD_STOP, STOP_INTERRUPT, stop_server};

use pumpkin_config::{AdvancedConfiguration, BasicConfiguration, LoadConfiguration};
use pumpkin_util::text::{TextComponent, color::NamedColor};
use std::time::Instant;

// Setup some tokens to allow us to identify which event is for which socket.

pub mod block;
pub mod command;
pub mod data;
pub mod entity;
pub mod error;
pub mod item;
pub mod logging;
pub mod net;
pub mod plugin;
pub mod server;
pub mod world;

pub static LOGGER_IMPL: LazyLock<Arc<OnceLock<LoggerOption>>> =
    LazyLock::new(|| Arc::new(OnceLock::new()));

const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

// WARNING: All rayon calls from the tokio runtime must be non-blocking! This includes things
// like `par_iter`. These should be spawned in the the rayon pool and then passed to the tokio
// runtime with a channel! See `Level::fetch_chunks` as an example!
#[tokio::main]
async fn main() {
    #[cfg(feature = "console-subscriber")]
    console_subscriber::init();
    let time = Instant::now();

    let exec_dir = std::env::current_dir().unwrap();
    let config_dir = exec_dir.join("config");

    let basic_config = BasicConfiguration::load(&config_dir);
    let advanced_config = AdvancedConfiguration::load(&config_dir);

    let vanilla_data = VanillaData::load();

    pumpkin::init_logger(&advanced_config);

    let default_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        default_panic(info);
        // TODO: Gracefully exit?
        // We need to abide by the panic rules here.
        std::process::exit(1);
    }));
    log::info!(
        "{}",
        TextComponent::text(format!(
            "Starting {} {} Minecraft (Protocol {})",
            TextComponent::text("Pumpkin")
                .color_named(NamedColor::Gold)
                .to_pretty_console(),
            TextComponent::text(CARGO_PKG_VERSION.to_string())
                .color_named(NamedColor::Green)
                .to_pretty_console(),
            TextComponent::text(CURRENT_MC_PROTOCOL.to_string())
                .color_named(NamedColor::DarkBlue)
                .to_pretty_console()
        ))
        .to_pretty_console(),
    );

    log::debug!(
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
    print_support_links_and_warning();

    tokio::spawn(async {
        setup_sighandler()
            .await
            .expect("Unable to setup signal handlers");
    });

    let pumpkin_server = PumpkinServer::new(basic_config, advanced_config, vanilla_data).await;
    pumpkin_server.init_plugins().await;

    log::info!(
        "Started server; took {}",
        TextComponent::text(format!("{}ms", time.elapsed().as_millis()))
            .color_named(NamedColor::Gold)
            .to_pretty_console()
    );
    let basic_config = &pumpkin_server.server.basic_config;
    log::info!(
        "Server is now running. Connect using port: {}{}{}",
        if basic_config.java_edition {
            format!(
                "{} {}",
                TextComponent::text("Java Edition:")
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
            " | " // Separator if both are enabled
        } else {
            ""
        },
        if basic_config.bedrock_edition {
            format!(
                "{} {}",
                TextComponent::text("Bedrock Edition:")
                    .color_named(NamedColor::Gold)
                    .to_pretty_console(),
                TextComponent::text(format!("{}", basic_config.bedrock_edition_address))
                    .color_named(NamedColor::DarkBlue)
                    .to_pretty_console()
            )
        } else {
            TextComponent::text(String::new()).to_pretty_console()
        }
    );

    pumpkin_server.start().await;
    log::info!(
        "{}",
        TextComponent::text("The server has stopped.")
            .color_named(NamedColor::Red)
            .to_pretty_console()
    );
}
fn print_support_links_and_warning() {
    log::warn!(
        "{}",
        TextComponent::text("Pumpkin is currently under heavy development!")
            .color_named(NamedColor::DarkRed)
            .to_pretty_console(),
    );
    log::info!(
        "Report issues on {}",
        TextComponent::text("https://github.com/Pumpkin-MC/Pumpkin/issues")
            .color_named(NamedColor::DarkAqua)
            .to_pretty_console()
    );
    log::info!(
        "Join our {} for community support: {}",
        TextComponent::text("Discord")
            .color_named(NamedColor::DarkBlue)
            .to_pretty_console(),
        TextComponent::text("https://discord.gg/pumpkinmc")
            .color_named(NamedColor::Aqua)
            .to_pretty_console()
    );
}
fn handle_interrupt() {
    log::warn!(
        "{}",
        TextComponent::text("Received interrupt signal; stopping server...")
            .color_named(NamedColor::Red)
            .to_pretty_console()
    );
    stop_server();
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
