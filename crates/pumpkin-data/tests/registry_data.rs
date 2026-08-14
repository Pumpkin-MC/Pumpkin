#![allow(clippy::unwrap_used)]

use pumpkin_data::{
    chunk_gen_settings::GenerationSettings, dimension::Dimension, structures::StructureSet,
    world_preset::WorldPreset,
};
use pumpkin_registry::{
    BOOTSTRAP, DataKey, Registry, RegistryBuilder, bootstrap::BootstrapManager,
};
use pumpkin_util::identifier::Identifier;
use std::sync::Arc;

fn root() -> Arc<dyn Registry> {
    let _ = BOOTSTRAP.set(BootstrapManager::new());

    RegistryBuilder::<Arc<dyn Registry>>::frozen(&Identifier::vanilla_static("root"))
        .unwrap()
        .arc_dyn()
}

#[test]
fn dimension_type_registry_contains_vanilla_data() {
    let root = root();

    let cases = [
        (
            DataKey::<Dimension>::new("minecraft:dimension_type/minecraft:overworld"),
            &Dimension::OVERWORLD,
        ),
        (
            DataKey::<Dimension>::new("minecraft:dimension_type/minecraft:overworld_caves"),
            &Dimension::OVERWORLD_CAVES,
        ),
        (
            DataKey::<Dimension>::new("minecraft:dimension_type/minecraft:the_end"),
            &Dimension::THE_END,
        ),
        (
            DataKey::<Dimension>::new("minecraft:dimension_type/minecraft:the_nether"),
            &Dimension::THE_NETHER,
        ),
    ];

    for (key, expected) in cases {
        let actual = key.get_blocking(root.as_ref()).unwrap();
        assert_eq!(*actual, *expected);
    }
}

#[test]
fn noise_settings_registry_contains_vanilla_data() {
    let root = root();

    let cases = [
        (
            DataKey::<GenerationSettings>::new(
                "minecraft:worldgen/minecraft:noise_settings/minecraft:amplified",
            ),
            &GenerationSettings::AMPLIFIED,
        ),
        (
            DataKey::<GenerationSettings>::new(
                "minecraft:worldgen/minecraft:noise_settings/minecraft:caves",
            ),
            &GenerationSettings::CAVES,
        ),
        (
            DataKey::<GenerationSettings>::new(
                "minecraft:worldgen/minecraft:noise_settings/minecraft:end",
            ),
            &GenerationSettings::END,
        ),
        (
            DataKey::<GenerationSettings>::new(
                "minecraft:worldgen/minecraft:noise_settings/minecraft:floating_islands",
            ),
            &GenerationSettings::FLOATING_ISLANDS,
        ),
        (
            DataKey::<GenerationSettings>::new(
                "minecraft:worldgen/minecraft:noise_settings/minecraft:large_biomes",
            ),
            &GenerationSettings::LARGE_BIOMES,
        ),
        (
            DataKey::<GenerationSettings>::new(
                "minecraft:worldgen/minecraft:noise_settings/minecraft:nether",
            ),
            &GenerationSettings::NETHER,
        ),
        (
            DataKey::<GenerationSettings>::new(
                "minecraft:worldgen/minecraft:noise_settings/minecraft:overworld",
            ),
            &GenerationSettings::OVERWORLD,
        ),
    ];

    for (key, expected) in cases {
        let actual = key.get_blocking(root.as_ref()).unwrap();

        assert_eq!(actual.sea_level, expected.sea_level);
        assert_eq!(actual.shape.min_y, expected.shape.min_y);
        assert_eq!(actual.shape.height, expected.shape.height);
        assert_eq!(actual.aquifers_enabled, expected.aquifers_enabled);
        assert_eq!(actual.ore_veins_enabled, expected.ore_veins_enabled);
        assert_eq!(actual.default_block, expected.default_block);
        assert_eq!(actual.default_fluid, expected.default_fluid);
    }
}

#[test]
fn structure_set_registry_contains_vanilla_data() {
    let root = root();
    let key = DataKey::<StructureSet>::new(
        "minecraft:worldgen/minecraft:structure_set/minecraft:strongholds",
    );
    let set = key.get_blocking(root.as_ref()).unwrap();

    assert_eq!(set.placement.salt, StructureSet::STRONGHOLDS.placement.salt);
    assert_eq!(set.structures.len(), StructureSet::STRONGHOLDS.structures.len());
}

#[test]
fn world_preset_registry_contains_vanilla_presets() {
    let root = root();

    for name in [
        "normal",
        "flat",
        "large_biomes",
        "amplified",
        "single_biome_surface",
    ] {
        let key = DataKey::<WorldPreset>::owned(format!(
            "minecraft:worldgen/minecraft:world_preset/minecraft:{name}"
        ));
        let preset = key.get_blocking(root.as_ref()).unwrap();
        assert_eq!(preset.dimensions.len(), 3);
    }
}

#[test]
fn flat_world_preset_preserves_generator_settings() {
    let root = root();
    let key =
        DataKey::<WorldPreset>::new("minecraft:worldgen/minecraft:world_preset/minecraft:flat");
    let preset = key.get_blocking(root.as_ref()).unwrap();
    let overworld = preset
        .dimensions
        .iter()
        .find(|dimension| dimension.identifier == Identifier::vanilla_static("overworld"))
        .unwrap();

    let stem: serde_json::Value = serde_json::from_str(overworld.stem).unwrap();
    assert_eq!(stem["generator"]["type"], "minecraft:flat");
    assert_eq!(stem["generator"]["settings"]["biome"], "minecraft:plains");
    assert_eq!(
        stem["generator"]["settings"]["layers"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    assert_eq!(
        stem["generator"]["settings"]["structure_overrides"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
}
