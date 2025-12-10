use pumpkin_data::{Block, BlockDirection};
use pumpkin_util::{
    HeightMap,
    math::{block_box::BlockBox, position::BlockPos, vector3::Vector3},
};
use serde::Deserialize;

use crate::{
    ProtoChunk,
    generation::{
        positions::chunk_pos::{get_center_x, get_center_z, get_offset_x, get_offset_z},
        structure::structures::{StructureGenerator, StructurePiecesCollector, StructurePosition},
    },
};

#[derive(Deserialize, Clone, Debug)]
pub struct BuriedTreasureGenerator;

impl StructureGenerator for BuriedTreasureGenerator {
    fn get_structure_position(&self, chunk: &ProtoChunk) -> super::StructurePosition {
        let x = get_center_x(chunk.x);
        let z = get_center_z(chunk.z);
        let y = chunk.get_top_y(&HeightMap::OceanFloorWg, x, z) - 1;
        let generator = StructurePiecesCollector {
            pieces_positions: vec![BlockBox::from_pos(BlockPos::new(
                get_offset_x(chunk.x, 9),
                90,
                get_offset_z(chunk.z, 9),
            ))],
        };
        StructurePosition {
            position: BlockPos(Vector3::new(x, y, z)),
            generator,
        }
    }

    fn generate(&self, bounding_box: BlockBox, chunk: &mut crate::ProtoChunk) {
        let y = chunk.get_top_y(
            &HeightMap::OceanFloorWg,
            bounding_box.min.x,
            bounding_box.min.z,
        );
        let mut pos = BlockPos::new(bounding_box.min.x, y, bounding_box.min.z);
        for _ in y..chunk.bottom_y() as i32 {
            let state = chunk.get_block_state(&pos.0);
            let down_raw_state = chunk.get_block_state(&pos.down().0);
            let down_block = down_raw_state.to_block();
            if down_block == &Block::SANDSTONE
                || down_block == &Block::STONE
                || down_block == &Block::ANDESITE
                || down_block == &Block::GRANITE
                || down_block == &Block::DIORITE
            {
                for dir in BlockDirection::all() {
                    let pos = pos.offset(dir.to_offset());
                    let dir_state = chunk.get_block_state(&pos.0);
                    if !dir_state.to_state().is_air() && !Self::is_liquid(dir_state.to_block()) {
                        continue;
                    }
                    let down_pos = pos.down();
                    let down_state = chunk.get_block_state(&down_pos.0);
                    if (down_state.to_state().is_air() || Self::is_liquid(down_state.to_block()))
                        && dir != BlockDirection::Up
                    {
                        chunk.set_block_state(&pos.0, down_raw_state.to_state());
                        continue;
                    }
                    let state1 = if state.to_state().is_air() || Self::is_liquid(state.to_block()) {
                        Block::SAND.default_state
                    } else {
                        state.to_state()
                    };
                    chunk.set_block_state(&pos.0, state1);
                }
                // TODO: add loot
                chunk.set_block_state(&pos.0, Block::CHEST.default_state);
                return;
            }
            pos = pos.down();
        }
    }
}

impl BuriedTreasureGenerator {
    fn is_liquid(block: &Block) -> bool {
        block == &Block::WATER || block == &Block::LAVA
    }
}
