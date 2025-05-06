use serde::{Deserialize, Serialize};

use crate::math::position::BlockPos;

#[derive(Debug, PartialEq, Eq)]
pub struct ParseGameModeError;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct RespawnPoint {
    // TODO: Multi dimension
    // pub world: DimensionType,
    pub position: BlockPos,
    pub yaw: f32,
    pub force: bool,
}
