use pumpkin_data::chunk::Biome;
use pumpkin_util::math::vector3::Vector3;

use crate::{
    biome::BiomeSupplier,
    dimension::Dimension,
    generation::{
        biome_coords, noise_router::multi_noise_sampler::MultiNoiseSampler, section_coords,
    },
};

pub struct TheEndBiomeSupplier;

impl TheEndBiomeSupplier {
    const CENTER_BIOME: Biome = Biome::THE_END;
    const HIGHLANDS_BIOME: Biome = Biome::END_HIGHLANDS;
    const MIDLANDS_BIOME: Biome = Biome::END_MIDLANDS;
    const SMALL_ISLANDS_BIOME: Biome = Biome::SMALL_END_ISLANDS;
    const BARRENS_BIOME: Biome = Biome::END_BARRENS;
}

impl BiomeSupplier for TheEndBiomeSupplier {
    fn biome(
        global_biome_pos: &Vector3<i32>,
        noise: &mut MultiNoiseSampler<'_>,
        _dimension: Dimension,
    ) -> &'static Biome {
        let x = biome_coords::to_block(global_biome_pos.x);
        let y = biome_coords::to_block(global_biome_pos.y);
        let z = biome_coords::to_block(global_biome_pos.z);
        let section_x = section_coords::block_to_section(x);
        let section_z = section_coords::block_to_section(z);
        if section_x * section_x + section_z * section_z <= 4096 {
            return &Self::CENTER_BIOME;
        }
        let x = (section_x * 2 + 1) * 8;
        let z = (section_z * 2 + 1) * 8;
        let noise = noise.sample_erosion(x, y, z);
        if noise > 0.25 {
            return &Self::HIGHLANDS_BIOME;
        }
        if noise >= -0.0625 {
            return &Self::MIDLANDS_BIOME;
        }
        if noise < -0.21875 {
            return &Self::SMALL_ISLANDS_BIOME;
        }

        &Self::BARRENS_BIOME
    }
}
