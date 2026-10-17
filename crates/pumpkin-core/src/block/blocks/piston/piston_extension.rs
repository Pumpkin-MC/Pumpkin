use pumpkin_data::BlockState;
use pumpkin_macros::pumpkin_block;

use crate::block::{BlockBehaviour, PathComputationType};

#[pumpkin_block("minecraft:moving_piston")]
pub struct PistonExtensionBlock;

impl BlockBehaviour for PistonExtensionBlock {
    fn is_pathfindable(&self, _state: &BlockState, _computation_type: PathComputationType) -> bool {
        false
    }
}
