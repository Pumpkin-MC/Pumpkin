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
            stronghold::{EntranceType, StoneBrickRandomizer, StrongholdPiece},
        },
    },
};

#[derive(Clone)]
pub struct PortalRoomPiece {
    pub piece: StrongholdPiece,
    pub spawner_placed: bool,
}

impl PortalRoomPiece {
    pub fn create(
        collector: &mut StructurePiecesCollector,
        _random: &mut impl RandomImpl,
        x: i32,
        y: i32,
        z: i32,
        orientation: BlockDirection,
        chain_length: u32,
    ) -> Option<Box<dyn StructurePieceBase>> {
        let bounding_box = BlockBox::rotated(x, y, z, -4, -1, 0, 11, 8, 16, &orientation);

        if !StrongholdPiece::is_in_bounds(&bounding_box)
            || collector.get_intersecting(&bounding_box).is_some()
        {
            return None;
        }

        let mut piece = StrongholdPiece::new(
            StructurePieceType::StrongholdPortalRoom,
            chain_length,
            bounding_box,
        );
        piece.piece.set_facing(Some(orientation));

        Some(Box::new(Self {
            piece,
            spawner_placed: false,
        }))
    }
}

impl StructurePieceBase for PortalRoomPiece {
    fn get_structure_piece(&self) -> &StructurePiece {
        &self.piece.piece
    }

    fn get_structure_piece_mut(&mut self) -> &mut StructurePiece {
        &mut self.piece.piece
    }

    fn place(&mut self, chunk: &mut ProtoChunk, random: &mut RandomGenerator, _seed: i64) {
        let randomizer = StoneBrickRandomizer;
        let box_limit = self.piece.piece.bounding_box;
        let p = &self.piece;
        let inner = &p.piece;

        // 1. Main Shell (11x8x16)
        inner.fill_outline_random(
            0,
            0,
            0,
            10,
            7,
            15,
            &randomizer,
            chunk,
            false,
            random,
            &box_limit,
        );

        // 2. Entrance (Grates)
        p.generate_entrance(chunk, &box_limit, EntranceType::Grates, 4, 1, 0);

        // 3. Upper Walkways
        inner.fill_outline_random(
            1,
            6,
            1,
            1,
            6,
            14,
            &randomizer,
            chunk,
            false,
            random,
            &box_limit,
        );
        inner.fill_outline_random(
            9,
            6,
            1,
            9,
            6,
            14,
            &randomizer,
            chunk,
            false,
            random,
            &box_limit,
        );
        inner.fill_outline_random(
            2,
            6,
            1,
            8,
            6,
            2,
            &randomizer,
            chunk,
            false,
            random,
            &box_limit,
        );
        inner.fill_outline_random(
            2,
            6,
            14,
            8,
            6,
            14,
            &randomizer,
            chunk,
            false,
            random,
            &box_limit,
        );

        // 4. Lava Pools (Side pools and central pool)
        let lava = Block::LAVA.default_state;
        inner.fill_outline_random(
            1,
            1,
            1,
            2,
            1,
            4,
            &randomizer,
            chunk,
            false,
            random,
            &box_limit,
        );
        inner.fill_outline_random(
            8,
            1,
            1,
            9,
            1,
            4,
            &randomizer,
            chunk,
            false,
            random,
            &box_limit,
        );
        inner.fill_with_outline(chunk, &box_limit, false, 1, 1, 1, 1, 1, 3, lava, lava);
        inner.fill_with_outline(chunk, &box_limit, false, 9, 1, 1, 9, 1, 3, lava, lava);

        inner.fill_outline_random(
            3,
            1,
            8,
            7,
            1,
            12,
            &randomizer,
            chunk,
            false,
            random,
            &box_limit,
        );
        inner.fill_with_outline(chunk, &box_limit, false, 4, 1, 9, 6, 1, 11, lava, lava);

        // 5. Iron Bars (Directional panes)
        // let bar_ns = Block::IRON_BARS.default_state.with("north", "true").with("south", "true");
        // let bar_we = Block::IRON_BARS.default_state.with("west", "true").with("east", "true");

        // for j in (3..14).step_by(2) {
        //     inner.fill_with_outline(chunk, &box_limit, false, 0, 3, j, 0, 4, j, &bar_ns, &bar_ns);
        //     inner.fill_with_outline(chunk, &box_limit, false, 10, 3, j, 10, 4, j, &bar_ns, &bar_ns);
        // }
        // for j in (2..9).step_by(2) {
        //     inner.fill_with_outline(chunk, &box_limit, false, j, 3, 15, j, 4, 15, &bar_we, &bar_we);
        // }

        // // 6. Stairs leading to Portal
        // let stairs_n = Block::STONE_BRICK_STAIRS.default_state.with("facing", "north");
        // inner.fill_outline_random(4, 1, 5, 6, 1, 7, &randomizer, chunk, false, random, &box_limit);
        // inner.fill_outline_random(4, 2, 6, 6, 2, 7, &randomizer, chunk, false, random, &box_limit);
        // inner.fill_outline_random(4, 3, 7, 6, 3, 7, &randomizer, chunk, false, random, &box_limit);

        // for k in 4..=6 {
        //     inner.add_block(chunk, &stairs_n, k, 1, 4, &box_limit);
        //     inner.add_block(chunk, &stairs_n, k, 2, 5, &box_limit);
        //     inner.add_block(chunk, &stairs_n, k, 3, 6, &box_limit);
        // }

        // 7. End Portal Frames and Eyes
        let eyes: [bool; 12] = std::array::from_fn(|_| random.next_f32() > 0.9);
        let all_eyes = eyes.iter().all(|&eye| eye);

        // let f_north = Block::END_PORTAL_FRAME.default_state.with("facing", "north");
        // let f_south = Block::END_PORTAL_FRAME.default_state.with("facing", "south");
        // let f_east = Block::END_PORTAL_FRAME.default_state.with("facing", "east");
        // let f_west = Block::END_PORTAL_FRAME.default_state.with("facing", "west");

        // Helper to stringify eye boolean for state mapping
        //let eye_str = |b: bool| if b { "true" } else { "false" };

        // inner.add_block(chunk, &f_north.with("eye", eye_str(eyes[0])), 4, 3, 8, &box_limit);
        // inner.add_block(chunk, &f_north.with("eye", eye_str(eyes[1])), 5, 3, 8, &box_limit);
        // inner.add_block(chunk, &f_north.with("eye", eye_str(eyes[2])), 6, 3, 8, &box_limit);

        // inner.add_block(chunk, &f_south.with("eye", eye_str(eyes[3])), 4, 3, 12, &box_limit);
        // inner.add_block(chunk, &f_south.with("eye", eye_str(eyes[4])), 5, 3, 12, &box_limit);
        // inner.add_block(chunk, &f_south.with("eye", eye_str(eyes[5])), 6, 3, 12, &box_limit);

        // inner.add_block(chunk, &f_east.with("eye", eye_str(eyes[6])), 3, 3, 9, &box_limit);
        // inner.add_block(chunk, &f_east.with("eye", eye_str(eyes[7])), 3, 3, 10, &box_limit);
        // inner.add_block(chunk, &f_east.with("eye", eye_str(eyes[8])), 3, 3, 11, &box_limit);

        // inner.add_block(chunk, &f_west.with("eye", eye_str(eyes[9])), 7, 3, 9, &box_limit);
        // inner.add_block(chunk, &f_west.with("eye", eye_str(eyes[10])), 7, 3, 10, &box_limit);
        // inner.add_block(chunk, &f_west.with("eye", eye_str(eyes[11])), 7, 3, 11, &box_limit);

        if all_eyes {
            let portal = Block::END_PORTAL.default_state;
            inner.fill_with_outline(chunk, &box_limit, false, 4, 3, 9, 6, 3, 11, portal, portal);
        }

        // 8. Spawner (Silverfish)
        if !self.spawner_placed {
            let pos = inner.offset_pos(5, 3, 6);
            if box_limit.contains_pos(&pos) {
                self.spawner_placed = true;
                let spawner = Block::SPAWNER.default_state;
                inner.add_block(chunk, spawner, 5, 3, 6, &box_limit);
                // In a real implementation, you'd add the TileEntityData for Silverfish here
            }
        }
    }

    fn clone_box(&self) -> Box<dyn StructurePieceBase> {
        Box::new((*self).clone())
    }
}
