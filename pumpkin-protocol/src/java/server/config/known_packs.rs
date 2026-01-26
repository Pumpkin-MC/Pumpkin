use pumpkin_data::packet::serverbound::CONFIG_SELECT_KNOWN_PACKS;
use pumpkin_macros::java_packet;
use serde::Serialize;

use crate::VarInt;

#[derive(serde::Deserialize, Serialize)]
#[java_packet(CONFIG_SELECT_KNOWN_PACKS)]
pub struct SKnownPacks {
    pub known_pack_count: VarInt,
    // known_packs: &'a [KnownPack]
}
