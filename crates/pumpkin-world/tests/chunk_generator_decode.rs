#![allow(clippy::unwrap_used)]

use pumpkin_codecs::{DataResult, Decode, DynamicOps, json_ops::JsonOps};
use pumpkin_data::structures::StructureSet;
use pumpkin_nbt::{nbt_ops::NbtOps, tag::NbtTag};
use pumpkin_util::world_seed::Seed;
use pumpkin_world::dimension_type::DimensionType as Dimension;
use std::sync::{Arc, LazyLock};

use pumpkin_registry::{
    BOOTSTRAP, DataKey, ROOT, Registry, RegistryBuilder, bootstrap::BootstrapManager,
};
use pumpkin_util::identifier::Identifier;
use pumpkin_world::{
    ProtoChunk,
    chunk_system::generation_cache::Cache,
    generation::generator::{
        ChunkGenerator, ChunkGeneratorDecode, ChunkGeneratorType, VanillaGenerator,
        flat::{FlatGenerator as RuntimeFlatGenerator, FlatGeneratorConfig},
    },
    world::WorldPortalExt,
};
use serde_json::json;

static OVERWORLD: LazyLock<Dimension> = LazyLock::new(|| {
    Dimension::parse(
        json!({
            "ambient_light": 0.0,
            "attributes": {},
            "coordinate_scale": 1.0,
            "has_ceiling": false,
            "has_ender_dragon_fight": false,
            "has_skylight": true,
            "height": 384,
            "infiniburn": "#minecraft:infiniburn_overworld",
            "logical_height": 384,
            "min_y": -64,
            "monster_spawn_block_light_limit": 0,
            "monster_spawn_light_level": 7
        }),
        &JsonOps,
    )
    .into_result()
    .unwrap()
});

fn overworld() -> Dimension {
    (*OVERWORLD).clone()
}

struct NoiseGenerator {
    seed: u64,
}

impl ChunkGenerator for NoiseGenerator {
    fn dimension(&self) -> &Dimension {
        &OVERWORLD
    }

    fn seed(&self) -> u64 {
        self.seed
    }

    fn sea_level(&self) -> i32 {
        63
    }

    fn step_to_biomes(&self, _chunk: &mut ProtoChunk) {}

    fn step_to_structure_start(
        &self,
        _cache: &mut Cache,
        _chunk_index: usize,
        _block_registry: &dyn WorldPortalExt,
    ) {
    }

    fn step_to_structure_references(
        &self,
        _cache: &mut Cache,
        _chunk_index: usize,
        _block_registry: &dyn WorldPortalExt,
    ) {
    }

    fn rebuild_structure_starts(&self, _chunk: &mut ProtoChunk) {}

    fn rebuild_structure_references(&self, _chunk: &mut ProtoChunk) {}

    fn step_to_noise(&self, _chunk: &mut ProtoChunk) {}

    fn step_to_surface(&self, _chunk: &mut ProtoChunk) {}

    fn step_to_carvers(&self, _chunk: &mut ProtoChunk) {}

    fn step_to_features(
        &self,
        _cache: &mut Cache,
        _chunk_index: usize,
        _block_registry: &dyn WorldPortalExt,
    ) {
    }
}

impl Decode for NoiseGenerator {
    fn decode<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<(Self, O::Value)> {
        u64::decode(input, ops).map(|(seed, remainder)| (Self { seed }, remainder))
    }
}

impl ChunkGeneratorDecode for NoiseGenerator {
    type Config = u64;

    fn from_config(seed: Seed, _dimension: Dimension, _config: Self::Config) -> DataResult<Self> {
        DataResult::new_success(Self { seed: seed.0 })
    }
}

struct FlatGenerator {
    seed: u64,
}

impl ChunkGenerator for FlatGenerator {
    fn dimension(&self) -> &Dimension {
        &OVERWORLD
    }

    fn seed(&self) -> u64 {
        self.seed
    }

    fn sea_level(&self) -> i32 {
        63
    }

    fn step_to_biomes(&self, _chunk: &mut ProtoChunk) {}

    fn step_to_structure_start(
        &self,
        _cache: &mut Cache,
        _chunk_index: usize,
        _block_registry: &dyn WorldPortalExt,
    ) {
    }

    fn step_to_structure_references(
        &self,
        _cache: &mut Cache,
        _chunk_index: usize,
        _block_registry: &dyn WorldPortalExt,
    ) {
    }

    fn rebuild_structure_starts(&self, _chunk: &mut ProtoChunk) {}

    fn rebuild_structure_references(&self, _chunk: &mut ProtoChunk) {}

    fn step_to_noise(&self, _chunk: &mut ProtoChunk) {}

    fn step_to_surface(&self, _chunk: &mut ProtoChunk) {}

    fn step_to_carvers(&self, _chunk: &mut ProtoChunk) {}

    fn step_to_features(
        &self,
        _cache: &mut Cache,
        _chunk_index: usize,
        _block_registry: &dyn WorldPortalExt,
    ) {
    }
}

impl Decode for FlatGenerator {
    fn decode<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<(Self, O::Value)> {
        u64::decode(input, ops).map(|(seed, remainder)| (Self { seed }, remainder))
    }
}

impl ChunkGeneratorDecode for FlatGenerator {
    type Config = u64;

    fn from_config(seed: Seed, _dimension: Dimension, _config: Self::Config) -> DataResult<Self> {
        DataResult::new_success(Self { seed: seed.0 })
    }
}

fn init_registries() {
    BOOTSTRAP.get_or_init(BootstrapManager::new);
    ROOT.get_or_init(|| {
        RegistryBuilder::<Arc<dyn Registry>>::frozen(&Identifier::vanilla_static("root")).unwrap()
    });
}

#[test]
fn builtin_generator_type_resolves_from_registry() {
    init_registries();

    let root = ROOT.get().unwrap();
    let generator_type = DataKey::<ChunkGeneratorType>::new(
        "minecraft:worldgen/minecraft:chunk_generator_type/minecraft:noise",
    )
    .get(root)
    .unwrap();

    let generator = generator_type
        .decode(
            json!({
                "settings": "minecraft:overworld",
                "biome_source": {
                    "type": "minecraft:multi_noise",
                    "preset": "minecraft:overworld"
                }
            }),
            &JsonOps,
            Seed(1234),
            overworld(),
        )
        .into_result()
        .unwrap();

    assert_eq!(generator.seed(), 1234);
}

#[test]
fn generator_type_decodes_from_json_ops() {
    let generator_type = ChunkGeneratorType::new::<NoiseGenerator>();

    let generator = generator_type
        .decode(json!(42), &JsonOps, Seed(42), overworld())
        .into_result()
        .unwrap();

    assert_eq!(generator.seed(), 42);
}

#[test]
fn same_generator_type_decodes_from_nbt_ops() {
    let generator_type = ChunkGeneratorType::new::<NoiseGenerator>();

    let generator = generator_type
        .decode(NbtTag::Long(42), &NbtOps, Seed(42), overworld())
        .into_result()
        .unwrap();

    assert_eq!(generator.seed(), 42);
}

#[test]
fn flat_generator_config_decodes_structure_overrides() {
    init_registries();

    let settings = json!({
        "biome": "minecraft:plains",
        "features": false,
        "lakes": false,
        "layers": [
            { "block": "minecraft:bedrock", "height": 1 },
            { "block": "minecraft:dirt", "height": 2 },
            { "block": "minecraft:grass_block", "height": 1 }
        ],
        "structure_overrides": [
            "minecraft:strongholds",
            "minecraft:villages"
        ]
    });

    let config = FlatGeneratorConfig::parse(json!({ "settings": settings }), &JsonOps)
        .into_result()
        .unwrap();
    assert_eq!(config.biome, "minecraft:plains");
    assert_eq!(config.layers.len(), 3);
    assert_eq!(config.structure_overrides.len(), 2);

    let generator = RuntimeFlatGenerator::from_config(Seed(42), overworld(), config).unwrap();

    let root = ROOT.get().unwrap();
    let strongholds = DataKey::<StructureSet>::new(
        "minecraft:worldgen/minecraft:structure_set/minecraft:strongholds",
    )
    .get(root)
    .unwrap();
    let villages = DataKey::<StructureSet>::new(
        "minecraft:worldgen/minecraft:structure_set/minecraft:villages",
    )
    .get(root)
    .unwrap();

    let resolved_salts: Vec<_> = generator
        .structure_overrides
        .iter()
        .map(|key| key.get(root).unwrap().placement.salt)
        .collect();

    assert!(resolved_salts.contains(&strongholds.placement.salt));
    assert!(resolved_salts.contains(&villages.placement.salt));
    assert_eq!(generator.structure_overrides.len(), 2);
}

#[test]
fn vanilla_noise_generator_decodes_overworld_config() {
    init_registries();
    let generator_type = ChunkGeneratorType::new::<VanillaGenerator>();
    let generator = generator_type
        .decode(
            json!({
                "settings": "minecraft:overworld",
                "biome_source": {
                    "type": "minecraft:multi_noise",
                    "preset": "minecraft:overworld"
                }
            }),
            &JsonOps,
            Seed(1234),
            overworld(),
        )
        .into_result()
        .unwrap();

    assert_eq!(generator.seed(), 1234);
    assert!(generator.dimension().is_overworld_like());
}

#[test]
fn vanilla_noise_generator_decodes_amplified_config() {
    init_registries();
    let generator_type = ChunkGeneratorType::new::<VanillaGenerator>();
    let generator = generator_type
        .decode(
            json!({
                "settings": "minecraft:amplified",
                "biome_source": {
                    "type": "minecraft:multi_noise",
                    "preset": "minecraft:overworld"
                }
            }),
            &JsonOps,
            Seed(7),
            overworld(),
        )
        .into_result()
        .unwrap();

    assert_eq!(generator.seed(), 7);
}

#[test]
fn vanilla_noise_generator_decodes_fixed_biome_source() {
    init_registries();
    let generator_type = ChunkGeneratorType::new::<VanillaGenerator>();
    let generator = generator_type
        .decode(
            json!({
                "settings": "minecraft:overworld",
                "biome_source": {
                    "type": "minecraft:fixed",
                    "biome": "minecraft:plains"
                }
            }),
            &JsonOps,
            Seed(99),
            overworld(),
        )
        .into_result()
        .unwrap();

    assert_eq!(generator.seed(), 99);
}

#[test]
fn generator_types_can_live_in_one_dispatch_table() {
    let generator_types = [
        (
            "minecraft:noise",
            ChunkGeneratorType::new::<NoiseGenerator>(),
        ),
        ("minecraft:flat", ChunkGeneratorType::new::<FlatGenerator>()),
    ];

    let generator_type = generator_types
        .iter()
        .find(|(identifier, _)| *identifier == "minecraft:flat")
        .map(|(_, generator_type)| generator_type)
        .unwrap();

    let generator = generator_type
        .decode(json!(9001), &JsonOps, Seed(9001), overworld())
        .into_result()
        .unwrap();

    assert_eq!(generator.seed(), 9001);
}
