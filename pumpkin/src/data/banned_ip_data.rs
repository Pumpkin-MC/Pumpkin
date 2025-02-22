use std::{net::IpAddr, path::Path, sync::LazyLock};

use chrono::Local;
use serde::{Deserialize, Serialize};

use super::{LoadJSONConfiguration, SaveJSONConfiguration, banlist_serializer::BannedIpEntry};

pub static BANNED_IP_LIST: LazyLock<tokio::sync::RwLock<BannedIpList>> =
    LazyLock::new(|| tokio::sync::RwLock::new(BannedIpList::load()));

#[derive(Deserialize, Serialize, Default)]
#[serde(transparent)]
pub struct BannedIpList {
    pub banned_ips: Vec<BannedIpEntry>,
}

impl BannedIpList {
    #[must_use]
    pub fn get_entry(&mut self, ip: &IpAddr) -> Option<&BannedIpEntry> {
        self.remove_invalid_entries();
        self.banned_ips.iter().find(|entry| entry.ip == *ip)
    }

    fn remove_invalid_entries(&mut self) {
        let original_len = self.banned_ips.len();

        self.banned_ips
            .retain(|entry| entry.expires.is_none_or(|expires| expires >= Local::now()));

        if original_len != self.banned_ips.len() {
            self.save();
        }
    }
}

impl LoadJSONConfiguration for BannedIpList {
    fn get_path() -> &'static Path {
        Path::new("banned-ips.json")
    }
    fn validate(&self) {
        // TODO: Validate the list
    }
}

impl SaveJSONConfiguration for BannedIpList {}
