use std::{path::Path, sync::LazyLock};

use pumpkin_config::whitelist::WhitelistEntry;
use serde::{Deserialize, Serialize};

use crate::net::GameProfile;

use super::{LoadJSONConfiguration, SaveJSONConfiguration};

pub static WHITELIST_CONFIG: LazyLock<tokio::sync::RwLock<WhitelistConfig>> =
    LazyLock::new(|| tokio::sync::RwLock::new(WhitelistConfig::load()));

#[derive(Deserialize, Serialize, Default)]
#[serde(transparent)]
pub struct WhitelistConfig {
    pub whitelist: Vec<WhitelistEntry>,
}

impl WhitelistConfig {
    #[must_use]
    pub fn is_whitelisted(&self, profile: &GameProfile) -> bool {
        self.whitelist
            .iter()
            .any(|entry| entry.uuid == profile.id && entry.name == profile.name)
    }
}

impl LoadJSONConfiguration for WhitelistConfig {
    fn get_path() -> &'static Path {
        Path::new("whitelist.json")
    }
    fn validate(&self) {
        // TODO: Validate the whitelist configuration
    }
}

impl SaveJSONConfiguration for WhitelistConfig {}
