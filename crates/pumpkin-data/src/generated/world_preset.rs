/* This file is generated. Do not edit manually. */
use pumpkin_registry::{Registry, RegistryBuilder, bootstrap::RegistryEntry, bootstrap_provider};
use pumpkin_util::identifier::Identifier;
use std::sync::Arc;
#[derive(Debug)]
pub struct WorldPresetDimension {
    pub identifier: Identifier,
    pub stem: &'static str,
}
#[derive(Debug)]
pub struct WorldPreset {
    pub dimensions: &'static [WorldPresetDimension],
}
impl WorldPreset {
    const NORMAL_DIMENSIONS: [WorldPresetDimension; 3usize] = [
        WorldPresetDimension {
            identifier: Identifier::parse_static("minecraft:overworld"),
            stem: "{\"type\":\"minecraft:overworld\",\"generator\":{\"type\":\"minecraft:noise\",\"biome_source\":{\"type\":\"minecraft:multi_noise\",\"preset\":\"minecraft:overworld\"},\"settings\":\"minecraft:overworld\"}}",
        },
        WorldPresetDimension {
            identifier: Identifier::parse_static("minecraft:the_end"),
            stem: "{\"type\":\"minecraft:the_end\",\"generator\":{\"type\":\"minecraft:noise\",\"biome_source\":{\"type\":\"minecraft:the_end\"},\"settings\":\"minecraft:end\"}}",
        },
        WorldPresetDimension {
            identifier: Identifier::parse_static("minecraft:the_nether"),
            stem: "{\"type\":\"minecraft:the_nether\",\"generator\":{\"type\":\"minecraft:noise\",\"biome_source\":{\"type\":\"minecraft:multi_noise\",\"preset\":\"minecraft:nether\"},\"settings\":\"minecraft:nether\"}}",
        },
    ];
    pub const NORMAL: WorldPreset = WorldPreset {
        dimensions: &Self::NORMAL_DIMENSIONS,
    };
    const FLAT_DIMENSIONS: [WorldPresetDimension; 3usize] = [
        WorldPresetDimension {
            identifier: Identifier::parse_static("minecraft:overworld"),
            stem: "{\"type\":\"minecraft:overworld\",\"generator\":{\"type\":\"minecraft:flat\",\"settings\":{\"biome\":\"minecraft:plains\",\"features\":false,\"lakes\":false,\"layers\":[{\"block\":\"minecraft:bedrock\",\"height\":1},{\"block\":\"minecraft:dirt\",\"height\":2},{\"block\":\"minecraft:grass_block\",\"height\":1}],\"structure_overrides\":[\"minecraft:strongholds\",\"minecraft:villages\"]}}}",
        },
        WorldPresetDimension {
            identifier: Identifier::parse_static("minecraft:the_end"),
            stem: "{\"type\":\"minecraft:the_end\",\"generator\":{\"type\":\"minecraft:noise\",\"biome_source\":{\"type\":\"minecraft:the_end\"},\"settings\":\"minecraft:end\"}}",
        },
        WorldPresetDimension {
            identifier: Identifier::parse_static("minecraft:the_nether"),
            stem: "{\"type\":\"minecraft:the_nether\",\"generator\":{\"type\":\"minecraft:noise\",\"biome_source\":{\"type\":\"minecraft:multi_noise\",\"preset\":\"minecraft:nether\"},\"settings\":\"minecraft:nether\"}}",
        },
    ];
    pub const FLAT: WorldPreset = WorldPreset {
        dimensions: &Self::FLAT_DIMENSIONS,
    };
    const LARGE_BIOMES_DIMENSIONS: [WorldPresetDimension; 3usize] = [
        WorldPresetDimension {
            identifier: Identifier::parse_static("minecraft:overworld"),
            stem: "{\"type\":\"minecraft:overworld\",\"generator\":{\"type\":\"minecraft:noise\",\"biome_source\":{\"type\":\"minecraft:multi_noise\",\"preset\":\"minecraft:overworld\"},\"settings\":\"minecraft:large_biomes\"}}",
        },
        WorldPresetDimension {
            identifier: Identifier::parse_static("minecraft:the_end"),
            stem: "{\"type\":\"minecraft:the_end\",\"generator\":{\"type\":\"minecraft:noise\",\"biome_source\":{\"type\":\"minecraft:the_end\"},\"settings\":\"minecraft:end\"}}",
        },
        WorldPresetDimension {
            identifier: Identifier::parse_static("minecraft:the_nether"),
            stem: "{\"type\":\"minecraft:the_nether\",\"generator\":{\"type\":\"minecraft:noise\",\"biome_source\":{\"type\":\"minecraft:multi_noise\",\"preset\":\"minecraft:nether\"},\"settings\":\"minecraft:nether\"}}",
        },
    ];
    pub const LARGE_BIOMES: WorldPreset = WorldPreset {
        dimensions: &Self::LARGE_BIOMES_DIMENSIONS,
    };
    const AMPLIFIED_DIMENSIONS: [WorldPresetDimension; 3usize] = [
        WorldPresetDimension {
            identifier: Identifier::parse_static("minecraft:overworld"),
            stem: "{\"type\":\"minecraft:overworld\",\"generator\":{\"type\":\"minecraft:noise\",\"biome_source\":{\"type\":\"minecraft:multi_noise\",\"preset\":\"minecraft:overworld\"},\"settings\":\"minecraft:amplified\"}}",
        },
        WorldPresetDimension {
            identifier: Identifier::parse_static("minecraft:the_end"),
            stem: "{\"type\":\"minecraft:the_end\",\"generator\":{\"type\":\"minecraft:noise\",\"biome_source\":{\"type\":\"minecraft:the_end\"},\"settings\":\"minecraft:end\"}}",
        },
        WorldPresetDimension {
            identifier: Identifier::parse_static("minecraft:the_nether"),
            stem: "{\"type\":\"minecraft:the_nether\",\"generator\":{\"type\":\"minecraft:noise\",\"biome_source\":{\"type\":\"minecraft:multi_noise\",\"preset\":\"minecraft:nether\"},\"settings\":\"minecraft:nether\"}}",
        },
    ];
    pub const AMPLIFIED: WorldPreset = WorldPreset {
        dimensions: &Self::AMPLIFIED_DIMENSIONS,
    };
    const SINGLE_BIOME_SURFACE_DIMENSIONS: [WorldPresetDimension; 3usize] = [
        WorldPresetDimension {
            identifier: Identifier::parse_static("minecraft:overworld"),
            stem: "{\"type\":\"minecraft:overworld\",\"generator\":{\"type\":\"minecraft:noise\",\"biome_source\":{\"type\":\"minecraft:fixed\",\"biome\":\"minecraft:plains\"},\"settings\":\"minecraft:overworld\"}}",
        },
        WorldPresetDimension {
            identifier: Identifier::parse_static("minecraft:the_end"),
            stem: "{\"type\":\"minecraft:the_end\",\"generator\":{\"type\":\"minecraft:noise\",\"biome_source\":{\"type\":\"minecraft:the_end\"},\"settings\":\"minecraft:end\"}}",
        },
        WorldPresetDimension {
            identifier: Identifier::parse_static("minecraft:the_nether"),
            stem: "{\"type\":\"minecraft:the_nether\",\"generator\":{\"type\":\"minecraft:noise\",\"biome_source\":{\"type\":\"minecraft:multi_noise\",\"preset\":\"minecraft:nether\"},\"settings\":\"minecraft:nether\"}}",
        },
    ];
    pub const SINGLE_BIOME_SURFACE: WorldPreset = WorldPreset {
        dimensions: &Self::SINGLE_BIOME_SURFACE_DIMENSIONS,
    };
}
const STATIC_ENTRIES: [WorldPreset; 5usize] = [
    WorldPreset::NORMAL,
    WorldPreset::FLAT,
    WorldPreset::LARGE_BIOMES,
    WorldPreset::AMPLIFIED,
    WorldPreset::SINGLE_BIOME_SURFACE,
];
const STATIC_IDENTIFIERS: [Identifier; 5usize] = [
    Identifier::parse_static("minecraft:normal"),
    Identifier::parse_static("minecraft:flat"),
    Identifier::parse_static("minecraft:large_biomes"),
    Identifier::parse_static("minecraft:amplified"),
    Identifier::parse_static("minecraft:single_biome_surface"),
];
bootstrap_provider! { WORLD_PRESET_REGISTRY : Arc < dyn Registry > => "minecraft:worldgen" , || { vec ! [RegistryEntry :: new (Identifier :: vanilla_static ("world_preset") , RegistryBuilder :: < WorldPreset > :: new_static (& Identifier :: parse_static ("minecraft:worldgen/world_preset") , & STATIC_ENTRIES , & STATIC_IDENTIFIERS ,) . unwrap () . arc_dyn () ,)] } }
