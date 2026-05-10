use std::sync::Arc;

use pumpkin_data::{Block, BlockDirection};
use pumpkin_util::{
    HeightMap,
    math::{block_box::BlockBox, position::BlockPos},
    random::RandomGenerator,
};

use crate::{
    ProtoChunk,
    generation::{
        positions::chunk_pos::{get_center_x, get_center_z},
        structure::{
            piece::StructurePieceType,
            structures::{
                StructureGenerator, StructureGeneratorContext, StructurePiece, StructurePieceBase,
                StructurePiecesCollector, StructurePosition,
            },
        },
    },
};

pub struct BuriedTreasureGenerator;

impl StructureGenerator for BuriedTreasureGenerator {
    fn get_structure_position(
        &self,
        context: StructureGeneratorContext,
    ) -> Option<StructurePosition> {
        let x = get_center_x(context.chunk_x);
        let z = get_center_z(context.chunk_z);

        let bounding_box = BlockBox::new(x, -64, z, x, 320, z);

        let mut collector = StructurePiecesCollector::default();

        collector.add_piece(Box::new(BuriedTreasurePiece {
            piece: StructurePiece::new(StructurePieceType::BuriedTreasure, bounding_box, 0),
        }));

        Some(StructurePosition {
            start_pos: BlockPos::new(x, 90, z),
            collector: Arc::new(collector.into()),
        })
    }
}

#[derive(Clone)]
pub struct BuriedTreasurePiece {
    piece: StructurePiece,
}

impl StructurePieceBase for BuriedTreasurePiece {
    fn clone_box(&self) -> Box<dyn StructurePieceBase> {
        Box::new((*self).clone())
    }

    fn place(
        &mut self,
        chunk: &mut ProtoChunk,
        _random: &mut RandomGenerator,
        _seed: i64,
        _chunk_box: &BlockBox,
    ) {
        let boundingbox = self.bounding_box();
        let y = chunk.get_top_y(
            &HeightMap::OceanFloorWg,
            boundingbox.min.x,
            boundingbox.min.z,
        );

        let mut pos = BlockPos::new(boundingbox.min.x, y, boundingbox.min.z);
        let bottom_y = chunk.bottom_y() as i32;

        for _ in (bottom_y..=y).rev() {
            let state = chunk.get_block_state(&pos.0);
            let down_pos = pos.down();
            let down_raw_state = chunk.get_block_state(&down_pos.0);
            let down_block = down_raw_state.to_block_id();

            if down_block == Block::SANDSTONE
                || down_block == Block::STONE
                || down_block == Block::ANDESITE
                || down_block == Block::GRANITE
                || down_block == Block::DIORITE
            {
                for dir in BlockDirection::all() {
                    let offset_pos = pos.offset(dir.to_offset());
                    let dir_state = chunk.get_block_state(&offset_pos.0);

                    if !dir_state.to_state().is_air() && !Self::is_liquid(dir_state.to_block_id()) {
                        continue;
                    }

                    let down_offset_pos = offset_pos.down();
                    let down_offset_state = chunk.get_block_state(&down_offset_pos.0);

                    if (down_offset_state.to_state().is_air()
                        || Self::is_liquid(down_offset_state.to_block_id()))
                        && dir != BlockDirection::Up
                    {
                        chunk.set_block_state(
                            offset_pos.0.x,
                            offset_pos.0.y,
                            offset_pos.0.z,
                            down_raw_state.to_state(),
                        );
                        continue;
                    }

                    let state1 =
                        if state.to_state().is_air() || Self::is_liquid(state.to_block_id()) {
                            Block::SAND.default_state
                        } else {
                            state.to_state()
                        };
                    chunk.set_block_state(offset_pos.0.x, offset_pos.0.y, offset_pos.0.z, state1);
                }

                // Place the Chest
                // TODO: Add loot table logic here (requires seed)
                chunk.set_block_state(pos.0.x, pos.0.y, pos.0.z, Block::CHEST.default_state);
                return;
            }
            pos = pos.down();
        }
    }

    fn get_structure_piece(&self) -> &StructurePiece {
        &self.piece
    }

    fn get_structure_piece_mut(&mut self) -> &mut StructurePiece {
        &mut self.piece
    }
}

impl BuriedTreasurePiece {
    fn is_liquid(block: u16) -> bool {
        block == Block::WATER || block == Block::LAVA
    }
}
