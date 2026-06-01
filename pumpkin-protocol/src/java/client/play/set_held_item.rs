use pumpkin_data::packet::clientbound::{PLAY_SET_CARRIED_ITEM, PLAY_SET_HELD_SLOT};
use pumpkin_util::version::JavaMinecraftVersion;
use serde::Serialize;

use crate::packet::MultiVersionJavaPacket;

#[derive(Serialize)]
pub struct CSetSelectedSlot {
    pub slot: i8,
}

impl CSetSelectedSlot {
    #[must_use]
    pub const fn new(slot: i8) -> Self {
        Self { slot }
    }
}

impl MultiVersionJavaPacket for CSetSelectedSlot {
    fn to_id(version: JavaMinecraftVersion) -> i32 {
        if version >= JavaMinecraftVersion::V_1_21_2 {
            PLAY_SET_HELD_SLOT.to_id(version)
        } else if version == JavaMinecraftVersion::V_1_21 {
            PLAY_SET_CARRIED_ITEM.to_id(version)
        } else {
            PLAY_SET_HELD_SLOT.to_id(version)
        }
    }
}
