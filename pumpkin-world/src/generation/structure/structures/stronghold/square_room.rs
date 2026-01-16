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
            stronghold::{StoneBrickRandomizer, StrongholdPiece},
        },
    },
};

#[derive(Clone)]
pub struct SquareRoomPiece {
    piece: StrongholdPiece,
    room_type: u32,
}

impl SquareRoomPiece {
    pub fn create(
        collector: &mut StructurePiecesCollector,
        random: &mut impl RandomImpl,
        x: i32,
        y: i32,
        z: i32,
        orientation: BlockDirection,
        chain_length: u32,
    ) -> Option<Box<dyn StructurePieceBase>> {
        let bounding_box = BlockBox::rotated(x, y, z, -4, -1, 0, 11, 7, 11, &orientation);

        // 2. Check world bounds and intersections
        if !StrongholdPiece::is_in_bounds(&bounding_box)
            || collector.get_intersecting(&bounding_box).is_some()
        {
            return None;
        }

        // 3. Construct the piece
        let mut piece = StrongholdPiece::new(
            StructurePieceType::StrongholdSquareRoom,
            chain_length,
            bounding_box,
        );
        piece.piece.set_facing(Some(orientation));

        let room_type = random.next_bounded_i32(5) as u32;

        Some(Box::new(Self { piece, room_type }))
    }
}

impl StructurePieceBase for SquareRoomPiece {
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
        collector: &mut StructurePiecesCollector,
        pieces_to_process: &mut Vec<Box<dyn StructurePieceBase>>,
    ) {
        // 1. Forward Opening
        // Java: this.fillForwardOpening((Start)start, holder, random, 4, 1);
        self.piece.fill_forward_opening(
            start,
            collector,
            random,
            4, // left_right_offset
            1, // height_offset
            pieces_to_process,
            None,
        );

        // 2. Left Opening (NW)
        // Java: this.fillNWOpening((Start)start, holder, random, 1, 4);
        self.piece.fill_nw_opening(
            start,
            collector,
            random,
            1, // height_offset
            4, // left_right_offset
            pieces_to_process,
        );

        // 3. Right Opening (SE)
        // Java: this.fillSEOpening((Start)start, holder, random, 1, 4);
        self.piece.fill_se_opening(
            start,
            collector,
            random,
            1, // height_offset
            4, // left_right_offset
            pieces_to_process,
        );
    }

    fn place(&mut self, chunk: &mut ProtoChunk, random: &mut RandomGenerator, _seed: i64) {
        let randomizer = StoneBrickRandomizer;
        let box_limit = self.piece.piece.bounding_box;
        let p = &self.piece;
        let inner = &p.piece;
        let air = Block::AIR.default_state;

        // 1. Main Outline (10x6x10) - Matches Java: fillWithOutline(..., 0,0,0, 10,6,10, ...)
        inner.fill_outline_random(
            0,
            0,
            0,
            10,
            6,
            10,
            &randomizer,
            chunk,
            true,
            random,
            &box_limit,
        );

        // 2. Main Entrance (Z=0, aligned to X=4)
        // Matches Java: generateEntrance(..., 4, 1, 0)
        p.generate_entrance(chunk, &box_limit, self.piece.entry_door, 4, 1, 0);

        // 3. Clear Exits
        // Forward Exit (Z=10) -> fillWithOutline(..., 4,1,10, 6,3,10, AIR, AIR, false)
        inner.fill_with_outline(chunk, &box_limit, false, 4, 1, 10, 6, 3, 10, air, air);

        // Left Side Exit (X=0) -> fillWithOutline(..., 0,1,4, 0,3,6, AIR, AIR, false)
        inner.fill_with_outline(chunk, &box_limit, false, 0, 1, 4, 0, 3, 6, air, air);

        // Right Side Exit (X=10) -> fillWithOutline(..., 10,1,4, 10,3,6, AIR, AIR, false)
        inner.fill_with_outline(chunk, &box_limit, false, 10, 1, 4, 10, 3, 6, air, air);

        // 4. Room Specific Decorations
        match self.room_type {
            0 => {
                let stone_brick = Block::STONE_BRICKS.default_state;
                let smooth_slab = Block::SMOOTH_STONE_SLAB.default_state;

                // Central Pillar
                inner.add_block(chunk, stone_brick, 5, 1, 5, &box_limit);
                inner.add_block(chunk, stone_brick, 5, 2, 5, &box_limit);
                inner.add_block(chunk, stone_brick, 5, 3, 5, &box_limit);

                // // Torches
                // inner.add_block(chunk, &Block::WALL_TORCH.default_state.with("facing", "west"), 4, 3, 5, &box_limit);
                // inner.add_block(chunk, &Block::WALL_TORCH.default_state.with("facing", "east"), 6, 3, 5, &box_limit);
                // inner.add_block(chunk, &Block::WALL_TORCH.default_state.with("facing", "south"), 5, 3, 4, &box_limit);
                // inner.add_block(chunk, &Block::WALL_TORCH.default_state.with("facing", "north"), 5, 3, 6, &box_limit);

                // Floor Pattern (Slabs)
                inner.add_block(chunk, smooth_slab, 4, 1, 4, &box_limit);
                inner.add_block(chunk, smooth_slab, 4, 1, 5, &box_limit);
                inner.add_block(chunk, smooth_slab, 4, 1, 6, &box_limit);
                inner.add_block(chunk, smooth_slab, 6, 1, 4, &box_limit);
                inner.add_block(chunk, smooth_slab, 6, 1, 5, &box_limit);
                inner.add_block(chunk, smooth_slab, 6, 1, 6, &box_limit);
                inner.add_block(chunk, smooth_slab, 5, 1, 4, &box_limit);
                inner.add_block(chunk, smooth_slab, 5, 1, 6, &box_limit);
            }
            1 => {
                let stone_brick = Block::STONE_BRICKS.default_state;

                // Ring of bricks
                for i in 0..5 {
                    inner.add_block(chunk, stone_brick, 3, 1, 3 + i, &box_limit);
                    inner.add_block(chunk, stone_brick, 7, 1, 3 + i, &box_limit);
                    inner.add_block(chunk, stone_brick, 3 + i, 1, 3, &box_limit);
                    inner.add_block(chunk, stone_brick, 3 + i, 1, 7, &box_limit);
                }

                // Central Water Column
                inner.add_block(chunk, stone_brick, 5, 1, 5, &box_limit);
                inner.add_block(chunk, stone_brick, 5, 2, 5, &box_limit);
                inner.add_block(chunk, stone_brick, 5, 3, 5, &box_limit);
                inner.add_block(chunk, Block::WATER.default_state, 5, 4, 5, &box_limit);
            }
            2 => {
                let cobble = Block::COBBLESTONE.default_state;

                // Cobblestone shelves/walls
                for i in 1..=9 {
                    inner.add_block(chunk, cobble, 1, 3, i, &box_limit);
                    inner.add_block(chunk, cobble, 9, 3, i, &box_limit);
                }
                for i in 1..=9 {
                    inner.add_block(chunk, cobble, i, 3, 1, &box_limit);
                    inner.add_block(chunk, cobble, i, 3, 9, &box_limit);
                }

                // Central structure supports
                inner.add_block(chunk, cobble, 5, 1, 4, &box_limit);
                inner.add_block(chunk, cobble, 5, 1, 6, &box_limit);
                inner.add_block(chunk, cobble, 5, 3, 4, &box_limit);
                inner.add_block(chunk, cobble, 5, 3, 6, &box_limit);
                inner.add_block(chunk, cobble, 4, 1, 5, &box_limit);
                inner.add_block(chunk, cobble, 6, 1, 5, &box_limit);
                inner.add_block(chunk, cobble, 4, 3, 5, &box_limit);
                inner.add_block(chunk, cobble, 6, 3, 5, &box_limit);

                // Pillars
                for i in 1..=3 {
                    inner.add_block(chunk, cobble, 4, i, 4, &box_limit);
                    inner.add_block(chunk, cobble, 6, i, 4, &box_limit);
                    inner.add_block(chunk, cobble, 4, i, 6, &box_limit);
                    inner.add_block(chunk, cobble, 6, i, 6, &box_limit);
                }

                inner.add_block(chunk, Block::WALL_TORCH.default_state, 5, 3, 5, &box_limit);

                // Upper floor (Oak Planks)
                let oak = Block::OAK_PLANKS.default_state;
                for i in 2..=8 {
                    inner.add_block(chunk, oak, 2, 3, i, &box_limit);
                    inner.add_block(chunk, oak, 3, 3, i, &box_limit);

                    // Holes for ladders/structure
                    if i <= 3 || i >= 7 {
                        inner.add_block(chunk, oak, 4, 3, i, &box_limit);
                        inner.add_block(chunk, oak, 5, 3, i, &box_limit);
                        inner.add_block(chunk, oak, 6, 3, i, &box_limit);
                    }
                    inner.add_block(chunk, oak, 7, 3, i, &box_limit);
                    inner.add_block(chunk, oak, 8, 3, i, &box_limit);
                }

                // Ladder
                //  let ladder = Block::LADDER.default_state.with("facing", "west");
                //  inner.add_block(chunk, &ladder, 9, 1, 3, &box_limit);
                //  inner.add_block(chunk, &ladder, 9, 2, 3, &box_limit);
                //  inner.add_block(chunk, &ladder, 9, 3, 3, &box_limit);

                // Chest
                // Note: Ensure you have `LootTables` defined or use a string ID
                // p.add_chest(chunk, &box_limit, random, 3, 4, 8, LootTables::StrongholdCrossingChest);
                inner.add_block(chunk, Block::CHEST.default_state, 3, 4, 8, &box_limit);
            }
            _ => {}
        }
    }

    fn clone_box(&self) -> Box<dyn StructurePieceBase> {
        Box::new((*self).clone())
    }
}
