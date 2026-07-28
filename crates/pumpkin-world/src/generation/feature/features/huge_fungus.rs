use crate::generation::proto_chunk::GenerationCache;
use pumpkin_data::{Block, BlockState, chunk::Biome, tag};
use pumpkin_util::{
    math::{position::BlockPos, vector3::Vector3},
    random::{RandomGenerator, RandomImpl},
};

pub struct HugeFungusFeature;

impl HugeFungusFeature {
    /// Replaceable block list
    const REPLACEABLE_BLOCKS: &[Block] = &[
        Block::OAK_SAPLING,
        Block::SPRUCE_SAPLING,
        Block::BIRCH_SAPLING,
        Block::JUNGLE_SAPLING,
        Block::ACACIA_SAPLING,
        Block::CHERRY_SAPLING,
        Block::DARK_OAK_SAPLING,
        Block::PALE_OAK_SAPLING,
        Block::MANGROVE_PROPAGULE,
        Block::DANDELION,
        Block::TORCHFLOWER,
        Block::POPPY,
        Block::BLUE_ORCHID,
        Block::ALLIUM,
        Block::AZURE_BLUET,
        Block::RED_TULIP,
        Block::ORANGE_TULIP,
        Block::WHITE_TULIP,
        Block::PINK_TULIP,
        Block::OXEYE_DAISY,
        Block::CORNFLOWER,
        Block::WITHER_ROSE,
        Block::LILY_OF_THE_VALLEY,
        Block::BROWN_MUSHROOM,
        Block::RED_MUSHROOM,
        Block::WHEAT,
        Block::SUGAR_CANE,
        Block::ATTACHED_PUMPKIN_STEM,
        Block::ATTACHED_MELON_STEM,
        Block::PUMPKIN_STEM,
        Block::MELON_STEM,
        Block::LILY_PAD,
        Block::NETHER_WART,
        Block::COCOA,
        Block::CARROTS,
        Block::POTATOES,
        Block::CHORUS_PLANT,
        Block::CHORUS_FLOWER,
        Block::TORCHFLOWER_CROP,
        Block::PITCHER_CROP,
        Block::BEETROOTS,
        Block::SWEET_BERRY_BUSH,
        Block::WARPED_FUNGUS,
        Block::CRIMSON_FUNGUS,
        Block::WEEPING_VINES,
        Block::WEEPING_VINES_PLANT,
        Block::TWISTING_VINES,
        Block::TWISTING_VINES_PLANT,
        Block::CAVE_VINES,
        Block::CAVE_VINES_PLANT,
        Block::SPORE_BLOSSOM,
        Block::AZALEA,
        Block::FLOWERING_AZALEA,
        Block::MOSS_CARPET,
        Block::PINK_PETALS,
        Block::WILDFLOWERS,
        Block::BIG_DRIPLEAF,
        Block::BIG_DRIPLEAF_STEM,
        Block::SMALL_DRIPLEAF,
    ];

    #[allow(clippy::unused_self)]
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        _min_y: i8,
        height: u16,
        _feature: pumpkin_data::placed_feature::PlacedFeature,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        /* Here we'd like to check something about the
         * position, like if it's valid for a huge fungus to grow there.
         */

        // Check: Is this generated in a warped/crimson forest? (Expected: yes)
        let is_warped = {
            // Get current biome
            let biome = chunk.get_biome_for_terrain_gen(pos.0.x, pos.0.y, pos.0.z);

            // And match...
            match biome {
                b if b == &Biome::CRIMSON_FOREST => false,
                b if b == &Biome::WARPED_FOREST => true,
                _ => return false, // Should not be generated if not in those biome
            }
        };

        // Check: Is the block below a valid ground (nylium or netherrack) (Expected: yes)?
        let below_state = GenerationCache::get_block_state(chunk, &pos.down().0);
        let is_valid_block = below_state
            .to_block_id()
            .has_tag(tag::Block::MINECRAFT_NYLIUM);
        if !is_valid_block {
            return false;
        }

        /* Main generation logic */
        // Get the height of the fungus (vanilla range: 4-13 blocks, might double in 1/12 chance)
        let total_height = {
            let first_height = random.next_inbetween_i32(4, 13);
            if random.next_bounded_i32(12) == 0 {
                first_height * 2
            } else {
                first_height
            }
        };

        // TODO: In vanilla, it will pass us a config which tells us is this manual or native generation.
        // TODO: Current it is no, so here we check the height directly.
        // Check: Is computed height out of given max height?
        if pos.0.y + total_height + 1 >= height as i32 {
            return false;
        }

        // Get the stem and wart block states
        let stem_state = if is_warped {
            Block::WARPED_STEM.default_state
        } else {
            Block::CRIMSON_STEM.default_state
        };
        let wart_state = if is_warped {
            Block::WARPED_WART_BLOCK.default_state
        } else {
            Block::NETHER_WART_BLOCK.default_state
        };
        let is_huge = random.next_f32() < 0.06;

        // Start the main generation logic.
        chunk.set_block_state(&pos.0, Block::AIR.default_state);
        self.generate_stem(chunk, pos, stem_state, total_height, random, is_huge, false);
        self.generate_hat(chunk, pos, wart_state, total_height, random, is_huge, false);

        true
    }

    /// Check is given position's block in replace list
    fn is_replaceable<T: GenerationCache>(
        &self,
        chunk: &mut T,
        pos: &BlockPos,
        check_non_replaceable_plants: bool,
    ) -> bool {
        let block_state = GenerationCache::get_block_state(chunk, &pos.0);
        let block = block_state.to_block();

        // TODO: Attend a field `is_replaceable` to check if the block is replaceable.
        // In vanilla, only `air` is assigned as replaceable. So here we just check it is air or not.
        if block.is_air() {
            return true;
        }

        if check_non_replaceable_plants {
            return Self::REPLACEABLE_BLOCKS.contains(&block);
        }

        false
    }

    /// Generate the stem of this current fungus.
    fn generate_stem<T: GenerationCache>(
        &self,
        chunk: &mut T,
        pos: BlockPos,
        stem_state: &BlockState,
        total_height: i32,
        random: &mut RandomGenerator,
        is_huge: bool,
        planted: bool,
    ) {
        let stem_radius: i32 = if is_huge { 1 } else { 0 };

        for dx in -stem_radius..=stem_radius {
            for dz in -stem_radius..=stem_radius {
                let corner_of_huge_stem =
                    is_huge && dx.abs() == stem_radius && dz.abs() == stem_radius;
                for dy in 0..total_height {
                    let block_pos = pos.offset(Vector3::new(dx, dy as i32, dz));
                    if self.is_replaceable(chunk, &block_pos, true) {
                        if planted {
                            if !chunk.is_air(&block_pos.down().0) {
                                chunk.set_block_state(&block_pos.0, Block::AIR.default_state);
                            }

                            chunk.set_block_state(&block_pos.0, stem_state);
                        } else if corner_of_huge_stem {
                            if random.next_f32() < 0.1 {
                                chunk.set_block_state(&block_pos.0, stem_state);
                            }
                        } else {
                            chunk.set_block_state(&block_pos.0, stem_state);
                        }
                    }
                }
            }
        }
    }

    /// Generate the hat of this current fungus.
    fn generate_hat<T: GenerationCache>(
        &self,
        chunk: &mut T,
        pos: BlockPos,
        hat_state: &BlockState,
        total_height: i32,
        random: &mut RandomGenerator,
        is_huge: bool,
        planted: bool,
    ) {
        let place_vines = hat_state == Block::NETHER_WART_BLOCK.default_state;
        let hat_height = (random.next_bounded_i32(1 + total_height / 3) + 5).min(total_height);
        let hat_start_y = total_height - hat_height;

        for y in hat_start_y..=total_height {
            let mut r = if y < total_height - random.next_bounded_i32(3) {
                2
            } else {
                1
            };
            if hat_height > 8 && y < hat_start_y + 4 {
                r = 3;
            }

            if is_huge {
                r += 1;
            }

            for x in -r..=r {
                for z in -r..=r {
                    let edge_x = x == -r || x == r;
                    let edge_z = z == -r || z == r;
                    let inner = !edge_x && !edge_z && y != total_height;
                    let corner = edge_x && edge_z;
                    let bottom = y < hat_start_y + 3;
                    let block_pos = pos.offset(Vector3::new(x, y as i32, z));
                    if self.is_replaceable(chunk, &block_pos, true) {
                        if planted && !chunk.is_air(&block_pos.down().0) {
                            chunk.set_block_state(&block_pos.0, Block::AIR.default_state);
                        }

                        if bottom {
                            if !inner {
                                self.generate_hat_drop_block(chunk, random, block_pos, hat_state, place_vines);
                            }
                        } else if inner {
                            let vine_prob = if place_vines { 0.1 } else { 0.0 };
                            self.generate_hat_block(
                                chunk,
                                random,
                                block_pos,
                                hat_state,
                                0.1,
                                0.2,
                                vine_prob,
                            );
                        } else if corner {
                            let vine_prob = if place_vines { 0.083 } else { 0.0 };
                            self.generate_hat_block(
                                chunk,
                                random,
                                block_pos,
                                hat_state,
                                0.01,
                                0.7,
                                vine_prob,
                            );
                        } else {
                            let vine_prob = if place_vines { 0.07 } else { 0.0 };
                            self.generate_hat_block(
                                chunk,
                                random,
                                block_pos,
                                hat_state,
                                5.0e-4,
                                0.98,
                                vine_prob,
                            );
                        }
                    }
                }
            }
        }
    }

    fn generate_hat_block<T: GenerationCache>(
        &self,
        chunk: &mut T,
        random: &mut RandomGenerator,
        pos: BlockPos,
        hat_state: &BlockState,
        decor_prob: f32,
        hat_prob: f32,
        vine_prob: f32,
    ) {
        // Set decor block in chance...
        if random.next_f32() < decor_prob {
            chunk.set_block_state(&pos.0, Block::SHROOMLIGHT.default_state);
        } else if random.next_f32() < hat_prob {
            chunk.set_block_state(&pos.0, hat_state);
            if random.next_f32() < vine_prob {
                Self::try_place_weeping_vines(pos, chunk, random);
            }
        }
    }

    fn generate_hat_drop_block<T: GenerationCache>(
        &self,
        chunk: &mut T,
        random: &mut RandomGenerator,
        pos: BlockPos,
        hat_state: &BlockState,
        place_vines: bool,
    ) {
        let block_below = GenerationCache::get_block_state(chunk, &pos.down().0);
        if block_below == hat_state.id {
            chunk.set_block_state(&pos.0, hat_state);
        } else if random.next_f32() < 0.15 {
            chunk.set_block_state(&pos.0, hat_state);
            if place_vines && random.next_bounded_i32(11) == 0 {
                Self::try_place_weeping_vines(pos, chunk, random);
            }
        }
    }

    fn try_place_weeping_vines<T: GenerationCache>(
        hat_block_pos: BlockPos,
        chunk: &mut T,
        random: &mut RandomGenerator,
    ) {
        let vine_pos = hat_block_pos.down();
        if chunk.is_air(&vine_pos.0) {
            let mut vine_length = random.next_inbetween_i32(1, 5);
            if random.next_bounded_i32(7) == 0 {
                vine_length *= 2;
            }

            // Simplified weeping vine column placement; vanilla uses age 23-25.
            // We place WEEPING_VINES_PLANT for the stem, and WEEPING_VINES at the tip.
            for i in 0..vine_length {
                let current = vine_pos.offset(Vector3::new(0, -i, 0));
                if i == vine_length - 1 {
                    // Tip of the vine
                    chunk.set_block_state(&current.0, Block::WEEPING_VINES.default_state);
                } else {
                    // Stem piece
                    chunk.set_block_state(&current.0, Block::WEEPING_VINES_PLANT.default_state);
                }
            }
        }
    }
}