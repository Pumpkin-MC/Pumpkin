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

/// Descending interior corridor staircase (5 × 14 × 10).
#[derive(Clone)]
pub struct CorridorStairsPiece {
    pub piece: NetherFortressPiece,
}

impl CorridorStairsPiece {
    pub fn create(
        collector: &mut StructurePiecesCollector,
        x: i32,
        y: i32,
        z: i32,
        orientation: BlockDirection,
        chain_length: u32,
    ) -> Option<Box<dyn StructurePieceBase>> {
        // Offset: -1 left/right, -7 height (deep stairs), 14 tall, 10 long
        let bbox = BlockBox::rotated(x, y, z, -1, -7, 0, 5, 14, 10, &orientation);
        if !NetherFortressPiece::is_in_bounds(&bbox) || collector.get_intersecting(&bbox).is_some()
        {
            return None;
        }

        let mut piece = NetherFortressPiece::new(
            StructurePieceType::NetherFortressCorridorStairs,
            chain_length,
            bbox,
        );
        piece.piece.set_facing(Some(orientation));
        Some(Box::new(Self { piece }))
    }
}

impl StructurePieceBase for CorridorStairsPiece {
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

        // Stair block facing south (going deeper into the structure)
        let mut stair_props = OakStairsLikeProperties::default(&Block::NETHER_BRICK_STAIRS);
        stair_props.facing = HorizontalFacing::South;
        let stair =
            pumpkin_data::BlockState::from_id(stair_props.to_state_id(&Block::NETHER_BRICK_STAIRS));

        let fence = |north: bool, south: bool| {
            let mut props = OakFenceLikeProperties::default(&Block::NETHER_BRICK_FENCE);
            props.north = north;
            props.south = south;
            pumpkin_data::BlockState::from_id(props.to_state_id(&Block::NETHER_BRICK_FENCE))
        };
        let f_ns = fence(true, true);

        for i in 0..=9i32 {
            let j = 1.max(7 - i);
            let k = (j + 5).min(14 - i).min(13);
            let l = i;

            // Solid base
            p.fill_with_outline(chunk, &bb, false, 0, 0, i, 4, j, i, nb, nb);

            // Air interior
            p.fill_with_outline(chunk, &bb, false, 1, j + 1, i, 3, k - 1, i, air, air);

            // Stair treads (only for the descending section)
            if i <= 6 {
                p.add_block(chunk, stair, 1, j + 1, i, &bb);
                p.add_block(chunk, stair, 2, j + 1, i, &bb);
                p.add_block(chunk, stair, 3, j + 1, i, &bb);
            }

            // Ceiling row
            p.fill_with_outline(chunk, &bb, false, 0, k, i, 4, k, i, nb, nb);

            // Side walls
            p.fill_with_outline(chunk, &bb, false, 0, j + 1, i, 0, k - 1, i, nb, nb);
            p.fill_with_outline(chunk, &bb, false, 4, j + 1, i, 4, k - 1, i, nb, nb);

            // Fence railings every 2 slices
            if (i & 1) == 0 {
                p.fill_with_outline(chunk, &bb, false, 0, j + 2, i, 0, j + 3, i, f_ns, f_ns);
                p.fill_with_outline(chunk, &bb, false, 4, j + 2, i, 4, j + 3, i, f_ns, f_ns);
            }

            // Fill downwards beneath each slice
            for m in 0..=4i32 {
                p.fill_downwards(chunk, nb, m, -1, l, &bb);
            }
        }
    }

    fn clone_box(&self) -> Box<dyn StructurePieceBase> {
        Box::new(self.clone())
    }
}
