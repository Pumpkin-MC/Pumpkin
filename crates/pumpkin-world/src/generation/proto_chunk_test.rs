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

    /// Verifies that references use the structure selected for the shared Nether start.
    #[test]
    fn nether_complex_references_use_selected_structure() {
        use crate::generation::structure::placement::get_structure_chunk_in_region;
        use pumpkin_data::structures::{StructureKeys, StructurePlacementType, StructureSet};

        let seed = Seed(0);
        let world_gen = get_world_gen(
            seed,
            Dimension::THE_NETHER,
            false,
            Vec::new(),
            String::new(),
        );
        let WorldGenerator::Noise(generator) = &*world_gen else {
            unreachable!()
        };
        let set = &StructureSet::NETHER_COMPLEXES;
        let StructurePlacementType::RandomSpread(spread) = &set.placement.placement_type else {
            unreachable!()
        };

        for region_x in 0..8 {
            for region_z in 0..8 {
                let (chunk_x, chunk_z) = get_structure_chunk_in_region(
                    spread,
                    seed.0 as i64,
                    region_x,
                    region_z,
                    set.placement.salt,
                );
                let mut proto = ProtoChunk::new(chunk_x, chunk_z, &world_gen);
                proto.step_to_biomes(generator);
                proto.set_structure_starts(generator);

                if !proto.has_structure(StructureKeys::BastionRemnant) {
                    continue;
                }

                proto.set_structure_references(generator);
                assert!(proto.has_structure(StructureKeys::BastionRemnant));
                assert!(!proto.has_structure(StructureKeys::Fortress));
                return;
            }
        }

        panic!("no bastion remnant start found in sampled nether complex regions");
    }

    /// Verifies that structure references survive a partial-generation round trip.
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

    #[test]
    fn block_entities_survive_chunk_data_resume() {
        use crate::chunk_system::chunk_state::Chunk;
        use pumpkin_config::lighting::LightingEngineConfig;
        use pumpkin_nbt::compound::NbtCompound;

        let seed = Seed(4242);
        let world_gen = get_world_gen(seed, Dimension::OVERWORLD, false, Vec::new(), String::new());

        let mut proto = ProtoChunk::new(3, -2, &world_gen);
        let mut chest = NbtCompound::new();
        chest.put_string("id", "minecraft:chest".to_string());
        chest.put_int("x", 49);
        chest.put_int("y", 64);
        chest.put_int("z", -31);
        proto.add_block_entity(chest);

        let mut staged = Chunk::Proto(Box::new(proto));
        staged.upgrade_to_level_chunk(&Dimension::OVERWORLD, &LightingEngineConfig::Default);
        let Chunk::Level(chunk_data) = staged else {
            unreachable!()
        };

        let resumed = ProtoChunk::from_chunk_data(&chunk_data, &world_gen);
        assert_eq!(resumed.pending_block_entities.len(), 1);
        assert_eq!(
            resumed.pending_block_entities[0].get_string("id"),
            Some("minecraft:chest")
        );

        let mut staged_again = Chunk::Proto(Box::new(resumed));
        staged_again.upgrade_to_level_chunk(&Dimension::OVERWORLD, &LightingEngineConfig::Default);
        let Chunk::Level(chunk_again) = staged_again else {
            unreachable!()
        };
        assert_eq!(chunk_again.pending_block_entities.lock().unwrap().len(), 1);
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
                expected_heights[z as usize][x as usize] = proto.top_block_height_exclusive(x, z);
            }
        }

        let mut staged = Chunk::Proto(Box::new(proto));
        staged.upgrade_to_level_chunk(&Dimension::OVERWORLD, &LightingEngineConfig::Default);
        let Chunk::Level(chunk_data) = staged else {
            unreachable!()
        };
        assert_eq!(chunk_data.status, pumpkin_data::chunk::ChunkStatus::Terrain);

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
        if mismatches > 0 {
            println!("[{test_name}] Total mismatches: {mismatches}");
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
        let allowed_mismatches = 7500;
        assert!(
            mismatches <= allowed_mismatches,
            "[{test_name}] Chunk surface generation mismatches vanilla! (got {mismatches} mismatches, allowed {allowed_mismatches})"
        );
    }

    #[derive(serde::Deserialize)]
    struct BiomeTestData {
        x: i32,
        z: i32,
        data: Vec<(i32, i32, i32, u8)>,
    }

    #[derive(serde::Deserialize)]
    struct ChunkGridData {
        x: i32,
        z: i32,
        blocks: Vec<u16>,
    }

    fn verify_chunk_biomes(
        seed: u64,
        dimension: Dimension,
        expected_data: &[BiomeTestData],
        test_name: &str,
    ) {
        let seed = Seed(seed);
        let world_gen = get_world_gen(seed, dimension, false, Vec::new(), String::new());
        let WorldGenerator::Noise(generator) = &*world_gen else {
            unreachable!()
        };

        for data in expected_data {
            let mut chunk = ProtoChunk::new(data.x, data.z, &world_gen);
            chunk.step_to_biomes(generator);

            for &(biome_x, biome_y, biome_z, expected_id) in &data.data {
                let actual_biome = chunk.get_biome(biome_x, biome_y, biome_z);
                assert_eq!(
                    actual_biome.id,
                    expected_id,
                    "[{test_name}] Biome mismatch at quart ({biome_x}, {biome_y}, {biome_z}) in chunk ({}, {}): got {} ({}), expected {} ({})",
                    data.x,
                    data.z,
                    actual_biome.registry_id,
                    actual_biome.id,
                    pumpkin_data::chunk::Biome::from_id(expected_id)
                        .map_or("unknown", |b| b.registry_id),
                    expected_id
                );
            }
        }
    }

    fn verify_chunk_carvers(
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
        chunk.step_to_carvers(generator);

        let mismatches = count_dump_mismatches(&chunk, expected_data, test_name);
        assert_air_above_dumped_window(&chunk, expected_data, test_name);
        let allowed_mismatches = 8000;
        assert!(
            mismatches <= allowed_mismatches,
            "[{test_name}] Chunk carver generation mismatches vanilla! (got {mismatches} mismatches, allowed {allowed_mismatches})"
        );
    }

    struct TestBlockRegistry;
    impl crate::world::WorldPortalExt for TestBlockRegistry {
        fn can_place_at(
            &self,
            _block: &pumpkin_data::Block,
            _state: &pumpkin_data::BlockState,
            _block_accessor: &dyn crate::world::BlockAccessor,
            _block_pos: &pumpkin_util::math::position::BlockPos,
        ) -> bool {
            true
        }

        fn mirror(
            &self,
            block: &pumpkin_data::Block,
            state_id: pumpkin_data::BlockStateId,
            mirror: pumpkin_data::Mirror,
        ) -> &'static pumpkin_data::BlockState {
            block.mirror(state_id, mirror)
        }

        fn rotate(
            &self,
            block: &pumpkin_data::Block,
            state_id: pumpkin_data::BlockStateId,
            rotation: pumpkin_data::Rotation,
        ) -> &'static pumpkin_data::BlockState {
            block.rotate(state_id, rotation)
        }

        fn spawn_mobs_for_chunk_generation(
            &self,
            _cache: &mut dyn crate::generation::proto_chunk::GenerationCache,
            _biome: &'static pumpkin_data::biome::Biome,
            _chunk_x: i32,
            _chunk_z: i32,
        ) {
        }
    }

    struct TestGridCache {
        center_pos: (i32, i32),
        chunks: std::collections::HashMap<(i32, i32), ProtoChunk>,
    }

    impl crate::generation::height_limit::HeightLimitView for TestGridCache {
        fn height(&self) -> u16 {
            384
        }
        fn bottom_y(&self) -> i8 {
            -64
        }
    }

    impl crate::world::BlockAccessor for TestGridCache {
        fn get_block(
            &self,
            position: &pumpkin_util::math::position::BlockPos,
        ) -> &'static pumpkin_data::Block {
            self.get_block_state_id(position).to_block()
        }
        fn get_block_state(
            &self,
            position: &pumpkin_util::math::position::BlockPos,
        ) -> &'static pumpkin_data::BlockState {
            self.get_block_state_id(position).to_state()
        }
        fn get_block_state_id(
            &self,
            position: &pumpkin_util::math::position::BlockPos,
        ) -> pumpkin_data::BlockStateId {
            crate::generation::proto_chunk::GenerationCache::get_block_state(self, &position.0)
        }
        fn get_block_and_state(
            &self,
            position: &pumpkin_util::math::position::BlockPos,
        ) -> (
            &'static pumpkin_data::Block,
            &'static pumpkin_data::BlockState,
        ) {
            let id = self.get_block_state_id(position);
            pumpkin_data::BlockState::from_id_with_block(id)
        }
    }

    impl crate::generation::proto_chunk::GenerationCache for TestGridCache {
        fn get_center_chunk_mut(&mut self) -> &mut ProtoChunk {
            self.chunks.get_mut(&self.center_pos).unwrap()
        }
        fn get_center_chunk(&self) -> &ProtoChunk {
            self.chunks.get(&self.center_pos).unwrap()
        }
        fn get_chunk_mut(&mut self, chunk_x: i32, chunk_z: i32) -> Option<&mut ProtoChunk> {
            self.chunks.get_mut(&(chunk_x, chunk_z))
        }
        fn get_chunk(&self, chunk_x: i32, chunk_z: i32) -> Option<&ProtoChunk> {
            self.chunks.get(&(chunk_x, chunk_z))
        }
        fn try_get_proto_chunk(&self, chunk_x: i32, chunk_z: i32) -> Option<&ProtoChunk> {
            self.chunks.get(&(chunk_x, chunk_z))
        }
        fn get_block_state(
            &self,
            pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ) -> pumpkin_data::BlockStateId {
            let cx = pos.x >> 4;
            let cz = pos.z >> 4;
            self.chunks
                .get(&(cx, cz))
                .map_or(pumpkin_data::BlockStateId::AIR, |chunk| {
                    chunk.get_block_state(pos)
                })
        }
        fn get_fluid_and_fluid_state(
            &self,
            _pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ) -> (pumpkin_data::fluid::Fluid, pumpkin_data::fluid::FluidState) {
            (
                pumpkin_data::fluid::Fluid::EMPTY,
                pumpkin_data::fluid::FluidState {
                    height: 0.0,
                    level: 0,
                    is_empty: true,
                    blast_resistance: 0.0,
                    block_state_id: pumpkin_data::BlockStateId::AIR,
                    is_still: false,
                    is_source: false,
                    falling: false,
                },
            )
        }
        fn set_block_state(
            &mut self,
            pos: &pumpkin_util::math::vector3::Vector3<i32>,
            block_state: &pumpkin_data::BlockState,
        ) {
            let cx = pos.x >> 4;
            let cz = pos.z >> 4;
            if let Some(chunk) = self.chunks.get_mut(&(cx, cz)) {
                chunk.set_block_state(pos.x, pos.y, pos.z, block_state);
            }
        }
        fn add_block_entity(
            &mut self,
            _pos: &pumpkin_util::math::vector3::Vector3<i32>,
            nbt: pumpkin_nbt::compound::NbtCompound,
        ) {
            self.chunks
                .get_mut(&self.center_pos)
                .unwrap()
                .add_block_entity(nbt);
        }
        fn top_motion_blocking_block_height_exclusive(&self, x: i32, z: i32) -> i32 {
            let cx = x >> 4;
            let cz = z >> 4;
            self.chunks.get(&(cx, cz)).map_or(-64, |chunk| {
                chunk.top_motion_blocking_block_height_exclusive(x, z)
            })
        }
        fn top_motion_blocking_block_no_leaves_height_exclusive(&self, x: i32, z: i32) -> i32 {
            let cx = x >> 4;
            let cz = z >> 4;
            self.chunks.get(&(cx, cz)).map_or(-64, |chunk| {
                chunk.top_motion_blocking_block_no_leaves_height_exclusive(x, z)
            })
        }
        fn get_top_y(&self, heightmap: &pumpkin_util::HeightMap, x: i32, z: i32) -> i32 {
            let cx = x >> 4;
            let cz = z >> 4;
            self.chunks
                .get(&(cx, cz))
                .map_or(-64, |chunk| chunk.get_top_y(heightmap, x, z))
        }
        fn top_block_height_exclusive(&self, x: i32, z: i32) -> i32 {
            let cx = x >> 4;
            let cz = z >> 4;
            self.chunks
                .get(&(cx, cz))
                .map_or(-64, |chunk| chunk.top_block_height_exclusive(x, z))
        }
        fn ocean_floor_height_exclusive(&self, x: i32, z: i32) -> i32 {
            let cx = x >> 4;
            let cz = z >> 4;
            self.chunks
                .get(&(cx, cz))
                .map_or(-64, |chunk| chunk.ocean_floor_height_exclusive(x, z))
        }
        fn is_air(&self, local_pos: &pumpkin_util::math::vector3::Vector3<i32>) -> bool {
            pumpkin_data::block_properties::is_air(
                crate::generation::proto_chunk::GenerationCache::get_block_state(self, local_pos),
            )
        }
        fn get_biome_for_terrain_gen(
            &self,
            x: i32,
            y: i32,
            z: i32,
        ) -> &'static pumpkin_data::biome::Biome {
            let biome_pos = self.get_center_chunk().get_terrain_gen_biome_pos(x, y, z);
            let cx = biome_pos.x >> 2;
            let cz = biome_pos.z >> 2;
            self.chunks.get(&(cx, cz)).map_or(
                pumpkin_data::biome::Biome::from_id(pumpkin_data::chunk::Biome::PLAINS.id).unwrap(),
                |chunk| chunk.get_biome(biome_pos.x, biome_pos.y, biome_pos.z),
            )
        }
        fn get_blending_data(
            &self,
            chunk_x: i32,
            chunk_z: i32,
        ) -> Option<&crate::generation::blender::blending_data::BlendingData> {
            self.chunks
                .get(&(chunk_x, chunk_z))
                .and_then(|c| c.blending_data.as_ref())
        }
    }

    fn verify_grid_features(
        seed: u64,
        dimension: Dimension,
        grid: &[ChunkGridData],
        size: i32,
        test_name: &str,
    ) {
        let seed = Seed(seed);
        let world_gen = get_world_gen(seed, dimension, false, Vec::new(), String::new());
        let WorldGenerator::Noise(generator) = &*world_gen else {
            unreachable!()
        };
        let block_registry = TestBlockRegistry;

        let margin = 1;
        let min_x = -margin;
        let max_x = size + margin - 1;
        let min_z = -margin;
        let max_z = size + margin - 1;

        let mut chunks = std::collections::HashMap::new();
        for cx in min_x..=max_x {
            for cz in min_z..=max_z {
                let mut chunk = ProtoChunk::new(cx, cz, &world_gen);
                chunk.step_to_biomes(generator);
                chunk.stage = StagedChunkEnum::StructureReferences;
                chunk.step_to_noise(generator);
                let surface_biomes = surface_biomes(&world_gen, cx, cz);
                chunk.step_to_surface(generator, &surface_biomes);
                chunk.step_to_carvers(generator);
                chunks.insert((cx, cz), chunk);
            }
        }

        let mut grid_cache = TestGridCache {
            center_pos: (0, 0),
            chunks,
        };

        for cx in 0..size {
            for cz in 0..size {
                grid_cache.center_pos = (cx, cz);
                ProtoChunk::generate_features_and_structure(
                    &mut grid_cache,
                    &block_registry,
                    &generator.random_config,
                );
            }
        }

        let mut total_mismatches = 0;
        for chunk_data in grid {
            let chunk = grid_cache
                .chunks
                .get(&(chunk_data.x, chunk_data.z))
                .unwrap();
            let chunk_test_name = format!("{}_{}_{}", test_name, chunk_data.x, chunk_data.z);
            let mismatches = count_dump_mismatches(chunk, &chunk_data.blocks, &chunk_test_name);
            assert_air_above_dumped_window(chunk, &chunk_data.blocks, &chunk_test_name);
            println!("[{chunk_test_name}] Mismatches: {mismatches}");
            total_mismatches += mismatches;
        }
        println!("[{test_name}] Total mismatches across grid: {total_mismatches}");
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

    #[test]
    fn biomes_no_blend_no_beard_5_5() {
        let expected: Vec<BiomeTestData> = pumpkin_util::read_data_from_file!(
            "../../../../assets/tests/biome_no_blend_no_beard_0.json"
        );
        verify_chunk_biomes(
            0,
            Dimension::OVERWORLD,
            &expected,
            "biomes_no_blend_no_beard_5_5",
        );
    }

    #[test]
    fn no_blend_no_beard_surface_7_4() {
        let expected: Vec<u16> = pumpkin_util::read_data_from_file!(
            "../../../../assets/tests/no_blend_no_beard_surface_7_4.chunk"
        );
        verify_chunk_surface(
            0,
            Dimension::OVERWORLD,
            7,
            4,
            &expected,
            "no_blend_no_beard_surface_7_4",
        );
    }

    #[test]
    fn no_blend_no_beard_surface_13579_minus6_11() {
        let expected: Vec<u16> = pumpkin_util::read_data_from_file!(
            "../../../../assets/tests/no_blend_no_beard_surface_13579_-6_11.chunk"
        );
        verify_chunk_surface(
            13579,
            Dimension::OVERWORLD,
            -6,
            11,
            &expected,
            "no_blend_no_beard_surface_13579_minus6_11",
        );
    }

    #[test]
    fn no_blend_no_beard_surface_13579_minus2_15() {
        let expected: Vec<u16> = pumpkin_util::read_data_from_file!(
            "../../../../assets/tests/no_blend_no_beard_surface_13579_-2_15.chunk"
        );
        verify_chunk_surface(
            13579,
            Dimension::OVERWORLD,
            -2,
            15,
            &expected,
            "no_blend_no_beard_surface_13579_minus2_15",
        );
    }

    #[test]
    fn no_blend_no_beard_surface_13579_minus7_9() {
        let expected: Vec<u16> = pumpkin_util::read_data_from_file!(
            "../../../../assets/tests/no_blend_no_beard_surface_13579_-7_9.chunk"
        );
        verify_chunk_surface(
            13579,
            Dimension::OVERWORLD,
            -7,
            9,
            &expected,
            "no_blend_no_beard_surface_13579_minus7_9",
        );
    }

    #[test]
    fn no_blend_no_beard_carvers_0_0() {
        let expected: Vec<u16> = pumpkin_util::read_data_from_file!(
            "../../../../assets/tests/no_blend_no_beard_carvers_0_0.chunk"
        );
        verify_chunk_carvers(
            0,
            Dimension::OVERWORLD,
            0,
            0,
            &expected,
            "no_blend_no_beard_carvers_0_0",
        );
    }

    #[test]
    fn no_blend_no_beard_carvers_7_4() {
        let expected: Vec<u16> = pumpkin_util::read_data_from_file!(
            "../../../../assets/tests/no_blend_no_beard_carvers_7_4.chunk"
        );
        verify_chunk_carvers(
            0,
            Dimension::OVERWORLD,
            7,
            4,
            &expected,
            "no_blend_no_beard_carvers_7_4",
        );
    }

    #[test]
    fn nether_carvers_no_blend_no_beard_0_0() {
        let expected: Vec<u16> = pumpkin_util::read_data_from_file!(
            "../../../../assets/tests/nether_carvers_no_blend_no_beard_0_0.chunk"
        );
        verify_chunk_carvers(
            0,
            Dimension::THE_NETHER,
            0,
            0,
            &expected,
            "nether_carvers_no_blend_no_beard_0_0",
        );
    }

    #[test]
    fn nether_carvers_no_blend_no_beard_7_4() {
        let expected: Vec<u16> = pumpkin_util::read_data_from_file!(
            "../../../../assets/tests/nether_carvers_no_blend_no_beard_7_4.chunk"
        );
        verify_chunk_carvers(
            0,
            Dimension::THE_NETHER,
            7,
            4,
            &expected,
            "nether_carvers_no_blend_no_beard_7_4",
        );
    }

    #[test]
    fn grid_8x8_biomes() {
        let expected: Vec<BiomeTestData> =
            pumpkin_util::read_data_from_file!("../../../../assets/tests/grid_8x8_biomes.json");
        verify_chunk_biomes(0, Dimension::OVERWORLD, &expected, "grid_8x8_biomes");
    }

    #[test]
    fn grid_8x8_noise() {
        let grid: Vec<ChunkGridData> =
            pumpkin_util::read_data_from_file!("../../../../assets/tests/grid_8x8_noise.json");
        for chunk in &grid {
            verify_chunk_noise(
                0,
                Dimension::OVERWORLD,
                chunk.x,
                chunk.z,
                &chunk.blocks,
                &format!("grid_8x8_noise_{}_{}", chunk.x, chunk.z),
            );
        }
    }

    #[test]
    fn grid_8x8_surface() {
        let grid: Vec<ChunkGridData> =
            pumpkin_util::read_data_from_file!("../../../../assets/tests/grid_8x8_surface.json");
        for chunk in &grid {
            verify_chunk_surface(
                0,
                Dimension::OVERWORLD,
                chunk.x,
                chunk.z,
                &chunk.blocks,
                &format!("grid_8x8_surface_{}_{}", chunk.x, chunk.z),
            );
        }
    }

    #[test]
    fn grid_8x8_carvers() {
        let grid: Vec<ChunkGridData> =
            pumpkin_util::read_data_from_file!("../../../../assets/tests/grid_8x8_carvers.json");
        for chunk in &grid {
            verify_chunk_carvers(
                0,
                Dimension::OVERWORLD,
                chunk.x,
                chunk.z,
                &chunk.blocks,
                &format!("grid_8x8_carvers_{}_{}", chunk.x, chunk.z),
            );
        }
    }

    #[test]
    fn grid_8x8_features() {
        let grid: Vec<ChunkGridData> =
            pumpkin_util::read_data_from_file!("../../../../assets/tests/grid_8x8_features.json");
        verify_grid_features(0, Dimension::OVERWORLD, &grid, 8, "grid_8x8_features");
    }

    #[test]
    fn nether_grid_8x8_features() {
        let grid: Vec<ChunkGridData> = pumpkin_util::read_data_from_file!(
            "../../../../assets/tests/nether_grid_8x8_features.json"
        );
        verify_grid_features(
            0,
            Dimension::THE_NETHER,
            &grid,
            8,
            "nether_grid_8x8_features",
        );
    }

    #[test]
    fn end_grid_8x8_features() {
        let grid: Vec<ChunkGridData> = pumpkin_util::read_data_from_file!(
            "../../../../assets/tests/end_grid_8x8_features.json"
        );
        verify_grid_features(0, Dimension::THE_END, &grid, 8, "end_grid_8x8_features");
    }

    #[derive(serde::Deserialize, Debug)]
    struct StructurePieceDump {
        bb: [i32; 6],
        #[serde(rename = "type")]
        piece_type: String,
    }

    #[derive(serde::Deserialize, Debug)]
    struct StructureStartDump {
        id: String,
        chunk_x: i32,
        chunk_z: i32,
        bb: [i32; 6],
        pieces: Vec<StructurePieceDump>,
    }

    #[derive(serde::Deserialize, Debug)]
    struct StructureRefChunkDump {
        chunk_x: i32,
        chunk_z: i32,
    }

    #[derive(serde::Deserialize, Debug)]
    struct StructureRefDump {
        id: String,
        chunks: Vec<StructureRefChunkDump>,
    }

    #[derive(serde::Deserialize, Debug)]
    struct ChunkStructuresDump {
        x: i32,
        z: i32,
        starts: Vec<StructureStartDump>,
        references: Vec<StructureRefDump>,
    }

    #[derive(serde::Deserialize, Debug)]
    struct AreaStructureStartChunkDump {
        chunk_x: i32,
        chunk_z: i32,
        starts: Vec<StructureStartDump>,
    }

    fn verify_grid_structures(grid: &[ChunkStructuresDump]) {
        let seed = Seed(0);
        let world_gen = get_world_gen(seed, Dimension::OVERWORLD, false, Vec::new(), String::new());
        let WorldGenerator::Noise(generator) = &*world_gen else {
            unreachable!()
        };

        for chunk_dump in grid {
            let mut chunk = ProtoChunk::new(chunk_dump.x, chunk_dump.z, &world_gen);
            chunk.step_to_biomes(generator);
            chunk.set_structure_starts(generator);
            chunk.set_structure_references(generator);

            println!(
                "Verifying chunk ({}, {}) structure starts...",
                chunk_dump.x, chunk_dump.z
            );
            for expected_start in &chunk_dump.starts {
                let key = match expected_start.id.as_str() {
                    "minecraft:mineshaft" => pumpkin_data::structures::StructureKeys::Mineshaft,
                    "minecraft:mineshaft_mesa" => {
                        pumpkin_data::structures::StructureKeys::MineshaftMesa
                    }
                    "minecraft:ruined_portal" => {
                        pumpkin_data::structures::StructureKeys::RuinedPortal
                    }
                    "minecraft:trial_chambers" => {
                        pumpkin_data::structures::StructureKeys::TrialChambers
                    }
                    "minecraft:ancient_city" => {
                        pumpkin_data::structures::StructureKeys::AncientCity
                    }
                    other => panic!("Unimplemented structure key {other}"),
                };

                let instance = chunk.structure_starts.get(&key);
                assert!(
                    instance.is_some(),
                    "Chunk ({}, {}) missing structure start for {}",
                    chunk_dump.x,
                    chunk_dump.z,
                    expected_start.id
                );
                if let Some(crate::generation::structure::structures::StructureInstance::Start(
                    start,
                )) = instance
                {
                    assert_eq!(chunk_dump.x, expected_start.chunk_x);
                    assert_eq!(chunk_dump.z, expected_start.chunk_z);
                    let mut collector = start.collector.lock().unwrap();
                    let bb = collector.get_bounding_box();
                    assert_eq!(
                        [bb.min.x, bb.min.y, bb.min.z, bb.max.x, bb.max.y, bb.max.z],
                        expected_start.bb,
                        "Bounding box mismatch for {} at chunk ({}, {})",
                        expected_start.id,
                        chunk_dump.x,
                        chunk_dump.z
                    );
                    assert_eq!(
                        collector.pieces.len(),
                        expected_start.pieces.len(),
                        "Piece count mismatch for {} at chunk ({}, {})",
                        expected_start.id,
                        chunk_dump.x,
                        chunk_dump.z
                    );
                } else {
                    panic!(
                        "Expected Start instance for {} at chunk ({}, {}), got reference",
                        expected_start.id, chunk_dump.x, chunk_dump.z
                    );
                }
            }
        }
    }

    fn verify_area_structures(area: &[AreaStructureStartChunkDump]) {
        let seed = Seed(0);
        let world_gen = get_world_gen(seed, Dimension::OVERWORLD, false, Vec::new(), String::new());
        let WorldGenerator::Noise(generator) = &*world_gen else {
            unreachable!()
        };

        for chunk_dump in area {
            let mut chunk = ProtoChunk::new(chunk_dump.chunk_x, chunk_dump.chunk_z, &world_gen);
            chunk.step_to_biomes(generator);
            chunk.set_structure_starts(generator);

            println!(
                "Verifying area chunk ({}, {}) structure starts...",
                chunk_dump.chunk_x, chunk_dump.chunk_z
            );
            for expected_start in &chunk_dump.starts {
                let key = pumpkin_data::structures::StructureKeys::from_name(&expected_start.id)
                    .unwrap_or_else(|| panic!("Unimplemented structure key {}", expected_start.id));

                let instance = chunk.structure_starts.get(&key);
                assert!(
                    instance.is_some(),
                    "Area chunk ({}, {}) missing structure start for {}",
                    chunk_dump.chunk_x,
                    chunk_dump.chunk_z,
                    expected_start.id
                );
                if let Some(crate::generation::structure::structures::StructureInstance::Start(
                    start,
                )) = instance
                {
                    assert_eq!(chunk_dump.chunk_x, expected_start.chunk_x);
                    assert_eq!(chunk_dump.chunk_z, expected_start.chunk_z);
                    let mut collector = start.collector.lock().unwrap();
                    let bb = collector.get_adjusted_bounding_box(&key);
                    println!("Expected BB: {:?}", expected_start.bb);
                    println!(
                        "Got BB: [{}, {}, {}, {}, {}, {}]",
                        bb.min.x, bb.min.y, bb.min.z, bb.max.x, bb.max.y, bb.max.z
                    );
                    for (i, p) in collector.pieces.iter().take(3).enumerate() {
                        let pbb = p.bounding_box();
                        println!(
                            "Piece {}: [{}, {}, {}, {}, {}, {}]",
                            i, pbb.min.x, pbb.min.y, pbb.min.z, pbb.max.x, pbb.max.y, pbb.max.z
                        );
                    }
                    assert_eq!(
                        [bb.min.x, bb.min.y, bb.min.z, bb.max.x, bb.max.y, bb.max.z],
                        expected_start.bb,
                        "Bounding box mismatch for {} at chunk ({}, {})",
                        expected_start.id,
                        chunk_dump.chunk_x,
                        chunk_dump.chunk_z
                    );
                    assert_eq!(
                        collector.pieces.len(),
                        expected_start.pieces.len(),
                        "Piece count mismatch for {} at chunk ({}, {})",
                        expected_start.id,
                        chunk_dump.chunk_x,
                        chunk_dump.chunk_z
                    );
                    for (i, (actual, expected)) in collector
                        .pieces
                        .iter()
                        .zip(&expected_start.pieces)
                        .enumerate()
                    {
                        let pbb = actual.bounding_box();
                        assert_eq!(
                            [
                                pbb.min.x, pbb.min.y, pbb.min.z, pbb.max.x, pbb.max.y, pbb.max.z
                            ],
                            expected.bb,
                            "Piece {} bounding box mismatch for {} at chunk ({}, {})",
                            i,
                            expected_start.id,
                            chunk_dump.chunk_x,
                            chunk_dump.chunk_z
                        );
                    }
                } else {
                    panic!(
                        "Expected Start instance for {} at chunk ({}, {}), got reference",
                        expected_start.id, chunk_dump.chunk_x, chunk_dump.chunk_z
                    );
                }
            }
        }
    }

    #[test]
    fn grid_8x8_structures() {
        let grid: Vec<ChunkStructuresDump> =
            pumpkin_util::read_data_from_file!("../../../../assets/tests/grid_8x8_structures.json");
        verify_grid_structures(&grid);
    }

    #[test]
    fn area_structures() {
        let area: Vec<AreaStructureStartChunkDump> =
            pumpkin_util::read_data_from_file!("../../../../assets/tests/area_structures.json");
        verify_area_structures(&area);
    }
}
