use crate::generation::{
    generator::WorldGenerator, get_world_gen, proto_chunk::ProtoChunk,
    surface::terrain::SurfaceTerrainBuilder,
};
use pumpkin_data::{Block, dimension::Dimension};
use pumpkin_util::{math::vector3::Vector3, random::xoroshiro128::Xoroshiro, world_seed::Seed};

use super::surface_biomes;

#[test]
fn terracotta_bands_negative_y_wrapping() {
    let mut rand = Xoroshiro::from_seed(12345);
    let deriver = rand.next_splitter();
    let terrain_builder = SurfaceTerrainBuilder::new(&deriver);
    for y in -64..320 {
        let block = terrain_builder.get_terracotta_block(0, y, 0);
        assert!(!block.is_air());
    }
}

#[test]
fn eroded_badlands_pillars_are_terracotta_not_stone() {
    let seed = Seed(0);
    let world_gen = get_world_gen(seed, Dimension::OVERWORLD, false, Vec::new(), String::new());
    let WorldGenerator::Noise(generator) = &*world_gen else {
        unreachable!()
    };

    let chunk_x = -595;
    let chunk_z = 544;
    let mut chunk = ProtoChunk::new(chunk_x, chunk_z, &world_gen);
    chunk.step_to_biomes(generator);
    chunk.stage = crate::chunk_system::chunk_state::StagedChunkEnum::StructureReferences;
    chunk.step_to_noise(generator);

    let mut original_heights = [0i32; 256];
    for x in 0..16 {
        for z in 0..16 {
            original_heights[x * 16 + z] = chunk.top_block_height_exclusive(x as i32, z as i32);
        }
    }

    let surface_biomes = surface_biomes(&world_gen, chunk_x, chunk_z);
    chunk.step_to_surface(generator, &surface_biomes);

    let mut elevated_columns = 0;
    let mut pillar_blocks_checked = 0;

    for x in 0..16 {
        for z in 0..16 {
            let original_top = original_heights[x * 16 + z];
            let new_top = chunk.top_block_height_exclusive(x as i32, z as i32);
            if new_top > original_top {
                elevated_columns += 1;
                for y in original_top..new_top {
                    let state = chunk.get_block_state(&Vector3::new(
                        chunk_x * 16 + x as i32,
                        y,
                        chunk_z * 16 + z as i32,
                    ));
                    let block = state.to_state().id.to_block();
                    assert_ne!(
                        *block,
                        Block::STONE,
                        "Badlands pillar block at ({}, {}, {}) must not be stone! Found: {}",
                        chunk_x * 16 + x as i32,
                        y,
                        chunk_z * 16 + z as i32,
                        block.name
                    );
                    pillar_blocks_checked += 1;
                }
            }
        }
    }

    assert!(
        elevated_columns > 0,
        "Chunk (-595, 544) should contain eroded badlands pillars"
    );
    assert!(
        pillar_blocks_checked > 0,
        "Should have verified pillar blocks"
    );
}
