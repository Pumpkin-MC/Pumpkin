use pumpkin_data::{
    Block, BlockState,
    block_properties::{BlockProperties, ResinBrickSlabLikeProperties, SlabType},
};
use pumpkin_util::{
    BlockDirection,
    math::block_box::BlockBox,
    random::{RandomGenerator, RandomImpl},
};

use crate::{
    ProtoChunk,
    generation::structure::{
        piece::StructurePieceType,
        structures::{
            StructurePiece, StructurePieceBase, StructurePiecesCollector,
            stronghold::{
                EntranceType, PieceWeight, StoneBrickRandomizer, StrongholdPiece,
                StrongholdPieceType,
            },
        },
    },
};

#[derive(Clone)]
pub struct FiveWayCrossingPiece {
    pub piece: StrongholdPiece,
    pub lower_left_exists: bool,
    pub upper_left_exists: bool,
    pub lower_right_exists: bool,
    pub upper_right_exists: bool,
}

impl FiveWayCrossingPiece {
    pub fn create(
        collector: &mut StructurePiecesCollector,
        random: &mut impl RandomImpl,
        x: i32,
        y: i32,
        z: i32,
        orientation: BlockDirection,
        chain_length: u32,
    ) -> Option<Box<dyn StructurePieceBase>> {
        let bounding_box = BlockBox::rotated(x, y, z, -4, -3, 0, 10, 9, 11, &orientation);

        if !StrongholdPiece::is_in_bounds(&bounding_box)
            || collector.get_intersecting(&bounding_box).is_some()
        {
            return None;
        }

        let mut piece = StrongholdPiece::new(
            StructurePieceType::StrongholdFiveWayCrossing,
            chain_length,
            bounding_box,
        );
        piece.piece.set_facing(Some(orientation));
        piece.entry_door = EntranceType::get_random(random);

        Some(Box::new(Self {
            piece,
            lower_left_exists: random.next_bool(),
            upper_left_exists: random.next_bool(),
            lower_right_exists: random.next_bool(),
            upper_right_exists: random.next_bounded_i32(3) > 0,
        }))
    }
}

impl StructurePieceBase for FiveWayCrossingPiece {
    fn get_structure_piece(&self) -> &StructurePiece {
        &self.piece.piece
    }

    fn get_structure_piece_mut(&mut self) -> &mut StructurePiece {
        &mut self.piece.piece
    }

    fn fill_openings(
        &self,
        start: &StructurePiece,
        random: &mut RandomGenerator,
        weights: &mut Vec<PieceWeight>,
        last_piece_type: &mut Option<StrongholdPieceType>,
        _has_portal_room: &mut bool,

        collector: &mut StructurePiecesCollector,
        pieces_to_process: &mut Vec<Box<dyn StructurePieceBase>>,
    ) {
        let mut i = 3;
        let mut j = 5;
        let facing = self.piece.piece.facing.unwrap_or(BlockDirection::North);

        if facing == BlockDirection::West || facing == BlockDirection::North {
            i = 8 - i;
            j = 8 - j;
        }

        self.piece.fill_forward_opening(
            start,
            collector,
            random,
            weights,
            last_piece_type,
            5,
            1,
            pieces_to_process,
            None,
        );

        if self.lower_left_exists {
            self.piece.fill_nw_opening(
                start,
                collector,
                random,
                weights,
                last_piece_type,
                i,
                1,
                pieces_to_process,
            );
        }
        if self.upper_left_exists {
            self.piece.fill_nw_opening(
                start,
                collector,
                random,
                weights,
                last_piece_type,
                j,
                7,
                pieces_to_process,
            );
        }

        if self.lower_right_exists {
            self.piece.fill_se_opening(
                start,
                collector,
                random,
                weights,
                last_piece_type,
                i,
                1,
                pieces_to_process,
            );
        }
        if self.upper_right_exists {
            self.piece.fill_se_opening(
                start,
                collector,
                random,
                weights,
                last_piece_type,
                j,
                7,
                pieces_to_process,
            );
        }
    }

    fn place(
        &mut self,
        chunk: &mut ProtoChunk,
        random: &mut RandomGenerator,
        _seed: i64,
        chunk_box: &BlockBox,
    ) {
        let randomizer = StoneBrickRandomizer;
        let box_limit = *chunk_box;
        let p = &self.piece;
        let inner = &p.piece;
        let air = Block::AIR.default_state;

        inner.fill_outline_random(
            0,
            0,
            0,
            9,
            8,
            10,
            &randomizer,
            chunk,
            true,
            random,
            &box_limit,
        );

        p.generate_entrance(chunk, &box_limit, self.piece.entry_door, 4, 3, 0);

        if self.lower_left_exists {
            inner.fill_with_outline(chunk, &box_limit, false, 0, 3, 1, 0, 5, 3, air, air);
        }
        if self.lower_right_exists {
            inner.fill_with_outline(chunk, &box_limit, false, 9, 3, 1, 9, 5, 3, air, air);
        }
        if self.upper_left_exists {
            inner.fill_with_outline(chunk, &box_limit, false, 0, 5, 7, 0, 7, 9, air, air);
        }
        if self.upper_right_exists {
            inner.fill_with_outline(chunk, &box_limit, false, 9, 5, 7, 9, 7, 9, air, air);
        }

        inner.fill_with_outline(chunk, &box_limit, false, 5, 1, 10, 7, 3, 10, air, air);

        inner.fill_outline_random(
            1,
            2,
            1,
            8,
            2,
            6,
            &randomizer,
            chunk,
            false,
            random,
            &box_limit,
        );
        inner.fill_outline_random(
            4,
            1,
            5,
            4,
            4,
            9,
            &randomizer,
            chunk,
            false,
            random,
            &box_limit,
        );
        inner.fill_outline_random(
            8,
            1,
            5,
            8,
            4,
            9,
            &randomizer,
            chunk,
            false,
            random,
            &box_limit,
        );
        inner.fill_outline_random(
            1,
            4,
            7,
            3,
            4,
            9,
            &randomizer,
            chunk,
            false,
            random,
            &box_limit,
        );
        inner.fill_outline_random(
            1,
            3,
            5,
            3,
            3,
            6,
            &randomizer,
            chunk,
            false,
            random,
            &box_limit,
        );

        let slab = Block::SMOOTH_STONE_SLAB.default_state;

        inner.fill_with_outline(chunk, &box_limit, false, 1, 3, 4, 3, 3, 4, slab, slab);
        inner.fill_with_outline(chunk, &box_limit, false, 1, 4, 6, 3, 4, 6, slab, slab);

        inner.fill_outline_random(
            5,
            1,
            7,
            7,
            1,
            8,
            &randomizer,
            chunk,
            false,
            random,
            &box_limit,
        );
        inner.fill_with_outline(chunk, &box_limit, false, 5, 1, 9, 7, 1, 9, slab, slab);
        inner.fill_with_outline(chunk, &box_limit, false, 5, 2, 7, 7, 2, 7, slab, slab);

        inner.fill_with_outline(chunk, &box_limit, false, 4, 5, 7, 4, 5, 9, slab, slab);
        inner.fill_with_outline(chunk, &box_limit, false, 8, 5, 7, 8, 5, 9, slab, slab);

        let mut props = ResinBrickSlabLikeProperties::default(&Block::SMOOTH_STONE_SLAB);
        props.r#type = SlabType::Double;
        let double_slab = BlockState::from_id(props.to_state_id(&Block::SMOOTH_STONE_SLAB));
        inner.fill_with_outline(
            chunk,
            &box_limit,
            false,
            5,
            5,
            7,
            7,
            5,
            9,
            double_slab,
            double_slab,
        );

        // let torch = Block::WALL_TORCH.default_state.with("facing", "south");
        // inner.add_block(chunk, &torch, 6, 5, 6, &box_limit);
    }

    fn clone_box(&self) -> Box<dyn StructurePieceBase> {
        Box::new((*self).clone())
    }
}
