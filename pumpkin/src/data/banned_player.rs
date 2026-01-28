use std::path::Path;

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::net::GameProfile;

use super::{LoadJSONConfiguration, SaveJSONConfiguration, banlist_serializer::BannedPlayerEntry};

#[derive(Deserialize, Serialize, Default)]
#[serde(transparent)]
pub struct BannedPlayerList {
    pub banned_players: Vec<BannedPlayerEntry>,
}

impl BannedPlayerList {
    #[must_use]
    pub fn get_entry(&mut self, profile: &GameProfile) -> Option<&BannedPlayerEntry> {
        self.remove_invalid_entries();
        self.banned_players
            .iter()
            .find(|entry| entry.name == profile.name && entry.uuid == profile.id)
    }

    fn remove_invalid_entries(&mut self) {
        let original_len = self.banned_players.len();

        self.banned_players.retain(|entry| {
            entry
                .expires
                .is_none_or(|expires| expires >= OffsetDateTime::now_utc())
        });

        if original_len != self.banned_players.len() {
            self.save();
        }
    }
}

impl LoadJSONConfiguration for BannedPlayerList {
    fn get_path() -> &'static Path {
        Path::new("banned-players.json")
    }
    fn validate(&self) {
        // TODO: Validate the list
    }
}

impl SaveJSONConfiguration for BannedPlayerList {}
