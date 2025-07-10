use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

use crate::{bedrock::client::start_game::Experiments, codec::var_uint::VarUInt};

#[derive(Serialize, Deserialize)]
#[packet(0x07)]
pub struct CResourcePackStackPacket {
    // https://mojang.github.io/bedrock-protocol-docs/html/ResourcePackStackPacket.html
    resource_pack_required: bool,
    addons_list_size: VarUInt,
    texture_pack_list_size: VarUInt,
    game_version: String,
    experiments: Experiments,
    /// When connecting to an Editor world, include the vanilla editor packs in the stack
    include_editor_packs: bool,
}

impl CResourcePackStackPacket {
    pub fn new(
        resource_pack_required: bool,
        addons_list_size: VarUInt,
        texture_pack_list_size: VarUInt,
        game_version: String,
        experiments: Experiments,
        include_editor_packs: bool,
    ) -> Self {
        Self {
            resource_pack_required,
            addons_list_size,
            texture_pack_list_size,
            game_version,
            experiments,
            include_editor_packs,
        }
    }
}
