use pumpkin_data::packet::serverbound::PLAY_TEST_INSTANCE_BLOCK_ACTION;
use pumpkin_macros::java_packet;
use pumpkin_util::math::position::BlockPos;

use crate::VarInt;

#[derive(serde::Deserialize)]
#[java_packet(PLAY_TEST_INSTANCE_BLOCK_ACTION)]
pub struct STestInstanceBlockAction {
    pub pos: BlockPos,
    pub action: TestInstanceBlockAction,
    pub data: TestInstanceBlockData,
}

#[derive(serde::Deserialize, Clone, Copy, Debug)]
pub enum TestInstanceBlockAction {
    Init,
    Query,
    Set,
    Reset,
    Save,
    Export,
    Run,
}

#[derive(serde::Deserialize)]
pub struct VarIntVector3 {
    pub x: VarInt,
    pub y: VarInt,
    pub z: VarInt,
}

#[derive(serde::Deserialize)]
pub struct TestInstanceBlockData {
    pub test: Option<String>,
    pub size: VarIntVector3,
    pub rotation: pumpkin_data::block_rotation::Rotation,
    pub ignore_entities: bool,
    pub status: TestInstanceBlockStatus,
    pub error_message: Option<String>,
}

#[derive(serde::Deserialize, Clone, Copy, Debug)]
pub enum TestInstanceBlockStatus {
    Cleared,
    Running,
    Success,
    Failed,
}
