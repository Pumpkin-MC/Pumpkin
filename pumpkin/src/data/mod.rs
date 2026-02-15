use std::{env, fs, path::Path};

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, error, warn};

const DATA_FOLDER: &str = "data/";

pub mod op;

pub mod banlist_serializer;
pub mod banned_ip;
pub mod banned_player;
pub mod player_server;
pub mod whitelist;

pub struct VanillaData {
    pub banned_ip_list: RwLock<banned_ip::BannedIpList>,
    pub banned_player_list: RwLock<banned_player::BannedPlayerList>,
    pub operator_config: RwLock<op::OperatorConfig>,
    pub whitelist_config: RwLock<whitelist::WhitelistConfig>,
}

impl VanillaData {
    #[must_use]
    pub fn load() -> Self {
        Self {
            banned_ip_list: RwLock::new(banned_ip::BannedIpList::load()),
            banned_player_list: RwLock::new(banned_player::BannedPlayerList::load()),
            operator_config: RwLock::new(op::OperatorConfig::load()),
            whitelist_config: RwLock::new(whitelist::WhitelistConfig::load()),
        }
    }
}

pub trait LoadJSONConfiguration {
    #[must_use]
    fn load() -> Self
    where
        Self: Sized + Default + Serialize + for<'de> Deserialize<'de>,
    {
        let exe_dir = env::current_dir().unwrap();
        let data_dir = exe_dir.join(DATA_FOLDER);
        if !data_dir.exists() {
            debug!("creating new data root folder");
            fs::create_dir(&data_dir).expect("Failed to create data root folder");
        }
        let path = data_dir.join(Self::get_path());

        let config = if path.exists() {
            let file_content = fs::read_to_string(&path).unwrap_or_else(|_| {
                panic!("Couldn't read configuration file at {}", path.display())
            });

            serde_json::from_str(&file_content).unwrap_or_else(|err| {
                panic!(
                    "Couldn't parse data config at {}. Reason: {err}. This is probably caused by a config update. Just delete the old data config and restart.", path.display(),
                )
            })
        } else {
            let content = Self::default();

            if let Err(err) = fs::write(&path, serde_json::to_string_pretty(&content).unwrap()) {
                error!(
                    "Couldn't write default data config to {}. Reason: {err}. This is probably caused by a config update. Just delete the old data config and restart.",
                    path.display(),
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

pub trait SaveJSONConfiguration: LoadJSONConfiguration {
    fn save(&self)
    where
        Self: Sized + Default + Serialize + for<'de> Deserialize<'de>,
    {
        let exe_dir = env::current_dir().unwrap();
        let data_dir = exe_dir.join(DATA_FOLDER);
        if !data_dir.exists() {
            debug!("creating new data root folder");
            fs::create_dir(&data_dir).expect("Failed to create data root folder");
        }
        let path = data_dir.join(Self::get_path());

        let content = match serde_json::to_string_pretty(self) {
            Ok(content) => content,
            Err(err) => {
                warn!(
                    "Couldn't serialize operator data config to {}. Reason: {err}",
                    path.display()
                );
                return;
            }
        };

        if let Err(err) = std::fs::write(&path, content) {
            warn!(
                "Couldn't write operator config to {}. Reason: {err}",
                path.display()
            );
        }
    }
}
