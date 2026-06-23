use crate::biome::{BiomeSupplier, MultiNoiseBiomeSupplier, end::TheEndBiomeSupplier};
use crate::generation::biome_coords;
use crate::generation::generator::structure_finder::find_nearest_structure;
use crate::generation::noise::router::multi_noise_sampler::{
    MultiNoiseSampler, MultiNoiseSamplerBuilderOptions,
};
use pumpkin_data::dimension::Dimension;
use pumpkin_data::structures::StructurePlacement;
use pumpkin_util::math::position::BlockPos;

pub fn find_nearest_biome(
    world_gen: &crate::generation::generator::VanillaGenerator,
    dimension: Dimension,
    source_pos: BlockPos,
    biome_mask: &[bool; 256],
    min_y: i32,
    height: i32,
) -> Option<(BlockPos, f64)> {
    let px = source_pos.0.x;
    let py = source_pos.0.y;
    let pz = source_pos.0.z;

    let max_y = min_y + height - 1;

    let mut y_coords = [0i32; 64];
    let mut count = 0;
    let mut y = min_y;
    while y <= max_y && count < 64 {
        y_coords[count] = y;
        count += 1;
        y += 64;
    }
    let slice = &mut y_coords[..count];
    slice.sort_by_key(|&val| (val - py).abs());

    let overworld_supplier = MultiNoiseBiomeSupplier::OVERWORLD;
    let nether_supplier = MultiNoiseBiomeSupplier::NETHER;
    let end_supplier = TheEndBiomeSupplier;

    let base_supplier: &dyn BiomeSupplier = if dimension == Dimension::OVERWORLD {
        &overworld_supplier
    } else if dimension == Dimension::THE_NETHER {
        &nether_supplier
    } else if dimension == Dimension::THE_END {
        &end_supplier
    } else {
        &overworld_supplier
    };

    let multi_noise_config = MultiNoiseSamplerBuilderOptions::new(0, 0, 0);
    let mut multi_noise_sampler =
        MultiNoiseSampler::generate(&world_gen.base_router.multi_noise, &multi_noise_config);

    let mut best_match: Option<(BlockPos, f64)> = None;

    for r_step in 0..=200 {
        let r = r_step * 32;
        if let Some((_, best_d)) = best_match
            && r as f64 > best_d
        {
            break;
        }

        let min_limit = i32::MIN + r + 32;
        let max_limit = i32::MAX - r - 32;
        if px < min_limit || px > max_limit || pz < min_limit || pz > max_limit {
            break;
        }

        let perimeter_points = get_perimeter_points(px, pz, r);
        for (x, z) in perimeter_points {
            for &y in slice.iter() {
                let bx = biome_coords::from_block(x);
                let by = biome_coords::from_block(y);
                let bz = biome_coords::from_block(z);

                let sampled_biome = base_supplier.biome(bx, by, bz, &mut multi_noise_sampler);
                if biome_mask[sampled_biome.id as usize] {
                    let dx = x - px;
                    let dy = y - py;
                    let dz = z - pz;
                    let dist = ((dx * dx + dy * dy + dz * dz) as f64).sqrt();
                    if best_match.as_ref().is_none_or(|&(_, d)| dist < d) {
                        best_match = Some((BlockPos::new(x, y, z), dist));
                    }
                }
            }
        }
    }

    best_match
}

pub fn get_perimeter_points(px: i32, pz: i32, r: i32) -> impl Iterator<Item = (i32, i32)> {
    let min_limit = i32::MIN + r + 32;
    let max_limit = i32::MAX - r - 32;
    let safe = r >= 0 && px >= min_limit && px <= max_limit && pz >= min_limit && pz <= max_limit;

    let left = (0i32..)
        .map(move |i| if safe { pz - r + i * 32 } else { i32::MAX })
        .take_while(move |&z| safe && z <= pz + r)
        .flat_map(move |z| [(px - r, z), (px + r, z)]);

    let top_bottom = (0i32..)
        .map(move |i| if safe { px - r + 32 + i * 32 } else { i32::MAX })
        .take_while(move |&x| safe && x <= px + r - 32)
        .flat_map(move |x| [(x, pz - r), (x, pz + r)]);

    let origin = if safe && r == 0 { Some((px, pz)) } else { None }.into_iter();

    origin.chain(left).chain(top_bottom)
}

pub fn find_nearest_structure_pos(
    world_gen: &crate::generation::generator::VanillaGenerator,
    dimension: Dimension,
    source_pos: BlockPos,
    placements: &[&'static StructurePlacement],
    allowed_biomes_mask: [bool; 256],
) -> Option<BlockPos> {
    let overworld_supplier = MultiNoiseBiomeSupplier::OVERWORLD;
    let nether_supplier = MultiNoiseBiomeSupplier::NETHER;
    let end_supplier = TheEndBiomeSupplier;

    let base_supplier: &dyn BiomeSupplier = if dimension == Dimension::OVERWORLD {
        &overworld_supplier
    } else if dimension == Dimension::THE_NETHER {
        &nether_supplier
    } else if dimension == Dimension::THE_END {
        &end_supplier
    } else {
        &overworld_supplier
    };

    let multi_noise_config = MultiNoiseSamplerBuilderOptions::new(0, 0, 0);
    let mut multi_noise_sampler =
        MultiNoiseSampler::generate(&world_gen.base_router.multi_noise, &multi_noise_config);

    let world_seed = world_gen.random_config.seed as i64;
    find_nearest_structure(
        source_pos,
        placements,
        100,
        world_seed,
        &world_gen.global_structure_cache,
        |pos, _placement| {
            let bx = biome_coords::from_block(pos.0.x);
            let by = biome_coords::from_block(64);
            let bz = biome_coords::from_block(pos.0.z);
            let sampled_biome = base_supplier.biome(bx, by, bz, &mut multi_noise_sampler);
            allowed_biomes_mask[sampled_biome.id as usize]
        },
    )
}
