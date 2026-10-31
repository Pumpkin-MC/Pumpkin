#![allow(clippy::print_stdout, clippy::needless_pass_by_value)]

use crate::chunk_system::{
    chunk_state::{Chunk, StagedChunkEnum},
    generation_cache::SurfaceBiomeNeighborhood,
};
use crate::generation::{
    biome_coords, generator::WorldGenerator, get_world_gen, proto_chunk::ProtoChunk,
};
use pumpkin_data::dimension::Dimension;
use pumpkin_util::world_seed::Seed;

use super::surface_biomes;

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
