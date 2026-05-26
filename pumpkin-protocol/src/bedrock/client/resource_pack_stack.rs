use crate::{bedrock::client::start_game::Experiments, serial::PacketWrite};
use pumpkin_macros::packet;

#[derive(PacketWrite)]
pub struct ResourcePackStackEntry {
    pub uuid: String,
    pub version: String,
    pub sub_pack_name: String,
}

#[derive(PacketWrite)]
#[packet(7)]
pub struct CResourcePackStackPacket {
    pub resource_pack_required: bool,
    pub resource_packs: Vec<ResourcePackStackEntry>,
    pub game_version: String,
    pub experiments: Experiments,
    pub include_editor_packs: bool,
}

impl CResourcePackStackPacket {
    #[must_use]
    pub const fn new(
        resource_pack_required: bool,
        resource_packs: Vec<ResourcePackStackEntry>,
        game_version: String,
        experiments: Experiments,
        include_editor_packs: bool,
    ) -> Self {
        Self {
            resource_pack_required,
            resource_packs,
            game_version,
            experiments,
            include_editor_packs,
        }
    }
}
