//! Configuration management and serialization for the Pumpkin Minecraft server.
#![deny(missing_docs)]
#![deny(clippy::unwrap_used)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use fun::FunConfig;

use logging::LoggingConfig;
use pumpkin_util::world_seed::Seed;
use pumpkin_util::{Difficulty, GameMode, PermissionLvl, random};
use recipe::RecipeConfig;
use serde::{Deserialize, Serialize};
use toml_spanner::{Context, Failed, FromToml, Item};

use std::path::PathBuf;
use std::{fs, num::NonZero, path::Path};
use tracing::{debug, warn};

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
pub use networking::management::ManagementServerConfig;
pub use networking::packet_limiter::PacketLimiterConfig;
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
mod toml_adapters;
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

impl<'de> FromToml<'de> for PumpkinConfig {
    fn from_toml(ctx: &mut Context<'de>, item: &Item<'de>) -> Result<Self, Failed> {
        // The file has two flattened root sections; each reader ignores the other's keys.
        let basic = BasicConfiguration::from_toml(ctx, item)?;
        let advanced = AdvancedConfiguration::from_toml(ctx, item)?;
        let telemetry = item
            .table_helper(ctx)?
            .optional("telemetry")
            .unwrap_or_default();
        Ok(Self {
            basic,
            advanced,
            telemetry,
        })
    }
}

impl LoadConfiguration for PumpkinConfig {
    fn get_path() -> &'static Path {
        Path::new("pumpkin.toml")
    }

    fn validate(&self) {
        self.basic.validate();
        self.advanced.validate();
        self.telemetry.validate();

        let Some(min_vd) = NonZero::new(2) else {
            return;
        };
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
#[derive(Deserialize, Serialize, Default, toml_spanner::Toml)]
#[serde(default)]
#[toml(ignore_unknown_fields, recoverable)]
pub struct AdvancedConfiguration {
    /// Logging-related configuration such as log levels and output behaviour.
    #[toml(default = Self::default().logging)]
    pub logging: LoggingConfig,
    /// Resource pack configuration, including enforcement and pack metadata.
    #[toml(default = Self::default().resource_pack)]
    pub resource_pack: ResourcePackConfig,
    /// World and level-related settings beyond basic configuration.
    #[toml(default = Self::default().world)]
    pub world: LevelConfig,
    /// Networking-related features such as compression, authentication, and LAN broadcast.
    #[toml(default = Self::default().networking)]
    pub networking: NetworkingConfig,
    /// Command system configuration, including availability and permissions.
    #[toml(default = Self::default().commands)]
    pub commands: CommandsConfig,
    /// Chat-related features such as formatting, filtering, and message behaviour.
    #[toml(default = Self::default().chat)]
    pub chat: ChatConfig,
    /// Player-vs-player rules and mechanics.
    #[toml(default = Self::default().pvp)]
    pub pvp: PVPConfig,
    /// Server links configuration exposed to clients.
    #[toml(default = Self::default().server_links)]
    pub server_links: ServerLinksConfig,
    /// Persistent player data handling and storage behaviour.
    #[toml(default = Self::default().player_data)]
    pub player_data: PlayerDataConfig,
    /// Optional fun and experimental features.
    #[toml(default = Self::default().fun)]
    pub fun: FunConfig,
    /// Recipe-related configuration.
    #[toml(default = Self::default().recipe)]
    pub recipe: RecipeConfig,
    /// Plugin-related configuration.
    #[toml(default = Self::default().plugins)]
    pub plugins: PluginsConfig,
    /// Advancement configuration
    #[toml(default = Self::default().advancement)]
    pub advancement: AdvancementConfig,
}

/// Basic configuration for core server settings.
///
/// Covers edition support, world, networking, gameplay rules, and security options.
#[derive(Serialize, Deserialize, toml_spanner::Toml)]
#[serde(default)]
#[toml(ignore_unknown_fields, recoverable)]
pub struct BasicConfiguration {
    /// The seed for the world generation.
    #[toml(default = Self::default().seed, with = crate::toml_adapters::serde_value)]
    pub seed: Seed,
    /// The default game difficulty.
    #[toml(default = Self::default().default_difficulty, with = crate::toml_adapters::serde_value)]
    pub default_difficulty: Difficulty,
    /// The op level assigned by the /op command.
    #[toml(default = Self::default().op_permission_level, with = crate::toml_adapters::serde_value)]
    pub op_permission_level: PermissionLvl,
    /// Whether the Nether dimension is enabled.
    #[toml(default = Self::default().allow_nether)]
    pub allow_nether: bool,
    /// Whether the End dimension is enabled.
    #[toml(default = Self::default().allow_end)]
    pub allow_end: bool,
    /// Whether the server is in hardcore mode.
    #[toml(default = Self::default().hardcore)]
    pub hardcore: bool,
    /// The server's ticks per second.
    #[toml(default = Self::default().tps)]
    pub tps: f32,
    /// The default gamemode for players.
    #[toml(default = Self::default().default_gamemode, with = crate::toml_adapters::serde_value)]
    pub default_gamemode: GameMode,
    /// If the server forces the gamemode on-join.
    #[toml(default = Self::default().force_gamemode)]
    pub force_gamemode: bool,
    /// Whether to remove IPs from logs or not.
    #[toml(default = Self::default().scrub_ips)]
    pub scrub_ips: bool,
    /// Whether to use a server favicon.
    #[toml(default = Self::default().use_favicon)]
    pub use_favicon: bool,
    /// Path to optional server favicon.
    #[toml(default = Self::default().favicon_path)]
    pub favicon_path: Option<String>,
    /// The default level name
    #[toml(default = Self::default().default_level_name)]
    pub default_level_name: String,
    /// Whether chat messages should be signed or not.
    #[toml(default = Self::default().allow_chat_reports)]
    pub allow_chat_reports: bool,
    /// Whether to enable the whitelist.
    #[toml(default = Self::default().white_list)]
    pub white_list: bool,
    /// Whether to enforce the whitelist.
    #[toml(default = Self::default().enforce_whitelist)]
    pub enforce_whitelist: bool,
    /// Whether this server accepts incoming transfers from other servers.
    #[serde(alias = "accepts-transfers", alias = "accepts_transfers")]
    #[toml(default = Self::default().accepts_transfers, alias = "accepts-transfers")]
    pub accepts_transfers: bool,
    /// The radius of the spawn protection area around the world spawn point.
    /// Players without operator privileges cannot break or place blocks within this radius.
    /// Set to 0 to disable spawn protection.
    #[toml(default = Self::default().spawn_protection)]
    pub spawn_protection: u32,
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
            accepts_transfers: true,
            spawn_protection: 16,
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
    /// Reports errors to stderr and replaces invalid settings with defaults while retaining
    /// valid settings.
    #[must_use]
    // NOTE: Logger may not be ready.
    #[expect(clippy::print_stdout, clippy::print_stderr)]
    fn load(config_dir: &Path) -> Self
    where
        Self: Sized + Default + Serialize + for<'de> FromToml<'de>,
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
                    eprintln!(
                        "Couldn't read configuration file at {}: {err}\nFalling back to default configuration settings.",
                        path.display()
                    );
                    return Self::default();
                }
            };

            let (merged_config, changed, diagnostics) =
                parse_configuration::<Self>(&file_content, &path);
            for diagnostic in &diagnostics {
                eprintln!("{diagnostic}");
            }

            if changed && diagnostics.is_empty() {
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

    /// Returns the path to the configuration file relative to the config directory.
    fn get_path() -> &'static Path;

    /// Validates the configuration after loading or merging.
    fn validate(&self);
}

fn parse_configuration<T>(source: &str, path: &Path) -> (T, bool, Vec<String>)
where
    T: Default + Serialize + for<'de> FromToml<'de>,
{
    let arena = toml_spanner::Arena::new();
    let mut document = toml_spanner::parse_recoverable(source, &arena);
    let (config, errors) = match document.to_allowing_errors::<T>() {
        Ok(result) => result,
        Err(errors) => {
            let mut diagnostics: Vec<_> = errors
                .errors
                .iter()
                .map(|error| configuration_error(path, source, error))
                .collect();
            diagnostics.push("Falling back to default configuration settings.".to_string());
            return (T::default(), false, diagnostics);
        }
    };
    let changed = errors.errors.is_empty()
        && toml::Value::try_from(&config)
            .is_ok_and(|values| missing_values(&values, document.table().as_item()));
    let mut diagnostics: Vec<_> = errors
        .errors
        .iter()
        .map(|error| configuration_error(path, source, error))
        .collect();
    if !diagnostics.is_empty() {
        diagnostics.push(
            "Falling back to default configuration settings for invalid fields. Valid settings have been retained."
                .to_string(),
        );
    }
    (config, changed, diagnostics)
}

fn missing_values(values: &toml::Value, item: &Item<'_>) -> bool {
    let Some(fields) = values.as_table() else {
        return false;
    };
    fields.iter().any(|(key, value)| {
        item.as_table()
            .and_then(|table| table.get(key))
            .is_none_or(|item| missing_values(value, item))
    })
}

fn configuration_error(path: &Path, source: &str, error: &toml_spanner::Error) -> String {
    let setting = error.path().map_or_else(
        || "configuration".to_string(),
        |field| format!("setting `{field}`"),
    );
    let span = error.span().range();
    let offset = source.floor_char_boundary(span.start.min(source.len()));
    let line_start = source[..offset].rfind('\n').map_or(0, |index| index + 1);
    let line_end = source[offset..]
        .find('\n')
        .map_or(source.len(), |index| offset + index);
    let line = source[..line_start]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count()
        + 1;
    let column = source[line_start..offset].chars().count() + 1;
    let end = source
        .floor_char_boundary(span.end.min(line_end))
        .max(offset);
    let width = source[offset..end].chars().count().max(1);
    let gutter = line.to_string().len();
    format!(
        "Invalid {setting} in {}:\nTOML error at line {line}, column {column}\n{space:>gutter$} |\n{line} | {}\n{space:>gutter$} | {}{}\n{}\n",
        path.display(),
        &source[line_start..line_end],
        " ".repeat(column - 1),
        "^".repeat(width),
        error.message(source).trim_end(),
        space = ""
    )
}

#[cfg(test)]
mod tests {
    use toml::from_str;

    use super::{BasicConfiguration, PumpkinConfig, parse_configuration};
    use pumpkin_util::GameMode;

    #[test]
    fn configuration_load_diagnostics() {
        let path = std::path::Path::new("pumpkin.toml");
        for (source, diagnostic, expected_mode, field) in [
            (
                "default_gamemode = Creative\n",
                "default_gamemode = Creative",
                GameMode::Survival,
                "default_gamemode",
            ),
            (
                "default_gamemode =\n",
                "default_gamemode =",
                GameMode::Survival,
                "default_gamemode",
            ),
            (
                "default_gamemode = \"Invalid\"\n",
                "default_gamemode = \"Invalid\"",
                GameMode::Survival,
                "default_gamemode",
            ),
            (
                "default_gamemode = \"Creative\"\n[networking.java]\nmax_players = \"six\"\n",
                "max_players = \"six\"",
                GameMode::Creative,
                "networking.java.max_players",
            ),
            (
                "default_gamemode = \"Creative\"\n[telemetry]\nenabled = \"yes\"\n",
                "enabled = \"yes\"",
                GameMode::Creative,
                "telemetry.enabled",
            ),
            (
                "default_gamemode = \"Creative\"\n[logging]\nfile = latest.log\n",
                "file = latest.log",
                GameMode::Creative,
                "logging.file",
            ),
            (
                "default_gamemode = \"Creative\"\n[world.chunk]\ntype = \"anvil\"\n[world.chunk.compression]\nlevel = \"six\"\n",
                "level = \"six\"",
                GameMode::Creative,
                "world.chunk.compression.level",
            ),
        ] {
            let source = format!(
                "force_gamemode = true\ndefault_level_name = \"kids\"\ntps = 12.5\n{source}"
            );
            let (config, _, diagnostics) = parse_configuration::<PumpkinConfig>(&source, path);
            assert_eq!(config.basic.default_gamemode, expected_mode);
            assert!(config.basic.force_gamemode);
            assert_eq!(config.basic.default_level_name, "kids");
            assert_eq!(config.basic.tps, 12.5);
            assert_eq!(diagnostics.len(), 2, "{diagnostics:?}");
            let error = &diagnostics[0];
            assert!(error.contains("pumpkin.toml"), "{error}");
            assert!(error.contains(diagnostic), "{error}");
            assert!(
                error.contains(&format!("Invalid setting `{field}`")),
                "{error}"
            );
            assert!(error.contains("line "), "{error}");
            assert!(error.contains("column "), "{error}");
            assert_eq!(
                diagnostics[1],
                "Falling back to default configuration settings for invalid fields. Valid settings have been retained."
            );
        }
    }

    #[test]
    fn valid_configuration_has_no_diagnostics() {
        let source = "default_gamemode = \"Creative\"\nforce_gamemode = true\n";
        let (config, changed, diagnostics) =
            parse_configuration::<PumpkinConfig>(source, std::path::Path::new("pumpkin.toml"));
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        assert!(changed);
        assert_eq!(config.basic.default_gamemode, GameMode::Creative);
        assert!(config.basic.force_gamemode);
    }

    #[test]
    fn spanner_reads_the_existing_generated_configuration() {
        let expected = PumpkinConfig::default();
        let source = toml::to_string(&expected).unwrap();
        let (actual, changed, diagnostics) =
            parse_configuration::<PumpkinConfig>(&source, std::path::Path::new("pumpkin.toml"));
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        assert!(!changed);
        assert_eq!(
            toml::Value::try_from(actual).unwrap(),
            toml::Value::try_from(expected).unwrap()
        );
    }

    #[test]
    fn spanner_preserves_aliases_and_custom_field_formats() {
        let source = r#"
seed = "-42"
op_permission_level = 3
accepts-transfers = false
[networking.java]
keep-alive-time = 25
view_distance = 8
address = "127.0.0.1:25566"
[networking.java.authentication]
fallback_urls = ["https://example.com/auth"]
[networking.java.authentication.player_profile]
allowed_actions = ["USING_BANNED_SKIN"]
[networking.bedrock.nethernet]
external_ip = ""
[world.chunk]
type = "anvil"
[world.chunk.compression]
algorithm = "ZLib"
level = 3
[world.entity_islands]
unaudited_reach = -1.0
knockback_reach = nan
[resource_pack.bedrock]
packs = [{ uuid = "00000000-0000-0000-0000-000000000001", version = "1.0.0", size = 128, download_url = "https://example.com/pack", path = "packs/family.mcpack" }]
"#;
        let expected: PumpkinConfig = toml::from_str(source).unwrap();
        let (actual, _, diagnostics) =
            parse_configuration::<PumpkinConfig>(source, std::path::Path::new("pumpkin.toml"));
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        assert_eq!(
            toml::Value::try_from(actual).unwrap(),
            toml::Value::try_from(expected).unwrap()
        );
    }

    #[test]
    fn new_flattened_and_tagged_sections_need_no_loader_registration() {
        use serde::Serialize;
        use toml_spanner::Toml;

        #[derive(Default, Serialize, Toml)]
        struct FutureConfig {
            #[toml(default)]
            future: FutureSection,
        }
        #[derive(Default, Serialize, Toml)]
        struct FutureSection {
            #[toml(flatten, with = toml_spanner::helper::flatten_any)]
            options: FutureOptions,
        }
        #[derive(Default, Serialize, Toml)]
        struct FutureOptions {
            #[toml(default = 7)]
            limit: u32,
            #[toml(default)]
            strategy: FutureStrategy,
        }
        #[derive(Default, Serialize, Toml)]
        #[toml(tag = "kind")]
        enum FutureStrategy {
            #[default]
            Automatic,
            Custom {
                #[toml(default = 5)]
                value: u32,
                #[toml(default)]
                enabled: bool,
            },
        }
        let source = r#"
[future]
limit = "invalid"
[future.strategy]
kind = "Custom"
value = "invalid"
enabled = true
"#;
        let (config, _, diagnostics) =
            parse_configuration::<FutureConfig>(source, std::path::Path::new("pumpkin.toml"));
        assert_eq!(diagnostics.len(), 3, "{diagnostics:?}");
        assert!(diagnostics[0].contains("Invalid setting `future.limit`"));
        assert!(diagnostics[1].contains("Invalid setting `future.strategy.value`"));
        assert_eq!(config.future.options.limit, 7);
        assert!(matches!(
            config.future.options.strategy,
            FutureStrategy::Custom {
                value: 5,
                enabled: true
            }
        ));
    }

    #[test]
    fn accepts_transfers_defaults_to_true() {
        let config: BasicConfiguration = BasicConfiguration::default();
        assert!(config.accepts_transfers);
    }

    #[test]
    fn accepts_transfers_reads_vanilla_aliases() {
        let config: BasicConfiguration = from_str("accepts-transfers = false").unwrap();
        assert!(!config.accepts_transfers);

        let config: BasicConfiguration = from_str("accepts_transfers = false").unwrap();
        assert!(!config.accepts_transfers);
    }
}
