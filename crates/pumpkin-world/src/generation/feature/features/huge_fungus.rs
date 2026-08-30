use crate::generation::proto_chunk::GenerationCache;
use pumpkin_data::{
    Block, BlockState,
    chunk::Biome,
    tag::{self, Taggable},
};
use pumpkin_util::{
    math::{position::BlockPos, vector3::Vector3},
    random::{RandomGenerator, RandomImpl},
};

#[derive(Clone)]
pub struct HugeFungusFeature;

impl HugeFungusFeature {
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
        let hat_state = if is_warped {
            Block::WARPED_WART_BLOCK.default_state
        } else {
            Block::NETHER_WART_BLOCK.default_state
        };
        let is_huge = random.next_f32() < 0.06;

        // Pack them into a config struct.
        let context = HugeFungusContext {
            hat_state,
            stem_state,
            total_height,
            is_huge,
            planted: false,
        };

        // Start the main generation logic.
        chunk.set_block_state(&pos.0, Block::AIR.default_state);
        self.generate_stem(chunk, pos, random, &context);
        self.generate_hat(chunk, pos, random, &context);

        true
    }

    /// Check is given position's block in replace list
    #[allow(clippy::unused_self)]
    fn is_replaceable<T: GenerationCache>(
        &self,
        chunk: &T,
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
            return block.has_tag(&tag::Block::MINECRAFT_REPLACEABLE);
        }

        false
    }

    /// Generate the stem of this current fungus.
    #[allow(clippy::unused_self)]
    fn generate_stem<T: GenerationCache>(
        &self,
        chunk: &mut T,
        pos: BlockPos,
        random: &mut RandomGenerator,
        ctx: &HugeFungusContext,
    ) {
        let stem_radius = i32::from(ctx.is_huge);

        for dx in -stem_radius..=stem_radius {
            for dz in -stem_radius..=stem_radius {
                let corner_of_huge_stem =
                    ctx.is_huge && dx.abs() == stem_radius && dz.abs() == stem_radius;
                for dy in 0..ctx.total_height {
                    let block_pos = pos.offset(Vector3::new(dx, dy, dz));
                    if !self.is_replaceable(chunk, &block_pos, true) {
                        continue;
                    }

                    if ctx.planted {
                        if !chunk.is_air(&block_pos.down().0) {
                            chunk.set_block_state(&block_pos.0, Block::AIR.default_state);
                        }

                        chunk.set_block_state(&block_pos.0, ctx.stem_state);
                    } else if corner_of_huge_stem {
                        if random.next_f32() < 0.1 {
                            chunk.set_block_state(&block_pos.0, ctx.stem_state);
                        }
                    } else {
                        chunk.set_block_state(&block_pos.0, ctx.stem_state);
                    }
                }
            }
        }
    }

    /// Generate the hat of this current fungus.
    #[allow(clippy::unused_self)]
    fn generate_hat<T: GenerationCache>(
        &self,
        chunk: &mut T,
        pos: BlockPos,
        random: &mut RandomGenerator,
        ctx: &HugeFungusContext,
    ) {
        let place_vines = ctx.hat_state == Block::NETHER_WART_BLOCK.default_state;
        let hat_height =
            (random.next_bounded_i32(1 + ctx.total_height / 3) + 5).min(ctx.total_height);
        let hat_start_y = ctx.total_height - hat_height;

        for y in hat_start_y..=ctx.total_height {
            let mut r = if hat_height > 8 && y < hat_start_y + 4 {
                3
            } else if y < ctx.total_height - random.next_bounded_i32(3) {
                2
            } else {
                1
            };

            if ctx.is_huge {
                r += 1;
            }

            for x in -r..=r {
                for z in -r..=r {
                    let offset = Vector3::new(x, y, z);
                    let conditions = HatPlaceCond::from_pos(
                        &offset,
                        r,
                        ctx.total_height,
                        hat_start_y,
                        place_vines,
                    );
                    let block_pos = pos.offset(offset);

                    // Check: Is the block replaceable?
                    if !self.is_replaceable(chunk, &block_pos, true) {
                        continue;
                    }

                    if ctx.planted && !chunk.is_air(&block_pos.down().0) {
                        chunk.set_block_state(&block_pos.0, Block::AIR.default_state);
                    }

                    if conditions.bottom && !conditions.inner {
                        self.generate_hat_drop_block(chunk, random, block_pos, ctx, place_vines);
                    } else {
                        // Basic config
                        let prob_cfg = HatProbConfig::from_conditions(&conditions);
                        self.generate_hat_block(chunk, random, block_pos, ctx, prob_cfg);
                    }
                }
            }
        }
    }

    #[allow(clippy::unused_self)]
    fn generate_hat_block<T: GenerationCache>(
        &self,
        chunk: &mut T,
        random: &mut RandomGenerator,
        pos: BlockPos,
        ctx: &HugeFungusContext,
        prob_config: HatProbConfig,
    ) {
        let HatProbConfig {
            decor: decor_prob,
            hat: hat_prob,
            vine: vine_prob,
        } = prob_config;

        // Set decor block in chance...
        if random.next_f32() < decor_prob {
            chunk.set_block_state(&pos.0, Block::SHROOMLIGHT.default_state);
        } else if random.next_f32() < hat_prob {
            chunk.set_block_state(&pos.0, ctx.hat_state);
            if random.next_f32() < vine_prob {
                self.try_place_weeping_vines(pos, chunk, random);
            }
        }
    }

    #[allow(clippy::unused_self)]
    fn generate_hat_drop_block<T: GenerationCache>(
        &self,
        chunk: &mut T,
        random: &mut RandomGenerator,
        pos: BlockPos,
        ctx: &HugeFungusContext,
        place_vines: bool,
    ) {
        let block_below = GenerationCache::get_block_state(chunk, &pos.down().0);
        if block_below == ctx.hat_state.id {
            chunk.set_block_state(&pos.0, ctx.hat_state);
        } else if random.next_f32() < 0.15 {
            chunk.set_block_state(&pos.0, ctx.hat_state);
            if place_vines && random.next_bounded_i32(11) == 0 {
                self.try_place_weeping_vines(pos, chunk, random);
            }
        }
    }

    #[allow(clippy::unused_self)]
    fn try_place_weeping_vines<T: GenerationCache>(
        &self,
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
                // Check: Is current block is replaceable
                if !GenerationCache::get_block_state(chunk, &current.0)
                    .to_block_id()
                    .has_tag(tag::Block::MINECRAFT_REPLACEABLE)
                {
                    break;
                }

                // Place the vine block in different positions...
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

/// Configurations of the huge fungus feature.
struct HugeFungusContext {
    /// The block state of the hat.
    pub hat_state: &'static BlockState,

    /// The block state of the stem.
    pub stem_state: &'static BlockState,

    /// The total height of the fungus.
    pub total_height: i32,

    /// Whether is this a huge fungus.
    pub is_huge: bool,

    /// Whether the hat block is planted by player.
    pub planted: bool,
}

/// Conditions of the hat block placement.
struct HatPlaceCond {
    /// Whether the hat block is placed in the edge of x-axis.
    pub edge_x: bool,

    /// Whether the hat block is placed in the edge of z-axis.
    pub edge_z: bool,

    /// Whether the hat block is placed in the inner corner.
    pub inner: bool,

    /// Whether the hat block is placed in the corner.
    pub corner: bool,

    /// Whether is the bottom block.
    pub bottom: bool,

    /// Whether the hat should place weeping vines.
    pub place_vines: bool,
}

impl HatPlaceCond {
    /// Create this conditions through x, y, z, and radius.
    pub const fn from_pos(
        pos: &Vector3<i32>,
        radius: i32,
        total_height: i32,
        hat_start_y: i32,
        place_vines: bool,
    ) -> Self {
        let edge_x = pos.x == -radius || pos.x == radius;
        let edge_z = pos.z == -radius || pos.z == radius;
        let inner = !edge_x && !edge_z && pos.y != total_height;
        let corner = edge_x && edge_z;
        let bottom = pos.y < hat_start_y + 3;
        Self {
            edge_x,
            edge_z,
            inner,
            corner,
            bottom,
            place_vines,
        }
    }
}

/// Configuration of the probability of the hat.
#[derive(Debug, Clone, Copy)]
struct HatProbConfig {
    /// Probability of placing a decor block.
    pub decor: f32,

    /// Probability of placing a hat block.
    pub hat: f32,

    /// Probability of placing weeping vines.
    pub vine: f32,
}

impl HatProbConfig {
    // Constants (from vanilla source code)
    /* Inner corner */
    /// Base probability of placing a decor block in the inner corner.
    const INNER_DECOR_PROB: f32 = 0.1;
    /// Base probability of placing a hat block in the inner corner.
    const INNER_HAT_PROB: f32 = 0.2;

    /* Corner */
    /// Base probability of placing a decor block in the corner.
    const CORNER_DECOR_PROB: f32 = 0.01;
    /// Base probability of placing a hat block in the corner.
    const CORNER_HAT_PROB: f32 = 0.7;

    /* Outer corner */
    /// Base probability of placing a decor block in the outer corner.
    const OUTER_DECOR_PROB: f32 = 5.0e-4;
    /// Base probability of placing a hat block in the outer corner.
    const OUTER_HAT_PROB: f32 = 0.98;

    /* Weeping vines */
    /// Probability of placing weeping vines in generic.
    const VINE_PROB: f32 = 0.0;
    /// Probability of placing weeping vines in the inner corner.
    const INNER_VINE_PROB: f32 = 0.1;
    /// Probability of placing weeping vines in the corner.
    const CORNER_VINE_PROB: f32 = 0.083;
    /// Probability of placing weeping vines in the outer corner.
    const OUTER_VINE_PROB: f32 = 0.07;

    /// Create this configuration through conditions.
    pub const fn from_conditions(conditions: &HatPlaceCond) -> Self {
        let base = match (conditions.inner, conditions.corner) {
            (true, _) => Self {
                decor: Self::INNER_DECOR_PROB,
                hat: Self::INNER_HAT_PROB,
                vine: Self::VINE_PROB,
            },
            (false, true) => Self {
                decor: Self::CORNER_DECOR_PROB,
                hat: Self::CORNER_HAT_PROB,
                vine: Self::VINE_PROB,
            },
            (false, false) => Self {
                decor: Self::OUTER_DECOR_PROB,
                hat: Self::OUTER_HAT_PROB,
                vine: Self::VINE_PROB,
            },
        };

        if conditions.place_vines {
            match (conditions.inner, conditions.corner) {
                (true, _) => Self {
                    vine: Self::INNER_VINE_PROB,
                    ..base
                },
                (false, true) => Self {
                    vine: Self::CORNER_VINE_PROB,
                    ..base
                },
                (false, false) => Self {
                    vine: Self::OUTER_VINE_PROB,
                    ..base
                },
            }
        } else {
            base
        }
    }
}
