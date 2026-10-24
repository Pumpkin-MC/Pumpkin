use pumpkin_data::{
    Block, BlockState,
    block_properties::{
        HorizontalFacing, LadderLikeProperties, OakFenceLikeProperties, WallTorchLikeProperties,
    },
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
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn get_structure_piece(&self) -> &StructurePiece {
        &self.piece.piece
    }

    fn get_structure_piece_mut(&mut self) -> &mut StructurePiece {
        &mut self.piece.piece
    }

    #[expect(clippy::too_many_lines)]
    fn place(
        &mut self,
        chunk: &mut ProtoChunk,
        _block_registry: &dyn crate::world::WorldPortalExt,
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
        let mut torch_props = WallTorchLikeProperties::default(&Block::WALL_TORCH);
        torch_props.facing = HorizontalFacing::East;
        let torch_e = BlockState::from_id(torch_props.to_state_id(&Block::WALL_TORCH));
        torch_props.facing = HorizontalFacing::West;
        let torch_w = BlockState::from_id(torch_props.to_state_id(&Block::WALL_TORCH));

        for l in 1..=13 {
            if (l - 1) % 4 == 0 {
                // Pillars
                inner.fill_with_outline(chunk, &box_limit, false, 1, 1, l, 1, 4, l, planks, planks);
                inner.fill_with_outline(
                    chunk, &box_limit, false, 12, 1, l, 12, 4, l, planks, planks,
                );
                // Torches
                inner.add_block(chunk, torch_e, 2, 3, l, &box_limit);
                inner.add_block(chunk, torch_w, 11, 3, l, &box_limit);

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
            let make_fence =
                |north: bool, south: bool, east: bool, west: bool| -> &'static BlockState {
                    let mut props = OakFenceLikeProperties::default(&Block::OAK_FENCE);
                    props.north = north;
                    props.south = south;
                    props.east = east;
                    props.west = west;
                    BlockState::from_id(props.to_state_id(&Block::OAK_FENCE))
                };

            let f_ns = make_fence(true, true, false, false);
            let f_ew = make_fence(false, false, true, true);

            inner.fill_with_outline(chunk, &box_limit, false, 3, 6, 3, 3, 6, 11, f_ns, f_ns);
            inner.fill_with_outline(chunk, &box_limit, false, 10, 6, 3, 10, 6, 9, f_ns, f_ns);
            inner.fill_with_outline(chunk, &box_limit, false, 4, 6, 2, 9, 6, 2, f_ew, f_ew);
            inner.fill_with_outline(chunk, &box_limit, false, 4, 6, 12, 7, 6, 12, f_ew, f_ew);

            // Complex fence junctions (matching the m-loop and specific offsets)
            inner.add_block(
                chunk,
                make_fence(true, false, true, false),
                3,
                6,
                2,
                &box_limit,
            );
            inner.add_block(
                chunk,
                make_fence(false, true, true, false),
                3,
                6,
                12,
                &box_limit,
            );
            inner.add_block(
                chunk,
                make_fence(true, false, false, true),
                10,
                6,
                2,
                &box_limit,
            );
            for m in 0..=2 {
                inner.add_block(
                    chunk,
                    make_fence(false, true, false, true),
                    8 + m,
                    6,
                    12 - m,
                    &box_limit,
                );
                if m != 2 {
                    inner.add_block(
                        chunk,
                        make_fence(true, false, true, false),
                        8 + m,
                        6,
                        11 - m,
                        &box_limit,
                    );
                }
            }

            // Ladder
            let mut props = LadderLikeProperties::default(&Block::LADDER);
            props.facing = HorizontalFacing::South;
            let ladder = BlockState::from_id(props.to_state_id(&Block::LADDER));
            for y in 1..=7 {
                inner.add_block(chunk, ladder, 10, y, 13, &box_limit);
            }

            // Chandelier (The central structure)
            let f_e = make_fence(false, false, true, false);
            let f_w = make_fence(false, false, false, true);
            inner.add_block(chunk, f_e, 6, 9, 7, &box_limit);
            inner.add_block(chunk, f_w, 7, 9, 7, &box_limit);
            inner.add_block(chunk, f_e, 6, 8, 7, &box_limit);
            inner.add_block(chunk, f_w, 7, 8, 7, &box_limit);

            let f_full = make_fence(true, true, true, true);
            inner.add_block(chunk, f_full, 6, 7, 7, &box_limit);
            inner.add_block(chunk, f_full, 7, 7, 7, &box_limit);

            inner.add_block(chunk, f_e, 5, 7, 7, &box_limit);
            inner.add_block(chunk, f_w, 8, 7, 7, &box_limit);
            inner.add_block(
                chunk,
                make_fence(true, false, true, false),
                6,
                7,
                6,
                &box_limit,
            );
            inner.add_block(
                chunk,
                make_fence(false, true, true, false),
                6,
                7,
                8,
                &box_limit,
            );
            inner.add_block(
                chunk,
                make_fence(true, false, false, true),
                7,
                7,
                6,
                &box_limit,
            );
            inner.add_block(
                chunk,
                make_fence(false, true, false, true),
                7,
                7,
                8,
                &box_limit,
            );

            // Chandelier Torches (Total 6 standing torches at Y=8)
            let t = Block::TORCH.default_state;
            inner.add_block(chunk, t, 5, 8, 7, &box_limit);
            inner.add_block(chunk, t, 8, 8, 7, &box_limit);
            inner.add_block(chunk, t, 6, 8, 6, &box_limit);
            inner.add_block(chunk, t, 6, 8, 8, &box_limit);
            inner.add_block(chunk, t, 7, 8, 6, &box_limit);
            inner.add_block(chunk, t, 7, 8, 8, &box_limit);
        }

        // 6. Chests
        inner.add_chest(
            chunk,
            &box_limit,
            random,
            3,
            3,
            5,
            "minecraft:chests/stronghold_library",
        );
        if self.tall {
            inner.add_block(chunk, Block::AIR.default_state, 12, 9, 1, &box_limit); // Clear space above top chest
            inner.add_chest(
                chunk,
                &box_limit,
                random,
                12,
                8,
                1,
                "minecraft:chests/stronghold_library",
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generation::get_world_gen;
    use crate::world::{BlockAccessor, WorldPortalExt};
    use pumpkin_data::BlockId;
    use pumpkin_data::dimension::Dimension;
    use pumpkin_util::math::position::BlockPos;
    use pumpkin_util::random::legacy_rand::LegacyRand;
    use pumpkin_util::world_seed::Seed;

    struct DummyPortalExt;
    impl WorldPortalExt for DummyPortalExt {
        fn can_place_at(
            &self,
            _block: &Block,
            _state: &BlockState,
            _block_accessor: &dyn BlockAccessor,
            _block_pos: &BlockPos,
        ) -> bool {
            true
        }

        fn mirror(
            &self,
            _block: &Block,
            state_id: pumpkin_data::BlockStateId,
            mirror: pumpkin_data::Mirror,
        ) -> &'static BlockState {
            BlockState::from_id(state_id).mirror(mirror)
        }

        fn rotate(
            &self,
            _block: &Block,
            state_id: pumpkin_data::BlockStateId,
            rotation: pumpkin_data::Rotation,
        ) -> &'static BlockState {
            BlockState::from_id(state_id).rotate(rotation)
        }

        fn spawn_mobs_for_chunk_generation(
            &self,
            _cache: &mut dyn crate::generation::proto_chunk::GenerationCache,
            _biome: &'static pumpkin_data::biome::Biome,
            _chunk_x: i32,
            _chunk_z: i32,
        ) {
        }
    }

    #[test]
    fn tall_library_places_torches_and_fences() {
        let world_gen = get_world_gen(
            Seed(0),
            Dimension::OVERWORLD,
            false,
            Vec::new(),
            String::new(),
        );
        let mut chunk = ProtoChunk::new(0, 0, &world_gen);
        let chunk_box = BlockBox::new(0, 0, 0, 15, 127, 15);
        let mut random = RandomGenerator::Legacy(LegacyRand::from_seed(12345));

        let mut piece = StrongholdPiece::new(
            StructurePieceType::StrongholdLibrary,
            0,
            BlockBox::new(0, 0, 0, 13, 10, 14),
        );
        piece.piece.set_facing(Some(BlockDirection::North));
        let mut library = LibraryPiece { piece, tall: true };

        library.place(&mut chunk, &DummyPortalExt, &mut random, 0, &chunk_box);

        // Verify pillar wall torches
        let torch_e = library
            .piece
            .piece
            .get_block_at(&chunk, 2, 3, 1, &chunk_box);
        assert_eq!(torch_e.id.to_block_id(), BlockId::WALL_TORCH);
        let props_e = WallTorchLikeProperties::from_state_id(torch_e.id);
        assert_eq!(props_e.facing, HorizontalFacing::East);

        let torch_w = library
            .piece
            .piece
            .get_block_at(&chunk, 11, 3, 1, &chunk_box);
        assert_eq!(torch_w.id.to_block_id(), BlockId::WALL_TORCH);
        let props_w = WallTorchLikeProperties::from_state_id(torch_w.id);
        assert_eq!(props_w.facing, HorizontalFacing::West);

        // Verify all 6 chandelier torches at Y=8
        let torch_coords = [
            (5, 8, 7),
            (8, 8, 7),
            (6, 8, 6),
            (6, 8, 8),
            (7, 8, 6),
            (7, 8, 8),
        ];
        for (x, y, z) in torch_coords {
            let torch = library
                .piece
                .piece
                .get_block_at(&chunk, x, y, z, &chunk_box);
            assert_eq!(
                torch.id.to_block_id(),
                BlockId::TORCH,
                "Expected standing torch at ({x}, {y}, {z})"
            );
        }

        // Verify chandelier fences
        let fence_6_9 = library
            .piece
            .piece
            .get_block_at(&chunk, 6, 9, 7, &chunk_box);
        assert_eq!(fence_6_9.id.to_block_id(), BlockId::OAK_FENCE);
        let props = OakFenceLikeProperties::from_state_id(fence_6_9.id);
        assert!(props.east);

        let fence_7_9 = library
            .piece
            .piece
            .get_block_at(&chunk, 7, 9, 7, &chunk_box);
        assert_eq!(fence_7_9.id.to_block_id(), BlockId::OAK_FENCE);
        let props = OakFenceLikeProperties::from_state_id(fence_7_9.id);
        assert!(props.west);

        let fence_center = library
            .piece
            .piece
            .get_block_at(&chunk, 6, 7, 7, &chunk_box);
        assert_eq!(fence_center.id.to_block_id(), BlockId::OAK_FENCE);
        let props = OakFenceLikeProperties::from_state_id(fence_center.id);
        assert!(props.north && props.south && props.east && props.west);
    }

    #[test]
    fn short_library_places_pillar_torches_only() {
        let world_gen = get_world_gen(
            Seed(0),
            Dimension::OVERWORLD,
            false,
            Vec::new(),
            String::new(),
        );
        let mut chunk = ProtoChunk::new(0, 0, &world_gen);
        let chunk_box = BlockBox::new(0, 0, 0, 15, 127, 15);
        let mut random = RandomGenerator::Legacy(LegacyRand::from_seed(54321));

        let mut piece = StrongholdPiece::new(
            StructurePieceType::StrongholdLibrary,
            0,
            BlockBox::new(0, 0, 0, 13, 5, 14),
        );
        piece.piece.set_facing(Some(BlockDirection::North));
        let mut library = LibraryPiece { piece, tall: false };

        library.place(&mut chunk, &DummyPortalExt, &mut random, 0, &chunk_box);

        // Pillar wall torches must still be placed
        let torch_e = library
            .piece
            .piece
            .get_block_at(&chunk, 2, 3, 1, &chunk_box);
        assert_eq!(torch_e.id.to_block_id(), BlockId::WALL_TORCH);
        let torch_w = library
            .piece
            .piece
            .get_block_at(&chunk, 11, 3, 1, &chunk_box);
        assert_eq!(torch_w.id.to_block_id(), BlockId::WALL_TORCH);

        // Chandelier torches at Y=8 must NOT be placed
        let torch_upper = library
            .piece
            .piece
            .get_block_at(&chunk, 5, 8, 7, &chunk_box);
        assert_ne!(torch_upper.id.to_block_id(), BlockId::TORCH);
    }
}
