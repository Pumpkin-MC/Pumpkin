use pumpkin_data::packet::serverbound::CONFIG_SELECT_KNOWN_PACKS;
use pumpkin_macros::java_packet;
use serde::{Deserialize, Serialize};

/// A single known pack entry received from the client.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct KnownPackEntry {
    pub namespace: String,
    pub id: String,
    pub version: String,
}

#[derive(Deserialize, Serialize)]
#[java_packet(CONFIG_SELECT_KNOWN_PACKS)]
pub struct SKnownPacks {
    pub known_packs: Vec<KnownPackEntry>,
}
