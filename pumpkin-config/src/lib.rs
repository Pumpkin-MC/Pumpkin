use fun::FunConfig;
use logging::LoggingConfig;
use pumpkin_util::world_seed::Seed;
use pumpkin_util::{Difficulty, GameMode, PermissionLvl, random};
use recipe::RecipeConfig;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use std::path::PathBuf;
use std::{fs, num::NonZeroU8, path::Path};
use tracing::{debug, warn};
pub mod fun;
pub mod logging;
pub use logging::{development_mode, set_development_mode};
pub mod networking;
pub mod performance;
pub mod plugins;
pub mod recipe;

pub mod resource_pack;

pub use chat::ChatConfig;
pub use commands::CommandsConfig;
pub use networking::auth::AuthenticationConfig;
pub use networking::bedrock::BedrockConfig;
pub use networking::compression::CompressionConfig;
pub use networking::java::JavaConfig;
pub use networking::lan_broadcast::LANBroadcastConfig;
pub use networking::rcon::RCONConfig;
pub use performance::{AllocatorBackend, CompressionBackend, PerformanceConfig};
pub use plugins::PluginsConfig;
pub use pvp::PVPConfig;
pub use server_links::ServerLinksConfig;

mod commands;

mod chat;
pub mod chunk;
pub mod lighting;
pub mod op;

mod advancement;
mod player_data;
mod pvp;
mod server_links;
pub mod whitelist;
pub mod world;

use advancement::AdvancementConfig;
use networking::NetworkingConfig;
use player_data::PlayerDataConfig;
use resource_pack::ResourcePackConfig;
use world::LevelConfig;

#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub struct PumpkinConfig {
    #[serde(flatten)]
    pub basic: BasicConfiguration,
    #[serde(flatten)]
    pub advanced: AdvancedConfiguration,
}

impl LoadConfiguration for PumpkinConfig {
    fn get_path() -> &'static Path {
        Path::new("pumpkin.toml")
    }

    fn first_run_template() -> Option<String> {
        // Pick the template whose comments match the operator's locale
        // (LC_ALL > LC_MESSAGES > LANG > LANGUAGE, first that is set).
        let locale = ["LC_ALL", "LC_MESSAGES", "LANG", "LANGUAGE"]
            .iter()
            .find_map(|name| std::env::var(name).ok().filter(|value| !value.is_empty()))
            .unwrap_or_default()
            .to_lowercase();
        let template = if locale.starts_with("zh") {
            include_str!("../../config/pumpkin.zh_cn.toml")
        } else {
            include_str!("../../config/pumpkin.en_us.toml")
        };
        // Bake a concrete random seed so the file records the world's seed
        // like vanilla, instead of re-rolling from an empty value every boot.
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |duration| duration.as_nanos() as u64);
        // splitmix64 scramble of the timestamp.
        let mut seed_value = nanos.wrapping_add(0x9E37_79B9_7F4A_7C15);
        seed_value = (seed_value ^ (seed_value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        seed_value = (seed_value ^ (seed_value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        seed_value ^= seed_value >> 31;
        let seed_line = format!("seed = \"{}\"", seed_value as i64);

        let mut in_top_level = true;
        let template = template
            .lines()
            .map(|line| {
                if in_top_level && line.starts_with('[') {
                    in_top_level = false;
                }
                if in_top_level && line.starts_with("seed = ") {
                    seed_line.clone()
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        Some(template + "\n")
    }

    // NOTE: Logger may not be ready during config validation.
    #[expect(clippy::print_stderr)]
    fn validate(&self) {
        self.basic.validate();
        self.advanced.validate();

        let min_vd = NonZeroU8::new(2).unwrap();
        let max_vd = NonZeroU8::new(64).unwrap();

        // Validate Java
        assert!(
            self.advanced.networking.java.view_distance >= min_vd,
            "Java View distance must be at least 2"
        );
        assert!(
            self.advanced.networking.java.view_distance <= max_vd,
            "Java View distance must be less than 64"
        );
        assert!(
            self.advanced.networking.java.simulation_distance >= min_vd,
            "Java simulation distance must be at least 2"
        );
        assert!(
            self.advanced.networking.java.simulation_distance <= max_vd,
            "Java simulation distance must be less than 64"
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
        assert!(
            self.advanced.networking.bedrock.simulation_distance >= min_vd,
            "Bedrock simulation distance must be at least 2"
        );
        assert!(
            self.advanced.networking.bedrock.simulation_distance <= max_vd,
            "Bedrock simulation distance must be less than 64"
        );
        if self.advanced.networking.bedrock.online_mode {
            assert!(
                self.advanced.networking.bedrock.encryption,
                "When online mode is enabled, bedrock_encryption must be enabled"
            );
        }

        if self.basic.allow_chat_reports {
            assert!(
                self.advanced.networking.java.online_mode,
                "When allow_chat_reports is enabled, java.online_mode must be enabled"
            );
        }

        // Common misconfiguration: [networking.java.authentication].enabled is
        // only the auth-service sub-config; the premium-verification switch is
        // [networking.java].online_mode. If the two disagree, clients still get
        // should_authenticate from online_mode and see "Invalid session".
        if self.advanced.networking.java.online_mode
            && !self.advanced.networking.java.authentication.enabled
        {
            eprintln!(
                "WARNING: networking.java.online_mode = true but \
                 networking.java.authentication.enabled = false. Premium \
                 verification is controlled by online_mode — set \
                 [networking.java] online_mode = false to disable it; \
                 authentication.enabled only configures the auth service."
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
    /// Optional performance backends (allocator / zlib). Defaults match the
    /// production build; Cargo features select the actual backend.
    pub performance: performance::PerformanceConfig,
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

    pub const fn validate(&self) {}
}

impl AdvancedConfiguration {
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
                    warn!(
                        "Couldn't write merged config to {}. Reason: {}",
                        path.display(),
                        err
                    );
                }
            }

            merged_config
        } else if let Some(template) = Self::first_run_template() {
            // PowerNukkitX-style first run: write the commented template that
            // matches the operator's system language, then load it through the
            // normal merge path so file and memory agree.
            if let Err(err) = fs::write(&path, &template) {
                warn!(
                    "Couldn't write default config to {:?}. Reason: {}",
                    path.display(),
                    err
                );
            }
            let parsed: toml::Value = toml::from_str(&template)
                .unwrap_or_else(|err| panic!("Built-in config template is invalid TOML: {err}"));
            let (merged_config, _) = Self::merge_with_default_toml(parsed);
            merged_config
        } else {
            let content = Self::default();
            if let Err(err) = fs::write(&path, toml::to_string(&content).unwrap()) {
                warn!(
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

    /// Commented template to write on first run instead of a bare default dump.
    /// `None` keeps the plain serialized defaults.
    #[must_use]
    fn first_run_template() -> Option<String> {
        None
    }

    /// Validates the configuration after loading or merging.
    fn validate(&self);
}

#[cfg(test)]
mod tests {
    use super::{LoadConfiguration, PumpkinConfig};
    use std::num::NonZeroU8;

    #[test]
    #[should_panic(expected = "Java simulation distance must be at least 2")]
    fn rejects_too_small_java_simulation_distance() {
        let mut config = PumpkinConfig::default();
        config.advanced.networking.java.simulation_distance = NonZeroU8::new(1).unwrap();
        config.validate();
    }

    #[test]
    #[should_panic(expected = "Bedrock simulation distance must be less than 64")]
    fn rejects_too_large_bedrock_simulation_distance() {
        let mut config = PumpkinConfig::default();
        config.advanced.networking.bedrock.simulation_distance = NonZeroU8::new(65).unwrap();
        config.validate();
    }
}
