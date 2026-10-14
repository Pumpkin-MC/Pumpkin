#![allow(clippy::print_stdout, clippy::needless_pass_by_value)]

use crate::chunk_system::chunk_state::StagedChunkEnum;
use crate::generation::{generator::WorldGenerator, get_world_gen, proto_chunk::ProtoChunk};
use pumpkin_data::dimension::Dimension;
use pumpkin_util::world_seed::Seed;

use super::surface_biomes;

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
                    let act_block = pumpkin_data::BlockState::from_id(actual).id.to_block().name;
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

#[test]
fn no_blend_no_beard_0_0() {
    let expected: Vec<u16> = pumpkin_util::read_data_from_file!(
        "../../../../../assets/tests/noise_no_blend_no_beard_0_0.chunk"
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
        "../../../../../assets/tests/noise_no_blend_no_beard_7_4.chunk"
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
        "../../../../../assets/tests/noise_no_blend_no_beard_only_cell_cache_interpolated_0_0.chunk"
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
        "../../../../../assets/tests/noise_no_blend_no_beard_-595_544.chunk"
    );
    verify_chunk_noise(
        0,
        Dimension::OVERWORLD,
        -595,
        544,
        &expected,
        "no_blend_no_beard_badlands_-595_544",
    );
}

#[test]
fn no_blend_no_beard_frozen_ocean_minus119_183() {
    let expected: Vec<u16> = pumpkin_util::read_data_from_file!(
        "../../../../../assets/tests/noise_no_blend_no_beard_-119_183.chunk"
    );
    verify_chunk_noise(
        0,
        Dimension::OVERWORLD,
        -119,
        183,
        &expected,
        "no_blend_no_beard_frozen_ocean_-119_183",
    );
}

#[test]
fn no_blend_no_beard_13579_minus6_11() {
    let expected: Vec<u16> = pumpkin_util::read_data_from_file!(
        "../../../../../assets/tests/noise_no_blend_no_beard_13579_-6_11.chunk"
    );
    verify_chunk_noise(
        13579,
        Dimension::OVERWORLD,
        -6,
        11,
        &expected,
        "no_blend_no_beard_13579_-6_11",
    );
}

#[test]
fn no_blend_no_beard_13579_minus2_15() {
    let expected: Vec<u16> = pumpkin_util::read_data_from_file!(
        "../../../../../assets/tests/noise_no_blend_no_beard_13579_-2_15.chunk"
    );
    verify_chunk_noise(
        13579,
        Dimension::OVERWORLD,
        -2,
        15,
        &expected,
        "no_blend_no_beard_13579_-2_15",
    );
}

#[test]
fn no_blend_no_beard_13579_minus7_9() {
    let expected: Vec<u16> = pumpkin_util::read_data_from_file!(
        "../../../../../assets/tests/noise_no_blend_no_beard_13579_-7_9.chunk"
    );
    verify_chunk_noise(
        13579,
        Dimension::OVERWORLD,
        -7,
        9,
        &expected,
        "no_blend_no_beard_13579_-7_9",
    );
}

#[test]
fn nether_noise_no_blend_no_beard_0_0() {
    let expected: Vec<u16> = pumpkin_util::read_data_from_file!(
        "../../../../../assets/tests/noise_nether_no_blend_no_beard_0_0.chunk"
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
        "../../../../../assets/tests/noise_nether_no_blend_no_beard_7_4.chunk"
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
        "../../../../../assets/tests/noise_end_no_blend_no_beard_0_0.chunk"
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
        "../../../../../assets/tests/noise_end_no_blend_no_beard_7_4.chunk"
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
        "../../../../../assets/tests/no_blend_no_beard_surface_0_0.chunk"
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
        "../../../../../assets/tests/no_blend_no_beard_surface_badlands_-595_544.chunk"
    );
    verify_chunk_surface(
        0,
        Dimension::OVERWORLD,
        -595,
        544,
        &expected,
        "no_blend_no_beard_surface_badlands_-595_544",
    );
}

#[test]
fn no_blend_no_beard_surface_frozen_ocean_minus119_183() {
    let expected: Vec<u16> = pumpkin_util::read_data_from_file!(
        "../../../../../assets/tests/no_blend_no_beard_surface_frozen_ocean_-119_183.chunk"
    );
    verify_chunk_surface(
        0,
        Dimension::OVERWORLD,
        -119,
        183,
        &expected,
        "no_blend_no_beard_surface_frozen_ocean_-119_183",
    );
}

#[test]
fn nether_surface_no_blend_no_beard_0_0() {
    let expected: Vec<u16> = pumpkin_util::read_data_from_file!(
        "../../../../../assets/tests/nether_surface_no_blend_no_beard_0_0.chunk"
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
        "../../../../../assets/tests/nether_surface_no_blend_no_beard_7_4.chunk"
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
        "../../../../../assets/tests/end_surface_no_blend_no_beard_0_0.chunk"
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
        "../../../../../assets/tests/end_surface_no_blend_no_beard_7_4.chunk"
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
        "../../../../../assets/tests/biome_no_blend_no_beard_0.json"
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
        "../../../../../assets/tests/no_blend_no_beard_surface_7_4.chunk"
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
        "../../../../../assets/tests/no_blend_no_beard_surface_13579_-6_11.chunk"
    );
    verify_chunk_surface(
        13579,
        Dimension::OVERWORLD,
        -6,
        11,
        &expected,
        "no_blend_no_beard_surface_13579_-6_11",
    );
}

#[test]
fn no_blend_no_beard_surface_13579_minus2_15() {
    let expected: Vec<u16> = pumpkin_util::read_data_from_file!(
        "../../../../../assets/tests/no_blend_no_beard_surface_13579_-2_15.chunk"
    );
    verify_chunk_surface(
        13579,
        Dimension::OVERWORLD,
        -2,
        15,
        &expected,
        "no_blend_no_beard_surface_13579_-2_15",
    );
}

#[test]
fn no_blend_no_beard_surface_13579_minus7_9() {
    let expected: Vec<u16> = pumpkin_util::read_data_from_file!(
        "../../../../../assets/tests/no_blend_no_beard_surface_13579_-7_9.chunk"
    );
    verify_chunk_surface(
        13579,
        Dimension::OVERWORLD,
        -7,
        9,
        &expected,
        "no_blend_no_beard_surface_13579_-7_9",
    );
}

#[test]
fn no_blend_no_beard_carvers_0_0() {
    let expected: Vec<u16> = pumpkin_util::read_data_from_file!(
        "../../../../../assets/tests/no_blend_no_beard_carvers_0_0.chunk"
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
        "../../../../../assets/tests/no_blend_no_beard_carvers_7_4.chunk"
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
        "../../../../../assets/tests/nether_carvers_no_blend_no_beard_0_0.chunk"
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
        "../../../../../assets/tests/nether_carvers_no_blend_no_beard_7_4.chunk"
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
        pumpkin_util::read_data_from_file!("../../../../../assets/tests/grid_8x8_biomes.json");
    verify_chunk_biomes(0, Dimension::OVERWORLD, &expected, "grid_8x8_biomes");
}
