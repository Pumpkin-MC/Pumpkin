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

/// Small 3-way crossing for the exterior bridge network (7 × 9 × 7).
#[derive(Clone)]
pub struct BridgeSmallCrossingPiece {
    pub piece: NetherFortressPiece,
}

impl BridgeSmallCrossingPiece {
    pub fn create(
        collector: &mut StructurePiecesCollector,
        x: i32,
        y: i32,
        z: i32,
        orientation: BlockDirection,
        chain_length: u32,
    ) -> Option<Box<dyn StructurePieceBase>> {
        let bbox = BlockBox::rotated(x, y, z, -2, 0, 0, 7, 9, 7, &orientation);
        if !NetherFortressPiece::is_in_bounds(&bbox) || collector.get_intersecting(&bbox).is_some()
        {
            return None;
        }

        let mut piece = NetherFortressPiece::new(
            StructurePieceType::NetherFortressBridgeSmallCrossing,
            chain_length,
            bbox,
        );
        piece.piece.set_facing(Some(orientation));
        Some(Box::new(Self { piece }))
    }
}

impl StructurePieceBase for BridgeSmallCrossingPiece {
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
            2,
            0,
            false,
            pieces_to_process,
        );
        self.piece.fill_nw_opening(
            start,
            collector,
            random,
            bridge_pieces,
            corridor_pieces,
            &mut last,
            0,
            2,
            false,
            pieces_to_process,
        );
        self.piece.fill_se_opening(
            start,
            collector,
            random,
            bridge_pieces,
            corridor_pieces,
            &mut last,
            0,
            2,
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

        p.fill_with_outline(chunk, &bb, false, 0, 0, 0, 6, 1, 6, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 0, 2, 0, 6, 7, 6, air, air);

        // Walls
        p.fill_with_outline(chunk, &bb, false, 0, 2, 0, 1, 6, 0, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 0, 2, 6, 1, 6, 6, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 5, 2, 0, 6, 6, 0, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 5, 2, 6, 6, 6, 6, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 0, 2, 0, 0, 6, 1, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 0, 2, 5, 0, 6, 6, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 6, 2, 0, 6, 6, 1, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 6, 2, 5, 6, 6, 6, nb, nb);

        // Fence helpers
        let fence = |west: bool, east: bool, north: bool, south: bool| {
            let mut props = OakFenceLikeProperties::default(&Block::NETHER_BRICK_FENCE);
            props.west = west;
            props.east = east;
            props.north = north;
            props.south = south;
            pumpkin_data::BlockState::from_id(props.to_state_id(&Block::NETHER_BRICK_FENCE))
        };
        let f_ew = fence(true, true, false, false);
        let f_ns = fence(false, false, true, true);

        // Battlements: top of the four open sides
        p.fill_with_outline(chunk, &bb, false, 2, 6, 0, 4, 6, 0, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 2, 5, 0, 4, 5, 0, f_ew, f_ew);
        p.fill_with_outline(chunk, &bb, false, 2, 6, 6, 4, 6, 6, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 2, 5, 6, 4, 5, 6, f_ew, f_ew);
        p.fill_with_outline(chunk, &bb, false, 0, 6, 2, 0, 6, 4, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 0, 5, 2, 0, 5, 4, f_ns, f_ns);
        p.fill_with_outline(chunk, &bb, false, 6, 6, 2, 6, 6, 4, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 6, 5, 2, 6, 5, 4, f_ns, f_ns);

        for i in 0..=6i32 {
            for j in 0..=6i32 {
                p.fill_downwards(chunk, nb, i, -1, j, &bb);
            }
        }
    }

    fn clone_box(&self) -> Box<dyn StructurePieceBase> {
        Box::new(self.clone())
    }
}
