#[cfg(test)]
mod test {
    use crate::chunk::CHUNK_AREA;
    use crate::chunk_system::chunk_state::StagedChunkEnum;
    use crate::generation::{generator::WorldGenerator, get_world_gen, proto_chunk::ProtoChunk};
    use pumpkin_data::dimension::Dimension;
    use pumpkin_util::world_seed::Seed;

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

    /// The Nether reports `logical_height` 128 (the portal/mob-AI cap) but its chunks
    /// are `height` 256 blocks tall. Sizing the proto chunk by the logical height made
    /// `Chunk::build_level_sections` — which walks the full dimension height — index
    /// past the end of `flat_block_map` and panic during chunk generation.
    #[test]
    fn nether_proto_chunk_covers_full_dimension_height() {
        let seed = Seed(0);
        let world_gen =
            get_world_gen(seed, Dimension::THE_NETHER, false, Vec::new(), String::new());
        let mut chunk = ProtoChunk::new(-5, 3, &world_gen);
        let WorldGenerator::Noise(generator) = &*world_gen else {
            unreachable!()
        };

        let height = Dimension::THE_NETHER.height;
        assert_eq!(
            i32::from(chunk.height()),
            height,
            "Proto chunk must be allocated with the dimension's storage height"
        );
        assert_eq!(chunk.flat_block_map.len(), CHUNK_AREA * height as usize);

        chunk.step_to_biomes(generator);

        // Mirrors the access pattern of `Chunk::build_level_sections`.
        for y in 0..height {
            for x in 0..16 {
                for z in 0..16 {
                    let _ = chunk.get_block_state_raw(x, y, z);
                }
            }
        }
        for y in 0..(height / 4) {
            for x in 0..4 {
                for z in 0..4 {
                    let _ = chunk.get_biome_id(x, y, z);
                }
            }
        }
    }
}
