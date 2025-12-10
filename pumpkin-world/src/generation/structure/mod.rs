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

#[derive(Deserialize, Clone, Debug)]
pub enum StructureType {
    #[serde(rename = "minecraft:buried_treasure")]
    BuriedTreasure(BuriedTreasureGenerator),
    #[serde(rename = "minecraft:fortress")]
    NetherFortress(NetherFortressGenerator),
}

impl StructureType {
    pub fn get_structure_position(
        &self,
        name: &str,
        chunk: &ProtoChunk,
    ) -> Option<StructurePosition> {
        let position = match self {
            StructureType::BuriedTreasure(generator) => generator.get_structure_position(chunk),
            StructureType::NetherFortress(generator) => generator.get_structure_position(chunk),
        };
        if let Some(structure) = STRUCTURES.get(name) {
            let current_biome = chunk.get_biome(
                position.position.0.x,
                position.position.0.y,
                position.position.0.z,
            );
            if Biome::get_tag_values(&structure.biomes)
                .unwrap()
                .contains(&current_biome.registry_id)
            {
                return Some(position);
            }
        }
        None
    }

    pub fn generate(&self, position: StructurePosition, chunk: &mut crate::ProtoChunk) {
        for pos in position.generator.pieces_positions {
            match self {
                StructureType::BuriedTreasure(generator) => generator.generate(pos, chunk),
                StructureType::NetherFortress(generator) => generator.generate(pos, chunk),
            }
        }
    }
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
