use pumpkin_data::{
    Block, BlockState,
    block_properties::{BlockProperties, HorizontalFacing, LadderLikeProperties},
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
            stronghold::{EntranceType, StoneBrickRandomizer, StrongholdPiece},
        },
    },
};

#[derive(Clone)]
pub struct LibraryPiece {
    pub piece: StrongholdPiece,
    pub tall: bool,
}

impl LibraryPiece {
    pub fn create(
        collector: &mut StructurePiecesCollector,
        random: &mut impl RandomImpl,
        x: i32,
        y: i32,
        z: i32,
        orientation: BlockDirection,
        chain_length: u32,
    ) -> Option<Box<dyn StructurePieceBase>> {
        let mut bounding_box = BlockBox::rotated(x, y, z, -4, -1, 0, 14, 11, 15, &orientation);

        if !StrongholdPiece::is_in_bounds(&bounding_box)
            || collector.get_intersecting(&bounding_box).is_some()
        {
            bounding_box = BlockBox::rotated(x, y, z, -4, -1, 0, 14, 6, 15, &orientation);
            if !StrongholdPiece::is_in_bounds(&bounding_box)
                || collector.get_intersecting(&bounding_box).is_some()
            {
                return None;
            }
        }

        let mut piece = StrongholdPiece::new(
            StructurePieceType::StrongholdLibrary,
            chain_length,
            bounding_box,
        );
        piece.piece.set_facing(Some(orientation));
        piece.entry_door = EntranceType::get_random(random);

        let tall = bounding_box.get_block_count_y() > 6;

        Some(Box::new(Self { piece, tall }))
    }
}

impl StructurePieceBase for LibraryPiece {
    fn get_structure_piece(&self) -> &StructurePiece {
        &self.piece.piece
    }

    fn get_structure_piece_mut(&mut self) -> &mut StructurePiece {
        &mut self.piece.piece
    }

    fn place(
        &mut self,
        chunk: &mut ProtoChunk,
        random: &mut RandomGenerator,
        _seed: i64,
        chunk_box: &BlockBox,
    ) {
        let randomizer = StoneBrickRandomizer;
        let box_limit = *chunk_box;
        let p = &self.piece;
        let inner = &p.piece;

        let max_y = if self.tall { 10 } else { 5 }; // Java: i-1 where i is 11 or 6

        // 1. Main Shell
        inner.fill_outline_random(
            0,
            0,
            0,
            13,
            max_y,
            14,
            &randomizer,
            chunk,
            true,
            random,
            &box_limit,
        );

        // 2. Entrance
        p.generate_entrance(chunk, &box_limit, self.piece.entry_door, 4, 1, 0);

        // 3. Cobwebs (Random probability placement)
        inner.fill_with_outline_under_sea_level(
            chunk,
            &box_limit,
            random,
            0.07,
            2,
            1,
            1,
            11,
            4,
            13,
            Block::COBWEB.default_state,
            Block::COBWEB.default_state,
            false,
            false,
        );

        // 4. Bookshelves, Pillars, and Wall Torches
        let planks = Block::OAK_PLANKS.default_state;
        let books = Block::BOOKSHELF.default_state;
        // let torch_e = Block::WALL_TORCH.default_state.with("facing", "east");
        // let torch_w = Block::WALL_TORCH.default_state.with("facing", "west");

        for l in 1..=13 {
            if (l - 1) % 4 == 0 {
                // Pillars
                inner.fill_with_outline(chunk, &box_limit, false, 1, 1, l, 1, 4, l, planks, planks);
                inner.fill_with_outline(
                    chunk, &box_limit, false, 12, 1, l, 12, 4, l, planks, planks,
                );
                // Torches
                // inner.add_block(chunk, &torch_e, 2, 3, l, &box_limit);
                // inner.add_block(chunk, &torch_w, 11, 3, l, &box_limit);

                if self.tall {
                    inner.fill_with_outline(
                        chunk, &box_limit, false, 1, 6, l, 1, 9, l, planks, planks,
                    );
                    inner.fill_with_outline(
                        chunk, &box_limit, false, 12, 6, l, 12, 9, l, planks, planks,
                    );
                }
            } else {
                // Shelf Walls
                inner.fill_with_outline(chunk, &box_limit, false, 1, 1, l, 1, 4, l, books, books);
                inner.fill_with_outline(chunk, &box_limit, false, 12, 1, l, 12, 4, l, books, books);
                if self.tall {
                    inner.fill_with_outline(
                        chunk, &box_limit, false, 1, 6, l, 1, 9, l, books, books,
                    );
                    inner.fill_with_outline(
                        chunk, &box_limit, false, 12, 6, l, 12, 9, l, books, books,
                    );
                }
            }
        }

        // 5. Central Freestanding Shelves
        for l in (3..12).step_by(2) {
            inner.fill_with_outline(chunk, &box_limit, false, 3, 1, l, 4, 3, l, books, books);
            inner.fill_with_outline(chunk, &box_limit, false, 6, 1, l, 7, 3, l, books, books);
            inner.fill_with_outline(chunk, &box_limit, false, 9, 1, l, 10, 3, l, books, books);
        }

        if self.tall {
            // Upper Walkways
            inner.fill_with_outline(chunk, &box_limit, false, 1, 5, 1, 3, 5, 13, planks, planks);
            inner.fill_with_outline(
                chunk, &box_limit, false, 10, 5, 1, 12, 5, 13, planks, planks,
            );
            inner.fill_with_outline(chunk, &box_limit, false, 4, 5, 1, 9, 5, 2, planks, planks);
            inner.fill_with_outline(chunk, &box_limit, false, 4, 5, 12, 9, 5, 13, planks, planks);

            // Walkway corners
            inner.add_block(chunk, planks, 9, 5, 11, &box_limit);
            inner.add_block(chunk, planks, 8, 5, 11, &box_limit);
            inner.add_block(chunk, planks, 9, 5, 10, &box_limit);

            // Fences / Railings
            // let f_ns = Block::OAK_FENCE.default_state.with("north", "true").with("south", "true");
            // let f_ew = Block::OAK_FENCE.default_state.with("east", "true").with("west", "true");

            // inner.fill_with_outline(chunk, &box_limit, false, 3, 6, 3, 3, 6, 11, &f_ns, &f_ns);
            // inner.fill_with_outline(chunk, &box_limit, false, 10, 6, 3, 10, 6, 9, &f_ns, &f_ns);
            // inner.fill_with_outline(chunk, &box_limit, false, 4, 6, 2, 9, 6, 2, &f_ew, &f_ew);
            // inner.fill_with_outline(chunk, &box_limit, false, 4, 6, 12, 7, 6, 12, &f_ew, &f_ew);

            // // Complex fence junctions (matching the m-loop and specific offsets)
            // inner.add_block(chunk, &Block::OAK_FENCE.default_state.with("north", "true").with("east", "true"), 3, 6, 2, &box_limit);
            // inner.add_block(chunk, &Block::OAK_FENCE.default_state.with("south", "true").with("east", "true"), 3, 6, 12, &box_limit);
            // inner.add_block(chunk, &Block::OAK_FENCE.default_state.with("north", "true").with("west", "true"), 10, 6, 2, &box_limit);
            // for m in 0..=2 {
            //     inner.add_block(chunk, &Block::OAK_FENCE.default_state.with("south", "true").with("west", "true"), 8 + m, 6, 12 - m, &box_limit);
            //     if m != 2 {
            //         inner.add_block(chunk, &Block::OAK_FENCE.default_state.with("north", "true").with("east", "true"), 8 + m, 6, 11 - m, &box_limit);
            //     }
            // }

            // // Ladder
            let mut props = LadderLikeProperties::default(&Block::LADDER);
            props.facing = HorizontalFacing::South;
            let ladder = BlockState::from_id(props.to_state_id(&Block::LADDER));
            for y in 1..=7 {
                inner.add_block(chunk, ladder, 10, y, 13, &box_limit);
            }

            // // Chandelier (The central structure)
            // let f_e = Block::OAK_FENCE.default_state.with("east", "true");
            // let f_w = Block::OAK_FENCE.default_state.with("west", "true");
            // inner.add_block(chunk, &f_e, 6, 9, 7, &box_limit);
            // inner.add_block(chunk, &f_w, 7, 9, 7, &box_limit);
            // inner.add_block(chunk, &f_e, 6, 8, 7, &box_limit);
            // inner.add_block(chunk, &f_w, 7, 8, 7, &box_limit);

            // let f_full = f_ns.with("west", "true").with("east", "true");
            // inner.add_block(chunk, &f_full, 6, 7, 7, &box_limit);
            // inner.add_block(chunk, &f_full, 7, 7, 7, &box_limit);

            // Chandelier Torches
            let t = Block::TORCH.default_state;
            inner.add_block(chunk, t, 5, 8, 7, &box_limit);
            inner.add_block(chunk, t, 8, 8, 7, &box_limit);
            inner.add_block(chunk, t, 6, 8, 6, &box_limit);
            inner.add_block(chunk, t, 6, 8, 8, &box_limit);
        }

        // 6. Chests
        // inner.add_chest(chunk, &box_limit, random, 3, 3, 5, "stronghold_library");
        // if self.tall {
        //     inner.add_block(chunk, &Block::AIR.default_state, 12, 9, 1, &box_limit); // Clear space above top chest
        //     inner.add_chest(chunk, &box_limit, random, 12, 8, 1, "stronghold_library");
        // }
    }

    fn clone_box(&self) -> Box<dyn StructurePieceBase> {
        Box::new((*self).clone())
    }
}
