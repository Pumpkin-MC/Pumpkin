use crate::{
    biome::Biome, biome::BiomeSupplier,
    generation::noise::router::multi_noise_sampler::MultiNoiseSampler,
};
use pumpkin_data::dimension::Dimension;
use pumpkin_util::math::position::BlockPos;

// NOT 100% parity with vanilla (Doesn't use "iterateInSquare")
pub fn find_nearest_biome(
    origin: BlockPos,
    target: &'static Biome,
    max_radius: i32,
    supplier: &impl BiomeSupplier,
    sampler: &mut MultiNoiseSampler<'_>,
    dim: &Dimension,
) -> Option<BlockPos> {
    const H_INTERVAL: i32 = 32;
    const V_INTERVAL: i32 = 64;
    let max_radius_cells = max_radius / H_INTERVAL;
    let top_y = dim.min_y + dim.height - 1;

    // This probably could be optimized to just use 2 for loops
    for radius in 0..=max_radius_cells {
        for dx in -radius..=radius {
            for dz in -radius..=radius {
                if dx.abs() != radius && dz.abs() != radius {
                    continue; // skip interior, keep ring edge only
                }
                // block coords: origin + offset*interval
                let bx = origin.0.x + dx * H_INTERVAL;
                let bz = origin.0.z + dz * H_INTERVAL;

                let mut by = dim.min_y;
                while by <= top_y {
                    if supplier.biome(bx >> 2, by >> 2, bz >> 2, sampler) == target {
                        return Some(BlockPos::new(bx, by, bz));
                    }

                    by += V_INTERVAL;
                }
            }
        }
    }

    None
}
