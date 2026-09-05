#[cfg(test)]
mod test {
    #![allow(clippy::print_stdout, clippy::needless_pass_by_value)]
    use crate::chunk_system::{
        Chunk, chunk_state::StagedChunkEnum, generation_cache::SurfaceBiomeNeighborhood,
    };
    use crate::generation::{
        biome_coords, generator::WorldGenerator, get_world_gen, proto_chunk::ProtoChunk,
    };
    use pumpkin_data::dimension::Dimension;
    use pumpkin_util::world_seed::Seed;

    fn surface_biomes(
        world_gen: &WorldGenerator,
        center_x: i32,
        center_z: i32,
    ) -> crate::chunk_system::generation_cache::SurfaceBiomeNeighborhood {
        let WorldGenerator::Noise(generator) = world_gen else {
            unreachable!()
        };
        let mut neighborhood = SurfaceBiomeNeighborhood::new(center_x, center_z);
        for chunk_x in center_x - 1..=center_x + 1 {
            for chunk_z in center_z - 1..=center_z + 1 {
                let mut chunk = ProtoChunk::new(chunk_x, chunk_z, world_gen);
                chunk.step_to_biomes(generator);
                assert!(neighborhood.push_chunk(&Chunk::Proto(Box::new(chunk))));
            }
        }
        assert!(neighborhood.is_complete());
        neighborhood
    }

    #[test]
    fn terrain_biome_lookup_crosses_chunk_boundary() {
        use pumpkin_data::chunk::Biome;

        let seed = Seed(1_786_192_857_164_469_025);
        let world_gen = get_world_gen(seed, Dimension::OVERWORLD, false, Vec::new(), String::new());
        let WorldGenerator::Noise(generator) = &*world_gen else {
            unreachable!()
        };

        let mut north = ProtoChunk::new(84, 599, &world_gen);
        let mut south = ProtoChunk::new(84, 600, &world_gen);
        north.step_to_biomes(generator);
        south.step_to_biomes(generator);

        // Biome zoom selects absolute quart (338, 17, 2399) on both sides of z=9600.
        // Vanilla resolves that quart through the owning chunk. It must not wrap the quart
        // coordinate into the chunk whose surface is currently being generated.
        let expected = north.get_biome_id(338, 17, 2399);
        assert_eq!(expected, Biome::SAVANNA.id);
        assert_eq!(north.get_terrain_gen_biome_id(1354, 68, 9599), expected);

        let mut surface_biomes = SurfaceBiomeNeighborhood::new(south.x, south.z);
        for chunk_x in south.x - 1..=south.x + 1 {
            for chunk_z in south.z - 1..=south.z + 1 {
                let mut chunk = ProtoChunk::new(chunk_x, chunk_z, &world_gen);
                chunk.step_to_biomes(generator);
                if (chunk_x, chunk_z) == (north.x, north.z) {
                    // Make stored authority observably differ from a fresh biome-source sample.
                    let index = chunk.local_biome_pos_to_biome_index(
                        338i32.rem_euclid(4),
                        17 - biome_coords::from_block(chunk.bottom_y() as i32),
                        2399i32.rem_euclid(4),
                    );
                    chunk.flat_biome_map[index] = Biome::DESERT.id;
                }
                assert!(surface_biomes.push_chunk(&Chunk::Proto(Box::new(chunk))));
            }
        }
        assert_eq!(
            south.get_terrain_gen_biome_id_from_neighborhood(&surface_biomes, 1354, 68, 9600),
            Some(Biome::DESERT.id)
        );
    }

    #[test]
    fn generation_cache_resolves_blended_biome_through_owning_chunk() {
        use crate::chunk_system::{Chunk, generation_cache::Cache};
        use crate::generation::proto_chunk::GenerationCache;
        use pumpkin_data::chunk::Biome;

        let seed = Seed(1_786_192_857_164_469_025);
        let world_gen = get_world_gen(seed, Dimension::OVERWORLD, false, Vec::new(), String::new());
        let WorldGenerator::Noise(generator) = &*world_gen else {
            unreachable!()
        };

        let mut cache = Cache::new(83, 599, 3);
        for chunk_x in 83..=85 {
            for chunk_z in 599..=601 {
                let mut chunk = ProtoChunk::new(chunk_x, chunk_z, &world_gen);
                chunk.step_to_biomes(generator);
                cache.chunks.push(Chunk::Proto(Box::new(chunk)));
            }
        }

        assert_eq!(
            cache.get_biome_for_terrain_gen(1354, 68, 9600).id,
            Biome::SAVANNA.id
        );
    }

    #[test]
    fn structure_references_are_rebuilt_when_resuming_generation() {
        use crate::chunk_system::chunk_state::Chunk;
        use pumpkin_config::lighting::LightingEngineConfig;
        use pumpkin_data::structures::StructureKeys;

        let seed = Seed(1_782_124_772_053_846_960);
        let world_gen = get_world_gen(seed, Dimension::OVERWORLD, false, Vec::new(), String::new());
        let WorldGenerator::Noise(generator) = &*world_gen else {
            unreachable!()
        };

        // This chunk contains the far edge of a monument starting at (-553, 173).
        let mut proto = ProtoChunk::new(-553, 174, &world_gen);
        proto.step_to_biomes(generator);
        proto.set_structure_starts(generator);
        proto.set_structure_references(generator);
        assert!(proto.has_structure(StructureKeys::Monument));

        let mut staged = Chunk::Proto(Box::new(proto));
        staged.upgrade_to_level_chunk(&Dimension::OVERWORLD, &LightingEngineConfig::Default);
        let Chunk::Level(chunk_data) = staged else {
            unreachable!()
        };

        let resumed = ProtoChunk::from_chunk_data(&chunk_data, &world_gen);
        assert_eq!(resumed.stage, StagedChunkEnum::StructureReferences);
        assert!(resumed.has_structure(StructureKeys::Monument));
    }

    /// Real vanilla 26.2 for seed 13579 stores exactly one structure start in the 16x16
    /// chunks x -16..-1, z 0..15: `minecraft:mineshaft_mesa` in chunk (-11, 11)
    /// (`structures.starts` of the saved region files, pumpkin-parity/vanilla/world-13579).
    /// Mineshafts use `legacy_type_3` frequency reduction, i.e. the large-feature seed.
    #[test]
    fn mineshaft_mesa_start_matches_vanilla_seed_13579() {
        use pumpkin_data::structures::StructureKeys;

        let world_gen = get_world_gen(
            Seed(13579),
            Dimension::OVERWORLD,
            false,
            Vec::new(),
            String::new(),
        );
        let WorldGenerator::Noise(generator) = &*world_gen else {
            unreachable!()
        };

        let mut proto = ProtoChunk::new(-11, 11, &world_gen);
        proto.step_to_biomes(generator);
        proto.set_structure_starts(generator);
        assert!(proto.has_structure(StructureKeys::MineshaftMesa));
        assert!(!proto.has_structure(StructureKeys::Mineshaft));

        for (cx, cz) in [(-10, 11), (-11, 10), (-12, 12)] {
            let mut proto = ProtoChunk::new(cx, cz, &world_gen);
            proto.step_to_biomes(generator);
            proto.set_structure_starts(generator);
            assert!(
                !proto.has_structure(StructureKeys::MineshaftMesa),
                "({cx}, {cz})"
            );
            assert!(
                !proto.has_structure(StructureKeys::Mineshaft),
                "({cx}, {cz})"
            );
        }
    }

    /// Vanilla `ChunkGenerator.createReferences` scans the starts of every chunk within 8
    /// chunks. In the saved vanilla world for seed 13579, chunk (-6, 8) carries a
    /// `minecraft:mineshaft_mesa` reference to the start in (-11, 11) (five chunks away).
    #[test]
    fn structure_references_reach_eight_chunks_seed_13579() {
        use pumpkin_data::structures::StructureKeys;

        let world_gen = get_world_gen(
            Seed(13579),
            Dimension::OVERWORLD,
            false,
            Vec::new(),
            String::new(),
        );
        let WorldGenerator::Noise(generator) = &*world_gen else {
            unreachable!()
        };

        // Only the positive case is asserted here: the exact extent of the start (vanilla
        // references it from x -15..-6, z 8..15 and *not* from (-5, 8) or (-6, 7)) also
        // depends on the pieces being generated depth-first, which is fixed in the mineshaft
        // series; the negative cases are asserted there.
        let mut proto = ProtoChunk::new(-6, 8, &world_gen);
        proto.step_to_biomes(generator);
        proto.set_structure_starts(generator);
        proto.set_structure_references(generator);
        assert!(proto.has_structure(StructureKeys::MineshaftMesa), "(-6, 8)");
    }

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
                expected_heights[z as usize][x as usize] =
                    proto.top_block_height_wg_exclusive(x, z);
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
                let got = resumed.top_block_height_wg_exclusive(x, z);
                if got != expected {
                    height_mismatches += 1;
                }
            }
        }
        assert_eq!(
            height_mismatches, 0,
            "heightmap corrupted by save/load roundtrip (transposed or lost)"
        );

        let surface_biomes = surface_biomes(&world_gen, cx, cz);
        resumed.step_to_surface(generator, &surface_biomes);

        let mut fresh = ProtoChunk::new(cx, cz, &world_gen);
        fresh.step_to_biomes(generator);
        fresh.set_structure_starts(generator);
        fresh.set_structure_references(generator);
        fresh.step_to_noise(generator);
        fresh.step_to_surface(generator, &surface_biomes);

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

        let mismatches = count_dump_mismatches(&chunk, expected_data, test_name);
        assert_air_above_dumped_window(&chunk, expected_data, test_name);
        let allowed_mismatches = 6000;
        assert!(
            mismatches <= allowed_mismatches,
            "[{test_name}] Chunk noise generation mismatches vanilla! (got {mismatches} mismatches, allowed {allowed_mismatches})"
        );
    }

    fn dumped_window_height(expected_data: &[u16]) -> usize {
        let columns = 16 * 16;
        assert_eq!(expected_data.len() % columns, 0);
        expected_data.len() / columns
    }

    fn count_dump_mismatches(chunk: &ProtoChunk, expected_data: &[u16], test_name: &str) -> usize {
        let dumped_height = dumped_window_height(expected_data);
        assert!(dumped_height <= chunk.height() as usize);

        let min_y = chunk.bottom_y() as i32;
        let mut mismatches = 0;
        for x in 0..16usize {
            for local_y in 0..dumped_height {
                for z in 0..16usize {
                    let expected = expected_data[(x * dumped_height + local_y) * 16 + z];
                    let actual = chunk.get_block_state_raw(x as i32, local_y as i32, z as i32);
                    if actual.as_u16() == expected {
                        continue;
                    }
                    if mismatches < 10 {
                        let y = local_y as i32 + min_y;
                        let act_block =
                            pumpkin_data::BlockState::from_id(actual).id.to_block().name;
                        let exp_block = pumpkin_data::BlockState::from_id(
                            pumpkin_data::BlockStateId::new(expected).unwrap(),
                        )
                        .id
                        .to_block()
                        .name;
                        println!(
                            "[{test_name}] Mismatch at local ({x}, {y}, {z}): got {act_block} ({}), expected {exp_block} ({expected})",
                            actual.as_u16()
                        );
                    }
                    mismatches += 1;
                }
            }
        }
        mismatches
    }

    fn assert_air_above_dumped_window(chunk: &ProtoChunk, expected_data: &[u16], test_name: &str) {
        let min_y = chunk.bottom_y() as i32;
        for x in 0..16usize {
            for local_y in dumped_window_height(expected_data)..chunk.height() as usize {
                for z in 0..16usize {
                    let actual = chunk.get_block_state_raw(x as i32, local_y as i32, z as i32);
                    assert!(
                        pumpkin_data::BlockState::from_id(actual).is_air(),
                        "[{test_name}] Block above the noise window at local ({x}, {}, {z}) is not air",
                        local_y as i32 + min_y
                    );
                }
            }
        }
    }

    /// Vanilla `FossilFeature.place` reads a neighbouring column through the region:
    ///
    /// ```java
    /// lowestSurfaceY = Math.min(lowestSurfaceY,
    ///     level.getHeight(Heightmap.Types.OCEAN_FLOOR_WG, lowCorner.getX() + xscan, lowCorner.getZ() + zscan));
    /// ```
    ///
    /// `WorldGenRegion.getHeight` forwards to the chunk holding that column, and
    /// `OCEAN_FLOOR_WG` / `WORLD_SURFACE_WG` are live on every proto chunk of the region:
    /// noise and the carvers maintained them. Only the chunk being decorated has the four
    /// `FINAL_HEIGHTMAPS` primed (`ChunkStatusTasks.generateFeatures`). Pumpkin's multi-chunk
    /// cache used to answer both `*_WG` maps out of the final maps, so any column outside the
    /// centre chunk reported `minY - 1`.
    #[test]
    fn the_worldgen_heightmaps_are_live_on_every_chunk_of_the_cache() {
        use crate::chunk_system::{Chunk, generation_cache::Cache};
        use crate::generation::proto_chunk::GenerationCache;
        use pumpkin_util::HeightMap;

        let seed = Seed(13579);
        let world_gen = get_world_gen(seed, Dimension::OVERWORLD, false, Vec::new(), String::new());
        let WorldGenerator::Noise(generator) = &*world_gen else {
            unreachable!()
        };

        let mut cache = Cache::new(-1, -1, 3);
        for chunk_x in -1..=1 {
            for chunk_z in -1..=1 {
                let mut chunk = ProtoChunk::new(chunk_x, chunk_z, &world_gen);
                chunk.step_to_biomes(generator);
                chunk.stage = StagedChunkEnum::StructureReferences;
                chunk.step_to_noise(generator);
                let surface_biomes = surface_biomes(&world_gen, chunk_x, chunk_z);
                chunk.step_to_surface(generator, &surface_biomes);
                chunk.stage = StagedChunkEnum::Carvers;
                cache.chunks.push(Chunk::Proto(Box::new(chunk)));
            }
        }
        // Exactly what `generate_features_and_structure` does before the first feature runs.
        cache.get_center_chunk_mut().prime_final_heightmaps();

        let bottom = cache.get_center_chunk().bottom_y() as i32;
        for (x, z) in [(0, 0), (-16, 0), (0, -16), (24, 24), (-3, 20)] {
            let wg = cache.get_top_y(&HeightMap::OceanFloorWg, x, z);
            assert!(
                wg > bottom,
                "OCEAN_FLOOR_WG at ({x}, {z}) is {wg}; the worldgen heightmap must be live \
                 outside the decorated chunk, not the unprimed final map ({bottom})"
            );
            let surface = cache.get_top_y(&HeightMap::WorldSurfaceWg, x, z);
            assert!(
                surface >= wg,
                "WORLD_SURFACE_WG ({surface}) at ({x}, {z}) must sit at or above OCEAN_FLOOR_WG ({wg})"
            );
        }
    }

    /// Vanilla `Heightmap.update` lowers a map when the block it was pointing at is replaced
    /// by one the map does not match:
    ///
    /// ```java
    /// } else if (i - 1 == y) {
    ///     for (int j = y - 1; j >= this.chunk.getMinY(); j--) {
    ///         if (this.isOpaque.test(this.chunk.getBlockState(mutableBlockPos.set(x, j, z)))) {
    ///             this.setHeight(x, z, j + 1);
    ///             return true;
    ///         }
    ///     }
    ///     this.setHeight(x, z, this.chunk.getMinY());
    ///     return true;
    /// }
    /// ```
    ///
    /// which is how carving pulls `WORLD_SURFACE_WG` / `OCEAN_FLOOR_WG` back down. Pumpkin
    /// used to keep a running `max`, so a carved-open column still reported its pre-carver
    /// surface for the whole feature step.
    #[test]
    fn a_write_over_the_top_block_lowers_the_worldgen_heightmap() {
        use pumpkin_data::Block;

        let seed = Seed(13579);
        let world_gen = get_world_gen(seed, Dimension::OVERWORLD, false, Vec::new(), String::new());
        let WorldGenerator::Noise(generator) = &*world_gen else {
            unreachable!()
        };
        let (chunk_x, chunk_z) = (0, 0);
        let mut chunk = ProtoChunk::new(chunk_x, chunk_z, &world_gen);
        chunk.step_to_biomes(generator);
        chunk.stage = StagedChunkEnum::StructureReferences;
        chunk.step_to_noise(generator);
        let surface_biomes = surface_biomes(&world_gen, chunk_x, chunk_z);
        chunk.step_to_surface(generator, &surface_biomes);
        assert_eq!(chunk.stage, StagedChunkEnum::Surface);

        // A carver-shaped write: replace the top block of every column with air.
        let bottom = chunk.bottom_y() as i32;
        for x in 0..16i32 {
            for z in 0..16i32 {
                let top = chunk.top_block_height_wg_exclusive(x, z) - 1;
                assert!(
                    top >= bottom,
                    "column ({x}, {z}) has no surface after the surface step"
                );
                chunk.set_block_state(x, top, z, Block::AIR.default_state);

                let expected = (bottom..top)
                    .rev()
                    .find(|&y| {
                        !pumpkin_data::BlockState::from_id(chunk.get_block_state_raw(
                            x,
                            y - bottom,
                            z,
                        ))
                        .is_air()
                    })
                    .unwrap_or(bottom - 1);
                assert_eq!(
                    chunk.top_block_height_wg_exclusive(x, z) - 1,
                    expected,
                    "WORLD_SURFACE_WG at ({x}, {z}) was not lowered onto the next non-air block"
                );
            }
        }

        // From the carver step on, vanilla maintains `FINAL_HEIGHTMAPS` instead: feature
        // writes move `WORLD_SURFACE` and leave `WORLD_SURFACE_WG` frozen where carving
        // left it.
        chunk.stage = StagedChunkEnum::Carvers;
        chunk.prime_final_heightmaps();
        let frozen = chunk.flat_surface_height_map;
        for x in 0..16i32 {
            for z in 0..16i32 {
                assert_eq!(
                    chunk.top_block_height_exclusive(x, z),
                    chunk.top_block_height_wg_exclusive(x, z),
                    "priming WORLD_SURFACE must reproduce the carved surface at ({x}, {z})"
                );
            }
        }

        let grown = chunk.top_block_height_exclusive(0, 0);
        chunk.set_block_state(0, grown, 0, Block::SHORT_GRASS.default_state);
        assert_eq!(
            chunk.flat_surface_height_map, frozen,
            "a feature write must not touch WORLD_SURFACE_WG"
        );
        assert_eq!(
            chunk.top_block_height_exclusive(0, 0),
            grown + 1,
            "a feature write must raise WORLD_SURFACE"
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
        let surface_biomes = surface_biomes(&world_gen, chunk_x, chunk_z);
        chunk.step_to_surface(generator, &surface_biomes);

        let mismatches = count_dump_mismatches(&chunk, expected_data, test_name);
        assert_air_above_dumped_window(&chunk, expected_data, test_name);
        let allowed_mismatches = 6000;
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
