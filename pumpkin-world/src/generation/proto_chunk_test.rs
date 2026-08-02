#[cfg(test)]
mod test {
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

    #[test]
    fn no_blend_no_beard_0_0() {
        let seed = Seed(0);
        let world_gen = get_world_gen(seed, Dimension::OVERWORLD, false, Vec::new(), String::new());
        let mut chunk = ProtoChunk::new(0, 0, &world_gen);
        let WorldGenerator::Noise(generator) = &*world_gen else {
            unreachable!()
        };

        chunk.step_to_biomes(generator);
        chunk.stage = StagedChunkEnum::StructureReferences;
        chunk.step_to_noise(generator);

        let mut non_air_count = 0;
        for block in &chunk.flat_block_map {
            if !block.to_state().id.to_block().name.eq("air") {
                non_air_count += 1;
            }
        }
        assert!(
            non_air_count > 0,
            "Chunk should generate non-air noise blocks"
        );
    }

    #[test]
    fn no_blend_no_beard_7_4() {
        let seed = Seed(0);
        let world_gen = get_world_gen(seed, Dimension::OVERWORLD, false, Vec::new(), String::new());
        let mut chunk = ProtoChunk::new(7, 4, &world_gen);
        let WorldGenerator::Noise(generator) = &*world_gen else {
            unreachable!()
        };

        chunk.step_to_biomes(generator);
        chunk.stage = StagedChunkEnum::StructureReferences;
        chunk.step_to_noise(generator);

        let mut non_air_count = 0;
        for block in &chunk.flat_block_map {
            if !block.to_state().id.to_block().name.eq("air") {
                non_air_count += 1;
            }
        }
        assert!(
            non_air_count > 0,
            "Chunk should generate non-air noise blocks"
        );
    }

    #[test]
    fn no_blend_no_beard_surface_0_0() {
        let seed = Seed(0);
        let world_gen = get_world_gen(seed, Dimension::OVERWORLD, false, Vec::new(), String::new());
        let mut chunk = ProtoChunk::new(0, 0, &world_gen);
        let WorldGenerator::Noise(generator) = &*world_gen else {
            unreachable!()
        };

        chunk.step_to_biomes(generator);
        chunk.stage = StagedChunkEnum::StructureReferences;
        chunk.step_to_noise(generator);
        chunk.step_to_surface(generator);

        let bottom_block = chunk.get_block_state_raw(0, 0, 0);
        assert_eq!(
            bottom_block.to_state().id.to_block().name,
            "bedrock",
            "Bottom of the world must be bedrock"
        );

        let mut has_deepslate_or_stone = false;
        for y in 10..100 {
            let block = chunk.get_block_state_raw(8, y, 8);
            let name = block.to_state().id.to_block().name;
            if name.contains("deepslate") || name.eq("stone") {
                has_deepslate_or_stone = true;
                break;
            }
        }
        assert!(
            has_deepslate_or_stone,
            "Middle of the world must contain deepslate or stone"
        );

        let mut has_surface_blocks = false;
        for y in 100..384 {
            let block = chunk.get_block_state_raw(8, y, 8);
            let name = block.to_state().id.to_block().name;
            if name.eq("grass_block") || name.eq("dirt") || name.eq("sand") || name.eq("water") {
                has_surface_blocks = true;
                break;
            }
        }
        assert!(
            has_surface_blocks,
            "Top of the world must contain surface blocks (grass/dirt/sand/water)"
        );
    }
}
