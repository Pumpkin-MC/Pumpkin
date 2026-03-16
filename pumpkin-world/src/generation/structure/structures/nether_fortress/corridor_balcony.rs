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

/// Interior corridor piece with a balcony overlook (9 × 7 × 9).
#[derive(Clone)]
pub struct CorridorBalconyPiece {
    pub piece: NetherFortressPiece,
}

impl CorridorBalconyPiece {
    pub fn create(
        collector: &mut StructurePiecesCollector,
        x: i32,
        y: i32,
        z: i32,
        orientation: BlockDirection,
        chain_length: u32,
    ) -> Option<Box<dyn StructurePieceBase>> {
        let bbox = BlockBox::rotated(x, y, z, -3, 0, 0, 9, 7, 9, &orientation);
        if !NetherFortressPiece::is_in_bounds(&bbox) || collector.get_intersecting(&bbox).is_some()
        {
            return None;
        }

        let mut piece = NetherFortressPiece::new(
            StructurePieceType::NetherFortressCorridorBalcony,
            chain_length,
            bbox,
        );
        piece.piece.set_facing(Some(orientation));
        Some(Box::new(Self { piece }))
    }
}

impl StructurePieceBase for CorridorBalconyPiece {
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

        // In Java: `i` is 1 for EAST/SOUTH facing, 5 for WEST/NORTH.
        let i = match self.piece.piece.facing {
            Some(BlockDirection::West) | Some(BlockDirection::North) => 5,
            _ => 1,
        };

        let nw_blocked = random.next_bounded_i32(8) > 0;
        let se_blocked = random.next_bounded_i32(8) > 0;

        self.piece.fill_nw_opening(
            start,
            collector,
            random,
            bridge_pieces,
            corridor_pieces,
            &mut last,
            0,
            i,
            nw_blocked,
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
            i,
            se_blocked,
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

        let fence = |west: bool, east: bool, north: bool, south: bool| {
            let mut props = OakFenceLikeProperties::default(&Block::NETHER_BRICK_FENCE);
            props.west = west;
            props.east = east;
            props.north = north;
            props.south = south;
            pumpkin_data::BlockState::from_id(props.to_state_id(&Block::NETHER_BRICK_FENCE))
        };
        let f_ns = fence(false, false, true, true);
        let f_ew = fence(true, true, false, false);

        p.fill_with_outline(chunk, &bb, false, 0, 0, 0, 8, 1, 8, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 0, 2, 0, 8, 5, 8, air, air);
        p.fill_with_outline(chunk, &bb, false, 0, 6, 0, 8, 6, 5, nb, nb);

        // Front wall (z=0) openings
        p.fill_with_outline(chunk, &bb, false, 0, 2, 0, 2, 5, 0, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 6, 2, 0, 8, 5, 0, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 1, 3, 0, 1, 4, 0, f_ew, f_ew);
        p.fill_with_outline(chunk, &bb, false, 7, 3, 0, 7, 4, 0, f_ew, f_ew);

        // Back floor and openings
        p.fill_with_outline(chunk, &bb, false, 0, 2, 4, 8, 2, 8, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 1, 1, 4, 2, 2, 4, air, air);
        p.fill_with_outline(chunk, &bb, false, 6, 1, 4, 7, 2, 4, air, air);

        // Back fence railing
        p.fill_with_outline(chunk, &bb, false, 1, 3, 8, 7, 3, 8, f_ew, f_ew);
        p.add_block(chunk, fence(false, true, false, true), 0, 3, 8, &bb);
        p.add_block(chunk, fence(true, false, false, true), 8, 3, 8, &bb);
        p.fill_with_outline(chunk, &bb, false, 0, 3, 6, 0, 3, 7, f_ns, f_ns);
        p.fill_with_outline(chunk, &bb, false, 8, 3, 6, 8, 3, 7, f_ns, f_ns);

        // Side wall pillars and fences
        p.fill_with_outline(chunk, &bb, false, 0, 3, 4, 0, 5, 5, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 8, 3, 4, 8, 5, 5, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 1, 3, 5, 2, 5, 5, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 6, 3, 5, 7, 5, 5, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 1, 4, 5, 1, 5, 5, f_ew, f_ew);
        p.fill_with_outline(chunk, &bb, false, 7, 4, 5, 7, 5, 5, f_ew, f_ew);

        for i in 0..=5i32 {
            for j in 0..=8i32 {
                p.fill_downwards(chunk, nb, j, -1, i, &bb);
            }
        }
    }

    fn clone_box(&self) -> Box<dyn StructurePieceBase> {
        Box::new(self.clone())
    }
}
