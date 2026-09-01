use crate::generation::proto_chunk::GenerationCache;
use pumpkin_data::block_properties::{BlockProperties, GrassBlockLikeProperties};
use pumpkin_data::tag;
use pumpkin_data::{Block, BlockId};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::random::RandomGenerator;

const SEA_LEVEL: i32 = 63; // TODO: getSeaLevel() from the worldgen context instead of hardcoding

pub struct FreezeTopLayerFeature;

impl FreezeTopLayerFeature {
    pub fn generate<T: GenerationCache>(
        chunk: &mut T,
        _min_y: i8,
        _height: u16,
        _feature_name: pumpkin_data::placed_feature::PlacedFeature,
        _random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let origin_x = pos.0.x;
        let origin_z = pos.0.z;

        for dx in 0..16i32 {
            for dz in 0..16i32 {
                let x = origin_x + dx;
                let z = origin_z + dz;

                let y = chunk.top_motion_blocking_block_height_exclusive(x, z);
                let below_y = y - 1;

                let top_vec = BlockPos::new(x, y, z).0;
                let below_vec = BlockPos::new(x, below_y, z).0;
                let below = GenerationCache::get_block_state(chunk, &below_vec);
                let below_block = below.to_block_id();

                let biome = chunk.get_biome_for_terrain_gen(x, y, z);

                // Freeze check
                if biome.weather.base_temperature() <= 0.15 && below_block == BlockId::WATER {
                    chunk.set_block_state(&below_vec, Block::ICE.default_state);
                }

                // Snow check
                let top_temp = biome
                    .weather
                    .compute_temperature(x as f64, y, z as f64, SEA_LEVEL);

                if top_temp < 0.15 {
                    let top_raw = GenerationCache::get_block_state(chunk, &top_vec);
                    // Re-read below (may have been replaced by ice in the freeze step above)

                    // topPos must be air; belowPos must not be air (something to stand on)
                    if top_raw.to_state().is_air()
                        && !below.to_state().is_air()
                        && !below_block.has_tag(tag::Block::MINECRAFT_CANNOT_SUPPORT_SNOW_LAYER)
                    {
                        chunk.set_block_state(&top_vec, Block::SNOW.default_state);

                        // Update the `snowy` block-state property on the block below if it has one
                        if GrassBlockLikeProperties::handles_block_id(below_block) {
                            let block = below_block.to_block();
                            let mut props = GrassBlockLikeProperties::from_state_id(below, block);
                            props.snowy = true;
                            chunk.set_block_state(&below_vec, props.to_state_id(block).to_state());
                        }
                    }
                }
            }
        }

        true
    }
}
