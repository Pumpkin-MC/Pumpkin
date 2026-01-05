use std::{collections::HashMap, sync::LazyLock};

use pumpkin_data::{chunk::Biome, tag::Taggable};
use pumpkin_util::include_json_static;
use serde::Deserialize;

use crate::{
    ProtoChunk,
    generation::{
        biome_coords,
        structure::{
            placement::StructurePlacement,
            structures::{
                StructureGenerator, StructurePosition, buried_treasure::BuriedTreasureGenerator,
                nether_fortress::NetherFortressGenerator, swamp_hut::SwampHutGenerator,
            },
        },
    },
};

pub mod placement;
pub mod structures;

#[derive(Deserialize)]
pub struct StructureSet {
    pub placement: StructurePlacement,
    pub structures: Vec<WeightedEntry>,
}

#[derive(Deserialize)]
pub struct WeightedEntry {
    pub structure: StructureKeys,
    pub weight: u32,
}

#[derive(Deserialize, Clone)]
pub enum StructureType {
    #[serde(rename = "minecraft:buried_treasure")]
    BuriedTreasure(BuriedTreasureGenerator),
    #[serde(rename = "minecraft:desert_pyramid")]
    DesertPyramid,
    #[serde(rename = "minecraft:end_city")]
    EndCity,
    #[serde(rename = "minecraft:fortress")]
    NetherFortress(NetherFortressGenerator),
    #[serde(rename = "minecraft:igloo")]
    Igloo,
    #[serde(rename = "minecraft:jigsaw")]
    Jigsaw,
    #[serde(rename = "minecraft:jungle_temple")]
    JungleTemple,
    #[serde(rename = "minecraft:mineshaft")]
    Mineshaft,
    #[serde(rename = "minecraft:nether_fossil")]
    NetherFossil,
    #[serde(rename = "minecraft:ocean_monument")]
    OceanMonument,
    #[serde(rename = "minecraft:ocean_ruin")]
    OceanRuin,
    #[serde(rename = "minecraft:ruined_portal")]
    RuinedPortal,
    #[serde(rename = "minecraft:shipwreck")]
    Shipwreck,
    #[serde(rename = "minecraft:stronghold")]
    Stronghold,
    #[serde(rename = "minecraft:swamp_hut")]
    SwampHut(SwampHutGenerator),
    #[serde(rename = "minecraft:woodland_mansion")]
    WoodlandMansion,
}

#[derive(Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum StructureKeys {
    #[serde(rename = "minecraft:pillager_outpost")]
    PillagerOutpost,
    #[serde(rename = "minecraft:mineshaft")]
    Mineshaft,
    #[serde(rename = "minecraft:mineshaft_mesa")]
    MineshaftMesa,
    #[serde(rename = "minecraft:mansion")]
    Mansion,
    #[serde(rename = "minecraft:jungle_pyramid")]
    JunglePyramid,
    #[serde(rename = "minecraft:desert_pyramid")]
    DesertPyramid,
    #[serde(rename = "minecraft:igloo")]
    Igloo,
    #[serde(rename = "minecraft:shipwreck")]
    Shipwreck,
    #[serde(rename = "minecraft:shipwreck_beached")]
    ShipwreckBeached,
    #[serde(rename = "minecraft:swamp_hut")]
    SwampHut,
    #[serde(rename = "minecraft:stronghold")]
    Stronghold,
    #[serde(rename = "minecraft:monument")]
    Monument,
    #[serde(rename = "minecraft:ocean_ruin_cold")]
    OceanRuinCold,
    #[serde(rename = "minecraft:ocean_ruin_warm")]
    OceanRuinWarm,
    #[serde(rename = "minecraft:fortress")]
    Fortress,
    #[serde(rename = "minecraft:nether_fossil")]
    NetherFossil,
    #[serde(rename = "minecraft:end_city")]
    EndCity,
    #[serde(rename = "minecraft:buried_treasure")]
    BuriedTreasure,
    #[serde(rename = "minecraft:bastion_remnant")]
    BastionRemnant,
    #[serde(rename = "minecraft:village_plains")]
    VillagePlains,
    #[serde(rename = "minecraft:village_desert")]
    VillageDesert,
    #[serde(rename = "minecraft:village_savanna")]
    VillageSavanna,
    #[serde(rename = "minecraft:village_snowy")]
    VillageSnowy,
    #[serde(rename = "minecraft:village_taiga")]
    VillageTaiga,
    #[serde(rename = "minecraft:ruined_portal")]
    RuinedPortal,
    #[serde(rename = "minecraft:ruined_portal_desert")]
    RuinedPortalDesert,
    #[serde(rename = "minecraft:ruined_portal_jungle")]
    RuinedPortalJungle,
    #[serde(rename = "minecraft:ruined_portal_swamp")]
    RuinedPortalSwamp,
    #[serde(rename = "minecraft:ruined_portal_mountain")]
    RuinedPortalMountain,
    #[serde(rename = "minecraft:ruined_portal_ocean")]
    RuinedPortalOcean,
    #[serde(rename = "minecraft:ruined_portal_nether")]
    RuinedPortalNether,
    #[serde(rename = "minecraft:ancient_city")]
    AncientCity,
    #[serde(rename = "minecraft:trail_ruins")]
    TrailRuins,
    #[serde(rename = "minecraft:trial_chambers")]
    TrialChambers,
}

#[derive(Deserialize, Clone)]
pub struct EmptyStruct {}

impl StructureKeys {
    pub fn try_generate(
        &self,
        seed: i64,
        chunk_x: i32,
        chunk_z: i32,
        chunk_ref_for_biomes: &ProtoChunk,
    ) -> Option<StructurePosition> {
        let structure_pos = match self {
            StructureKeys::BuriedTreasure => BuriedTreasureGenerator::get_structure_position(
                &BuriedTreasureGenerator,
                seed,
                chunk_x,
                chunk_z,
            ),
            StructureKeys::Fortress => NetherFortressGenerator::get_structure_position(
                &NetherFortressGenerator,
                seed,
                chunk_x,
                chunk_z,
            ),
            StructureKeys::SwampHut => SwampHutGenerator::get_structure_position(
                &SwampHutGenerator,
                seed,
                chunk_x,
                chunk_z,
            ),
            // TODO
            _ => None,
        };

        if let Some(pos) = structure_pos
            && let Some(structure_data) = STRUCTURES.get(self)
        {
            // Get the biome at the structure's starting position
            let current_biome = chunk_ref_for_biomes.get_biome(
                biome_coords::from_block(pos.start_pos.0.x),
                biome_coords::from_block(pos.start_pos.0.y),
                biome_coords::from_block(pos.start_pos.0.z),
            );
            let biomes = Biome::get_tag_values(&structure_data.biomes).unwrap();
            // Check if the biome is allowed for this structure
            if biomes.contains(&current_biome.registry_id) {
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

pub static STRUCTURES: LazyLock<HashMap<StructureKeys, Structure>> = LazyLock::new(
    || include_json_static!("../../../../assets/structures.json", HashMap<StructureKeys, Structure>),
);

pub static STRUCTURE_SETS: LazyLock<HashMap<String, StructureSet>> = LazyLock::new(
    || include_json_static!("../../../../assets/structure_set.json", HashMap<String, StructureSet>),
);
