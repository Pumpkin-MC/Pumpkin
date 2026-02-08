use pumpkin_data::packet::clientbound::PLAY_OPEN_BOOK;
use pumpkin_macros::java_packet;
use serde::Serialize;

use crate::VarInt;

/// Opens the client's book editing/viewing UI for the book in the given hand.
///
/// 0 = main hand, 1 = off hand.
#[derive(Serialize)]
#[java_packet(PLAY_OPEN_BOOK)]
pub struct COpenBook {
    pub hand: VarInt,
}

impl COpenBook {
    #[must_use]
    pub const fn new(hand: VarInt) -> Self {
        Self { hand }
    }
}
