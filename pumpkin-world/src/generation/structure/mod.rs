use pumpkin_data::{
    structures::{Structure, StructureKeys},
    tag::{RegistryKey, get_tag_ids},
};

use crate::{
    ProtoChunk,
    biome::BiomeSupplier,
    generation::{
        biome_coords,
        noise::router::multi_noise_sampler::MultiNoiseSampler,
        structure::structures::{
            StructureGenerator, StructureGeneratorContext, StructurePosition,
            buried_treasure::BuriedTreasureGenerator, create_chunk_random,
            desert_pyramid::DesertPyramidGenerator, igloo::IglooGenerator,
            nether_fortress::NetherFortressGenerator, nether_fossil::NetherFossilGenerator,
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
        StructureKeys::Fortress => {
            NetherFortressGenerator::get_structure_position(&NetherFortressGenerator, context)
        }
        StructureKeys::NetherFossil => {
            NetherFossilGenerator::get_structure_position(&NetherFossilGenerator, context)
        }
        StructureKeys::Igloo => IglooGenerator::get_structure_position(&IglooGenerator, context),
        StructureKeys::DesertPyramid => DesertPyramidGenerator.get_structure_position(context),
        // TODO: Implement other structure types
        _ => None,
    };

    if let Some(pos) = structure_pos {
        // Get the biome at the structure's starting position.
        // Clamp biome Y to the chunk's valid range — structure start_pos.y may exceed
        // the chunk's logical height (e.g. nether fossils use full height 256 but
        // ProtoChunk only covers logical_height 128).
        let biome_y = biome_coords::from_block(pos.start_pos.0.y);
        let biome_height = (chunk.height() >> 2) as i32;
        let biome_bottom = biome_coords::from_block(chunk.bottom_y() as i32);
        let clamped_biome_y = biome_y.clamp(biome_bottom, biome_bottom + biome_height - 1);

        let current_biome = chunk.get_biome_id(
            biome_coords::from_block(pos.start_pos.0.x),
            clamped_biome_y,
            biome_coords::from_block(pos.start_pos.0.z),
        ) as u16;

        let biomes = get_tag_ids(
            RegistryKey::WorldgenBiome,
            structure
                .biomes
                .strip_prefix('#')
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

#[must_use]
pub fn lazily_generate_structure(
    key: &StructureKeys,
    structure: &Structure,
    context: StructureGeneratorContext, // Replaces 5 separate arguments!
    biome_supplier: &dyn BiomeSupplier,
    multi_noise_sampler: &mut MultiNoiseSampler,
) -> Option<StructurePosition> {
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
        StructureKeys::Fortress => {
            NetherFortressGenerator::get_structure_position(&NetherFortressGenerator, context)
        }
        StructureKeys::NetherFossil => {
            NetherFossilGenerator::get_structure_position(&NetherFossilGenerator, context)
        }
        StructureKeys::Igloo => IglooGenerator::get_structure_position(&IglooGenerator, context),
        StructureKeys::DesertPyramid => DesertPyramidGenerator.get_structure_position(context),
        // TODO: Implement other structure types
        _ => None,
    };

    if let Some(pos) = structure_pos {
        // Get the biome mathematically, bypassing the chunk boundaries entirely!
        let biome_x = biome_coords::from_block(pos.start_pos.0.x);
        let biome_y = biome_coords::from_block(pos.start_pos.0.y);
        let biome_z = biome_coords::from_block(pos.start_pos.0.z);

        let biome = biome_supplier.biome(biome_x, biome_y, biome_z, multi_noise_sampler);

        let biomes = get_tag_ids(
            RegistryKey::WorldgenBiome,
            structure
                .biomes
                .strip_prefix('#')
                .unwrap_or(structure.biomes),
        )
        .unwrap();

        // Check if the biome is allowed for this structure
        if biomes.contains(&(biome.id as u16)) {
            return Some(pos);
        }
    }

    None
}
