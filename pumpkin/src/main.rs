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
use tracing::{debug, info, warn};

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
    info!("Starting Pumpkin {CARGO_PKG_VERSION} Minecraft (Protocol {CURRENT_MC_PROTOCOL})",);

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

    warn!("Pumpkin is currently under heavy development!");
    info!("Report issues on https://github.com/Pumpkin-MC/Pumpkin/issues");
    info!("Join our Discord for community support: https://discord.com/invite/wT8XjrjKkf");

    tokio::spawn(async {
        setup_sighandler()
            .await
            .expect("Unable to setup signal handlers");
    });

    let pumpkin_server = PumpkinServer::new(basic_config, advanced_config, vanilla_data).await;
    pumpkin_server.init_plugins().await;

    info!("Started server; took {}ms", time.elapsed().as_millis());
    let basic_config = &pumpkin_server.server.basic_config;
    info!(
        "Server is now running. Connect using port: {}{}{}",
        if basic_config.java_edition {
            format!("Java Edition: {}", basic_config.java_edition_address)
        } else {
            String::new()
        },
        if basic_config.java_edition && basic_config.bedrock_edition {
            " | " // Separator if both are enabled
        } else {
            ""
        },
        if basic_config.bedrock_edition {
            format!("Bedrock Edition: {}", basic_config.bedrock_edition_address)
        } else {
            String::new()
        }
    );

    pumpkin_server.start().await;
    info!("The server has stopped.");
}

fn handle_interrupt() {
    warn!(
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
