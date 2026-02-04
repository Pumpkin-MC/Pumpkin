use pumpkin_data::Block;
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
pub struct LeftTurnPiece {
    pub piece: StrongholdPiece,
}

impl LeftTurnPiece {
    pub fn create(
        collector: &mut StructurePiecesCollector,
        random: &mut impl RandomImpl,
        x: i32,
        y: i32,
        z: i32,
        orientation: BlockDirection,
        chain_length: u32,
    ) -> Option<Box<dyn StructurePieceBase>> {
        let bounding_box = BlockBox::rotated(x, y, z, -1, -1, 0, 5, 5, 5, &orientation);

        if !StrongholdPiece::is_in_bounds(&bounding_box)
            || collector.get_intersecting(&bounding_box).is_some()
        {
            return None;
        }

        let mut piece = StrongholdPiece::new(
            StructurePieceType::StrongholdLeftTurn,
            chain_length,
            bounding_box,
        );
        piece.piece.set_facing(Some(orientation));
        piece.entry_door = EntranceType::get_random(random);

        Some(Box::new(Self { piece }))
    }
}

impl StructurePieceBase for LeftTurnPiece {
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
        let facing = self.piece.piece.facing.unwrap_or(BlockDirection::North);

        if facing == BlockDirection::North || facing == BlockDirection::East {
            self.piece.fill_nw_opening(
                start,
                collector,
                random,
                weights,
                last_piece_type,
                1,
                1,
                pieces_to_process,
            );
        } else {
            self.piece.fill_se_opening(
                start,
                collector,
                random,
                weights,
                last_piece_type,
                1,
                1,
                pieces_to_process,
            );
        }
    }

    fn place(&mut self, chunk: &mut ProtoChunk, random: &mut RandomGenerator, _seed: i64) {
        let randomizer = StoneBrickRandomizer;
        let box_limit = self.piece.piece.bounding_box;
        let p = &self.piece;
        let inner = &p.piece;
        let air = Block::AIR.default_state;

        inner.fill_outline_random(
            0,
            0,
            0,
            4,
            4,
            4,
            &randomizer,
            chunk,
            true,
            random,
            &box_limit,
        );

        p.generate_entrance(chunk, &box_limit, self.piece.entry_door, 1, 1, 0);

        let direction = inner.facing.unwrap_or(BlockDirection::North);

        if direction == BlockDirection::North || direction == BlockDirection::East {
            inner.fill_with_outline(chunk, &box_limit, false, 0, 1, 1, 0, 3, 3, air, air);
        } else {
            inner.fill_with_outline(chunk, &box_limit, false, 4, 1, 1, 4, 3, 3, air, air);
        }
    }

    fn clone_box(&self) -> Box<dyn StructurePieceBase> {
        Box::new((*self).clone())
    }
}
