use fun::FunConfig;
use logging::LoggingConfig;
use pumpkin_util::world_seed::Seed;
use pumpkin_util::{Difficulty, GameMode, PermissionLvl, random};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use std::net::SocketAddr;
use std::path::PathBuf;
use std::{fs, num::NonZeroU8, path::Path};
pub mod fun;
pub mod logging;
pub mod networking;

pub mod resource_pack;

pub use chat::ChatConfig;
pub use commands::CommandsConfig;
pub use networking::auth::AuthenticationConfig;
pub use networking::compression::CompressionConfig;
pub use networking::lan_broadcast::LANBroadcastConfig;
pub use networking::rcon::RCONConfig;
pub use pvp::PVPConfig;
pub use server_links::ServerLinksConfig;

mod commands;

mod chat;
pub mod chunk;
pub mod op;
mod player_data;
mod pvp;
mod server_links;
pub mod whitelist;
pub mod world;

use networking::NetworkingConfig;
use player_data::PlayerDataConfig;
use resource_pack::ResourcePackConfig;
use world::LevelConfig;

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
}

/// Basic configuration for core server settings.
///
/// Covers edition support, world, networking, gameplay rules, and security options.
#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct BasicConfiguration {
    /// Whether Java Edition Clients are Accepted.
    pub java_edition: bool,

    /// The address and port to which the Java Edition server will bind.
    pub java_edition_address: SocketAddr,

    /// Whether Bedrock Edition Clients are Accepted.
    pub bedrock_edition: bool,

    /// Whether Bedrock Edition Clients are Accepted.
    pub bedrock_edition_address: SocketAddr,

    /// The seed for the world generation.
    pub seed: Seed,

    /// The maximum number of players allowed on the server. Specifying `0` disables the limit.
    pub max_players: u32,

    /// The maximum view distance for players.
    pub view_distance: NonZeroU8,

    /// The maximum simulated view distance.
    pub simulation_distance: NonZeroU8,

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

    /// Whether online mode is enabled. Requires valid Minecraft accounts.
    pub online_mode: bool,

    /// Whether packet encryption is enabled. Required when online mode is enabled.
    pub encryption: bool,

    /// Message of the Day; the server's description displayed on the status screen.
    pub motd: String,

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

    /// Path to server favicon.
    pub favicon_path: String,

    /// The default level name.
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
            java_edition: true,
            java_edition_address: "0.0.0.0:25565".parse().unwrap(),
            bedrock_edition: true,
            bedrock_edition_address: "0.0.0.0:19132".parse().unwrap(),
            seed: Seed(random::get_seed()),
            max_players: 1000,
            view_distance: NonZeroU8::new(16).unwrap(),
            simulation_distance: NonZeroU8::new(10).unwrap(),
            default_difficulty: Difficulty::Normal,
            op_permission_level: PermissionLvl::Four,
            allow_nether: true,
            allow_end: true,
            hardcore: false,
            online_mode: true,
            encryption: true,
            motd: "A blazingly fast Pumpkin server!".to_string(),
            tps: 20.0,
            default_gamemode: GameMode::Survival,
            force_gamemode: false,
            scrub_ips: true,
            use_favicon: true,
            favicon_path: "icon.png".to_string(),
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
            log::debug!("creating new config root folder");
            fs::create_dir(config_dir).expect("Failed to create config root folder");
        }
        let path = config_dir.join(Self::get_path());

        let config = if path.exists() {
            let file_content = fs::read_to_string(&path).unwrap_or_else(|_| {
                panic!("Couldn't read configuration file at {}", path.display())
            });

            let parsed_toml_value: toml::Value = toml::from_str(&file_content)
                .unwrap_or_else(|err| {
                    panic!(
                        "Couldn't parse TOML at {}. Reason: {}. This is probably caused by invalid TOML syntax",
                        path.display(), err
                    )
                });

            let (merged_config, changed) = Self::merge_with_default_toml(parsed_toml_value);

            if changed {
                println!(
                    "{} changed because values were missing. The missing values were filled with default values.",
                    path.file_name().unwrap().display()
                );
                if let Err(err) = fs::write(&path, toml::to_string(&merged_config).unwrap()) {
                    log::warn!(
                        "Couldn't write merged config to {}. Reason: {}",
                        path.display(),
                        err
                    );
                }
            }

            merged_config
        } else {
            let content = Self::default();

            if let Err(err) = fs::write(&path, toml::to_string(&content).unwrap()) {
                log::warn!(
                    "Couldn't write default config to {:?}. Reason: {}",
                    path.display(),
                    err
                );
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

        let default_toml_value =
            toml::Value::try_from(default_config).expect("Failed to parse default config");

        let (merged_value, changed) = Self::merge_toml_values(default_toml_value, parsed_toml);

        let config = merged_value
            .try_into()
            .expect("Failed to convert merged config");

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

impl LoadConfiguration for AdvancedConfiguration {
    fn get_path() -> &'static Path {
        Path::new("features.toml")
    }

    fn validate(&self) {
        self.resource_pack.validate();
    }
}

impl LoadConfiguration for BasicConfiguration {
    fn get_path() -> &'static Path {
        Path::new("configuration.toml")
    }

    fn validate(&self) {
        let min = NonZeroU8::new(2).unwrap();
        let max = NonZeroU8::new(64).unwrap();

        assert!(
            self.view_distance.ge(&min),
            "View distance must be at least 2"
        );
        assert!(
            self.view_distance.le(&max),
            "View distance must be less than 64"
        );
        if self.online_mode {
            assert!(
                self.encryption,
                "When online mode is enabled, encryption must be enabled"
            );
        }
        if self.allow_chat_reports {
            assert!(
                self.online_mode,
                "When allow_chat_reports is enabled, online_mode must be enabled"
            );
        }
    }
}
