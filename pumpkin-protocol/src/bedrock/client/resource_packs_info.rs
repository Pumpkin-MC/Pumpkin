use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[packet(0x06)]
pub struct CResourcePacksInfo {
    resource_pack_required: bool,
    has_addon_packs: bool,
    has_scripts: bool,
    // TODO: Add more
}

impl CResourcePacksInfo {
    pub fn new(resource_pack_required: bool, has_addon_packs: bool, has_scripts: bool) -> Self {
        Self {
            resource_pack_required,
            has_addon_packs,
            has_scripts,
        }
    }
}
