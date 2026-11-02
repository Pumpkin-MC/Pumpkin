#![allow(clippy::print_stdout, clippy::needless_pass_by_value)]

use crate::generation::{generator::WorldGenerator, get_world_gen, proto_chunk::ProtoChunk};
use pumpkin_data::dimension::Dimension;
use pumpkin_util::world_seed::Seed;

#[derive(serde::Deserialize)]
struct StructureStartsDump {
    seed: u64,
    scan_range: i32,
    max_generate: usize,
    overworld: DimensionStructuresDump,
    the_nether: DimensionStructuresDump,
    the_end: DimensionStructuresDump,
}

#[derive(serde::Deserialize)]
struct DimensionStructuresDump {
    candidate_chunks: usize,
    generated_chunks: usize,
    chunks_with_starts: usize,
    chunks_with_references: usize,
    total_starts: usize,
    total_pieces: usize,
    total_references: usize,
    chunks: Vec<ChunkStartsDump>,
}

#[derive(serde::Deserialize)]
struct ChunkStartsDump {
    x: i32,
    z: i32,
    starts: Vec<StructureStartEntryDump>,
    #[serde(default)]
    references: Vec<StructureReferenceEntryDump>,
}

#[derive(serde::Deserialize)]
struct StructureStartEntryDump {
    structure: String,
    chunk_x: i32,
    chunk_z: i32,
    references: i32,
    bounding_box: BoundingBoxDump,
    pieces: Vec<PieceEntryDump>,
}

#[derive(serde::Deserialize)]
struct BoundingBoxDump {
    min_x: i32,
    min_y: i32,
    min_z: i32,
    max_x: i32,
    max_y: i32,
    max_z: i32,
}

#[derive(serde::Deserialize)]
struct PieceEntryDump {
    #[serde(rename = "type")]
    piece_type: String,
    gen_depth: i32,
    orientation: i32,
    bounding_box: BoundingBoxDump,
    #[serde(default)]
    piece_data: Option<serde_json::Value>,
}

#[derive(serde::Deserialize)]
struct StructureReferenceEntryDump {
    structure: String,
    source_chunks: Vec<[i32; 2]>,
}

#[allow(clippy::too_many_lines)]
fn verify_dimension_structure_starts(seed: u64, dim: Dimension, dump: &DimensionStructuresDump) {
    let world_gen = get_world_gen(Seed(seed), dim, false, Vec::new(), String::new());
    let WorldGenerator::Noise(generator) = &*world_gen else {
        unreachable!()
    };

    for chunk_dump in &dump.chunks {
        if chunk_dump.starts.is_empty() {
            continue;
        }

        let mut chunk = ProtoChunk::new(chunk_dump.x, chunk_dump.z, &world_gen);
        chunk.step_to_biomes(generator);
        chunk.set_structure_starts(generator);

        for expected_start in &chunk_dump.starts {
            let Some(key) =
                pumpkin_data::structures::StructureKeys::from_name(&expected_start.structure)
            else {
                println!(
                    "Skipping unimplemented structure key {}",
                    expected_start.structure
                );
                continue;
            };

            let instance = chunk.structure_starts.get(&key);
            assert!(
                instance.is_some(),
                "Missing structure start for {} at chunk ({}, {})",
                expected_start.structure,
                chunk_dump.x,
                chunk_dump.z
            );

            if let Some(crate::generation::structure::structures::StructureInstance::Start(start)) =
                instance
            {
                let mut collector = start.collector.lock().unwrap();
                let bb = collector.get_adjusted_bounding_box(&key);
                let exp_bb = &expected_start.bounding_box;
                assert_eq!(
                    [bb.min.x, bb.min.y, bb.min.z, bb.max.x, bb.max.y, bb.max.z],
                    [
                        exp_bb.min_x,
                        exp_bb.min_y,
                        exp_bb.min_z,
                        exp_bb.max_x,
                        exp_bb.max_y,
                        exp_bb.max_z
                    ],
                    "Bounding box mismatch for {} at chunk ({}, {})",
                    expected_start.structure,
                    chunk_dump.x,
                    chunk_dump.z
                );
                assert_eq!(
                    collector.pieces.len(),
                    expected_start.pieces.len(),
                    "Piece count mismatch for {} at chunk ({}, {})",
                    expected_start.structure,
                    chunk_dump.x,
                    chunk_dump.z
                );
                for (i, (actual, expected)) in collector
                    .pieces
                    .iter()
                    .zip(&expected_start.pieces)
                    .enumerate()
                {
                    let pbb = actual.bounding_box();
                    let exp_pbb = &expected.bounding_box;
                    assert_eq!(
                        [
                            pbb.min.x, pbb.min.y, pbb.min.z, pbb.max.x, pbb.max.y, pbb.max.z
                        ],
                        [
                            exp_pbb.min_x,
                            exp_pbb.min_y,
                            exp_pbb.min_z,
                            exp_pbb.max_x,
                            exp_pbb.max_y,
                            exp_pbb.max_z
                        ],
                        "Piece {} bounding box mismatch for {} at chunk ({}, {})",
                        i,
                        expected_start.structure,
                        chunk_dump.x,
                        chunk_dump.z
                    );

                    if let Some(piece_data) = &expected.piece_data
                        && let Some(jpiece) = actual.as_any().downcast_ref::<crate::generation::structure::structures::jigsaw::PoolElementStructurePiece>()
                    {
                        if let Some(pos) = piece_data.get("position").and_then(|p| p.as_array()) {
                            let exp_pos = [
                                pos[0].as_i64().unwrap() as i32,
                                pos[1].as_i64().unwrap() as i32,
                                pos[2].as_i64().unwrap() as i32,
                            ];
                            assert_eq!(
                                [jpiece.pos.0.x, jpiece.pos.0.y, jpiece.pos.0.z],
                                exp_pos,
                                "Piece {i} position mismatch for {} at chunk ({}, {})",
                                expected_start.structure,
                                chunk_dump.x,
                                chunk_dump.z
                            );
                        }
                        if let Some(delta) = piece_data.get("ground_level_delta").and_then(serde_json::Value::as_i64) {
                            assert_eq!(
                                jpiece.ground_level_delta as i64,
                                delta,
                                "Piece {i} ground_level_delta mismatch for {} at chunk ({}, {})",
                                expected_start.structure,
                                chunk_dump.x,
                                chunk_dump.z
                            );
                        }
                        if let Some(junctions) = piece_data.get("junctions").and_then(|j| j.as_array()) {
                            assert_eq!(
                                jpiece.junctions.len(),
                                junctions.len(),
                                "Piece {i} junction count mismatch for {} at chunk ({}, {})",
                                expected_start.structure,
                                chunk_dump.x,
                                chunk_dump.z
                            );
                            for (j_idx, (act_j, exp_j)) in jpiece.junctions.iter().zip(junctions).enumerate() {
                                let exp_sx = exp_j.get("source_x").and_then(serde_json::Value::as_i64).unwrap() as i32;
                                let exp_sgy = exp_j.get("source_ground_y").and_then(serde_json::Value::as_i64).unwrap() as i32;
                                let exp_sz = exp_j.get("source_z").and_then(serde_json::Value::as_i64).unwrap() as i32;
                                let exp_dy = exp_j.get("delta_y").and_then(serde_json::Value::as_i64).unwrap() as i32;
                                    let exp_dest_proj = exp_j.get("dest_proj").and_then(|v| v.as_str()).unwrap();
                                    let act_dest_proj = match act_j.projection {
                                        crate::generation::structure::structures::jigsaw::JigsawProjection::Rigid => "rigid",
                                        crate::generation::structure::structures::jigsaw::JigsawProjection::TerrainMatching => "terrain_matching",
                                    };
                                    assert_eq!(
                                        act_j.source_x,
                                        exp_sx,
                                        "Piece {i} junction {j_idx} source_x mismatch for {} at ({}, {})",
                                        expected_start.structure,
                                        chunk_dump.x,
                                        chunk_dump.z
                                    );
                                    assert_eq!(
                                        act_j.source_ground_y,
                                        exp_sgy,
                                        "Piece {i} junction {j_idx} source_ground_y mismatch for {} at ({}, {})",
                                        expected_start.structure,
                                        chunk_dump.x,
                                        chunk_dump.z
                                    );
                                    assert_eq!(
                                        act_j.source_z,
                                        exp_sz,
                                        "Piece {i} junction {j_idx} source_z mismatch for {} at ({}, {})",
                                        expected_start.structure,
                                        chunk_dump.x,
                                        chunk_dump.z
                                    );
                                    assert_eq!(
                                        act_j.delta_y,
                                        exp_dy,
                                        "Piece {i} junction {j_idx} delta_y mismatch for {} at ({}, {})",
                                        expected_start.structure,
                                        chunk_dump.x,
                                        chunk_dump.z
                                    );
                                    assert_eq!(
                                        act_dest_proj,
                                        exp_dest_proj,
                                        "Piece {i} junction {j_idx} dest_proj mismatch for {} at ({}, {})",
                                        expected_start.structure,
                                        chunk_dump.x,
                                        chunk_dump.z
                                    );
                                }
                            }
                            if let Some(pool_element) = piece_data.get("pool_element").and_then(|pe| pe.as_object()) {
                                if let Some(exp_proj) = pool_element.get("projection").and_then(|p| p.as_str()) {
                                    let act_proj = match jpiece.element.projection {
                                        crate::generation::structure::structures::jigsaw::JigsawProjection::Rigid => "rigid",
                                        crate::generation::structure::structures::jigsaw::JigsawProjection::TerrainMatching => "terrain_matching",
                                    };
                                    assert_eq!(
                                        act_proj,
                                        exp_proj,
                                        "Piece {i} pool element projection mismatch for {} at ({}, {})",
                                        expected_start.structure,
                                        chunk_dump.x,
                                        chunk_dump.z
                                    );
                                }
                                if let Some(exp_type) = pool_element.get("element_type").and_then(|t| t.as_str()) {
                                    let act_type = match &jpiece.element.kind {
                                        crate::generation::structure::structures::jigsaw::PoolElementKind::Single { legacy: true, .. } => "minecraft:legacy_single_pool_element",
                                        crate::generation::structure::structures::jigsaw::PoolElementKind::Single { legacy: false, .. } => "minecraft:single_pool_element",
                                        crate::generation::structure::structures::jigsaw::PoolElementKind::Empty => "minecraft:empty_pool_element",
                                        crate::generation::structure::structures::jigsaw::PoolElementKind::List(_) => "minecraft:list_pool_element",
                                        crate::generation::structure::structures::jigsaw::PoolElementKind::Feature(_) => "minecraft:feature_pool_element",
                                    };
                                    assert_eq!(
                                        act_type,
                                        exp_type,
                                        "Piece {i} pool element type mismatch for {} at ({}, {})",
                                        expected_start.structure,
                                        chunk_dump.x,
                                        chunk_dump.z
                                    );
                                }
                                if let Some(exp_loc) = pool_element.get("location").and_then(|l| l.as_str())
                                    && let crate::generation::structure::structures::jigsaw::PoolElementKind::Single { template, .. } = &jpiece.element.kind
                                {
                                    assert_eq!(
                                        template.as_str(),
                                        exp_loc,
                                        "Piece {i} pool element location mismatch for {} at ({}, {})",
                                        expected_start.structure,
                                        chunk_dump.x,
                                        chunk_dump.z
                                    );
                                }
                            }
                        }
                }
                println!(
                    "MATCH: {} at ({}, {}) ({} pieces)",
                    expected_start.structure,
                    chunk_dump.x,
                    chunk_dump.z,
                    collector.pieces.len()
                );
            }
        }
    }
}

#[test]
fn nether_structure_starts_dump() {
    let dump: StructureStartsDump =
        pumpkin_util::read_data_from_file!("../../../../../assets/tests/structure_starts.json");
    verify_dimension_structure_starts(dump.seed, Dimension::THE_NETHER, &dump.the_nether);
}

#[test]
fn end_structure_starts_dump() {
    let dump: StructureStartsDump =
        pumpkin_util::read_data_from_file!("../../../../../assets/tests/structure_starts.json");
    verify_dimension_structure_starts(dump.seed, Dimension::THE_END, &dump.the_end);
}

#[test]
fn overworld_structure_starts_dump() {
    let dump: StructureStartsDump =
        pumpkin_util::read_data_from_file!("../../../../../assets/tests/structure_starts.json");
    verify_dimension_structure_starts(dump.seed, Dimension::OVERWORLD, &dump.overworld);
}
