use std::{collections::BTreeMap, fs};

use crate::placed_feature::value_to_int_provider;
use heck::ToShoutySnakeCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;

/// Raw deserialization shape for a single dimension entry from `dimension.json`.
#[derive(Deserialize)]
pub struct Dimension {
    /// Whether this dimension has a skylight source (i.e. is not a cave or the Nether).
    pub has_skylight: bool,
    /// Whether this dimension has a bedrock ceiling (e.g. the Nether).
    pub has_ceiling: bool,
    /// Ambient light level added to all blocks, bypassing the normal sky/block-light calculation.
    pub ambient_light: f32,
    /// Coordinate scale factor mapping a position in this dimension to overworld coordinates.
    pub coordinate_scale: f64,
    /// Minimum Y level (inclusive) of the buildable/chunk range.
    pub min_y: i32,
    /// Total height (in blocks) of the buildable/chunk range.
    pub height: i32,
    /// Maximum Y level usable by mob AI and portals (can be less than `min_y + height`).
    pub logical_height: i32,
    /// Tag key for blocks that act as infinite burn sources (e.g. `"minecraft:infiniburn_overworld"`).
    pub infiniburn: String,
    pub monster_spawn_light_level: serde_json::Value,
    pub monster_spawn_block_light_limit: u8,
    /// Environment Attribute values supplied by this dimension.
    #[serde(default)]
    pub attributes: BTreeMap<String, serde_json::Value>,
    /// Whether time-based behaviors should behave as though time is fixed.
    #[serde(default)]
    pub has_fixed_time: bool,
    /// Whether an Ender Dragon fight can exist in this dimension.
    #[serde(default)]
    pub has_ender_dragon_fight: bool,
    /// Skybox rendering type (`overworld`, `end`, or `none`).
    #[serde(default)]
    pub skybox: Option<String>,
    /// Cardinal lighting mode (`default` or `nether`).
    #[serde(default)]
    pub cardinal_light: Option<String>,
    /// Timeline ID, list of IDs, or timeline tag active in this dimension.
    #[serde(default)]
    pub timelines: Option<serde_json::Value>,
    /// Default world clock used by time commands and time markers.
    #[serde(default)]
    pub default_clock: Option<String>,
}

/// Generates the `TokenStream` for the `Dimension` struct, its constants, and `from_name` lookup.
pub fn build() -> TokenStream {
    let dimensions: BTreeMap<String, Dimension> = serde_json::from_str(
        &fs::read_to_string("../../assets/dimension.json").expect("Missing dimension.json"),
    )
    .expect("Failed to parse dimension.json");

    let mut variants = TokenStream::new();
    let mut static_entries = TokenStream::new();
    let mut identifiers = TokenStream::new();
    let len = dimensions.len();

    // Iterate with index to generate a unique numeric ID
    for (id, (name, dim)) in dimensions.into_iter().enumerate() {
        let id = id as u8; // Overworld=0, Nether=1, End=2 (usually)
        let format_name = format_ident!(
            "{}",
            name.strip_prefix("minecraft:")
                .unwrap_or(&name)
                .to_shouty_snake_case()
        );

        let attributes = dim.attributes.iter().map(|(identifier, value)| {
            let value =
                serde_json::to_string(value).expect("Failed to serialize environment attribute");
            quote! {
                EnvironmentAttribute {
                    identifier: #identifier,
                    value: #value,
                },
            }
        });

        let timelines: Vec<String> = match dim.timelines.as_ref() {
            None => Vec::new(),
            Some(serde_json::Value::String(value)) => vec![value.clone()],
            Some(serde_json::Value::Array(values)) => values
                .iter()
                .map(|value| {
                    value
                        .as_str()
                        .expect("Dimension timelines must contain string IDs")
                        .to_string()
                })
                .collect(),
            Some(_) => panic!("Dimension timelines must be an ID, tag, or list of IDs"),
        };

        let monster_spawn_light_level = value_to_int_provider(&dim.monster_spawn_light_level);

        let monster_spawn_block_light_limit = dim.monster_spawn_block_light_limit;
        let ambient_light = dim.ambient_light;
        let coordinate_scale = dim.coordinate_scale;
        let height = dim.height;
        let min_y = dim.min_y;
        let logical_height = dim.logical_height;
        let has_skylight = dim.has_skylight;
        let has_ceiling = dim.has_ceiling;
        // normalize infiniburn to always have namespace
        let infiniburn = if dim.infiniburn.contains(':') {
            dim.infiniburn.clone()
        } else {
            format!("minecraft:{}", dim.infiniburn)
        };
        let has_fixed_time = dim.has_fixed_time;
        let has_ender_dragon_fight = dim.has_ender_dragon_fight;
        let skybox = dim.skybox;
        let cardinal_light = dim.cardinal_light;
        let default_clock = dim.default_clock;

        let minecraft_name = if name.contains(':') {
            name.clone()
        } else {
            format!("minecraft:{name}")
        };

        let skybox = skybox.map_or_else(|| quote! { None }, |value| quote! { Some(#value) });
        let cardinal_light =
            cardinal_light.map_or_else(|| quote! { None }, |value| quote! { Some(#value) });
        let default_clock =
            default_clock.map_or_else(|| quote! { None }, |value| quote! { Some(#value) });

        variants.extend(quote! {
            pub const #format_name: Self = Self {
                id: #id,
                minecraft_name: #minecraft_name,
                has_skylight: #has_skylight,
                has_ceiling: #has_ceiling,
                coordinate_scale: #coordinate_scale,
                min_y: #min_y,
                height: #height,
                logical_height: #logical_height,
                infiniburn: #infiniburn,
                ambient_light: #ambient_light,
                monster_spawn_light_level: #monster_spawn_light_level,
                monster_spawn_block_light_limit: #monster_spawn_block_light_limit,
                attributes: &[#(#attributes)*],
                has_fixed_time: #has_fixed_time,
                has_ender_dragon_fight: #has_ender_dragon_fight,
                skybox: #skybox,
                cardinal_light: #cardinal_light,
                timelines: &[#(#timelines),*],
                default_clock: #default_clock,
            };
        });

        static_entries.extend(quote! {
            Dimension::#format_name,
        });

        identifiers.extend(quote! {
            Identifier::parse_static(#minecraft_name),
        });
    }

    quote!(
        use pumpkin_util::{identifier::Identifier, math::int_provider::{
            BiasedToBottomIntProvider, ClampedIntProvider, ClampedNormalIntProvider, ConstantIntProvider,
            IntProvider, NormalIntProvider, TrapezoidIntProvider, UniformIntProvider, WeightedEntry,
            WeightedListIntProvider,
        }};
        use std::sync::Arc;
        use pumpkin_registry::{
            Registry, RegistryBuilder, bootstrap::RegistryEntry, bootstrap_provider,
        };

        #[derive(Debug, Clone, Copy)]
        pub struct EnvironmentAttribute {
            pub identifier: &'static str,
            /// JSON representation of the Environment Attribute value or modifier.
            pub value: &'static str,
        }

        #[derive(Debug, Clone)]
        pub struct Dimension {
            pub id: u8,
            pub minecraft_name: &'static str,
            pub has_skylight: bool,
            pub has_ceiling: bool,
            pub coordinate_scale: f64,
            pub min_y: i32,
            pub height: i32,
            pub logical_height: i32,
            pub infiniburn: &'static str,
            pub ambient_light: f32,
            pub monster_spawn_light_level: IntProvider,
            pub monster_spawn_block_light_limit: u8,
            pub attributes: &'static [EnvironmentAttribute],
            pub has_fixed_time: bool,
            pub has_ender_dragon_fight: bool,
            pub skybox: Option<&'static str>,
            pub cardinal_light: Option<&'static str>,
            pub timelines: &'static [&'static str],
            pub default_clock: Option<&'static str>,
        }

        impl Dimension {
            #variants
        }

        const STATIC_ENTRIES: [Dimension; #len] = [
            #static_entries
        ];

        const STATIC_IDENTIFIERS: [Identifier; #len] = [
            #identifiers
        ];

        bootstrap_provider! {
            DIMENSION_TYPE_REGISTRY: Arc<dyn Registry> => "minecraft:root",
            || {
                vec![RegistryEntry::new(
                    Identifier::vanilla_static("dimension_type"),
                    RegistryBuilder::<Dimension>::new_static(
                        &Identifier::vanilla_static("dimension_type"),
                        &STATIC_ENTRIES,
                        &STATIC_IDENTIFIERS,
                    )
                    .unwrap()
                    .arc_dyn(),
                )]
            }
        }

        impl PartialEq for Dimension {
            fn eq(&self, other: &Self) -> bool {
                 self.id == other.id
            }
        }

        impl Eq for Dimension {}
    )
}
