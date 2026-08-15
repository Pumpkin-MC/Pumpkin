/* This file is generated. Do not edit manually. */
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub enum ChunkStatus {
    #[serde(rename = "minecraft:empty")]
    Empty,
    #[serde(rename = "minecraft:structure_starts")]
    StructureStarts,
    #[serde(rename = "minecraft:structure_references")]
    StructureReferences,
    #[serde(rename = "minecraft:biomes")]
    Biomes,
    #[serde(rename = "minecraft:noise")]
    Noise,
    #[serde(rename = "minecraft:surface")]
    Surface,
    #[serde(rename = "minecraft:carvers")]
    Carvers,
    #[serde(rename = "minecraft:features")]
    Features,
    #[serde(rename = "minecraft:initialize_light")]
    InitializeLight,
    #[serde(rename = "minecraft:light")]
    Light,
    #[serde(rename = "minecraft:spawn")]
    Spawn,
    #[serde(rename = "minecraft:full")]
    Full,
}
