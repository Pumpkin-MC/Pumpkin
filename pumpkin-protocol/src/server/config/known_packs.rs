use pumpkin_data::packet::serverbound::CONFIG_SELECT_KNOWN_PACKS;
use pumpkin_macros::packet;
use serde::Serialize;

use crate::VarInt;

#[derive(serde::Deserialize, Serialize)]
#[packet(CONFIG_SELECT_KNOWN_PACKS)]
pub struct SKnownPacks {
    pub known_pack_count: VarInt,
    // known_packs: &'a [KnownPack]
}
