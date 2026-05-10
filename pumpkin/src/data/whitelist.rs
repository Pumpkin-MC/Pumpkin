use std::path::Path;

use pumpkin_config::whitelist::WhitelistEntry;
use serde::{Deserialize, Serialize};

use crate::net::GameProfile;

use super::{LoadJSONConfiguration, SaveJSONConfiguration};

#[derive(Deserialize, Serialize, Default)]
#[serde(transparent)]
pub struct WhitelistConfig {
    pub whitelist: Vec<WhitelistEntry>,
}

impl WhitelistConfig {
    #[must_use]
    pub fn is_whitelisted(&self, profile: &GameProfile) -> bool {
        self.whitelist.iter().any(|entry| entry.uuid == profile.id)
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
