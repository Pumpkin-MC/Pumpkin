use pumpkin_data::BlockState;
use pumpkin_util::math::{block_box::BlockBox, position::BlockPos, vector3::Vector3};

use crate::ProtoChunk;

pub mod buried_treasure;
pub mod nether_fortress;
pub mod swamp_hut;

pub trait StructureGenerator {
    fn get_structure_position(&self, chunk: &ProtoChunk) -> StructurePosition;

    fn generate(&self, position: BlockBox, chunk: &mut crate::ProtoChunk);
}

#[expect(clippy::too_many_arguments)]
pub fn fill_with_outline(
    min_x: i32,
    min_y: i32,
    min_z: i32,
    max_x: i32,
    max_y: i32,
    max_z: i32,
    outline: &BlockState,
    inside: &BlockState,
    chunk: &mut crate::ProtoChunk,
) {
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            for z in min_z..=max_z {
                if y == min_y || y == max_y || x == min_x || x == max_x || z == min_z || z == max_z
                {
                    chunk.set_block_state(&Vector3::new(x, y, z), outline);
                } else {
                    chunk.set_block_state(&Vector3::new(x, y, z), inside);
                }
            }
        }
    }
}

#[expect(clippy::too_many_arguments)]
pub fn fill(
    min_x: i32,
    min_y: i32,
    min_z: i32,
    max_x: i32,
    max_y: i32,
    max_z: i32,
    state: &BlockState,
    chunk: &mut crate::ProtoChunk,
) {
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            for z in min_z..=max_z {
                chunk.set_block_state(&Vector3::new(x, y, z), state);
            }
        }
    }
}

pub fn fill_downwards(x: i32, y: i32, z: i32, state: &BlockState, chunk: &mut crate::ProtoChunk) {
    for y in y..chunk.bottom_y() as i32 {
        chunk.set_block_state(&Vector3::new(x, y, z), state);
    }
}

#[derive(Clone, Debug)]
pub struct StructurePosition {
    pub position: BlockPos,
    pub generator: StructurePiecesCollector,
}

#[derive(Default, Clone, Debug)]
pub struct StructurePiecesCollector {
    pub pieces_positions: Vec<BlockBox>,
}
