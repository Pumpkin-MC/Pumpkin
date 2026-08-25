pub use pumpkin_data::chunk::{
    Parameter, ParameterPoint, ParameterRange, TargetPoint, quantize_coord, unquantize_coord,
};

#[must_use]
pub const fn to_long(float: f32) -> i64 {
    quantize_coord(float)
}

pub type NoiseValuePoint = TargetPoint;

#[cfg(test)]
mod test {
    use pumpkin_data::dimension::Dimension;
    use pumpkin_util::read_data_from_file;

    use crate::ProtoChunk;

    #[test]
    fn sample_value() {
        use crate::generation::generator::{GeneratorInit, VanillaGenerator, WorldGenerator};
        use crate::generation::noise::router::multi_noise_sampler::{
            MultiNoiseSampler, MultiNoiseSamplerBuilderOptions,
        };
        use crate::generation::{biome_coords, positions::chunk_pos};
        use pumpkin_util::world_seed::Seed;
        type PosToPoint = (i32, i32, i32, i64, i64, i64, i64, i64, i64);
        let expected_data: Vec<PosToPoint> = read_data_from_file!(
            "../../../../assets/multi_noise_sample_no_blend_no_beard_0_0_0.json"
        );

        let seed = 0;
        let chunk_x = 0;
        let chunk_z = 0;

        let generator = Box::new(VanillaGenerator::new(
            Seed(seed as u64),
            Dimension::OVERWORLD,
        ));
        let world_gen = WorldGenerator::Noise(generator);
        let WorldGenerator::Noise(generator) = &world_gen else {
            unreachable!()
        };

        let _chunk = ProtoChunk::new(chunk_x, chunk_z, &world_gen);

        let start_x = chunk_pos::start_block_x(chunk_x);
        let start_z = chunk_pos::start_block_z(chunk_z);
        let horizontal_biome_end = biome_coords::from_block(16);
        let multi_noise_config = MultiNoiseSamplerBuilderOptions::new(
            biome_coords::from_block(start_x),
            biome_coords::from_block(start_z),
            horizontal_biome_end as usize,
        );
        let mut multi_noise_sampler =
            MultiNoiseSampler::generate(&generator.base_router.multi_noise, &multi_noise_config);

        for (x, y, z, tem, hum, con, ero, dep, wei) in expected_data {
            let point = multi_noise_sampler.sample(x, y, z);
            assert_eq!(point.temperature, tem);
            assert_eq!(point.humidity, hum);
            assert_eq!(point.continentalness, con);
            assert_eq!(point.erosion, ero);
            assert_eq!(point.depth, dep);
            assert_eq!(point.weirdness, wei);
        }
    }

    // #[test]
    // fn sample_multinoise_biome() {
    //     use crate::generation::generator::{GeneratorInit, VanillaGenerator};
    //     use pumpkin_util::world_seed::Seed;

    //     let expected_data: Vec<(i32, i32, i32, u8)> =
    //         read_data_from_file!("../../../assets/multi_noise_biome_source_test.json");

    //     let seed = 0;
    //     let generator = VanillaGenerator::new(Seed(seed as u64), Dimension::OVERWORLD);

    //     let mut sampler = MultiNoiseSampler::generate(
    //         &generator.base_router.multi_noise,
    //         &MultiNoiseSamplerBuilderOptions::new(0, 0, 4),
    //     );

    //     for (x, y, z, biome_id) in expected_data {
    //         let calculated_biome = MultiNoiseBiomeSupplier::OVERWORLD.biome(x, y, z, &mut sampler);

    //         assert_eq!(
    //             biome_id,
    //             calculated_biome.id,
    //             "Expected {:?} was {:?} at {},{},{}",
    //             Biome::from_id(biome_id),
    //             calculated_biome,
    //             x,
    //             y,
    //             z
    //         );
    //     }
    // }
}
