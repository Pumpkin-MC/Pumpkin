// Last verified for v2169

use crate::{bedrock::client::start_game::Experiments, serial::PacketWrite};
use pumpkin_macros::packet;

#[derive(PacketWrite)]
pub struct PackEntry {
    pub pack_id: String,
    pub version: String,
    pub sub_pack_name: String,
}

#[derive(PacketWrite)]
#[packet(7)]
pub struct CResourcePackStackPacket {
    pub texture_pack_required: bool,
    pub texture_pack_list: Vec<PackEntry>,
    pub base_game_version: String,
    pub experiments: Experiments,
    pub use_vanilla_editor_packs: bool,
}
