#![allow(clippy::unwrap_used)]

use pumpkin_data::{chunk_gen_settings::GenerationSettings, dimension::Dimension};
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
