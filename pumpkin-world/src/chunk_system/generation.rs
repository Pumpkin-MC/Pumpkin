use pumpkin_data::chunk_gen_settings::GenerationSettings;
use pumpkin_data::dimension::Dimension;

use crate::ProtoChunk;
use crate::generation::generator::VanillaGenerator;
use crate::world::BlockRegistryExt;
use pumpkin_config::lighting::LightingEngineConfig;

use super::{Cache, Chunk, StagedChunkEnum};

pub fn generate_single_chunk(
    dimension: &Dimension,
    biome_mixer_seed: i64,
    generator: &VanillaGenerator,
    block_registry: &dyn BlockRegistryExt,
    chunk_x: i32,
    chunk_z: i32,
    target_stage: StagedChunkEnum,
) -> Chunk {
    let settings = GenerationSettings::from_dimension(dimension);
    let radius = target_stage.get_direct_radius();

    let mut cache = Cache::new(chunk_x - radius, chunk_z - radius, radius * 2 + 1);

    for dx in -radius..=radius {
        for dz in -radius..=radius {
            let new_x = chunk_x + dx;
            let new_z = chunk_z + dz;

            let proto_chunk = Box::new(ProtoChunk::new(
                new_x,
                new_z,
                dimension,
                generator.default_block,
                biome_mixer_seed,
            ));

            cache.chunks.push(Chunk::Proto(proto_chunk));
        }
    }

    let stages = [
        StagedChunkEnum::StructureStart,
        StagedChunkEnum::StructureReferences,
        StagedChunkEnum::Biomes,
        StagedChunkEnum::Noise,
        StagedChunkEnum::Surface,
        StagedChunkEnum::Features,
        StagedChunkEnum::Lighting,
        StagedChunkEnum::Full,
    ];

    for &stage in &stages {
        if stage as u8 > target_stage as u8 {
            break;
        }

        cache.advance(
            stage,
            &LightingEngineConfig::Default,
            block_registry,
            settings,
            &generator.random_config,
            &generator.terrain_cache,
            &generator.base_router,
            *dimension,
        );
    }

    let mid = ((cache.size * cache.size) >> 1) as usize;
    cache.chunks.swap_remove(mid)
}

#[cfg(test)]
mod tests {
    use crate::biome::hash_seed;
    use crate::chunk::format::anvil::SingleChunkDataSerializer;
    use crate::chunk_system::{Chunk, StagedChunkEnum, generate_single_chunk};
    use crate::generation::get_world_gen;
    use crate::world::BlockRegistryExt;
    use futures::executor::block_on;
    use pumpkin_data::dimension::Dimension;
    use pumpkin_nbt::normalize_nbt_bytes;
    use pumpkin_util::world_seed::Seed;
    use std::sync::Arc;

    struct BlockRegistry;
    impl BlockRegistryExt for BlockRegistry {
        fn can_place_at(
            &self,
            _block: &pumpkin_data::Block,
            _state: &pumpkin_data::BlockState,
            _block_accessor: &dyn crate::world::BlockAccessor,
            _block_pos: &pumpkin_util::math::position::BlockPos,
        ) -> bool {
            true
        }
    }

    #[test]
    fn generate_chunk_should_return() {
        let dimension = Dimension::OVERWORLD;
        let seed = Seed(42);
        let block_registry = Arc::new(BlockRegistry);
        let world_gen = get_world_gen(seed, dimension);
        let biome_mixer_seed = hash_seed(world_gen.random_config.seed);

        let _ = generate_single_chunk(
            &dimension,
            biome_mixer_seed,
            &world_gen,
            block_registry.as_ref(),
            0,
            0,
            StagedChunkEnum::Full,
        );
    }

    async fn is_chunks_identical(chunk1: &Chunk, chunk2: &Chunk) -> bool {
        let (Chunk::Level(level1), Chunk::Level(level2)) = (chunk1, chunk2) else {
            panic!("Expected Level chunks");
        };

        let nbt1 = normalize_nbt_bytes(&level1.to_bytes().await.unwrap()).unwrap();
        let nbt2 = normalize_nbt_bytes(&level2.to_bytes().await.unwrap()).unwrap();
        if nbt1 == nbt2 {
            return true;
        }

        if level1.x != level2.x {
            println!("Chunk X coordinates differ");
            return false;
        }
        if level1.z != level2.z {
            println!("Chunk Z coordinates differ");
            return false;
        }

        let mut different_flag = false;

        let blocks1 = level1.section.dump_blocks();
        let blocks2 = level2.section.dump_blocks();
        if blocks1 != blocks2 {
            let sections1 = level1.section.block_sections.read().unwrap();
            let sections2 = level2.section.block_sections.read().unwrap();
            for (sec_idx, (sec1, sec2)) in sections1.iter().zip(sections2.iter()).enumerate() {
                for z in 0..16 {
                    for y in 0..16 {
                        for x in 0..16 {
                            let b1 = sec1.get(x, y, z);
                            let b2 = sec2.get(x, y, z);
                            if b1 != b2 {
                                different_flag = true;
                                println!(
                                    "Different on block: section index: {}, local XYZ: ({}, {}, {}), {} != {}",
                                    sec_idx, x, y, z, b1, b2
                                );
                            }
                        }
                    }
                }
            }
        }

        let biomes1 = level1.section.dump_biomes();
        let biomes2 = level2.section.dump_biomes();
        if biomes1 != biomes2 {
            let sections1 = level1.section.biome_sections.read().unwrap();
            let sections2 = level2.section.biome_sections.read().unwrap();
            for (sec_idx, (sec1, sec2)) in sections1.iter().zip(sections2.iter()).enumerate() {
                for z in 0..4 {
                    for y in 0..4 {
                        for x in 0..4 {
                            let b1 = sec1.get(x, y, z);
                            let b2 = sec2.get(x, y, z);
                            if b1 != b2 {
                                different_flag = true;
                                println!(
                                    "Different on biome: section index: {}, local XYZ: ({}, {}, {}), {} != {}",
                                    sec_idx, x, y, z, b1, b2
                                );
                            }
                        }
                    }
                }
            }
        }

        let heightmap1 = level1.heightmap.lock().unwrap();
        let heightmap2 = level2.heightmap.lock().unwrap();
        if heightmap1.world_surface.as_ref() != heightmap2.world_surface.as_ref() {
            println!("World surface heightmap differs",);
            different_flag = true;
        }
        if heightmap1.motion_blocking.as_ref() != heightmap2.motion_blocking.as_ref() {
            println!("Motion blocking heightmap differs",);
            different_flag = true;
        }
        if heightmap1.motion_blocking_no_leaves.as_ref()
            != heightmap2.motion_blocking_no_leaves.as_ref()
        {
            println!("Motion blocking no leaves heightmap differs",);
            different_flag = true;
        }

        return !different_flag;
    }

    #[tokio::test]
    #[ignore = "very slow, should be tested under release profile (-r)"]
    async fn slow_generate_chunk_should_identical() {
        use rayon::prelude::*;

        let chunk_x = 669;
        let chunk_z = 473;
        let world_seed = 657830420;

        let dimension = Dimension::OVERWORLD;
        let block_registry = Arc::new(BlockRegistry);
        let world_gen = get_world_gen(Seed(world_seed), dimension);
        let biome_mixer_seed = hash_seed(world_gen.random_config.seed);

        let initial_chunk = generate_single_chunk(
            &dimension,
            biome_mixer_seed,
            &world_gen,
            block_registry.as_ref(),
            chunk_x,
            chunk_z,
            StagedChunkEnum::Full,
        );

        let all_match = (0..3000).into_par_iter().all(|_| {
            let compared_chunk = generate_single_chunk(
                &dimension,
                biome_mixer_seed,
                &world_gen,
                block_registry.as_ref(),
                chunk_x,
                chunk_z,
                StagedChunkEnum::Full,
            );

            block_on(is_chunks_identical(&initial_chunk, &compared_chunk))
        });

        assert!(
            all_match,
            "Found at least one chunk that is different from the initial chunk"
        );
    }
}
