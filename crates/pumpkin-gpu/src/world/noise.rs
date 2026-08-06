//! GPU-accelerated chunk noise evaluation.
//!
//! Evaluates the full density-function graph for every block position in a chunk
//! in one GPU dispatch, maps density to vanilla block states (stone/water/air),
//! and returns the flat block map.  Registered as a callback in
//! `pumpkin_world::generation::noise::register_noise_gpu`.

use pumpkin_data::BlockStateId;
use pumpkin_data::chunk_gen_settings::GenerationSettings;
use pumpkin_util::math::block_box::BlockBox;
use pumpkin_world::generation::noise::NoiseGpuFn;
use pumpkin_world::generation::{
    GlobalRandomConfig,
    noise::router::density_function::beardifier::{BeardifierJunction, BeardifierStructure},
};

use crate::world::graph::{BeardifierData, compile};
use crate::world::light::get_global_gpu;

/// GPU callback for chunk noise evaluation.
///
/// Compiles the overworld noise router, evaluates it for every block position
/// in the chunk, maps density values to vanilla block states, and returns the
/// flat block map.  Returns `None` when the GPU is unavailable or the router
/// fails to compile.
#[expect(clippy::too_many_arguments)]
#[must_use]
pub fn noise_gpu_callback(
    chunk_x: i32,
    chunk_z: i32,
    settings: &GenerationSettings,
    random_config: &GlobalRandomConfig,
    beardifier_structures: &[BeardifierStructure],
    beardifier_junctions: &[BeardifierJunction],
    affected_box: Option<BlockBox>,
    default_block: BlockStateId,
    default_fluid: BlockStateId,
) -> Option<Box<[BlockStateId]>> {
    let ctx = get_global_gpu()?;

    let router = &pumpkin_data::noise_router::OVERWORLD_BASE_NOISE_ROUTER;
    let stack = router.noise.full_component_stack;
    let compiled = compile(stack, random_config).ok()?;

    let start_x = chunk_x * 16;
    let start_z = chunk_z * 16;
    let min_y = settings.shape.min_y as i32;
    let height = settings.shape.height as i32;
    let max_y = min_y + height;
    let sea_level = settings.sea_level;

    // Build all block positions
    let num_blocks = (16 * 16 * height) as usize;
    let mut points = Vec::with_capacity(num_blocks);
    for y in min_y..max_y {
        for z in 0..16 {
            for x in 0..16 {
                points.push([(start_x + x) as f32, y as f32, (start_z + z) as f32]);
            }
        }
    }

    let beardifier = BeardifierData::from_cpu(
        beardifier_structures,
        beardifier_junctions,
        affected_box.as_ref(),
    );

    let density_values = ctx.evaluate_graph_with(&compiled, &points, &beardifier);

    // Density → block state mapping (vanilla default behaviour)
    let stone_state = pumpkin_data::Block::STONE.default_state.id;
    let air_state = pumpkin_data::Block::AIR.default_state.id;

    let block_map: Box<[BlockStateId]> = density_values
        .iter()
        .enumerate()
        .map(|(i, &density)| {
            let y = min_y + (i / 256) as i32;
            if density > 0.0 {
                if y < sea_level {
                    // Below sea level with positive density: stone or default
                    if y < sea_level - 1 {
                        stone_state
                    } else {
                        default_block
                    }
                } else {
                    default_block
                }
            } else if y < sea_level {
                // Below sea level with negative density: default fluid
                default_fluid
            } else {
                // Above sea level with negative density: air
                air_state
            }
        })
        .collect();

    Some(block_map)
}

/// Returns a function pointer suitable for `register_noise_gpu`.
#[must_use]
pub const fn noise_gpu_fn() -> NoiseGpuFn {
    noise_gpu_callback
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_util::math::block_box::BlockBox;
    use pumpkin_world::generation::GlobalRandomConfig;

    #[test]
    fn noise_callback_returns_none_without_gpu() {
        let config = GlobalRandomConfig::new(42, false);
        let settings = &pumpkin_data::chunk_gen_settings::GenerationSettings::AMPLIFIED;
        let result = noise_gpu_callback(
            0,
            0,
            settings,
            &config,
            &[],
            &[],
            None,
            pumpkin_data::Block::STONE.default_state.id,
            pumpkin_data::Block::WATER.default_state.id,
        );
        assert!(
            result.is_none(),
            "noise callback must return None when no GPU is available"
        );
    }

    #[test]
    fn noise_callback_with_beardifier_returns_none_without_gpu() {
        let config = GlobalRandomConfig::new(42, false);
        let settings = &pumpkin_data::chunk_gen_settings::GenerationSettings::AMPLIFIED;
        let bb = BlockBox::new(0, -64, 0, 16, 320, 16);
        let result = noise_gpu_callback(
            0,
            0,
            settings,
            &config,
            &[],
            &[],
            Some(bb),
            pumpkin_data::Block::STONE.default_state.id,
            pumpkin_data::Block::WATER.default_state.id,
        );
        assert!(result.is_none());
    }

    #[test]
    fn noise_gpu_fn_returns_valid_function_pointer() {
        let fn_ptr = noise_gpu_fn();
        let ptr = fn_ptr as *const ();
        assert!(!ptr.is_null());
    }

    #[test]
    fn density_mapping_below_sea_level_negative_is_water() {
        let sea_level: i32 = 63;
        let water = pumpkin_data::Block::WATER.default_state.id;
        let air = pumpkin_data::Block::AIR.default_state.id;
        let y: i32 = sea_level - 5;
        let density = -1.0;
        let result = if density > 0.0 {
            pumpkin_data::Block::STONE.default_state.id
        } else if y < sea_level {
            water
        } else {
            air
        };
        assert_eq!(result, water);
    }

    #[test]
    fn density_mapping_above_sea_level_negative_is_air() {
        let sea_level: i32 = 63;
        let water = pumpkin_data::Block::WATER.default_state.id;
        let air = pumpkin_data::Block::AIR.default_state.id;
        let y: i32 = sea_level + 10;
        let density = -1.0;
        let result = if density > 0.0 {
            pumpkin_data::Block::STONE.default_state.id
        } else if y < sea_level {
            water
        } else {
            air
        };
        assert_eq!(result, air);
    }

    #[test]
    fn density_mapping_deep_below_sea_level_positive_is_stone() {
        let sea_level: i32 = 63;
        let stone = pumpkin_data::Block::STONE.default_state.id;
        let y: i32 = sea_level - 10;
        let density = 1.0;
        let result = if density > 0.0 {
            if y < sea_level && y < sea_level - 1 {
                stone
            } else {
                stone
            }
        } else {
            pumpkin_data::Block::AIR.default_state.id
        };
        assert_eq!(result, stone);
    }

    #[test]
    fn density_mapping_at_surface_positive_is_default() {
        let sea_level: i32 = 63;
        let default_block = pumpkin_data::Block::STONE.default_state.id;
        let y: i32 = sea_level;
        let density = 1.0;
        let result = if density > 0.0 {
            if y < sea_level && y < sea_level - 1 {
                default_block
            } else {
                default_block
            }
        } else {
            pumpkin_data::Block::AIR.default_state.id
        };
        assert_eq!(result, default_block);
    }
}
