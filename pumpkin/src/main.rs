#![deny(clippy::all)]
#![deny(clippy::pedantic)]
// #![warn(clippy::restriction)]
#![deny(clippy::cargo)]
// to keep consistency
#![deny(clippy::if_then_some_else_none)]
#![deny(clippy::empty_enum_variants_with_brackets)]
#![deny(clippy::empty_structs_with_brackets)]
#![deny(clippy::separated_literal_suffix)]
#![deny(clippy::semicolon_outside_block)]
#![deny(clippy::non_zero_suggestions)]
#![deny(clippy::string_lit_chars_any)]
#![deny(clippy::use_self)]
#![deny(clippy::useless_let_if_seq)]
#![deny(clippy::branches_sharing_code)]
#![deny(clippy::equatable_if_let)]
#![deny(clippy::option_if_let_else)]
// use log crate
#![deny(clippy::print_stdout)]
#![deny(clippy::print_stderr)]
// REMOVE SOME WHEN RELEASE
#![expect(clippy::cargo_common_metadata)]
#![expect(clippy::cast_precision_loss)]
#![expect(clippy::multiple_crate_versions)]
#![expect(clippy::single_call_fn)]
#![expect(clippy::cast_sign_loss)]
#![expect(clippy::cast_possible_truncation)]
#![expect(clippy::cast_possible_wrap)]
#![expect(clippy::missing_panics_doc)]
#![expect(clippy::missing_errors_doc)]
#![expect(clippy::module_name_repetitions)]
#![expect(clippy::struct_excessive_bools)]
// Not warn event sending macros
#![expect(unused_labels)]

#[cfg(target_os = "wasi")]
compile_error!("Compiling for WASI targets is not supported!");

use plugin::PluginManager;
use std::{
    io::{self},
    sync::LazyLock,
};
#[cfg(not(unix))]
use tokio::signal::ctrl_c;
#[cfg(unix)]
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::Mutex;

use crate::server::CURRENT_MC_VERSION;
use pumpkin::{init_log, stop_server, PumpkinServer, SHOULD_STOP};
use pumpkin_protocol::CURRENT_MC_PROTOCOL;
use pumpkin_util::text::{color::NamedColor, TextComponent};
use std::time::Instant;
// Setup some tokens to allow us to identify which event is for which socket.

pub mod block;
pub mod command;
pub mod data;
pub mod entity;
pub mod error;
pub mod item;
pub mod net;
pub mod plugin;
pub mod server;
pub mod world;

pub static PLUGIN_MANAGER: LazyLock<Mutex<PluginManager>> =
    LazyLock::new(|| Mutex::new(PluginManager::new()));

const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
const GIT_VERSION: &str = env!("GIT_VERSION");

// WARNING: All rayon calls from the tokio runtime must be non-blocking! This includes things
// like `par_iter`. These should be spawned in the the rayon pool and then passed to the tokio
// runtime with a channel! See `Level::fetch_chunks` as an example!
#[tokio::main]
async fn main() {
    let time = Instant::now();

    init_log!();

    let default_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        default_panic(info);
        // TODO: Gracefully exit?
        // we need to abide by the panic rules here
        std::process::exit(1);
    }));

    log::info!("Starting Pumpkin {CARGO_PKG_VERSION} ({GIT_VERSION}) for Minecraft {CURRENT_MC_VERSION} (Protocol {CURRENT_MC_PROTOCOL})",);

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

    log::warn!("Pumpkin is currently under heavy development!");
    log::info!("Report Issues on https://github.com/Pumpkin-MC/Pumpkin/issues");
    log::info!("Join our Discord for community support https://discord.com/invite/wT8XjrjKkf");

    tokio::spawn(async {
        setup_sighandler()
            .await
            .expect("Unable to setup signal handlers");
    });

    let pumpkin_server = PumpkinServer::new().await;
    pumpkin_server.init_plugins().await;

    log::info!("Started Server took {}ms", time.elapsed().as_millis());
    log::info!(
        "You now can connect to the server, Listening on {}",
        pumpkin_server.server_addr
    );

    pumpkin_server.start().await;
    log::info!("The server has stopped.");
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
