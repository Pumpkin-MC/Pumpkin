use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

use crate::codec::var_uint::VarUInt;

#[derive(Serialize, Deserialize)]
#[packet(0x07)]
pub struct CResourcePackStackPacket {
    resource_pack_required: bool,
    addons_list_size: VarUInt,
    texture_pack_list_size: VarUInt,
    game_version: String,
    experiments_size: i32,
    is_experiments_previously_toggled: bool,
    /// When connecting to an Editor world, include the vanilla editor packs in the stack
    include_editor_packs: bool,
}

impl CResourcePackStackPacket {
    pub fn new(
        resource_pack_required: bool,
        addons_list_size: VarUInt,
        texture_pack_list_size: VarUInt,
        game_version: String,
        experiments_size: i32,
        is_experiments_previously_toggled: bool,
        include_editor_packs: bool,
    ) -> Self {
        Self {
            resource_pack_required,
            addons_list_size,
            texture_pack_list_size,
            game_version,
            experiments_size,
            is_experiments_previously_toggled,
            include_editor_packs,
        }
    }
}
