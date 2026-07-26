use std::sync::Arc;

use pumpkin_data::Block;
use pumpkin_util::{
    math::{block_box::BlockBox, position::BlockPos},
    random::{RandomGenerator, RandomImpl},
};

use crate::{
    ProtoChunk,
    generation::{
        positions::chunk_pos::{get_center_x, start_block_x, start_block_z},
        structure::{
            piece::StructurePieceType,
            structures::{
                StructureGenerator, StructureGeneratorContext, StructurePiece, StructurePieceBase,
                StructurePiecesCollector, StructurePosition, WorldPortalExt,
            },
        },
    },
};

const INITIAL_Y: i32 = 50;

pub struct MineshaftGenerator {
    pub is_mesa: bool,
}

impl StructureGenerator for MineshaftGenerator {
    fn get_structure_position(
        &self,
        context: StructureGeneratorContext<'_>,
    ) -> Option<StructurePosition> {
        let mut random = context.random;

        // Vanilla consumes this value before creating the first piece.
        random.next_f64();

        let chunk_center_x = get_center_x(context.chunk_x);
        let room_x = start_block_x(context.chunk_x) + 2;
        let room_z = start_block_z(context.chunk_z) + 2;
        let bounding_box = BlockBox::new(
            room_x - 12,
            INITIAL_Y,
            room_z - 12,
            room_x + 12,
            INITIAL_Y + 2,
            room_z + 12,
        );

        let mut collector = StructurePiecesCollector::default();
        collector.add_piece(Box::new(MineshaftPiece {
            piece: StructurePiece::new(StructurePieceType::MineshaftCrossing, bounding_box, 0),
            is_mesa: self.is_mesa,
        }));

        let y_offset = if self.is_mesa {
            let bounding_box = collector.get_bounding_box();
            let center_x = i32::midpoint(bounding_box.min.x, bounding_box.max.x);
            let center_z = i32::midpoint(bounding_box.min.z, bounding_box.max.z);
            let surface_height = context.height_sampler.map_or(context.sea_level, |sampler| {
                sampler.estimate_height(center_x, center_z)
            });
            let target_y = if surface_height <= context.sea_level {
                context.sea_level
            } else {
                random.next_inbetween_i32(context.sea_level, surface_height)
            };
            let center_y = i32::midpoint(bounding_box.min.y, bounding_box.max.y);
            let offset = target_y - center_y;
            collector.shift(offset);
            offset
        } else {
            collector.shift_into(context.sea_level, context.min_y, &mut random, 10)
        };

        Some(StructurePosition {
            start_pos: BlockPos::new(
                chunk_center_x,
                INITIAL_Y + y_offset,
                start_block_z(context.chunk_z),
            ),
            collector: Arc::new(collector.into()),
        })
    }
}

pub struct MineshaftPiece {
    piece: StructurePiece,
    is_mesa: bool,
}

impl StructurePieceBase for MineshaftPiece {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn get_structure_piece(&self) -> &StructurePiece {
        &self.piece
    }
    fn get_structure_piece_mut(&mut self) -> &mut StructurePiece {
        &mut self.piece
    }
    fn place(
        &mut self,
        chunk: &mut ProtoChunk,
        _block_registry: &dyn WorldPortalExt,
        _random: &mut RandomGenerator,
        _seed: i64,
        chunk_box: &BlockBox,
    ) {
        let origin_x = i32::midpoint(self.piece.bounding_box.min.x, self.piece.bounding_box.max.x);
        let start_y = self.piece.bounding_box.min.y;
        let origin_z = i32::midpoint(self.piece.bounding_box.min.z, self.piece.bounding_box.max.z);

        let wood_planks = if self.is_mesa {
            Block::DARK_OAK_PLANKS
        } else {
            Block::OAK_PLANKS
        };
        let wood_fence = if self.is_mesa {
            Block::DARK_OAK_FENCE
        } else {
            Block::OAK_FENCE
        };

        // Draw underground corridors
        for y in start_y..(start_y + 3) {
            for x in (origin_x - 12)..=(origin_x + 12) {
                for z in (origin_z - 12)..=(origin_z + 12) {
                    let in_center = (x - origin_x).abs() <= 2 && (z - origin_z).abs() <= 2;
                    let in_ns_corridor = (x - origin_x).abs() <= 1;
                    let in_ew_corridor = (z - origin_z).abs() <= 1;

                    if (in_center || in_ns_corridor || in_ew_corridor)
                        && chunk_box.contains(x, y, z)
                    {
                        chunk.set_block_state(x, y, z, Block::AIR.default_state);

                        if (x - origin_x).abs() == 2 && y == start_y {
                            chunk.set_block_state(x, y, z, wood_fence.default_state);
                        }
                        if (z - origin_z).abs() == 2 && y == start_y {
                            chunk.set_block_state(x, y, z, wood_fence.default_state);
                        }
                    }
                }
            }
        }

        // Place rails and supports
        for x in (origin_x - 12)..=(origin_x + 12) {
            let y = start_y;
            let z = origin_z;
            if chunk_box.contains(x, y, z) {
                chunk.set_block_state(x, y, z, Block::RAIL.default_state);
            }
            if x % 5 == 0 {
                for sy in start_y..=(start_y + 2) {
                    if chunk_box.contains(x, sy, z - 1) {
                        chunk.set_block_state(x, sy, z - 1, wood_fence.default_state);
                    }
                    if chunk_box.contains(x, sy, z + 1) {
                        chunk.set_block_state(x, sy, z + 1, wood_fence.default_state);
                    }
                }
                if chunk_box.contains(x, start_y + 2, z) {
                    chunk.set_block_state(x, start_y + 2, z, wood_planks.default_state);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generation::structure::structures::{HeightSampler, create_chunk_random};

    struct FixedHeightSampler(i32);

    impl HeightSampler for FixedHeightSampler {
        fn estimate_height(&mut self, _block_x: i32, _block_z: i32) -> i32 {
            self.0
        }
    }

    fn context(height_sampler: Option<&mut dyn HeightSampler>) -> StructureGeneratorContext<'_> {
        StructureGeneratorContext {
            seed: 123,
            chunk_x: 4,
            chunk_z: -3,
            random: create_chunk_random(123, 4, -3),
            sea_level: 63,
            min_y: -64,
            height_sampler,
            structure_key: None,
        }
    }

    #[test]
    fn normal_mineshafts_are_shifted_below_sea_level() {
        let position = MineshaftGenerator { is_mesa: false }
            .get_structure_position(context(None))
            .expect("mineshaft has a generation position");
        let bounding_box = position.get_bounding_box();

        assert_eq!(bounding_box.get_block_count_y(), 3);
        assert!(bounding_box.min.y >= -63);
        assert!(bounding_box.max.y <= 52);
        assert_eq!(position.start_pos.0.y, bounding_box.min.y);
    }

    #[test]
    fn mesa_mineshafts_follow_surface_height_range() {
        let mut height_sampler = FixedHeightSampler(120);
        let position = MineshaftGenerator { is_mesa: true }
            .get_structure_position(context(Some(&mut height_sampler)))
            .expect("mineshaft has a generation position");
        let bounding_box = position.get_bounding_box();
        let center_y = i32::midpoint(bounding_box.min.y, bounding_box.max.y);

        assert!((63..=120).contains(&center_y));
        assert_eq!(position.start_pos.0.y, center_y - 1);
    }
}
