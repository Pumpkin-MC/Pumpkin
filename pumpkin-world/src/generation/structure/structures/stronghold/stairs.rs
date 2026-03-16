use pumpkin_data::{
    Block, BlockState,
    block_properties::{BlockProperties, HorizontalFacing, OakStairsLikeProperties},
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
pub struct StairsPiece {
    pub piece: StrongholdPiece,
}

impl StairsPiece {
    pub fn create(
        collector: &mut StructurePiecesCollector,
        random: &mut impl RandomImpl,
        x: i32,
        y: i32,
        z: i32,
        orientation: BlockDirection,
        chain_length: u32,
    ) -> Option<Box<dyn StructurePieceBase>> {
        let bounding_box = BlockBox::rotated(x, y, z, -1, -7, 0, 5, 11, 8, &orientation);

        if !StrongholdPiece::is_in_bounds(&bounding_box)
            || collector.get_intersecting(&bounding_box).is_some()
        {
            return None;
        }

        let mut piece = StrongholdPiece::new(
            StructurePieceType::StrongholdStairs,
            chain_length,
            bounding_box,
        );
        piece.piece.set_facing(Some(orientation));
        piece.entry_door = EntranceType::get_random(random);

        Some(Box::new(Self { piece }))
    }
}

impl StructurePieceBase for StairsPiece {
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
        self.piece.fill_forward_opening(
            start,
            collector,
            random,
            weights,
            last_piece_type,
            1,
            1,
            pieces_to_process,
            None,
        );
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

        // 1. Outer Box 5x11x8
        inner.fill_outline_random(
            0,
            0,
            0,
            4,
            10,
            7,
            &randomizer,
            chunk,
            true,
            random,
            &box_limit,
        );

        // 2. Entrance (Top)
        p.generate_entrance(chunk, &box_limit, self.piece.entry_door, 1, 7, 0);

        // 3. Exit (Bottom - Opening only)
        p.generate_entrance(chunk, &box_limit, EntranceType::Opening, 1, 1, 7);

        // 4. Stairs Generation
        let mut props = OakStairsLikeProperties::default(&Block::COBBLESTONE_STAIRS);
        props.facing = HorizontalFacing::South;
        let stair_state = BlockState::from_id(props.to_state_id(&Block::COBBLESTONE_STAIRS));

        let stone_bricks = Block::STONE_BRICKS.default_state;

        for i in 0..6 {
            // Place the stair blocks
            inner.add_block(chunk, stair_state, 1, 6 - i, 1 + i, &box_limit);
            inner.add_block(chunk, stair_state, 2, 6 - i, 1 + i, &box_limit);
            inner.add_block(chunk, stair_state, 3, 6 - i, 1 + i, &box_limit);

            // Fill underneath the stairs for stability
            if i < 5 {
                inner.add_block(chunk, stone_bricks, 1, 5 - i, 1 + i, &box_limit);
                inner.add_block(chunk, stone_bricks, 2, 5 - i, 1 + i, &box_limit);
                inner.add_block(chunk, stone_bricks, 3, 5 - i, 1 + i, &box_limit);
            }
        }
    }

    fn clone_box(&self) -> Box<dyn StructurePieceBase> {
        Box::new((*self).clone())
    }
}
