use pumpkin_macros::packet;

use crate::{
    bedrock::client::start_game::Experiments, codec::var_uint::VarUInt, serial::PacketWrite,
};

#[derive(PacketWrite)]
#[packet(7)]
pub struct CResourcePackStackPacket {
    // https://mojang.github.io/bedrock-protocol-docs/html/ResourcePackStackPacket.html
    resource_pack_required: bool,
    texture_pack_list_size: VarUInt,
    game_version: String,
    experiments: Experiments,
    /// When connecting to an Editor world, include the vanilla editor packs in the stack
    include_editor_packs: bool,
}

impl CResourcePackStackPacket {
    #[must_use]
    pub fn new(
        resource_pack_required: bool,
        texture_pack_list_size: VarUInt,
        game_version: String,
        experiments: Experiments,
        include_editor_packs: bool,
    ) -> Self {
        Self {
            resource_pack_required,
            texture_pack_list_size,
            game_version,
            experiments,
            include_editor_packs,
        }
    }
}
