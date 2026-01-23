use pumpkin_data::packet::clientbound::PLAY_OPEN_SIGN_EDITOR;
use pumpkin_macros::packet;
use pumpkin_util::math::position::BlockPos;
use serde::Serialize;

#[derive(Serialize)]
#[packet(PLAY_OPEN_SIGN_EDITOR)]
pub struct COpenSignEditor {
    pub location: BlockPos,
    pub is_front_text: bool,
}

impl COpenSignEditor {
    #[must_use]
    pub const fn new(location: BlockPos, is_front_text: bool) -> Self {
        Self {
            location,
            is_front_text,
        }
    }
}
