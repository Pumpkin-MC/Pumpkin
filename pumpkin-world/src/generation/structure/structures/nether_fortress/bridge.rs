use pumpkin_data::{
    Block,
    block_properties::{BlockProperties, OakFenceLikeProperties},
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
            nether_fortress::{NetherFortressPiece, NetherFortressPieceType, PieceWeight},
        },
    },
};

/// Straight exterior bridge segment (5 × 10 × 19).
#[derive(Clone)]
pub struct BridgePiece {
    pub piece: NetherFortressPiece,
}

impl BridgePiece {
    pub fn create(
        collector: &mut StructurePiecesCollector,
        _random: &mut impl RandomImpl,
        x: i32,
        y: i32,
        z: i32,
        orientation: BlockDirection,
        chain_length: u32,
    ) -> Option<Box<dyn StructurePieceBase>> {
        let bbox = BlockBox::rotated(x, y, z, -1, -3, 0, 5, 10, 19, &orientation);
        if !NetherFortressPiece::is_in_bounds(&bbox) || collector.get_intersecting(&bbox).is_some()
        {
            return None;
        }

        let mut piece =
            NetherFortressPiece::new(StructurePieceType::NetherFortressBridge, chain_length, bbox);
        piece.piece.set_facing(Some(orientation));

        Some(Box::new(Self { piece }))
    }
}

impl StructurePieceBase for BridgePiece {
    fn get_structure_piece(&self) -> &StructurePiece {
        &self.piece.piece
    }

    fn get_structure_piece_mut(&mut self) -> &mut StructurePiece {
        &mut self.piece.piece
    }

    fn fill_openings_nether(
        &self,
        start: &StructurePiece,
        random: &mut RandomGenerator,
        bridge_pieces: &mut Vec<PieceWeight>,
        corridor_pieces: &mut Vec<PieceWeight>,
        collector: &mut StructurePiecesCollector,
        pieces_to_process: &mut Vec<Box<dyn StructurePieceBase>>,
    ) {
        let mut last_piece: Option<NetherFortressPieceType> = None;
        self.piece.fill_forward_opening(
            start,
            collector,
            random,
            bridge_pieces,
            corridor_pieces,
            &mut last_piece,
            1,
            3,
            false,
            pieces_to_process,
        );
    }

    fn place(
        &mut self,
        chunk: &mut ProtoChunk,
        _random: &mut RandomGenerator,
        _seed: i64,
        chunk_box: &BlockBox,
    ) {
        let bb = *chunk_box;
        let p = &self.piece.piece;

        let nb = Block::NETHER_BRICKS.default_state;
        let air = Block::AIR.default_state;

        p.fill_with_outline(chunk, &bb, false, 0, 3, 0, 4, 4, 18, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 1, 5, 0, 3, 7, 18, air, air);
        p.fill_with_outline(chunk, &bb, false, 0, 5, 0, 0, 5, 18, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 4, 5, 0, 4, 5, 18, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 0, 2, 0, 4, 2, 5, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 0, 2, 13, 4, 2, 18, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 0, 0, 0, 4, 1, 3, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 0, 0, 15, 4, 1, 18, nb, nb);

        for i in 0..=4i32 {
            for j in 0..=2i32 {
                p.fill_downwards(chunk, nb, i, -1, j, &bb);
                p.fill_downwards(chunk, nb, i, -1, 18 - j, &bb);
            }
        }

        // North-South fence (connects along X axis on left/right walls)
        let mut props = OakFenceLikeProperties::default(&Block::NETHER_BRICK_FENCE);
        props.north = true;
        props.south = true;
        let _fence_ns =
            pumpkin_data::BlockState::from_id(props.to_state_id(&Block::NETHER_BRICK_FENCE));

        // fence_ns + east (left wall)
        props.east = true;
        props.north = true;
        props.south = true;
        let fence_east =
            pumpkin_data::BlockState::from_id(props.to_state_id(&Block::NETHER_BRICK_FENCE));

        // fence_ns + west (right wall)
        props.east = false;
        props.west = true;
        let fence_west =
            pumpkin_data::BlockState::from_id(props.to_state_id(&Block::NETHER_BRICK_FENCE));

        p.fill_with_outline(chunk, &bb, false, 0, 1, 1, 0, 4, 1, fence_east, fence_east);
        p.fill_with_outline(chunk, &bb, false, 0, 3, 4, 0, 4, 4, fence_east, fence_east);
        p.fill_with_outline(
            chunk, &bb, false, 0, 3, 14, 0, 4, 14, fence_east, fence_east,
        );
        p.fill_with_outline(
            chunk, &bb, false, 0, 1, 17, 0, 4, 17, fence_east, fence_east,
        );
        p.fill_with_outline(chunk, &bb, false, 4, 1, 1, 4, 4, 1, fence_west, fence_west);
        p.fill_with_outline(chunk, &bb, false, 4, 3, 4, 4, 4, 4, fence_west, fence_west);
        p.fill_with_outline(
            chunk, &bb, false, 4, 3, 14, 4, 4, 14, fence_west, fence_west,
        );
        p.fill_with_outline(
            chunk, &bb, false, 4, 1, 17, 4, 4, 17, fence_west, fence_west,
        );
    }

    fn clone_box(&self) -> Box<dyn StructurePieceBase> {
        Box::new(self.clone())
    }
}
