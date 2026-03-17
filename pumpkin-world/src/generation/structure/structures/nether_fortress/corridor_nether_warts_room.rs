use pumpkin_data::{
    Block,
    block_properties::{
        BlockProperties, HorizontalFacing, OakFenceLikeProperties, OakStairsLikeProperties,
    },
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

/// Large interior room with nether wart crops (13 × 14 × 13).
#[derive(Clone)]
pub struct CorridorNetherWartsRoomPiece {
    pub piece: NetherFortressPiece,
}

impl CorridorNetherWartsRoomPiece {
    pub fn create(
        collector: &mut StructurePiecesCollector,
        x: i32,
        y: i32,
        z: i32,
        orientation: BlockDirection,
        chain_length: u32,
    ) -> Option<Box<dyn StructurePieceBase>> {
        let bbox = BlockBox::rotated(x, y, z, -5, -3, 0, 13, 14, 13, &orientation);
        if !NetherFortressPiece::is_in_bounds(&bbox) || collector.get_intersecting(&bbox).is_some()
        {
            return None;
        }

        let mut piece = NetherFortressPiece::new(
            StructurePieceType::NetherFortressCorridorNetherWartsRoom,
            chain_length,
            bbox,
        );
        piece.piece.set_facing(Some(orientation));
        Some(Box::new(Self { piece }))
    }
}

impl StructurePieceBase for CorridorNetherWartsRoomPiece {
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
        // Two forward exits (one at the back, one even further)
        self.piece.fill_forward_opening(
            start,
            collector,
            random,
            bridge_pieces,
            corridor_pieces,
            &mut last,
            5,
            3,
            true,
            pieces_to_process,
        );
        self.piece.fill_forward_opening(
            start,
            collector,
            random,
            bridge_pieces,
            corridor_pieces,
            &mut last,
            5,
            11,
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
        let f_ns_west = fence(true, false, true, true);
        let f_ns_east = fence(false, true, true, true);

        // Shell (identical to CorridorExit outer shape)
        p.fill_with_outline(chunk, &bb, false, 0, 3, 0, 12, 4, 12, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 0, 5, 0, 12, 13, 12, air, air);
        p.fill_with_outline(chunk, &bb, false, 0, 5, 0, 1, 12, 12, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 11, 5, 0, 12, 12, 12, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 2, 5, 11, 4, 12, 12, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 8, 5, 11, 10, 12, 12, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 5, 9, 11, 7, 12, 12, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 2, 5, 0, 4, 12, 1, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 8, 5, 0, 10, 12, 1, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 5, 9, 0, 7, 12, 1, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 2, 11, 2, 10, 12, 10, nb, nb);

        // Battlements
        let mut i = 1i32;
        while i <= 11 {
            p.fill_with_outline(chunk, &bb, false, i, 10, 0, i, 11, 0, f_ew, f_ew);
            p.fill_with_outline(chunk, &bb, false, i, 10, 12, i, 11, 12, f_ew, f_ew);
            p.fill_with_outline(chunk, &bb, false, 0, 10, i, 0, 11, i, f_ns, f_ns);
            p.fill_with_outline(chunk, &bb, false, 12, 10, i, 12, 11, i, f_ns, f_ns);
            p.add_block(chunk, nb, i, 13, 0, &bb);
            p.add_block(chunk, nb, i, 13, 12, &bb);
            p.add_block(chunk, nb, 0, 13, i, &bb);
            p.add_block(chunk, nb, 12, 13, i, &bb);
            if i != 11 {
                p.add_block(chunk, f_ew, i + 1, 13, 0, &bb);
                p.add_block(chunk, f_ew, i + 1, 13, 12, &bb);
                p.add_block(chunk, f_ns, 0, 13, i + 1, &bb);
                p.add_block(chunk, f_ns, 12, 13, i + 1, &bb);
            }
            i += 2;
        }
        p.add_block(chunk, fence(false, true, true, false), 0, 13, 0, &bb);
        p.add_block(chunk, fence(false, true, false, true), 0, 13, 12, &bb);
        p.add_block(chunk, fence(true, false, false, true), 12, 13, 12, &bb);
        p.add_block(chunk, fence(true, false, true, false), 12, 13, 0, &bb);

        // Interior buttress fences
        let mut i = 3i32;
        while i <= 9 {
            p.fill_with_outline(chunk, &bb, false, 1, 7, i, 1, 8, i, f_ns_west, f_ns_west);
            p.fill_with_outline(chunk, &bb, false, 11, 7, i, 11, 8, i, f_ns_east, f_ns_east);
            i += 2;
        }

        // Staircase going up (north-facing stairs, going from z=4 up to z=10)
        let mut stair_props = OakStairsLikeProperties::default(&Block::NETHER_BRICK_STAIRS);
        stair_props.facing = HorizontalFacing::North;
        let stair_n =
            pumpkin_data::BlockState::from_id(stair_props.to_state_id(&Block::NETHER_BRICK_STAIRS));

        stair_props.facing = HorizontalFacing::East;
        let stair_e =
            pumpkin_data::BlockState::from_id(stair_props.to_state_id(&Block::NETHER_BRICK_STAIRS));

        stair_props.facing = HorizontalFacing::West;
        let stair_w =
            pumpkin_data::BlockState::from_id(stair_props.to_state_id(&Block::NETHER_BRICK_STAIRS));

        for j in 0..=6i32 {
            let k = j + 4;
            for l in 5..=7i32 {
                p.add_block(chunk, stair_n, l, 5 + j, k, &bb);
            }
            if (5..=8).contains(&k) {
                p.fill_with_outline(chunk, &bb, false, 5, 5, k, 7, j + 4, k, nb, nb);
            } else if (9..=10).contains(&k) {
                p.fill_with_outline(chunk, &bb, false, 5, 8, k, 7, j + 4, k, nb, nb);
            }
            if j >= 1 {
                p.fill_with_outline(chunk, &bb, false, 5, 6 + j, k, 7, 9 + j, k, air, air);
            }
        }
        for l in 5..=7i32 {
            p.add_block(chunk, stair_n, l, 12, 11, &bb);
        }

        // Stair balcony fences
        p.fill_with_outline(chunk, &bb, false, 5, 6, 7, 5, 7, 7, f_ns_east, f_ns_east);
        p.fill_with_outline(chunk, &bb, false, 7, 6, 7, 7, 7, 7, f_ns_west, f_ns_west);

        // Clear the arch above the top stair exit
        p.fill_with_outline(chunk, &bb, false, 5, 13, 12, 7, 13, 12, air, air);

        // Wart beds (raised platforms of soul sand with nether wart on top)
        p.fill_with_outline(chunk, &bb, false, 2, 5, 2, 3, 5, 3, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 2, 5, 9, 3, 5, 10, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 2, 5, 4, 2, 5, 8, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 9, 5, 2, 10, 5, 3, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 9, 5, 9, 10, 5, 10, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 10, 5, 4, 10, 5, 8, nb, nb);

        // Stair edgings for the wart beds
        p.add_block(chunk, stair_w, 4, 5, 2, &bb);
        p.add_block(chunk, stair_w, 4, 5, 3, &bb);
        p.add_block(chunk, stair_w, 4, 5, 9, &bb);
        p.add_block(chunk, stair_w, 4, 5, 10, &bb);
        p.add_block(chunk, stair_e, 8, 5, 2, &bb);
        p.add_block(chunk, stair_e, 8, 5, 3, &bb);
        p.add_block(chunk, stair_e, 8, 5, 9, &bb);
        p.add_block(chunk, stair_e, 8, 5, 10, &bb);

        let soul_sand = Block::SOUL_SAND.default_state;
        let nether_wart = Block::NETHER_WART.default_state;

        p.fill_with_outline(chunk, &bb, false, 3, 4, 4, 4, 4, 8, soul_sand, soul_sand);
        p.fill_with_outline(chunk, &bb, false, 8, 4, 4, 9, 4, 8, soul_sand, soul_sand);
        p.fill_with_outline(
            chunk,
            &bb,
            false,
            3,
            5,
            4,
            4,
            5,
            8,
            nether_wart,
            nether_wart,
        );
        p.fill_with_outline(
            chunk,
            &bb,
            false,
            8,
            5,
            4,
            9,
            5,
            8,
            nether_wart,
            nether_wart,
        );

        // Bridge support legs
        p.fill_with_outline(chunk, &bb, false, 4, 2, 0, 8, 2, 12, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 0, 2, 4, 12, 2, 8, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 4, 0, 0, 8, 1, 3, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 4, 0, 9, 8, 1, 12, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 0, 0, 4, 3, 1, 8, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 9, 0, 4, 12, 1, 8, nb, nb);

        for l in 4..=8i32 {
            for m in 0..=2i32 {
                p.fill_downwards(chunk, nb, l, -1, m, &bb);
                p.fill_downwards(chunk, nb, l, -1, 12 - m, &bb);
            }
        }
        for l in 0..=2i32 {
            for m in 4..=8i32 {
                p.fill_downwards(chunk, nb, l, -1, m, &bb);
                p.fill_downwards(chunk, nb, 12 - l, -1, m, &bb);
            }
        }
    }

    fn clone_box(&self) -> Box<dyn StructurePieceBase> {
        Box::new(self.clone())
    }
}
