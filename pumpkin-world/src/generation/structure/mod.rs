use std::{collections::HashMap, sync::LazyLock};

use pumpkin_data::{chunk::Biome, tag::Taggable};
use pumpkin_util::include_json_static;
use serde::Deserialize;

use crate::{
    ProtoChunk,
    generation::structure::{
        placement::StructurePlacement,
        structures::{
            StructureGenerator, StructurePosition, buried_treasure::BuriedTreasureGenerator,
            nether_fortress::NetherFortressGenerator,
        },
    },
};

pub mod placement;
pub mod structures;

#[derive(Deserialize)]
pub struct StructureSet {
    pub placement: StructurePlacement,
    pub structures: Vec<Structures>,
}

#[derive(Deserialize)]
pub struct Structures {
    pub structure: StructureType,
}

#[derive(Deserialize, Clone)]
pub enum StructureType {
    #[serde(rename = "minecraft:buried_treasure")]
    BuriedTreasure(BuriedTreasureGenerator),
    #[serde(rename = "minecraft:fortress")]
    NetherFortress(NetherFortressGenerator),
}

impl StructureType {
    pub fn try_generate(
        &self,
        name: &str,
        seed: i64,
        chunk_x: i32,
        chunk_z: i32,
        chunk_ref_for_biomes: &ProtoChunk,
    ) -> Option<StructurePosition> {
        let structure_pos = match self {
            StructureType::BuriedTreasure(generator) => {
                generator.try_generate(seed, chunk_x, chunk_z)
            }
            StructureType::NetherFortress(generator) => {
                generator.try_generate(seed, chunk_x, chunk_z)
            }
        };

        if let Some(pos) = structure_pos
            && let Some(structure_data) = STRUCTURES.get(name)
        {
            // Get the biome at the structure's starting position
            let current_biome = chunk_ref_for_biomes.get_biome(
                pos.start_pos.0.x,
                pos.start_pos.0.y,
                pos.start_pos.0.z,
            );

            // Check if the biome is allowed for this structure
            if Biome::get_tag_values(&structure_data.biomes)
                .unwrap_or_default()
                .contains(&current_biome.registry_id)
            {
                return Some(pos);
            }
        }

        None
    }
}

pub fn generate_structure_pieces(
    structure_pos: &StructurePosition,
    chunk: &mut ProtoChunk,
    seed: i64,
) {
    structure_pos.collector.generate_in_chunk(chunk, seed);
}

#[derive(Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Structure {
    biomes: String,
}

pub static STRUCTURES: LazyLock<HashMap<String, Structure>> = LazyLock::new(
    || include_json_static!("../../../../assets/structures.json", HashMap<String, Structure>),
);

pub static STRUCTURE_SETS: LazyLock<HashMap<String, StructureSet>> = LazyLock::new(
    || include_json_static!("../../../../assets/structure_set.json", HashMap<String, StructureSet>),
);
