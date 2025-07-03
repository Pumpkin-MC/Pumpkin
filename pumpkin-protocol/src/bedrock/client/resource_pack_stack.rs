use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

use crate::codec::var_uint::VarUInt;

#[derive(Serialize, Deserialize)]
#[packet(0x07)]
pub struct CResourcePackStackPacket {
    resource_pack_required: bool,
    addons_list_size: VarUInt,
}

impl CResourcePackStackPacket {
    pub fn new(resource_pack_required: bool, addons_list_size: VarUInt) -> Self {
        Self {
            resource_pack_required,
            addons_list_size,
        }
    }
}
