use pumpkin_data::packet::serverbound::PLAY_SET_CREATIVE_MODE_SLOT;
use pumpkin_macros::packet;

use crate::codec::slot::Slot;

#[derive(serde::Deserialize, Debug)]
#[packet(PLAY_SET_CREATIVE_MODE_SLOT)]
pub struct SSetCreativeSlot {
    pub slot: i16,
    pub clicked_item: Slot,
}
