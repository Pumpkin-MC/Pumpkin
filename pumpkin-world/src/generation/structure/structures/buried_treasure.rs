use pumpkin_data::{Block, BlockDirection};
use pumpkin_util::{
    HeightMap,
    math::{block_box::BlockBox, position::BlockPos},
};
use serde::Deserialize;

use crate::{
    ProtoChunk,
    generation::{
        positions::chunk_pos::{get_center_x, get_center_z},
        structure::structures::{
            StructureGenerator, StructurePiece, StructurePiecesCollector, StructurePosition,
        },
    },
};

#[derive(Deserialize, Clone)]
pub struct BuriedTreasureGenerator;

impl StructureGenerator for BuriedTreasureGenerator {
    fn try_generate(&self, _seed: i64, chunk_x: i32, chunk_z: i32) -> Option<StructurePosition> {
        let x = get_center_x(chunk_x);
        let z = get_center_z(chunk_z);

        let bounding_box = BlockBox::new(x, -64, z, x, 320, z);

        let mut collector = StructurePiecesCollector::default();

        collector.add_piece(Box::new(BuriedTreasurePiece {
            target_x: x,
            target_z: z,
            bounding_box,
        }));

        Some(StructurePosition {
            start_pos: BlockPos::new(x, 90, z),
            collector: collector.into(),
        })
    }
}

/// The Logic for actually placing the chest
#[derive(Debug, Clone)]
pub struct BuriedTreasurePiece {
    target_x: i32,
    target_z: i32,
    bounding_box: BlockBox,
}

impl StructurePiece for BuriedTreasurePiece {
    fn bounding_box(&self) -> &BlockBox {
        &self.bounding_box
    }

    fn place(&self, chunk: &mut ProtoChunk, _seed: i64) {
        let y = chunk.get_top_y(&HeightMap::OceanFloorWg, self.target_x, self.target_z);

        let mut pos = BlockPos::new(self.target_x, y, self.target_z);
        let bottom_y = chunk.bottom_y() as i32;

        for _ in (bottom_y..=y).rev() {
            let state = chunk.get_block_state(&pos.0);
            let down_pos = pos.down();
            let down_raw_state = chunk.get_block_state(&down_pos.0);
            let down_block = down_raw_state.to_block();

            if down_block == &Block::SANDSTONE
                || down_block == &Block::STONE
                || down_block == &Block::ANDESITE
                || down_block == &Block::GRANITE
                || down_block == &Block::DIORITE
            {
                for dir in BlockDirection::all() {
                    let offset_pos = pos.offset(dir.to_offset());
                    let dir_state = chunk.get_block_state(&offset_pos.0);

                    if !dir_state.to_state().is_air() && !Self::is_liquid(dir_state.to_block()) {
                        continue;
                    }

                    let down_offset_pos = offset_pos.down();
                    let down_offset_state = chunk.get_block_state(&down_offset_pos.0);

                    if (down_offset_state.to_state().is_air()
                        || Self::is_liquid(down_offset_state.to_block()))
                        && dir != BlockDirection::Up
                    {
                        chunk.set_block_state(&offset_pos.0, down_raw_state.to_state());
                        continue;
                    }

                    let state1 = if state.to_state().is_air() || Self::is_liquid(state.to_block()) {
                        Block::SAND.default_state
                    } else {
                        state.to_state()
                    };
                    chunk.set_block_state(&offset_pos.0, state1);
                }

                // Place the Chest
                // TODO: Add loot table logic here (requires seed)
                chunk.set_block_state(&pos.0, Block::CHEST.default_state);
                return;
            }
            pos = pos.down();
        }
    }
}

impl BuriedTreasurePiece {
    fn is_liquid(block: &Block) -> bool {
        block == &Block::WATER || block == &Block::LAVA
    }
}
