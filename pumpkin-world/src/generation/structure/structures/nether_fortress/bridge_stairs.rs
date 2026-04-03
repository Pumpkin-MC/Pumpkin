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

/// Stairway that transitions from bridge level to interior level (7 × 11 × 7).
#[derive(Clone)]
pub struct BridgeStairsPiece {
    pub piece: NetherFortressPiece,
}

impl BridgeStairsPiece {
    pub fn create(
        collector: &mut StructurePiecesCollector,
        x: i32,
        y: i32,
        z: i32,
        chain_length: u32,
        orientation: BlockDirection,
    ) -> Option<Box<dyn StructurePieceBase>> {
        let bbox = BlockBox::rotated(x, y, z, -2, 0, 0, 7, 11, 7, &orientation);
        if !NetherFortressPiece::is_in_bounds(&bbox) || collector.get_intersecting(&bbox).is_some()
        {
            return None;
        }

        let mut piece = NetherFortressPiece::new(
            StructurePieceType::NetherFortressBridgeStairs,
            chain_length,
            bbox,
        );
        piece.piece.set_facing(Some(orientation));
        Some(Box::new(Self { piece }))
    }
}

impl StructurePieceBase for BridgeStairsPiece {
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
        // SE opening connects into the corridor network (inside = true)
        self.piece.fill_se_opening(
            start,
            collector,
            random,
            bridge_pieces,
            corridor_pieces,
            &mut last,
            6,
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
        p.fill_with_outline(chunk, &bb, false, 0, 2, 0, 6, 10, 6, air, air);

        // Walls
        p.fill_with_outline(chunk, &bb, false, 0, 2, 0, 1, 8, 0, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 5, 2, 0, 6, 8, 0, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 0, 2, 1, 0, 8, 6, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 6, 2, 1, 6, 8, 6, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 1, 2, 6, 5, 8, 6, nb, nb);

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

        // Fence railings
        p.fill_with_outline(chunk, &bb, false, 0, 3, 2, 0, 5, 4, f_ns, f_ns);
        p.fill_with_outline(chunk, &bb, false, 6, 3, 2, 6, 5, 2, f_ns, f_ns);
        p.fill_with_outline(chunk, &bb, false, 6, 3, 4, 6, 5, 4, f_ns, f_ns);

        // Stair steps (bottom to top, going from z=5 → z=1)
        p.add_block(chunk, nb, 5, 2, 5, &bb);
        p.fill_with_outline(chunk, &bb, false, 4, 2, 5, 4, 3, 5, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 3, 2, 5, 3, 4, 5, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 2, 2, 5, 2, 5, 5, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 1, 2, 5, 1, 6, 5, nb, nb);

        // Upper floor platform
        p.fill_with_outline(chunk, &bb, false, 1, 7, 1, 5, 7, 4, nb, nb);

        // Open the SE exit gap
        p.fill_with_outline(chunk, &bb, false, 6, 8, 2, 6, 8, 4, air, air);

        // Battlement above forward opening
        p.fill_with_outline(chunk, &bb, false, 2, 6, 0, 4, 8, 0, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 2, 5, 0, 4, 5, 0, f_ew, f_ew);

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
