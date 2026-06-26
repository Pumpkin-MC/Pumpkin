use std::{env, fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::{localized_log, localized_log_format};
use tokio::sync::RwLock;
use tracing::{debug, error, warn};

const DATA_FOLDER: &str = "data/";

pub mod op;

pub mod advancement_data;
pub mod banlist_serializer;
pub mod banned_ip;
pub mod banned_player;
pub mod player_server;
pub mod usercache;
pub mod whitelist;

pub struct VanillaData {
    pub banned_ip_list: RwLock<banned_ip::BannedIpList>,
    pub banned_player_list: RwLock<banned_player::BannedPlayerList>,
    pub operator_config: RwLock<op::OperatorConfig>,
    pub user_cache: RwLock<usercache::UserCache>,
    pub whitelist_config: RwLock<whitelist::WhitelistConfig>,
}

impl VanillaData {
    #[must_use]
    pub fn load() -> Self {
        Self {
            banned_ip_list: RwLock::new(banned_ip::BannedIpList::load()),
            banned_player_list: RwLock::new(banned_player::BannedPlayerList::load()),
            operator_config: RwLock::new(op::OperatorConfig::load()),
            user_cache: RwLock::new(usercache::UserCache::load()),
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
        let exe_dir = env::current_dir()
            .unwrap_or_else(|_| panic!("{}", localized_log("debug.expect.current_dir_failed")));
        let data_dir = exe_dir.join(DATA_FOLDER);
        if !data_dir.exists() {
            debug!("{}", localized_log("server.log.creating_data_folder"));
            fs::create_dir(&data_dir).unwrap_or_else(|_| {
                panic!("{}", localized_log("debug.expect.create_data_root_failed"))
            });
        }
        let path = data_dir.join(Self::get_path());

        let config = if path.exists() {
            let file_content = fs::read_to_string(&path).unwrap_or_else(|_| {
                panic!(
                    "{}",
                    localized_log_format(
                        "server.log.failed_read_data_config",
                        &[path.display().to_string()]
                    )
                )
            });

            serde_json::from_str(&file_content).unwrap_or_else(|err| {
                panic!(
                    "{}",
                    localized_log_format(
                        "server.log.failed_parse_data_config",
                        &[path.display().to_string(), err.to_string()]
                    )
                )
            })
        } else {
            let content = Self::default();

            if let Err(err) = fs::write(
                &path,
                serde_json::to_string_pretty(&content).unwrap_or_else(|_| {
                    panic!(
                        "{}",
                        localized_log("debug.expect.serialize_default_data_config_failed",)
                    )
                }),
            ) {
                error!(
                    "{}",
                    localized_log_format(
                        "server.log.failed_write_default_data_config",
                        &[path.display().to_string(), err.to_string()]
                    )
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
        let exe_dir = env::current_dir()
            .unwrap_or_else(|_| panic!("{}", localized_log("debug.expect.current_dir_failed")));
        let data_dir = exe_dir.join(DATA_FOLDER);
        if !data_dir.exists() {
            debug!("{}", localized_log("server.log.creating_data_folder"));
            fs::create_dir(&data_dir).unwrap_or_else(|_| {
                panic!("{}", localized_log("debug.expect.create_data_root_failed"))
            });
        }
        let path = data_dir.join(Self::get_path());

        let content = match serde_json::to_string_pretty(self) {
            Ok(content) => content,
            Err(err) => {
                warn!(
                    "{}",
                    localized_log_format(
                        "server.log.failed_serialize_operator_config",
                        &[path.display().to_string(), err.to_string()]
                    )
                );
                return;
            }
        };

        if let Err(err) = std::fs::write(&path, content) {
            warn!(
                "{}",
                localized_log_format(
                    "server.log.failed_write_operator_config",
                    &[path.display().to_string(), err.to_string()]
                )
            );
        }
    }
}
