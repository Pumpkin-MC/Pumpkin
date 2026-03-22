use pumpkin_data::{
    Block,
    block_properties::{BlockProperties, OakFenceLikeProperties},
};
use pumpkin_util::{BlockDirection, math::block_box::BlockBox, random::RandomGenerator};

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

/// Straight interior corridor segment (5 × 7 × 5).
/// This is also the fallback piece returned by the generation logic.
#[derive(Clone)]
pub struct SmallCorridorPiece {
    pub piece: NetherFortressPiece,
}

impl SmallCorridorPiece {
    pub fn create(
        collector: &mut StructurePiecesCollector,
        x: i32,
        y: i32,
        z: i32,
        orientation: BlockDirection,
        chain_length: u32,
    ) -> Option<Box<dyn StructurePieceBase>> {
        let bbox = BlockBox::rotated(x, y, z, -1, 0, 0, 5, 7, 5, &orientation);
        if !NetherFortressPiece::is_in_bounds(&bbox) || collector.get_intersecting(&bbox).is_some()
        {
            return None;
        }

        let mut piece = NetherFortressPiece::new(
            StructurePieceType::NetherFortressSmallCorridor,
            chain_length,
            bbox,
        );
        piece.piece.set_facing(Some(orientation));
        Some(Box::new(Self { piece }))
    }
}

impl StructurePieceBase for SmallCorridorPiece {
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
        let mut last: Option<NetherFortressPieceType> = None;
        self.piece.fill_forward_opening(
            start,
            collector,
            random,
            bridge_pieces,
            corridor_pieces,
            &mut last,
            1,
            0,
            true,
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

        let fence = |north: bool, south: bool| {
            let mut props = OakFenceLikeProperties::default(&Block::NETHER_BRICK_FENCE);
            props.north = north;
            props.south = south;
            pumpkin_data::BlockState::from_id(props.to_state_id(&Block::NETHER_BRICK_FENCE))
        };
        let f_ns = fence(true, true);

        p.fill_with_outline(chunk, &bb, false, 0, 0, 0, 4, 1, 4, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 0, 2, 0, 4, 5, 4, air, air);

        // Side walls
        p.fill_with_outline(chunk, &bb, false, 0, 2, 0, 0, 5, 4, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 4, 2, 0, 4, 5, 4, nb, nb);

        // Fence railings on side walls
        p.fill_with_outline(chunk, &bb, false, 0, 3, 1, 0, 4, 1, f_ns, f_ns);
        p.fill_with_outline(chunk, &bb, false, 0, 3, 3, 0, 4, 3, f_ns, f_ns);
        p.fill_with_outline(chunk, &bb, false, 4, 3, 1, 4, 4, 1, f_ns, f_ns);
        p.fill_with_outline(chunk, &bb, false, 4, 3, 3, 4, 4, 3, f_ns, f_ns);

        // Ceiling
        p.fill_with_outline(chunk, &bb, false, 0, 6, 0, 4, 6, 4, nb, nb);

        for i in 0..=4i32 {
            for j in 0..=4i32 {
                p.fill_downwards(chunk, nb, i, -1, j, &bb);
            }
        }
    }

    fn clone_box(&self) -> Box<dyn StructurePieceBase> {
        Box::new(self.clone())
    }
}
