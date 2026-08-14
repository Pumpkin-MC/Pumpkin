use std::{collections::BTreeMap, fs};

use heck::ToShoutySnakeCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;

#[derive(Deserialize)]
struct WorldPresetFile {
    dimensions: BTreeMap<String, serde_json::Value>,
}

const PRESETS: [(&str, &str, Option<(&str, &[&str])>); 5] = [
    ("default", "minecraft:normal", None),
    (
        "super_flat",
        "minecraft:flat",
        Some(("minecraft:overworld", &["generator", "settings"])),
    ),
    ("large_biomes", "minecraft:large_biomes", None),
    ("amplified", "minecraft:amplified", None),
    (
        "single_biome",
        "minecraft:single_biome_surface",
        Some(("minecraft:overworld", &["generator", "biome_source"])),
    ),
];

pub fn build() -> TokenStream {
    let mut preset_consts = TokenStream::new();
    let mut generator_settings_statics = TokenStream::new();
    let mut static_entries = TokenStream::new();
    let mut identifiers = TokenStream::new();

    for (file_name, preset_name, generator_settings) in PRESETS {
        let path = format!("../../assets/world_preset/{file_name}.json");
        let preset: WorldPresetFile = serde_json::from_str(
            &fs::read_to_string(&path)
                .unwrap_or_else(|_| panic!("Missing world preset source: {path}")),
        )
        .unwrap_or_else(|_| panic!("Failed to parse world preset source: {path}"));

        let const_name = format_ident!(
            "{}",
            preset_name
                .strip_prefix("minecraft:")
                .unwrap_or(preset_name)
                .to_shouty_snake_case()
        );
        let dimensions_name = format_ident!("{const_name}_DIMENSIONS");
        let generator_settings = generator_settings.map_or_else(
            || quote! { None },
            |(world, path)| {
                let path_name = format_ident!("{const_name}_GENERATOR_SETTINGS_PATH");
                let config_name = format_ident!("{const_name}_GENERATOR_SETTINGS");
                let path_len = path.len();
                generator_settings_statics.extend(quote! {
                    static #path_name: [&str; #path_len] = [#(#path),*];
                    static #config_name: GeneratorSettingsConfig = GeneratorSettingsConfig {
                        world: Identifier::parse_static(#world),
                        path: &#path_name,
                    };
                });
                quote! { Some(&#config_name) }
            },
        );

        let dimensions = preset.dimensions.iter().map(|(name, stem)| {
            let stem = serde_json::to_string(stem).expect("Failed to serialize dimension stem");
            quote! {
                WorldPresetDimension {
                    identifier: Identifier::parse_static(#name),
                    stem: #stem,
                },
            }
        });
        let len = preset.dimensions.len();

        preset_consts.extend(quote! {
            const #dimensions_name: [WorldPresetDimension; #len] = [
                #(#dimensions)*
            ];

            pub const #const_name: WorldPreset = WorldPreset {
                dimensions: &Self::#dimensions_name,
                generator_settings: #generator_settings,
            };
        });

        static_entries.extend(quote! {
            WorldPreset::#const_name,
        });
        identifiers.extend(quote! {
            Identifier::parse_static(#preset_name),
        });
    }

    let len = PRESETS.len();

    quote! {
        use std::sync::Arc;

        use pumpkin_registry::{
            Registry, RegistryBuilder,
            bootstrap::RegistryEntry,
            bootstrap_provider,
        };
        use pumpkin_util::identifier::Identifier;

        #[derive(Debug)]
        pub struct WorldPresetDimension {
            pub identifier: Identifier,
            pub stem: &'static str,
        }

        #[derive(Debug)]
        pub struct GeneratorSettingsConfig {
            /// World whose saved dimension stem is modified by `generator-settings`.
            pub world: Identifier,
            /// Path within that dimension stem to the object receiving the overrides.
            pub path: &'static [&'static str],
        }

        #[derive(Debug)]
        pub struct WorldPreset {
            pub dimensions: &'static [WorldPresetDimension],
            /// Location modified by `generator-settings`, if supported.
            pub generator_settings: Option<&'static GeneratorSettingsConfig>,
        }

        #generator_settings_statics

        impl WorldPreset {
            #preset_consts
        }

        const STATIC_ENTRIES: [WorldPreset; #len] = [
            #static_entries
        ];

        const STATIC_IDENTIFIERS: [Identifier; #len] = [
            #identifiers
        ];

        bootstrap_provider! {
            WORLD_PRESET_REGISTRY: Arc<dyn Registry> => "minecraft:worldgen",
            || {
                vec![RegistryEntry::new(
                    Identifier::vanilla_static("world_preset"),
                    RegistryBuilder::<WorldPreset>::new_static(
                        &Identifier::parse_static("minecraft:worldgen/world_preset"),
                        &STATIC_ENTRIES,
                        &STATIC_IDENTIFIERS,
                    )
                    .unwrap()
                    .arc_dyn(),
                )]
            }
        }
    }
}
