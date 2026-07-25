use pumpkin_data::{Block, chunk::Biome, tag};
use pumpkin_util::{math::position::BlockPos, random::RandomGenerator, random::RandomImpl};

use crate::generation::proto_chunk::GenerationCache;

pub struct HugeFungusFeature;

impl HugeFungusFeature {
    #[allow(clippy::unused_self)]
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        _min_y: i8,
        _height: u16,
        _feature: pumpkin_data::placed_feature::PlacedFeature,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        /* Here we'd like to check something about the
         * position, like if it's valid for a huge fungus to grow there.
         */
        let root_pos = pos;

        // Check: Is this generated in air? (Expected: yes)
        if !chunk.is_air(&root_pos.0) {
            return false;
        }

        // Check: Is the block below a valid ground (nylium or netherrack) (Expected: yes)?
        let below_state = GenerationCache::get_block_state(chunk, &root_pos.down().0);
        let is_valid_block = below_state
            .to_block_id()
            .has_tag(tag::Block::MINECRAFT_NYLIUM)
            || below_state.to_block_id().has_tag(tag::Block::C_NETHERRACKS);
        if !is_valid_block {
            return false;
        }

        /* Main generation logic */
        let height = random.next_bounded_i32(4) + 6;
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

        for i in 0..height {
            let stem_pos = BlockPos::new(pos.0.x, pos.0.y + i, pos.0.z);
            chunk.set_block_state(&stem_pos.0, stem_state);
        }

        let cap_y = pos.0.y + height - 2;
        for dy in 0..=3 {
            let radius = if dy == 0 || dy == 3 { 1 } else { 2 };
            for dx in -radius..=radius {
                for dz in -radius..=radius {
                    let cap_pos = BlockPos::new(pos.0.x + dx, cap_y + dy, pos.0.z + dz);
                    let block_state = if (dx == 0 || dz == 0) && dy == 1 && random.next_f32() < 0.3
                    {
                        Block::SHROOMLIGHT.default_state
                    } else {
                        wart_state
                    };
                    chunk.set_block_state(&cap_pos.0, block_state);
                }
            }
        }
        true
    }
}
