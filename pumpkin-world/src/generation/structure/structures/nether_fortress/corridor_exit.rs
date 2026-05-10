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

/// Large entrance tower connecting the bridge network to the interior (13 × 14 × 13).
#[derive(Clone)]
pub struct CorridorExitPiece {
    pub piece: NetherFortressPiece,
}

impl CorridorExitPiece {
    pub fn create(
        collector: &mut StructurePiecesCollector,
        _random: &mut impl RandomImpl,
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
            StructurePieceType::NetherFortressCorridorExit,
            chain_length,
            bbox,
        );
        piece.piece.set_facing(Some(orientation));
        Some(Box::new(Self { piece }))
    }
}

impl StructurePieceBase for CorridorExitPiece {
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
            5,
            3,
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

        // Main shell
        p.fill_with_outline(chunk, &bb, false, 0, 3, 0, 12, 4, 12, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 0, 5, 0, 12, 13, 12, air, air);

        // Outer walls
        p.fill_with_outline(chunk, &bb, false, 0, 5, 0, 1, 12, 12, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 11, 5, 0, 12, 12, 12, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 2, 5, 11, 4, 12, 12, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 8, 5, 11, 10, 12, 12, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 5, 9, 11, 7, 12, 12, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 2, 5, 0, 4, 12, 1, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 8, 5, 0, 10, 12, 1, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 5, 9, 0, 7, 12, 1, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 2, 11, 2, 10, 12, 10, nb, nb);

        // Gate arch fence
        p.fill_with_outline(
            chunk,
            &bb,
            false,
            5,
            8,
            0,
            7,
            8,
            0,
            Block::NETHER_BRICK_FENCE.default_state,
            Block::NETHER_BRICK_FENCE.default_state,
        );

        // Battlement fences and corner blocks along top
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

        // Corner fence pieces
        p.add_block(chunk, fence(false, true, true, false), 0, 13, 0, &bb);
        p.add_block(chunk, fence(false, true, false, true), 0, 13, 12, &bb);
        p.add_block(chunk, fence(true, false, false, true), 12, 13, 12, &bb);
        p.add_block(chunk, fence(true, false, true, false), 12, 13, 0, &bb);

        // Interior buttress fences
        let mut i = 3i32;
        while i <= 9 {
            p.fill_with_outline(
                chunk,
                &bb,
                false,
                1,
                7,
                i,
                1,
                8,
                i,
                fence(false, false, true, true), // + west connection
                fence(false, false, true, true),
            );
            p.fill_with_outline(
                chunk,
                &bb,
                false,
                11,
                7,
                i,
                11,
                8,
                i,
                fence(false, false, true, true), // + east connection
                fence(false, false, true, true),
            );
            i += 2;
        }

        // Exterior bridge supports
        p.fill_with_outline(chunk, &bb, false, 4, 2, 0, 8, 2, 12, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 0, 2, 4, 12, 2, 8, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 4, 0, 0, 8, 1, 3, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 4, 0, 9, 8, 1, 12, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 0, 0, 4, 3, 1, 8, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 9, 0, 4, 12, 1, 8, nb, nb);

        for i in 4..=8i32 {
            for j in 0..=2i32 {
                p.fill_downwards(chunk, nb, i, -1, j, &bb);
                p.fill_downwards(chunk, nb, i, -1, 12 - j, &bb);
            }
        }
        for i in 0..=2i32 {
            for j in 4..=8i32 {
                p.fill_downwards(chunk, nb, i, -1, j, &bb);
                p.fill_downwards(chunk, nb, 12 - i, -1, j, &bb);
            }
        }

        // Central lava well
        p.fill_with_outline(chunk, &bb, false, 5, 5, 5, 7, 5, 7, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 6, 1, 6, 6, 4, 6, air, air);
        p.add_block(chunk, nb, 6, 0, 6, &bb);
        p.add_block(chunk, Block::LAVA.default_state, 6, 5, 6, &bb);

        // Schedule lava fluid tick
        // TODO
        // let lava_pos = p.offset_pos(6, 5, 6);
        // if bb.contains_pos(&lava_pos) {
        //     chunk.schedule_fluid_tick(&lava_pos, 0);
        // }
    }

    fn clone_box(&self) -> Box<dyn StructurePieceBase> {
        Box::new(self.clone())
    }
}
