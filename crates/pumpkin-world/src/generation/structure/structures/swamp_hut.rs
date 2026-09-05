use std::sync::Arc;

use pumpkin_data::{
    Block, BlockState,
    block_properties::{HorizontalFacing, OakStairsLikeProperties, StairsShape},
};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::{
    BlockDirection,
    math::{block_box::BlockBox, position::BlockPos},
    random::RandomGenerator,
};
use serde::Deserialize;

use crate::{
    ProtoChunk,
    generation::{
        positions::chunk_pos::{start_block_x, start_block_z},
        structure::{
            piece::StructurePieceType,
            shiftable_piece::ShiftableStructurePiece,
            structures::{
                StructureGenerator, StructureGeneratorContext, StructurePieceBase,
                StructurePiecesCollector, StructurePosition, WorldPortalExt,
            },
        },
    },
};

#[derive(Deserialize)]
pub struct SwampHutGenerator;

impl StructureGenerator for SwampHutGenerator {
    fn get_structure_position(
        &self,
        mut context: StructureGeneratorContext<'_>,
    ) -> Option<StructurePosition> {
        let x = start_block_x(context.chunk_x);
        let z = start_block_z(context.chunk_z);

        let facing = BlockDirection::get_random_horizontal_direction(&mut context.random);
        let mut shiftable_piece = ShiftableStructurePiece::new(
            StructurePieceType::SwampHut,
            x,
            64,
            z,
            7,
            7,
            9,
            facing.get_axis(),
        );
        shiftable_piece.piece.set_facing(Some(facing));

        let mut collector = StructurePiecesCollector::default();
        collector.add_piece(Box::new(SwampHutPiece {
            shiftable_structure_piece: shiftable_piece,
            spawned_witch: false,
            spawned_cat: false,
        }));

        Some(StructurePosition {
            start_pos: BlockPos::new(x, 64, z),
            collector: Arc::new(collector.into()),
        })
    }
}

pub struct SwampHutPiece {
    shiftable_structure_piece: ShiftableStructurePiece,
    spawned_witch: bool,
    spawned_cat: bool,
}

impl SwampHutPiece {
    fn spruce_stairs(facing: HorizontalFacing, shape: StairsShape) -> &'static BlockState {
        let mut props = OakStairsLikeProperties::default(&Block::SPRUCE_STAIRS);
        props.facing = facing;
        props.shape = shape;
        BlockState::from_id(props.to_state_id(&Block::SPRUCE_STAIRS))
    }

    fn spawn_witch(&mut self, chunk: &mut ProtoChunk, chunk_box: &BlockBox) {
        if !self.spawned_witch {
            let pos = self.shiftable_structure_piece.piece.offset_pos(2, 2, 5);
            if chunk_box.contains_pos(&pos) {
                self.spawned_witch = true;
                let mut nbt = NbtCompound::new();
                nbt.put_string("id", "minecraft:witch".to_string());
                nbt.put_list(
                    "Pos",
                    vec![
                        (f64::from(pos.x) + 0.5).into(),
                        f64::from(pos.y).into(),
                        (f64::from(pos.z) + 0.5).into(),
                    ],
                );
                nbt.put_bool("PersistenceRequired", true);
                chunk.add_structure_entity(nbt);
            }
        }
    }

    fn spawn_cat(&mut self, chunk: &mut ProtoChunk, chunk_box: &BlockBox) {
        if !self.spawned_cat {
            let pos = self.shiftable_structure_piece.piece.offset_pos(2, 2, 5);
            if chunk_box.contains_pos(&pos) {
                self.spawned_cat = true;
                let mut nbt = NbtCompound::new();
                nbt.put_string("id", "minecraft:cat".to_string());
                nbt.put_list(
                    "Pos",
                    vec![
                        (f64::from(pos.x) + 0.5).into(),
                        f64::from(pos.y).into(),
                        (f64::from(pos.z) + 0.5).into(),
                    ],
                );
                nbt.put_bool("PersistenceRequired", true);
                nbt.put_string("variant", "minecraft:all_black".to_string());
                chunk.add_structure_entity(nbt);
            }
        }
    }
}

impl StructurePieceBase for SwampHutPiece {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    #[allow(clippy::too_many_lines)]
    fn place(
        &mut self,
        chunk: &mut ProtoChunk,
        block_registry: &dyn WorldPortalExt,
        _random: &mut RandomGenerator,
        _seed: i64,
        chunk_box: &BlockBox,
    ) {
        if !self
            .shiftable_structure_piece
            .adjust_to_average_height(chunk)
        {
            return;
        }

        let box_limit = *chunk_box;
        let p = &self.shiftable_structure_piece.piece;

        let spruce_planks = Block::SPRUCE_PLANKS.default_state;
        let oak_log = Block::OAK_LOG.default_state;
        let oak_fence = Block::OAK_FENCE.default_state;
        let air = Block::AIR.default_state;

        p.fill_with_outline(
            chunk,
            &box_limit,
            false,
            1,
            1,
            1,
            5,
            1,
            7,
            spruce_planks,
            spruce_planks,
        );
        p.fill_with_outline(
            chunk,
            &box_limit,
            false,
            1,
            4,
            2,
            5,
            4,
            7,
            spruce_planks,
            spruce_planks,
        );
        p.fill_with_outline(
            chunk,
            &box_limit,
            false,
            2,
            1,
            0,
            4,
            1,
            0,
            spruce_planks,
            spruce_planks,
        );

        p.fill_with_outline(
            chunk,
            &box_limit,
            false,
            2,
            2,
            2,
            3,
            3,
            2,
            spruce_planks,
            spruce_planks,
        );
        p.fill_with_outline(
            chunk,
            &box_limit,
            false,
            1,
            2,
            3,
            1,
            3,
            6,
            spruce_planks,
            spruce_planks,
        );
        p.fill_with_outline(
            chunk,
            &box_limit,
            false,
            5,
            2,
            3,
            5,
            3,
            6,
            spruce_planks,
            spruce_planks,
        );
        p.fill_with_outline(
            chunk,
            &box_limit,
            false,
            2,
            2,
            7,
            4,
            3,
            7,
            spruce_planks,
            spruce_planks,
        );

        p.fill_with_outline(chunk, &box_limit, false, 1, 0, 2, 1, 3, 2, oak_log, oak_log);
        p.fill_with_outline(chunk, &box_limit, false, 5, 0, 2, 5, 3, 2, oak_log, oak_log);
        p.fill_with_outline(chunk, &box_limit, false, 1, 0, 7, 1, 3, 7, oak_log, oak_log);
        p.fill_with_outline(chunk, &box_limit, false, 5, 0, 7, 5, 3, 7, oak_log, oak_log);

        p.add_block(chunk, oak_fence, 2, 3, 2, &box_limit);
        p.add_block(chunk, oak_fence, 3, 3, 7, &box_limit);
        p.add_block(chunk, air, 1, 3, 4, &box_limit);
        p.add_block(chunk, air, 5, 3, 4, &box_limit);
        p.add_block(chunk, air, 5, 3, 5, &box_limit);
        p.add_block(
            chunk,
            Block::POTTED_RED_MUSHROOM.default_state,
            1,
            3,
            5,
            &box_limit,
        );
        p.add_block(
            chunk,
            Block::CRAFTING_TABLE.default_state,
            3,
            2,
            6,
            &box_limit,
        );
        p.add_block(chunk, Block::CAULDRON.default_state, 4, 2, 6, &box_limit);
        p.add_block(chunk, oak_fence, 1, 2, 1, &box_limit);
        p.add_block(chunk, oak_fence, 5, 2, 1, &box_limit);

        let stairs_n = Self::spruce_stairs(HorizontalFacing::North, StairsShape::Straight);
        let stairs_e = Self::spruce_stairs(HorizontalFacing::East, StairsShape::Straight);
        let stairs_w = Self::spruce_stairs(HorizontalFacing::West, StairsShape::Straight);
        let stairs_s = Self::spruce_stairs(HorizontalFacing::South, StairsShape::Straight);

        p.fill_with_outline(
            chunk, &box_limit, false, 0, 4, 1, 6, 4, 1, stairs_n, stairs_n,
        );
        p.fill_with_outline(
            chunk, &box_limit, false, 0, 4, 2, 0, 4, 7, stairs_e, stairs_e,
        );
        p.fill_with_outline(
            chunk, &box_limit, false, 6, 4, 2, 6, 4, 7, stairs_w, stairs_w,
        );
        p.fill_with_outline(
            chunk, &box_limit, false, 0, 4, 8, 6, 4, 8, stairs_s, stairs_s,
        );

        p.place_block(
            chunk,
            block_registry,
            Self::spruce_stairs(HorizontalFacing::North, StairsShape::OuterRight),
            0,
            4,
            1,
            &box_limit,
        );
        p.place_block(
            chunk,
            block_registry,
            Self::spruce_stairs(HorizontalFacing::North, StairsShape::OuterLeft),
            6,
            4,
            1,
            &box_limit,
        );
        p.place_block(
            chunk,
            block_registry,
            Self::spruce_stairs(HorizontalFacing::South, StairsShape::OuterLeft),
            0,
            4,
            8,
            &box_limit,
        );
        p.place_block(
            chunk,
            block_registry,
            Self::spruce_stairs(HorizontalFacing::South, StairsShape::OuterRight),
            6,
            4,
            8,
            &box_limit,
        );

        for i in [2, 7] {
            for j in [1, 5] {
                p.fill_downwards(chunk, oak_log, j, -1, i, &box_limit);
            }
        }

        self.spawn_witch(chunk, &box_limit);
        self.spawn_cat(chunk, &box_limit);
    }

    fn get_structure_piece(&self) -> &super::StructurePiece {
        &self.shiftable_structure_piece.piece
    }

    fn get_structure_piece_mut(&mut self) -> &mut super::StructurePiece {
        &mut self.shiftable_structure_piece.piece
    }
}
