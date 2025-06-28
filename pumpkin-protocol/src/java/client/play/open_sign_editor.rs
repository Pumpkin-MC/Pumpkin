use pumpkin_data::packet::clientbound::PLAY_OPEN_SIGN_EDITOR;
use pumpkin_macros::packet;
use pumpkin_util::math::position::BlockPos;
use serde::Serialize;

#[derive(Serialize)]
#[packet(PLAY_OPEN_SIGN_EDITOR)]
pub struct COpenSignEditor {
    location: BlockPos,
    is_front_text: bool,
}

impl COpenSignEditor {
    pub fn new(location: BlockPos, is_front_text: bool) -> Self {
        Self {
            location,
            is_front_text,
        }
    }
}
