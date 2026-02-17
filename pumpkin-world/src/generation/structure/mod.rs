use pumpkin_data::{
    structures::{Structure, StructureKeys},
    tag::{RegistryKey, get_tag_ids},
};

use crate::{
    ProtoChunk,
    generation::{
        biome_coords,
        structure::structures::{
            StructureGenerator, StructureGeneratorContext, StructurePosition,
            buried_treasure::BuriedTreasureGenerator, create_chunk_random, igloo::IglooGenerator,
            stronghold::StrongholdGenerator, swamp_hut::SwampHutGenerator,
        },
    },
};

pub mod piece;
pub mod placement;
pub mod shiftable_piece;
pub mod structures;
pub mod template;

#[must_use]
pub fn try_generate_structure(
    key: &StructureKeys,
    structure: &Structure,
    seed: i64,
    chunk: &ProtoChunk,
    sea_level: i32,
) -> Option<StructurePosition> {
    let random = create_chunk_random(seed, chunk.x, chunk.z);
    let context = StructureGeneratorContext {
        seed,
        chunk_x: chunk.x,
        chunk_z: chunk.z,
        random,
        sea_level,
        min_y: chunk.bottom_y() as i32,
    };

    let structure_pos = match key {
        StructureKeys::BuriedTreasure => {
            BuriedTreasureGenerator::get_structure_position(&BuriedTreasureGenerator, context)
        }
        StructureKeys::SwampHut => {
            SwampHutGenerator::get_structure_position(&SwampHutGenerator, context)
        }
        StructureKeys::Stronghold => {
            StrongholdGenerator::get_structure_position(&StrongholdGenerator, context)
        }
        StructureKeys::Igloo => IglooGenerator::get_structure_position(&IglooGenerator, context),
        // TODO: Implement other structure types
        _ => None,
    };

    if let Some(pos) = structure_pos {
        // Get the biome at the structure's starting position
        let current_biome = chunk.get_biome_id(
            biome_coords::from_block(pos.start_pos.0.x),
            biome_coords::from_block(pos.start_pos.0.y),
            biome_coords::from_block(pos.start_pos.0.z),
        ) as u16;

        let biomes = get_tag_ids(
            RegistryKey::WorldgenBiome,
            structure
                .biomes
                .strip_prefix("#")
                .unwrap_or(structure.biomes),
        )
        .unwrap();

        // Check if the biome is allowed for this structure
        if biomes.contains(&current_biome) {
            return Some(pos);
        }
    }

    None
}
