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
            nether_fortress::{NetherFortressPiece, NetherFortressPieceType, PieceWeight},
        },
    },
};

/// 4-way exterior bridge crossing (19 × 10 × 19).
/// Also serves as the generation start piece.
#[derive(Clone)]
pub struct BridgeCrossingPiece {
    pub piece: NetherFortressPiece,
}

impl BridgeCrossingPiece {
    /// Create as the very first (start) piece at a fixed y=64.
    pub fn new_start(random: &mut impl RandomImpl, x: i32, z: i32) -> Self {
        let orientation = BlockDirection::get_random_horizontal_direction(random);
        let bbox = BlockBox::create_box(x, 64, z, orientation.get_axis(), 19, 10, 19);
        let mut piece =
            NetherFortressPiece::new(StructurePieceType::NetherFortressBridgeCrossing, 0, bbox);
        piece.piece.set_facing(Some(orientation));
        Self { piece }
    }

    pub fn create(
        collector: &mut StructurePiecesCollector,
        x: i32,
        y: i32,
        z: i32,
        orientation: BlockDirection,
        chain_length: u32,
    ) -> Option<Box<dyn StructurePieceBase>> {
        let bbox = BlockBox::rotated(x, y, z, -8, -3, 0, 19, 10, 19, &orientation);
        if !NetherFortressPiece::is_in_bounds(&bbox) || collector.get_intersecting(&bbox).is_some()
        {
            return None;
        }

        let mut piece = NetherFortressPiece::new(
            StructurePieceType::NetherFortressBridgeCrossing,
            chain_length,
            bbox,
        );
        piece.piece.set_facing(Some(orientation));
        Some(Box::new(Self { piece }))
    }

    pub fn fill_openings(
        &self,
        start: &StructurePiece,
        random: &mut impl RandomImpl,
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
            8,
            3,
            false,
            pieces_to_process,
        );
        self.piece.fill_nw_opening(
            start,
            collector,
            random,
            bridge_pieces,
            corridor_pieces,
            &mut last_piece,
            3,
            8,
            false,
            pieces_to_process,
        );
        self.piece.fill_se_opening(
            start,
            collector,
            random,
            bridge_pieces,
            corridor_pieces,
            &mut last_piece,
            3,
            8,
            false,
            pieces_to_process,
        );
    }
}

impl StructurePieceBase for BridgeCrossingPiece {
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
        self.fill_openings(
            start,
            random,
            bridge_pieces,
            corridor_pieces,
            collector,
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

        p.fill_with_outline(chunk, &bb, false, 7, 3, 0, 11, 4, 18, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 0, 3, 7, 18, 4, 11, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 8, 5, 0, 10, 7, 18, air, air);
        p.fill_with_outline(chunk, &bb, false, 0, 5, 8, 18, 7, 10, air, air);
        p.fill_with_outline(chunk, &bb, false, 7, 5, 0, 7, 5, 7, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 7, 5, 11, 7, 5, 18, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 11, 5, 0, 11, 5, 7, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 11, 5, 11, 11, 5, 18, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 0, 5, 7, 7, 5, 7, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 11, 5, 7, 18, 5, 7, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 0, 5, 11, 7, 5, 11, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 11, 5, 11, 18, 5, 11, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 7, 2, 0, 11, 2, 5, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 7, 2, 13, 11, 2, 18, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 7, 0, 0, 11, 1, 3, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 7, 0, 15, 11, 1, 18, nb, nb);

        for i in 7..=11i32 {
            for j in 0..=2i32 {
                p.fill_downwards(chunk, nb, i, -1, j, &bb);
                p.fill_downwards(chunk, nb, i, -1, 18 - j, &bb);
            }
        }

        p.fill_with_outline(chunk, &bb, false, 0, 2, 7, 5, 2, 11, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 13, 2, 7, 18, 2, 11, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 0, 0, 7, 3, 1, 11, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 15, 0, 7, 18, 1, 11, nb, nb);

        for i in 0..=2i32 {
            for j in 7..=11i32 {
                p.fill_downwards(chunk, nb, i, -1, j, &bb);
                p.fill_downwards(chunk, nb, i, -1, 18 - j, &bb); // Note: Java uses `18 - i` but iterates j; faithful copy
            }
        }
    }

    fn clone_box(&self) -> Box<dyn StructurePieceBase> {
        Box::new(self.clone())
    }
}
