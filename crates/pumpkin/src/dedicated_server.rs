use std::{
    process::exit,
    sync::{Arc, atomic::Ordering},
    time::Instant,
};

use pumpkin_config::{LoadConfiguration, PumpkinConfig};
use pumpkin_core::{
    PumpkinServer, SERVER_EXIT_CODE, data::VanillaData, plugin::loader::PluginLoader,
};
use pumpkin_data::packet::CURRENT_MC_VERSION;
use pumpkin_util::text::{TextComponent, color::NamedColor};
use pumpkin_wasm_host::WasmPluginLoader;
use pumpkin_world::{CURRENT_BEDROCK_MC_PROTOCOL, CURRENT_BEDROCK_MC_VERSION};
use tracing::{debug, info, warn};

const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

#[allow(clippy::too_many_lines)]
pub async fn run() {
    let time = Instant::now();

    let exec_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

    let config = PumpkinConfig::load(&exec_dir);

    if has_init_settings_arg(std::env::args().skip(1)) {
        exit(0);
    }

    let vanilla_data = VanillaData::load();

    pumpkin_core::init_logger(&config.advanced);

    if let Some(target_arg) = parse_convert_world_arg(std::env::args().skip(1)) {
        let anvil_default = match &config.advanced.world.chunk {
            pumpkin_config::chunk::ChunkConfig::Anvil(cfg) => cfg.clone(),
            _ => pumpkin_config::chunk::AnvilChunkConfig::default(),
        };
        let Some(target_config) =
            pumpkin_world::chunk::io::converter::parse_chunk_config(&target_arg, anvil_default)
        else {
            tracing::error!(
                "Unknown chunk format '{target_arg}'. Expected 'pump', 'anvil', or 'linear'."
            );
            exit(1);
        };
        let target_format =
            pumpkin_world::chunk::io::converter::RegionFormat::from_config(&target_config);
        let world_dir = exec_dir.join(config.basic.get_world_path());
        info!(
            "Converting world at {} to {} format...",
            world_dir.display(),
            target_format.name()
        );
        match pumpkin_world::chunk::io::converter::convert_world(&world_dir, &target_config, true) {
            Ok(stats) => {
                info!(
                    "Converted {} region(s) ({} chunk(s)) to {} format in {}ms",
                    stats.regions_converted,
                    stats.chunks_converted,
                    target_format.name(),
                    time.elapsed().as_millis()
                );
                exit(0);
            }
            Err(err) => {
                tracing::error!("World conversion failed: {err}");
                exit(1);
            }
        }
    }

    info!(
        "{}",
        TextComponent::text(format!(
            "Starting {} {} Java Minecraft (Protocol {}) | {} Bedrock (Protocol {})",
            TextComponent::text("Pumpkin")
                .color_named(NamedColor::Gold)
                .to_pretty_console(),
            TextComponent::text(CARGO_PKG_VERSION.to_string())
                .color_named(NamedColor::Green)
                .to_pretty_console(),
            TextComponent::text(CURRENT_MC_VERSION.protocol_version().to_string())
                .color_named(NamedColor::DarkBlue)
                .to_pretty_console(),
            TextComponent::text(CURRENT_BEDROCK_MC_VERSION)
                .color_named(NamedColor::Gold)
                .to_pretty_console(),
            TextComponent::text(CURRENT_BEDROCK_MC_PROTOCOL.to_string())
                .color_named(NamedColor::DarkBlue)
                .to_pretty_console()
        ))
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
    if cfg!(debug_assertions) {
        warn!(
            "Pumpkin is running an unoptimized debug build. Do not use this build for performance testing; run `cargo run --release` or use a release binary."
        );
    }
    print_support_links_and_warning();

    let plugin_loaders: Vec<Arc<dyn PluginLoader>> = vec![Arc::new(WasmPluginLoader::new(
        config.advanced.plugins.verify_signatures,
    ))];

    let pumpkin_server = PumpkinServer::new(
        config.basic,
        config.advanced,
        config.telemetry,
        vanilla_data,
        plugin_loaders,
        true,
    )
    .await
    .unwrap_or_else(|error| {
        tracing::error!("{error}");
        exit(1);
    });
    let plugin_wait_time = pumpkin_server.init_plugins().await;

    let time_elapsed = time.elapsed().saturating_sub(plugin_wait_time);

    info!(
        "Started server; took {}",
        TextComponent::text(format!("{}ms", time_elapsed.as_millis()))
            .color_named(NamedColor::Gold)
            .to_pretty_console()
    );
    let advanced_config = &pumpkin_server.server.advanced_config;
    info!(
        "Server is now running. Connect using port: {}{}{}",
        if advanced_config.networking.java.enabled {
            format!(
                "{} {}",
                TextComponent::text("Java Edition:")
                    .color_named(NamedColor::Yellow)
                    .to_pretty_console(),
                TextComponent::text(format!("{}", advanced_config.networking.java.address))
                    .color_named(NamedColor::DarkBlue)
                    .to_pretty_console()
            )
        } else {
            TextComponent::text(String::new()).to_pretty_console()
        },
        if advanced_config.networking.java.enabled && advanced_config.networking.bedrock.enabled {
            " | " // Separator if both are enabled
        } else {
            ""
        },
        if advanced_config.networking.bedrock.enabled {
            format!(
                "{} {}",
                TextComponent::text("Bedrock Edition:")
                    .color_named(NamedColor::Gold)
                    .to_pretty_console(),
                TextComponent::text(format!(
                    "{}",
                    advanced_config.networking.bedrock.nethernet.address
                ))
                .color_named(NamedColor::DarkBlue)
                .to_pretty_console()
            )
        } else {
            TextComponent::text(String::new()).to_pretty_console()
        }
    );

    pumpkin_server.start().await;

    info!(
        "{}",
        TextComponent::text("The server has stopped.")
            .color_named(NamedColor::Red)
            .to_pretty_console()
    );

    exit(SERVER_EXIT_CODE.load(Ordering::Acquire));
}

fn print_support_links_and_warning() {
    warn!(
        "{}",
        TextComponent::text("Pumpkin is currently under heavy development!")
            .color_named(NamedColor::DarkRed)
            .to_pretty_console(),
    );
    info!(
        "Report issues on {}",
        TextComponent::text("https://github.com/Pumpkin-MC/Pumpkin/issues")
            .color_named(NamedColor::DarkAqua)
            .to_pretty_console()
    );
    info!(
        "Join our {} for community support: {}",
        TextComponent::text("Discord")
            .color_named(NamedColor::DarkBlue)
            .to_pretty_console(),
        TextComponent::text("https://discord.gg/wT8XjrjKkf")
            .color_named(NamedColor::Aqua)
            .to_pretty_console()
    );
    info!(
        "Consider {} to {}",
        TextComponent::text("Donating")
            .color_named(NamedColor::DarkPurple)
            .to_pretty_console(),
        TextComponent::text("https://pumpkinmc.org/donate/")
            .color_named(NamedColor::Gold)
            .to_pretty_console()
    );
}

fn parse_convert_world_arg(mut args: impl Iterator<Item = String>) -> Option<String> {
    while let Some(arg) = args.next() {
        if arg == "--convert-world" {
            return Some(args.next().unwrap_or_default());
        }
        if let Some(value) = arg.strip_prefix("--convert-world=") {
            return Some(value.to_string());
        }
    }
    None
}

fn has_init_settings_arg(mut args: impl Iterator<Item = String>) -> bool {
    args.any(|arg| arg == "--init-settings" || arg == "--initSettings")
}
