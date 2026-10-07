use std::{net::IpAddr, path::Path};

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use super::{LoadJSONConfiguration, SaveJSONConfiguration, banlist_serializer::BannedIpEntry};

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

        self.banned_ips.retain(|entry| {
            entry
                .expires
                .is_none_or(|expires| expires >= OffsetDateTime::now_utc())
        });

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
