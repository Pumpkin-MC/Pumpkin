#[cfg(test)]
mod test {
    #![allow(clippy::print_stdout, clippy::needless_pass_by_value)]
    use crate::chunk_system::chunk_state::StagedChunkEnum;
    use crate::generation::{generator::WorldGenerator, get_world_gen, proto_chunk::ProtoChunk};
    use pumpkin_data::dimension::Dimension;
    use pumpkin_util::world_seed::Seed;

    // Regression test for transposed heightmaps during Noise-stage chunk resume.
    // Flat terrain cannot expose this bug, so use a sloped chunk.
    #[test]
    fn heightmap_roundtrip_through_chunk_data_resume() {
        use crate::chunk_system::chunk_state::Chunk;
        use pumpkin_config::lighting::LightingEngineConfig;
        use pumpkin_util::math::vector3::Vector3;

        let seed = Seed(1779920288596261407);
        let (cx, cz) = (67i32, 63i32);
        let world_gen = get_world_gen(seed, Dimension::OVERWORLD, false, Vec::new(), String::new());
        let WorldGenerator::Noise(generator) = &*world_gen else {
            unreachable!()
        };

        let mut proto = ProtoChunk::new(cx, cz, &world_gen);
        proto.step_to_biomes(generator);
        proto.set_structure_starts(generator);
        proto.set_structure_references(generator);
        proto.step_to_noise(generator);

        let mut expected_heights = [[0i32; 16]; 16];
        for z in 0..16i32 {
            for x in 0..16i32 {
                expected_heights[z as usize][x as usize] = proto.top_block_height_exclusive(x, z);
            }
        }

        let mut staged = Chunk::Proto(Box::new(proto));
        staged.upgrade_to_level_chunk(&Dimension::OVERWORLD, &LightingEngineConfig::Default);
        let Chunk::Level(chunk_data) = staged else {
            unreachable!()
        };
        assert_eq!(chunk_data.status, pumpkin_data::chunk::ChunkStatus::Noise);

        let mut resumed = ProtoChunk::from_chunk_data(&chunk_data, &world_gen);
        assert_eq!(resumed.stage, StagedChunkEnum::Noise);

        let mut height_mismatches = 0;
        for z in 0..16i32 {
            for x in 0..16i32 {
                let expected = expected_heights[z as usize][x as usize];
                let got = resumed.top_block_height_exclusive(x, z);
                if got != expected {
                    height_mismatches += 1;
                }
            }
        }
        assert_eq!(
            height_mismatches, 0,
            "heightmap corrupted by save/load roundtrip (transposed or lost)"
        );

        resumed.step_to_surface(generator);

        let mut fresh = ProtoChunk::new(cx, cz, &world_gen);
        fresh.step_to_biomes(generator);
        fresh.set_structure_starts(generator);
        fresh.set_structure_references(generator);
        fresh.step_to_noise(generator);
        fresh.step_to_surface(generator);

        let bottom = fresh.bottom_y() as i32;
        let top = bottom + fresh.height() as i32;
        let mut surface_mismatches = 0;
        for lz in 0..16i32 {
            for lx in 0..16i32 {
                let (wx, wz) = (cx * 16 + lx, cz * 16 + lz);
                for y in (bottom..top).rev() {
                    let f = fresh.get_block_state(&Vector3::new(wx, y, wz)).to_state();
                    if f.is_air() || f.is_liquid() {
                        continue;
                    }
                    let r = resumed.get_block_state(&Vector3::new(wx, y, wz)).to_state();
                    if f.id != r.id {
                        surface_mismatches += 1;
                    }
                    break;
                }
            }
        }
        assert_eq!(
            surface_mismatches, 0,
            "resumed chunk surface differs from uninterrupted generation (stone-trail bug)"
        );
    }

    fn verify_chunk_noise(
        seed: u64,
        dimension: Dimension,
        chunk_x: i32,
        chunk_z: i32,
        expected_data: &[u16],
        test_name: &str,
    ) {
        let seed = Seed(seed);
        let world_gen = get_world_gen(seed, dimension, false, Vec::new(), String::new());
        let mut chunk = ProtoChunk::new(chunk_x, chunk_z, &world_gen);
        let WorldGenerator::Noise(generator) = &*world_gen else {
            unreachable!()
        };

        chunk.step_to_biomes(generator);
        chunk.stage = StagedChunkEnum::StructureReferences;
        chunk.step_to_noise(generator);

        assert_eq!(chunk.flat_block_map.len(), expected_data.len());
        let min_y = chunk.bottom_y() as i32;
        let height = chunk.height() as usize;
        let mut mismatches = 0;
        for (i, (&actual, &expected)) in chunk
            .flat_block_map
            .iter()
            .zip(expected_data.iter())
            .enumerate()
        {
            if actual.as_u16() != expected {
                if mismatches < 10 {
                    let x = i / (height * 16);
                    let rem = i % (height * 16);
                    let y_local = rem / 16;
                    let z = rem % 16;
                    let y = y_local as i32 + min_y;
                    let act_block = pumpkin_data::BlockState::from_id(actual).id.to_block().name;
                    let exp_block = pumpkin_data::BlockState::from_id(
                        pumpkin_data::BlockStateId::new(expected).unwrap(),
                    )
                    .id
                    .to_block()
                    .name;
                    println!(
                        "[{test_name}] Mismatch at local ({x}, {y}, {z}) index {i}: got {act_block} ({}), expected {exp_block} ({expected})",
                        actual.as_u16()
                    );
                }
                mismatches += 1;
            }
        }
        assert_eq!(
            mismatches, 0,
            "[{test_name}] Chunk noise generation mismatches vanilla!"
        );
    }

    fn verify_chunk_surface(
        seed: u64,
        dimension: Dimension,
        chunk_x: i32,
        chunk_z: i32,
        expected_data: &[u16],
        test_name: &str,
    ) {
        let seed = Seed(seed);
        let world_gen = get_world_gen(seed, dimension, false, Vec::new(), String::new());
        let mut chunk = ProtoChunk::new(chunk_x, chunk_z, &world_gen);
        let WorldGenerator::Noise(generator) = &*world_gen else {
            unreachable!()
        };

        chunk.step_to_biomes(generator);
        chunk.stage = StagedChunkEnum::StructureReferences;
        chunk.step_to_noise(generator);
        chunk.step_to_surface(generator);

        assert_eq!(chunk.flat_block_map.len(), expected_data.len());
        let min_y = chunk.bottom_y() as i32;
        let height = chunk.height() as usize;
        let mut mismatches = 0;
        for (i, (&actual, &expected)) in chunk
            .flat_block_map
            .iter()
            .zip(expected_data.iter())
            .enumerate()
        {
            if actual.as_u16() != expected {
                if mismatches < 10 {
                    let x = i / (height * 16);
                    let rem = i % (height * 16);
                    let y_local = rem / 16;
                    let z = rem % 16;
                    let y = y_local as i32 + min_y;
                    let act_block = pumpkin_data::BlockState::from_id(actual).id.to_block().name;
                    let exp_block = pumpkin_data::BlockState::from_id(
                        pumpkin_data::BlockStateId::new(expected).unwrap(),
                    )
                    .id
                    .to_block()
                    .name;
                    println!(
                        "[{test_name}] Mismatch at local ({x}, {y}, {z}) index {i}: got {act_block} ({}), expected {exp_block} ({expected})",
                        actual.as_u16()
                    );
                }
                mismatches += 1;
            }
        }
        let allowed_mismatches = 1060;
        assert!(
            mismatches <= allowed_mismatches,
            "[{test_name}] Chunk surface generation mismatches vanilla! (got {mismatches} mismatches, allowed {allowed_mismatches})"
        );
    }

    #[test]
    fn no_blend_no_beard_0_0() {
        let expected: Vec<u16> = pumpkin_util::read_data_from_file!(
            "../../../../assets/tests/noise_no_blend_no_beard_0_0.chunk"
        );
        verify_chunk_noise(
            0,
            Dimension::OVERWORLD,
            0,
            0,
            &expected,
            "no_blend_no_beard_0_0",
        );
    }

    #[test]
    fn no_blend_no_beard_7_4() {
        let expected: Vec<u16> = pumpkin_util::read_data_from_file!(
            "../../../../assets/tests/noise_no_blend_no_beard_7_4.chunk"
        );
        verify_chunk_noise(
            0,
            Dimension::OVERWORLD,
            7,
            4,
            &expected,
            "no_blend_no_beard_7_4",
        );
    }

    #[test]
    fn no_blend_no_beard_only_cell_cache_interpolated_0_0() {
        let expected: Vec<u16> = pumpkin_util::read_data_from_file!(
            "../../../../assets/tests/noise_no_blend_no_beard_only_cell_cache_interpolated_0_0.chunk"
        );
        verify_chunk_noise(
            0,
            Dimension::OVERWORLD,
            0,
            0,
            &expected,
            "no_blend_no_beard_only_cell_cache_interpolated_0_0",
        );
    }

    #[test]
    fn no_blend_no_beard_badlands_minus595_544() {
        let expected: Vec<u16> = pumpkin_util::read_data_from_file!(
            "../../../../assets/tests/noise_no_blend_no_beard_-595_544.chunk"
        );
        verify_chunk_noise(
            0,
            Dimension::OVERWORLD,
            -595,
            544,
            &expected,
            "no_blend_no_beard_badlands_minus595_544",
        );
    }

    #[test]
    fn no_blend_no_beard_frozen_ocean_minus119_183() {
        let expected: Vec<u16> = pumpkin_util::read_data_from_file!(
            "../../../../assets/tests/noise_no_blend_no_beard_-119_183.chunk"
        );
        verify_chunk_noise(
            0,
            Dimension::OVERWORLD,
            -119,
            183,
            &expected,
            "no_blend_no_beard_frozen_ocean_minus119_183",
        );
    }

    #[test]
    fn no_blend_no_beard_13579_minus6_11() {
        let expected: Vec<u16> = pumpkin_util::read_data_from_file!(
            "../../../../assets/tests/noise_no_blend_no_beard_13579_-6_11.chunk"
        );
        verify_chunk_noise(
            13579,
            Dimension::OVERWORLD,
            -6,
            11,
            &expected,
            "no_blend_no_beard_13579_minus6_11",
        );
    }

    #[test]
    fn no_blend_no_beard_13579_minus2_15() {
        let expected: Vec<u16> = pumpkin_util::read_data_from_file!(
            "../../../../assets/tests/noise_no_blend_no_beard_13579_-2_15.chunk"
        );
        verify_chunk_noise(
            13579,
            Dimension::OVERWORLD,
            -2,
            15,
            &expected,
            "no_blend_no_beard_13579_minus2_15",
        );
    }

    #[test]
    fn no_blend_no_beard_13579_minus7_9() {
        let expected: Vec<u16> = pumpkin_util::read_data_from_file!(
            "../../../../assets/tests/noise_no_blend_no_beard_13579_-7_9.chunk"
        );
        verify_chunk_noise(
            13579,
            Dimension::OVERWORLD,
            -7,
            9,
            &expected,
            "no_blend_no_beard_13579_minus7_9",
        );
    }

    #[test]
    fn nether_noise_no_blend_no_beard_0_0() {
        let expected: Vec<u16> = pumpkin_util::read_data_from_file!(
            "../../../../assets/tests/noise_nether_no_blend_no_beard_0_0.chunk"
        );
        verify_chunk_noise(
            0,
            Dimension::THE_NETHER,
            0,
            0,
            &expected,
            "nether_noise_no_blend_no_beard_0_0",
        );
    }

    #[test]
    fn nether_noise_no_blend_no_beard_7_4() {
        let expected: Vec<u16> = pumpkin_util::read_data_from_file!(
            "../../../../assets/tests/noise_nether_no_blend_no_beard_7_4.chunk"
        );
        verify_chunk_noise(
            0,
            Dimension::THE_NETHER,
            7,
            4,
            &expected,
            "nether_noise_no_blend_no_beard_7_4",
        );
    }

    #[test]
    fn end_noise_no_blend_no_beard_0_0() {
        let expected: Vec<u16> = pumpkin_util::read_data_from_file!(
            "../../../../assets/tests/noise_end_no_blend_no_beard_0_0.chunk"
        );
        verify_chunk_noise(
            0,
            Dimension::THE_END,
            0,
            0,
            &expected,
            "end_noise_no_blend_no_beard_0_0",
        );
    }

    #[test]
    fn end_noise_no_blend_no_beard_7_4() {
        let expected: Vec<u16> = pumpkin_util::read_data_from_file!(
            "../../../../assets/tests/noise_end_no_blend_no_beard_7_4.chunk"
        );
        verify_chunk_noise(
            0,
            Dimension::THE_END,
            7,
            4,
            &expected,
            "end_noise_no_blend_no_beard_7_4",
        );
    }

    #[test]
    fn no_blend_no_beard_surface_0_0() {
        let expected: Vec<u16> = pumpkin_util::read_data_from_file!(
            "../../../../assets/tests/no_blend_no_beard_surface_0_0.chunk"
        );
        verify_chunk_surface(
            0,
            Dimension::OVERWORLD,
            0,
            0,
            &expected,
            "no_blend_no_beard_surface_0_0",
        );
    }

    #[test]
    fn no_blend_no_beard_surface_badlands_minus595_544() {
        let expected: Vec<u16> = pumpkin_util::read_data_from_file!(
            "../../../../assets/tests/no_blend_no_beard_surface_badlands_-595_544.chunk"
        );
        verify_chunk_surface(
            0,
            Dimension::OVERWORLD,
            -595,
            544,
            &expected,
            "no_blend_no_beard_surface_badlands_minus595_544",
        );
    }

    #[test]
    fn no_blend_no_beard_surface_frozen_ocean_minus119_183() {
        let expected: Vec<u16> = pumpkin_util::read_data_from_file!(
            "../../../../assets/tests/no_blend_no_beard_surface_frozen_ocean_-119_183.chunk"
        );
        verify_chunk_surface(
            0,
            Dimension::OVERWORLD,
            -119,
            183,
            &expected,
            "no_blend_no_beard_surface_frozen_ocean_minus119_183",
        );
    }

    #[test]
    fn nether_surface_no_blend_no_beard_0_0() {
        let expected: Vec<u16> = pumpkin_util::read_data_from_file!(
            "../../../../assets/tests/nether_surface_no_blend_no_beard_0_0.chunk"
        );
        verify_chunk_surface(
            0,
            Dimension::THE_NETHER,
            0,
            0,
            &expected,
            "nether_surface_no_blend_no_beard_0_0",
        );
    }

    #[test]
    fn nether_surface_no_blend_no_beard_7_4() {
        let expected: Vec<u16> = pumpkin_util::read_data_from_file!(
            "../../../../assets/tests/nether_surface_no_blend_no_beard_7_4.chunk"
        );
        verify_chunk_surface(
            0,
            Dimension::THE_NETHER,
            7,
            4,
            &expected,
            "nether_surface_no_blend_no_beard_7_4",
        );
    }

    #[test]
    fn end_surface_no_blend_no_beard_0_0() {
        let expected: Vec<u16> = pumpkin_util::read_data_from_file!(
            "../../../../assets/tests/end_surface_no_blend_no_beard_0_0.chunk"
        );
        verify_chunk_surface(
            0,
            Dimension::THE_END,
            0,
            0,
            &expected,
            "end_surface_no_blend_no_beard_0_0",
        );
    }

    #[test]
    fn end_surface_no_blend_no_beard_7_4() {
        let expected: Vec<u16> = pumpkin_util::read_data_from_file!(
            "../../../../assets/tests/end_surface_no_blend_no_beard_7_4.chunk"
        );
        verify_chunk_surface(
            0,
            Dimension::THE_END,
            7,
            4,
            &expected,
            "end_surface_no_blend_no_beard_7_4",
        );
    }
}
