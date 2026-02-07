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
            buried_treasure::BuriedTreasureGenerator, create_chunk_random,
            desert_pyramid::DesertPyramidGenerator, igloo::IglooGenerator,
            jungle_temple::JungleTempleGenerator,
            nether_fossil::NetherFossilGenerator,
            ocean_ruin::{ColdOceanRuinGenerator, WarmOceanRuinGenerator},
            pillager_outpost::PillagerOutpostGenerator,
            ruined_portal::RuinedPortalGenerator, shipwreck::ShipwreckGenerator,
            stronghold::StrongholdGenerator, swamp_hut::SwampHutGenerator,
            woodland_mansion::WoodlandMansionGenerator,
        },
    },
};

pub mod piece;
pub mod placement;
pub mod shiftable_piece;
pub mod structures;

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
        StructureKeys::DesertPyramid => {
            DesertPyramidGenerator::get_structure_position(&DesertPyramidGenerator, context)
        }
        StructureKeys::JunglePyramid => {
            JungleTempleGenerator::get_structure_position(&JungleTempleGenerator, context)
        }
        StructureKeys::Igloo => IglooGenerator::get_structure_position(&IglooGenerator, context),
        StructureKeys::Shipwreck | StructureKeys::ShipwreckBeached => {
            ShipwreckGenerator::get_structure_position(&ShipwreckGenerator, context)
        }
        StructureKeys::OceanRuinCold => {
            ColdOceanRuinGenerator::get_structure_position(&ColdOceanRuinGenerator, context)
        }
        StructureKeys::OceanRuinWarm => {
            WarmOceanRuinGenerator::get_structure_position(&WarmOceanRuinGenerator, context)
        }
        StructureKeys::PillagerOutpost => {
            PillagerOutpostGenerator::get_structure_position(&PillagerOutpostGenerator, context)
        }
        StructureKeys::NetherFossil => {
            NetherFossilGenerator::get_structure_position(&NetherFossilGenerator, context)
        }
        StructureKeys::RuinedPortal
        | StructureKeys::RuinedPortalDesert
        | StructureKeys::RuinedPortalJungle
        | StructureKeys::RuinedPortalSwamp
        | StructureKeys::RuinedPortalMountain
        | StructureKeys::RuinedPortalOcean
        | StructureKeys::RuinedPortalNether => {
            RuinedPortalGenerator::get_structure_position(&RuinedPortalGenerator, context)
        }
        StructureKeys::Mansion => {
            WoodlandMansionGenerator::get_structure_position(&WoodlandMansionGenerator, context)
        }
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
