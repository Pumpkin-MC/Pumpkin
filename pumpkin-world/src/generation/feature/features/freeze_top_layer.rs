use crate::generation::proto_chunk::GenerationCache;
use pumpkin_data::{Block, BlockState};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::random::RandomGenerator;

const SEA_LEVEL: i32 = 63; // TODO: getSeaLevel() from the worldgen context instead of hardcoding

pub struct FreezeTopLayerFeature {}

impl FreezeTopLayerFeature {
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        _min_y: i8,
        _height: u16,
        _feature_name: &str,
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

                let biome = chunk.get_biome_for_terrain_gen(x, y, z);

                // Freeze check
                if biome.weather.base_temperature() <= 0.15
                    && GenerationCache::get_block_state(chunk, &below_vec).to_block_id()
                        == Block::WATER
                {
                    chunk.set_block_state(&below_vec, Block::ICE.default_state);
                }

                // Snow check
                let top_temp = biome
                    .weather
                    .compute_temperature(x as f64, y, z as f64, SEA_LEVEL);

                if top_temp < 0.15 {
                    let top_raw = GenerationCache::get_block_state(chunk, &top_vec);
                    // Re-read below (may have been replaced by ice in the freeze step above)
                    let below_raw = GenerationCache::get_block_state(chunk, &below_vec);

                    // topPos must be air; belowPos must not be air (something to stand on)
                    if top_raw.to_state().is_air() && !below_raw.to_state().is_air() {
                        chunk.set_block_state(&top_vec, Block::SNOW.default_state);

                        // Update the `snowy` block-state property on the block below if it has one
                        let below_block = below_raw.to_block();
                        if let Some(props) = below_block.properties(below_raw.0) {
                            let prop_list = props.to_props();
                            if prop_list.iter().any(|(k, _)| *k == "snowy") {
                                let new_props: Vec<(&str, &str)> = prop_list
                                    .iter()
                                    .map(|(k, v)| (*k, if *k == "snowy" { "true" } else { *v }))
                                    .collect();
                                let new_props_obj =
                                    below_block.from_properties(new_props.as_slice());
                                let new_state_id = new_props_obj.to_state_id(below_block);
                                chunk
                                    .set_block_state(&below_vec, BlockState::from_id(new_state_id));
                            }
                        }
                    }
                }
            }
        }

        true
    }
}
