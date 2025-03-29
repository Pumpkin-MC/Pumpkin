use chunk::ChunkConfig;
use log::warn;
use logging::LoggingConfig;
use pumpkin_util::{Difficulty, GameMode, PermissionLvl};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use std::path::PathBuf;
use std::{
    env, fs,
    net::{Ipv4Addr, SocketAddr},
    num::NonZeroU8,
    path::Path,
    sync::LazyLock,
};
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

use networking::NetworkingConfig;
use player_data::PlayerDataConfig;
use resource_pack::ResourcePackConfig;

const CONFIG_ROOT_FOLDER: &str = "config/";

pub static BASIC_CONFIG: LazyLock<BasicConfiguration> = LazyLock::new(|| {
    let exec_dir = env::current_dir().unwrap();
    BasicConfiguration::load(&exec_dir)
});

#[cfg(not(feature = "test_helper"))]
static ADVANCED_CONFIG: LazyLock<AdvancedConfiguration> = LazyLock::new(|| {
    let exec_dir = env::current_dir().unwrap();
    AdvancedConfiguration::load(&exec_dir)
});

#[cfg(not(feature = "test_helper"))]
pub fn advanced_config() -> &'static AdvancedConfiguration {
    &ADVANCED_CONFIG
}

// This is pretty jank but it works :(
// TODO: Can we refactor this better?
#[cfg(feature = "test_helper")]
use std::cell::RefCell;

// Yes, we are leaking memory here, but it is only for tests. Need to maintain pairity with the
// non-test code
#[cfg(feature = "test_helper")]
thread_local! {
    // Needs to be thread local so we don't override the config while another test is running
    static ADVANCED_CONFIG: RefCell<&'static AdvancedConfiguration> = RefCell::new(Box::leak(Box::new(AdvancedConfiguration::default())));
}

#[cfg(feature = "test_helper")]
pub fn override_config_for_testing(config: AdvancedConfiguration) {
    ADVANCED_CONFIG.with_borrow_mut(|ref_config| {
        *ref_config = Box::leak(Box::new(config));
    });
}

#[cfg(feature = "test_helper")]
pub fn advanced_config() -> &'static AdvancedConfiguration {
    ADVANCED_CONFIG.with_borrow(|config| *config)
}

/// The idea is that Pumpkin should very customizable.
/// You can enable or disable features depending on your needs.
///
/// This also allows you get some performance or resource boosts.
/// Important: The configuration should match vanilla by default.
#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub struct AdvancedConfiguration {
    pub logging: LoggingConfig,
    pub resource_pack: ResourcePackConfig,
    pub chunk: ChunkConfig,
    pub networking: NetworkingConfig,
    pub commands: CommandsConfig,
    pub chat: ChatConfig,
    pub pvp: PVPConfig,
    pub server_links: ServerLinksConfig,
    pub player_data: PlayerDataConfig,
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct BasicConfiguration {
    /// The address to bind the server to.
    pub server_address: SocketAddr,
    /// The seed for world generation.
    pub seed: String,
    /// The maximum number of players allowed on the server. Specifying `0` disables the limit.
    pub max_players: u32,
    /// The maximum view distance for players.
    pub view_distance: NonZeroU8,
    /// The maximum simulated view distance.
    pub simulation_distance: NonZeroU8,
    /// The default game difficulty.
    pub default_difficulty: Difficulty,
    /// The op level assigned by the /op command
    pub op_permission_level: PermissionLvl,
    /// Whether the Nether dimension is enabled.
    pub allow_nether: bool,
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
    /// If the server force the gamemode on join
    pub force_gamemode: bool,
    /// Whether to remove IPs from logs or not
    pub scrub_ips: bool,
    /// Whether to use a server favicon
    pub use_favicon: bool,
    /// Path to server favicon
    pub favicon_path: String,
    /// The default level name
    pub default_level_name: String,
    /// Whether chat messages should be signed or not
    pub allow_chat_reports: bool,
}

impl Default for BasicConfiguration {
    fn default() -> Self {
        Self {
            server_address: SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), 25565),
            seed: "".to_string(),
            max_players: 100000,
            view_distance: NonZeroU8::new(10).unwrap(),
            simulation_distance: NonZeroU8::new(10).unwrap(),
            default_difficulty: Difficulty::Normal,
            op_permission_level: PermissionLvl::Four,
            allow_nether: true,
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
        }
    }
}

impl BasicConfiguration {
    pub fn get_world_path(&self) -> PathBuf {
        format!("./{}", self.default_level_name).parse().unwrap()
    }
}

trait LoadConfiguration {
    fn load(exec_dir: &Path) -> Self
    where
        Self: Sized + Default + Serialize + DeserializeOwned,
    {
        let config_dir = exec_dir.join(CONFIG_ROOT_FOLDER);
        if !config_dir.exists() {
            log::debug!("creating new config root folder");
            fs::create_dir(&config_dir).expect("Failed to create config root folder");
        }
        let path = config_dir.join(Self::get_path());

        let config = if path.exists() {
            let file_content = fs::read_to_string(&path)
                .unwrap_or_else(|_| panic!("Couldn't read configuration file at {:?}", &path));

            toml::from_str(&file_content).unwrap_or_else(|err| {
                panic!(
                    "Couldn't parse config at {:?}. Reason: {}. This is probably caused by a config update; just delete the old config and start Pumpkin again",
                    &path,
                    err.message()
                )
            })
        } else {
            let content = Self::default();

            if let Err(err) = fs::write(&path, toml::to_string(&content).unwrap()) {
                warn!(
                    "Couldn't write default config to {:?}. Reason: {}. This is probably caused by a config update; just delete the old config and start Pumpkin again",
                    &path, err
                );
            }

            content
        };

        config.validate();
        config
    }

    fn get_path() -> &'static Path;

    fn validate(&self);
}

impl LoadConfiguration for AdvancedConfiguration {
    fn get_path() -> &'static Path {
        Path::new("features.toml")
    }

    fn validate(&self) {
        self.resource_pack.validate()
    }
}

impl LoadConfiguration for BasicConfiguration {
    fn get_path() -> &'static Path {
        Path::new("configuration.toml")
    }

    fn validate(&self) {
        let min = unsafe { NonZeroU8::new_unchecked(2) };
        let max = unsafe { NonZeroU8::new_unchecked(32) };

        assert!(
            self.view_distance.ge(&min),
            "View distance must be at least 2"
        );
        assert!(
            self.view_distance.le(&max),
            "View distance must be less than 32"
        );
        if self.online_mode {
            assert!(
                self.encryption,
                "When online mode is enabled, encryption must be enabled"
            )
        }
    }
}
