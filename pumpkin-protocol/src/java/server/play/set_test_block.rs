use pumpkin_data::packet::serverbound::PLAY_SET_TEST_BLOCK;
use pumpkin_macros::java_packet;
use pumpkin_util::math::position::BlockPos;

#[derive(serde::Deserialize)]
#[java_packet(PLAY_SET_TEST_BLOCK)]
pub struct SSetTestBlock {
    pub position: BlockPos,
    pub mode: TestBlockMode,
    pub message: String,
}

#[derive(serde::Deserialize, Clone, Copy, Debug)]
pub enum TestBlockMode {
    Start,
    Log,
    Fail,
    Accept,
}
