#![allow(clippy::print_stdout, clippy::needless_pass_by_value)]

use crate::generation::{generator::WorldGenerator, get_world_gen, proto_chunk::ProtoChunk};
use pumpkin_data::dimension::Dimension;
use pumpkin_util::world_seed::Seed;

#[derive(serde::Deserialize)]
struct BiomeHashesJson {
    seed: u64,
    overworld: DimensionBiomeHashes,
    the_nether: DimensionBiomeHashes,
    the_end: DimensionBiomeHashes,
}

#[derive(serde::Deserialize)]
struct DimensionBiomeHashes {
    min_section_y: i32,
    max_section_y: i32,
    hashes: Vec<(i32, i32, String)>,
}

fn verify_biome_hashes_for_dimension(dim: Dimension, data: &DimensionBiomeHashes, seed: u64) {
    use rayon::prelude::*;
    let world_gen = get_world_gen(Seed(seed), dim, false, Vec::new(), String::new());
    let WorldGenerator::Noise(generator) = &*world_gen else {
        unreachable!()
    };

    data.hashes.par_iter().for_each(|(cx, cz, expected_hash)| {
        let mut chunk = ProtoChunk::new(*cx, *cz, &world_gen);
        chunk.step_to_biomes(generator);
        let actual_hash =
            compute_proto_chunk_biome_hash(&chunk, data.min_section_y, data.max_section_y);
        assert_eq!(
            &actual_hash, expected_hash,
            "Biome hash mismatch at chunk ({cx}, {cz})"
        );
    });
}

#[test]
fn nether_biome_hashes_match_vanilla() {
    let json: BiomeHashesJson =
        pumpkin_util::read_data_from_file!("../../../../../assets/tests/biome_hashes.json");
    verify_biome_hashes_for_dimension(Dimension::THE_NETHER, &json.the_nether, json.seed);
}

#[test]
fn end_biome_hashes_match_vanilla() {
    let json: BiomeHashesJson =
        pumpkin_util::read_data_from_file!("../../../../../assets/tests/biome_hashes.json");
    verify_biome_hashes_for_dimension(Dimension::THE_END, &json.the_end, json.seed);
}

#[test]
fn overworld_biome_hashes_match_vanilla() {
    let json: BiomeHashesJson =
        pumpkin_util::read_data_from_file!("../../../../../assets/tests/biome_hashes.json");
    verify_biome_hashes_for_dimension(Dimension::OVERWORLD, &json.overworld, json.seed);
}

fn compute_proto_chunk_biome_hash(
    chunk: &ProtoChunk,
    min_section_y: i32,
    max_section_y: i32,
) -> String {
    let mut ctx = md5::Context::new();
    for section_y in min_section_y..=max_section_y {
        ctx.consume([section_y as u8]);
        for y in 0..4i32 {
            for z in 0..4i32 {
                for x in 0..4i32 {
                    let biome = chunk.get_biome(x, section_y * 4 + y, z);
                    let name = biome
                        .registry_id
                        .strip_prefix("minecraft:")
                        .unwrap_or(biome.registry_id);
                    ctx.consume(name.as_bytes());
                }
            }
        }
    }
    format!("{:x}", ctx.finalize())
}
