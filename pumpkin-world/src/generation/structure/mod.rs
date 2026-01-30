use std::{collections::HashMap, sync::LazyLock};

use pumpkin_data::tag::{RegistryKey, get_tag_ids};
use pumpkin_util::include_json_static;
use serde::Deserialize;

use crate::{
    ProtoChunk,
    generation::{
        biome_coords,
        structure::{
            placement::StructurePlacement,
            structures::{
                StructureGenerator, StructureGeneratorContext, StructurePosition,
                buried_treasure::BuriedTreasureGenerator, create_chunk_random,
                stronghold::StrongholdGenerator, swamp_hut::SwampHutGenerator,
            },
        },
    },
};

pub mod piece;
pub mod placement;
pub mod shiftable_piece;
pub mod structures;

#[derive(Deserialize)]
pub struct StructureSet {
    pub placement: StructurePlacement,
    pub structures: Vec<WeightedEntry>,
}

#[derive(Deserialize, Clone)]
pub struct WeightedEntry {
    pub structure: StructureKeys,
    pub weight: u32,
}

// #[derive(Deserialize)]
// pub enum StructureType {
//     #[serde(rename = "minecraft:buried_treasure")]
//     BuriedTreasure(BuriedTreasureGenerator),
//     #[serde(rename = "minecraft:desert_pyramid")]
//     DesertPyramid,
//     #[serde(rename = "minecraft:end_city")]
//     EndCity,
//     #[serde(rename = "minecraft:fortress")]
//     NetherFortress(NetherFortressGenerator),
//     #[serde(rename = "minecraft:igloo")]
//     Igloo,
//     #[serde(rename = "minecraft:jigsaw")]
//     Jigsaw,
//     #[serde(rename = "minecraft:jungle_temple")]
//     JungleTemple,
//     #[serde(rename = "minecraft:mineshaft")]
//     Mineshaft,
//     #[serde(rename = "minecraft:nether_fossil")]
//     NetherFossil,
//     #[serde(rename = "minecraft:ocean_monument")]
//     OceanMonument,
//     #[serde(rename = "minecraft:ocean_ruin")]
//     OceanRuin,
//     #[serde(rename = "minecraft:ruined_portal")]
//     RuinedPortal,
//     #[serde(rename = "minecraft:shipwreck")]
//     Shipwreck,
//     #[serde(rename = "minecraft:stronghold")]
//     Stronghold,
//     #[serde(rename = "minecraft:swamp_hut")]
//     SwampHut(SwampHutGenerator),
//     #[serde(rename = "minecraft:woodland_mansion")]
//     WoodlandMansion,
// }

#[derive(Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
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

impl StructureKeys {
    #[must_use]
    pub fn try_generate(
        &self,
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
        let structure_pos = match self {
            Self::BuriedTreasure => {
                BuriedTreasureGenerator::get_structure_position(&BuriedTreasureGenerator, context)
            }
            // StructureKeys::Fortress => {
            //     NetherFortressGenerator::get_structure_position(&NetherFortressGenerator, context)
            // }
            Self::SwampHut => {
                SwampHutGenerator::get_structure_position(&SwampHutGenerator, context)
            }
            Self::Stronghold => {
                StrongholdGenerator::get_structure_position(&StrongholdGenerator, context)
            }
            // StructureKeys::DesertPyramid => DesertTempleGenerator::get_structure_position(
            //     &DesertTempleGenerator,
            //    context
            // ),
            // TODO
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
                    .unwrap_or(&structure.biomes),
            )
            .unwrap();
            // Check if the biome is allowed for this structure
            if biomes.contains(&current_biome) {
                return Some(pos);
            }
        }

        None
    }
}

#[derive(Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Structure {
    pub biomes: String,
    pub step: GenerationStep,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GenerationStep {
    RawGeneration,
    Lakes,
    LocalModifications,
    UndergroundStructures,
    SurfaceStructures,
    Strongholds,
    UndergroundOres,
    UndergroundDecoration,
    FluidSprings,
    VegetalDecoration,
    TopLayerModification,
}

impl GenerationStep {
    #[must_use]
    pub const fn ordinal(&self) -> usize {
        *self as usize
    }
}

pub static STRUCTURES: LazyLock<HashMap<StructureKeys, Structure>> = LazyLock::new(
    || include_json_static!("../../../../assets/structures.json", HashMap<StructureKeys, Structure>),
);

pub static STRUCTURE_SETS: LazyLock<HashMap<String, StructureSet>> = LazyLock::new(
    || include_json_static!("../../../../assets/structure_set.json", HashMap<String, StructureSet>),
);
