//! Configuration management and serialization for the Pumpkin Minecraft server.
#![deny(missing_docs)]
#![deny(clippy::unwrap_used)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use fun::FunConfig;

use logging::LoggingConfig;
use pumpkin_util::world_seed::Seed;
use pumpkin_util::{Difficulty, GameMode, PermissionLvl, random};
use recipe::RecipeConfig;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use std::path::PathBuf;
use std::{fs, num::NonZero, path::Path};
use tracing::{debug, error, warn};

/// Fun and experimental configuration options.
pub mod fun;
/// Server logging configuration options.
pub mod logging;
/// Networking and protocol configuration options.
pub mod networking;
/// Plugin management configuration options.
pub mod plugins;
/// Recipe system configuration options.
pub mod recipe;

/// Resource pack configuration options.
pub mod resource_pack;

pub use chat::{AntiSpamConfig, ChatConfig};
pub use commands::{CommandOverride, CommandsConfig};
pub use networking::auth::AuthenticationConfig;
pub use networking::bedrock::BedrockConfig;
pub use networking::compression::CompressionConfig;
pub use networking::java::JavaConfig;
pub use networking::lan_broadcast::LANBroadcastConfig;
pub use networking::packet_limiter::PacketLimiterConfig;
pub use networking::rcon::RCONConfig;
pub use plugins::{PluginOverride, PluginsConfig};
pub use pvp::PVPConfig;
pub use server_links::ServerLinksConfig;
pub use telemetry::TelemetryConfig;

/// Telemetry configuration options.
pub mod telemetry;

mod commands;

mod chat;
/// Chunk loading and saving configuration options.
pub mod chunk;
/// Lighting engine configuration options.
pub mod lighting;
/// Operator permission level configuration options.
pub mod op;

mod advancement;
mod player_data;
mod pvp;
mod server_links;
/// Whitelist configuration options.
pub mod whitelist;
/// World generation and dimension configuration options.
pub mod world;

use advancement::AdvancementConfig;
use networking::NetworkingConfig;
use player_data::PlayerDataConfig;
use resource_pack::ResourcePackConfig;
use world::LevelConfig;

/// Root configuration container for Pumpkin server settings.
#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub struct PumpkinConfig {
    /// Core basic configuration settings.
    #[serde(flatten)]
    pub basic: BasicConfiguration,
    /// Advanced and feature-specific configuration settings.
    #[serde(flatten)]
    pub advanced: AdvancedConfiguration,
    /// Anonymous telemetry configuration settings.
    #[serde(default)]
    pub telemetry: TelemetryConfig,
}

impl LoadConfiguration for PumpkinConfig {
    fn get_path() -> &'static Path {
        Path::new("pumpkin.toml")
    }

    fn validate(&self) {
        self.basic.validate();
        self.advanced.validate();
        self.telemetry.validate();

        let min_vd = NonZero::<u8>::MIN;
        let Some(max_vd) = NonZero::new(64) else {
            return;
        };

        // Validate Java
        assert!(
            self.advanced.networking.java.keep_alive_time > 0,
            "Java Keep alive time must be greater than 0"
        );
        assert!(
            self.advanced.networking.java.view_distance >= min_vd,
            "Java View distance must be at least 2"
        );
        assert!(
            self.advanced.networking.java.view_distance <= max_vd,
            "Java View distance must be less than 64"
        );
        if self.advanced.networking.java.online_mode {
            assert!(
                self.advanced.networking.java.encryption,
                "When online mode is enabled, encryption must be enabled"
            );
        }

        // Validate Bedrock
        assert!(
            self.advanced.networking.bedrock.view_distance >= min_vd,
            "Bedrock View distance must be at least 2"
        );
        assert!(
            self.advanced.networking.bedrock.view_distance <= max_vd,
            "Bedrock View distance must be less than 64"
        );
        if self.basic.allow_chat_reports {
            assert!(
                self.advanced.networking.java.online_mode,
                "When allow_chat_reports is enabled, java.online_mode must be enabled"
            );
        }
    }
}

/// Advanced configuration for optional and feature-specific server settings.
///
/// Allows enabling/disabling features, customizing behaviour, and
/// tweaking performance or experimental options.
///
/// `Important`: The configuration should match vanilla by default.
#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub struct AdvancedConfiguration {
    /// Logging-related configuration such as log levels and output behaviour.
    pub logging: LoggingConfig,
    /// Resource pack configuration, including enforcement and pack metadata.
    pub resource_pack: ResourcePackConfig,
    /// World and level-related settings beyond basic configuration.
    pub world: LevelConfig,
    /// Networking-related features such as compression, authentication, and LAN broadcast.
    pub networking: NetworkingConfig,
    /// Command system configuration, including availability and permissions.
    pub commands: CommandsConfig,
    /// Chat-related features such as formatting, filtering, and message behaviour.
    pub chat: ChatConfig,
    /// Player-vs-player rules and mechanics.
    pub pvp: PVPConfig,
    /// Server links configuration exposed to clients.
    pub server_links: ServerLinksConfig,
    /// Persistent player data handling and storage behaviour.
    pub player_data: PlayerDataConfig,
    /// Optional fun and experimental features.
    pub fun: FunConfig,
    /// Recipe-related configuration.
    pub recipe: RecipeConfig,
    /// Plugin-related configuration.
    pub plugins: PluginsConfig,
    /// Advancement configuration
    pub advancement: AdvancementConfig,
}

/// Basic configuration for core server settings.
///
/// Covers edition support, world, networking, gameplay rules, and security options.
#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct BasicConfiguration {
    /// The seed for the world generation.
    pub seed: Seed,
    /// The default game difficulty.
    pub default_difficulty: Difficulty,
    /// The op level assigned by the /op command.
    pub op_permission_level: PermissionLvl,
    /// Whether the Nether dimension is enabled.
    pub allow_nether: bool,
    /// Whether the End dimension is enabled.
    pub allow_end: bool,
    /// Whether the server is in hardcore mode.
    pub hardcore: bool,
    /// The server's ticks per second.
    pub tps: f32,
    /// The default gamemode for players.
    pub default_gamemode: GameMode,
    /// If the server forces the gamemode on-join.
    pub force_gamemode: bool,
    /// Whether to remove IPs from logs or not.
    pub scrub_ips: bool,
    /// Whether to use a server favicon.
    pub use_favicon: bool,
    /// Path to optional server favicon.
    pub favicon_path: Option<String>,
    /// The default level name
    pub default_level_name: String,
    /// Whether chat messages should be signed or not.
    pub allow_chat_reports: bool,
    /// Whether to enable the whitelist.
    pub white_list: bool,
    /// Whether to enforce the whitelist.
    pub enforce_whitelist: bool,
}

impl Default for BasicConfiguration {
    fn default() -> Self {
        Self {
            seed: Seed(random::get_seed()),
            default_difficulty: Difficulty::Normal,
            op_permission_level: PermissionLvl::Four,
            allow_nether: true,
            allow_end: true,
            hardcore: false,
            tps: 20.0,
            default_gamemode: GameMode::Survival,
            force_gamemode: false,
            scrub_ips: true,
            use_favicon: true,
            favicon_path: None,
            default_level_name: "world".to_string(),
            allow_chat_reports: false,
            white_list: false,
            enforce_whitelist: false,
        }
    }
}

impl BasicConfiguration {
    /// Returns the path to the server's default world folder.
    #[must_use]
    pub fn get_world_path(&self) -> PathBuf {
        PathBuf::from(&self.default_level_name)
    }

    /// Validates basic configuration options.
    pub const fn validate(&self) {}
}

impl AdvancedConfiguration {
    /// Validates advanced configuration options.
    pub const fn validate(&self) {
        //self.resource_pack.validate();
    }
}

/// Trait for loading and validating configuration from a TOML file.
///
/// Provides default implementations for loading, merging with defaults,
/// and writing missing values back to disk. Also requires validation logic.
pub trait LoadConfiguration {
    /// Load configuration from the given directory.
    ///
    /// Creates the directory if it doesn't exist, reads the TOML file,
    /// merges it with defaults, writes missing fields, and validates the result.
    #[must_use]
    // NOTE: Logger may not be ready.
    #[expect(clippy::print_stdout)]
    fn load(config_dir: &Path) -> Self
    where
        Self: Sized + Default + Serialize + DeserializeOwned,
    {
        if !config_dir.exists() {
            debug!("creating new config root folder");
            let _ = fs::create_dir(config_dir);
        }
        let path = config_dir.join(Self::get_path());

        let config = if path.exists() {
            let file_content = match fs::read_to_string(&path) {
                Ok(content) => content,
                Err(err) => {
                    error!(
                        "Couldn't read configuration file at {}: {err}",
                        path.display()
                    );
                    return Self::default();
                }
            };

            let parsed_toml_value: toml::Value = match toml::from_str(&file_content) {
                Ok(val) => val,
                Err(err) => {
                    error!(
                        "Couldn't parse TOML at {}. Reason: {err}. Using default config.",
                        path.display()
                    );
                    return Self::default();
                }
            };

            let (merged_config, changed) = Self::merge_with_default_toml(parsed_toml_value);

            if changed {
                let file_name = path.file_name().map_or_else(
                    || path.display().to_string(),
                    |f| f.to_string_lossy().into_owned(),
                );
                println!(
                    "{file_name} changed because values were missing. The missing values were filled with default values."
                );
                match toml::to_string(&merged_config) {
                    Ok(toml_str) => {
                        if let Err(err) = fs::write(&path, toml_str) {
                            warn!(
                                "Couldn't write merged config to {}. Reason: {}",
                                path.display(),
                                err
                            );
                        }
                    }
                    Err(err) => {
                        warn!(
                            "Couldn't serialize merged config for {}. Reason: {}",
                            path.display(),
                            err
                        );
                    }
                }
            }

            merged_config
        } else {
            let content = Self::default();
            match toml::to_string(&content) {
                Ok(toml_str) => {
                    if let Err(err) = fs::write(&path, toml_str) {
                        warn!(
                            "Couldn't write default config to {:?}. Reason: {}",
                            path.display(),
                            err
                        );
                    }
                }
                Err(err) => {
                    warn!(
                        "Couldn't serialize default config for {:?}. Reason: {}",
                        path.display(),
                        err
                    );
                }
            }

            content
        };

        config.validate();
        config
    }

    /// Merge a parsed TOML value with the default configuration.
    ///
    /// Returns the merged configuration and a flag indicating if any values were filled.
    #[must_use]
    fn merge_with_default_toml(parsed_toml: toml::Value) -> (Self, bool)
    where
        Self: Sized + Default + Serialize + DeserializeOwned,
    {
        let default_config = Self::default();

        let Ok(default_toml_value) = toml::Value::try_from(&default_config) else {
            return (default_config, false);
        };

        let (merged_value, changed) = Self::merge_toml_values(default_toml_value, parsed_toml);

        let config = merged_value.try_into().unwrap_or_else(|_| Self::default());

        (config, changed)
    }

    /// Merge two TOML values recursively.
    ///
    /// Base is treated as default; overlay overwrites values.
    #[must_use]
    fn merge_toml_values(base: toml::Value, overlay: toml::Value) -> (toml::Value, bool) {
        match (base, overlay) {
            (toml::Value::Table(mut base_table), toml::Value::Table(overlay_table)) => {
                let mut changed = false;

                for key in base_table.keys() {
                    if !overlay_table.contains_key(key) {
                        changed = true;
                        break;
                    }
                }

                for (key, overlay_value) in overlay_table {
                    if let Some(base_value) = base_table.get(&key).cloned() {
                        let (merged_value, value_changed) =
                            Self::merge_toml_values(base_value, overlay_value);
                        base_table.insert(key, merged_value);
                        if value_changed {
                            changed = true;
                        }
                    } else {
                        base_table.insert(key, overlay_value);
                    }
                }
                (toml::Value::Table(base_table), changed)
            }
            (_, overlay) => (overlay, false),
        }
    }

    /// Returns the path to the configuration file relative to the config directory.
    fn get_path() -> &'static Path;

    /// Validates the configuration after loading or merging.
    fn validate(&self);
}
