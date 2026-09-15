use std::{collections::BTreeMap, fs};

use crate::placed_feature::value_to_int_provider;
use heck::ToShoutySnakeCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;

/// Parses a CSS-style hex color string (e.g. `"#78a7ff"` or `"#ccffffff"`) into a signed 32-bit integer.
///
/// # Returns
/// The color as an `i32`, or `None` if the input is not a valid hex color.
fn parse_hex_color(s: &str) -> Option<i32> {
    if let Some(stripped) = s.strip_prefix('#') {
        u32::from_str_radix(stripped, 16).ok().map(|v| v as i32)
    } else {
        None
    }
}

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
    /// Fixed day-time value in this dimension, or `None` if time progresses normally.
    #[serde(rename = "fixed_time")]
    pub fixed_time: Option<i64>,
    /// Whether time is fixed in this dimension (modern 26.2 field).
    #[serde(default, rename = "has_fixed_time")]
    pub has_fixed_time: Option<bool>,
    /// Environment attributes map (visual, gameplay, audio).
    #[serde(default)]
    pub attributes: Option<serde_json::Value>,
    /// Optional timeline resource key controlling day/night progression.
    #[serde(default)]
    pub timelines: Option<String>,
}

/// Generates the `TokenStream` for the `Dimension` struct, its constants, and `from_name` lookup.
pub fn build() -> TokenStream {
    let dir = std::path::Path::new("../../assets/datapacks/26_2/data/minecraft/dimension_type");
    let mut dimensions: BTreeMap<String, Dimension> = BTreeMap::new();
    let mut entries: Vec<_> = fs::read_dir(dir)
        .expect("Missing dimension_type directory")
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
        .collect();
    entries.sort_by_key(|e| e.path());

    for entry in entries {
        let path = entry.path();
        let stem = path.file_stem().unwrap().to_string_lossy().into_owned();
        let key = format!("minecraft:{stem}");
        let content = fs::read_to_string(&path).expect("Failed to read dimension file");
        let dim: Dimension =
            serde_json::from_str(&content).expect("Failed to parse dimension JSON");
        dimensions.insert(key, dim);
    }

    let mut variants = TokenStream::new();
    let mut name_to_type = TokenStream::new();

    // Iterate with index to generate a unique numeric ID
    for (id, (name, dim)) in dimensions.into_iter().enumerate() {
        let id = id as u8; // Overworld=0, Nether=1, End=2 (usually)
        let format_name = format_ident!(
            "{}",
            name.strip_prefix("minecraft:")
                .unwrap_or(&name)
                .to_shouty_snake_case()
        );

        let attrs = dim.attributes.as_ref();

        // Visual environment attributes
        let sky_color = attrs
            .and_then(|a| a.get("minecraft:visual/sky_color"))
            .and_then(|v| v.as_str())
            .and_then(parse_hex_color);
        let fog_color = attrs
            .and_then(|a| a.get("minecraft:visual/fog_color"))
            .and_then(|v| v.as_str())
            .and_then(parse_hex_color);
        let cloud_color = attrs
            .and_then(|a| a.get("minecraft:visual/cloud_color"))
            .and_then(|v| v.as_str())
            .and_then(parse_hex_color);
        let ambient_light_color = attrs
            .and_then(|a| a.get("minecraft:visual/ambient_light_color"))
            .and_then(|v| v.as_str())
            .and_then(parse_hex_color);
        let sky_light_color = attrs
            .and_then(|a| a.get("minecraft:visual/sky_light_color"))
            .and_then(|v| v.as_str())
            .and_then(parse_hex_color);

        let sky_light_factor = attrs
            .and_then(|a| a.get("minecraft:visual/sky_light_factor"))
            .and_then(|v| v.as_f64())
            .map(|f| f as f32);
        let fog_start_distance = attrs
            .and_then(|a| a.get("minecraft:visual/fog_start_distance"))
            .and_then(|v| v.as_f64())
            .map(|f| f as f32);
        let fog_end_distance = attrs
            .and_then(|a| a.get("minecraft:visual/fog_end_distance"))
            .and_then(|v| v.as_f64())
            .map(|f| f as f32);
        let cloud_height = attrs
            .and_then(|a| a.get("minecraft:visual/cloud_height"))
            .and_then(|v| v.as_f64())
            .map(|f| f as f32);

        // Gameplay environment attributes
        let sky_light_level = attrs
            .and_then(|a| a.get("minecraft:gameplay/sky_light_level"))
            .and_then(|v| v.as_f64())
            .map(|f| f as f32);
        let water_evaporates = attrs
            .and_then(|a| a.get("minecraft:gameplay/water_evaporates"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let fast_lava = attrs
            .and_then(|a| a.get("minecraft:gameplay/fast_lava"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let respawn_anchor_works = attrs
            .and_then(|a| a.get("minecraft:gameplay/respawn_anchor_works"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let piglins_zombify = attrs
            .and_then(|a| a.get("minecraft:gameplay/piglins_zombify"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let snow_golem_melts = attrs
            .and_then(|a| a.get("minecraft:gameplay/snow_golem_melts"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let can_start_raid = attrs
            .and_then(|a| a.get("minecraft:gameplay/can_start_raid"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let nether_portal_spawns_piglin = attrs
            .and_then(|a| a.get("minecraft:gameplay/nether_portal_spawns_piglin"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let bed_rule_obj = attrs.and_then(|a| a.get("minecraft:gameplay/bed_rule"));
        let (can_sleep_token, can_set_spawn_token, explodes) = if let Some(b) = bed_rule_obj {
            let can_sleep = b
                .get("can_sleep")
                .and_then(|v| v.as_str())
                .unwrap_or("when_dark");
            let can_set_spawn = b
                .get("can_set_spawn")
                .and_then(|v| v.as_str())
                .unwrap_or("always");
            let explodes = b.get("explodes").and_then(|v| v.as_bool()).unwrap_or(false);

            let sleep_ident = match can_sleep {
                "always" => quote!(BedRuleOption::Always),
                "never" => quote!(BedRuleOption::Never),
                _ => quote!(BedRuleOption::WhenDark),
            };
            let spawn_ident = match can_set_spawn {
                "when_dark" => quote!(BedRuleOption::WhenDark),
                "never" => quote!(BedRuleOption::Never),
                _ => quote!(BedRuleOption::Always),
            };
            (sleep_ident, spawn_ident, explodes)
        } else {
            (
                quote!(BedRuleOption::WhenDark),
                quote!(BedRuleOption::Always),
                false,
            )
        };

        let fixed_time = if let Some(t) = dim.fixed_time {
            quote! { Some(#t) }
        } else {
            quote! { None }
        };
        let has_fixed_time = dim.has_fixed_time.unwrap_or(dim.fixed_time.is_some());

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
        let timelines = dim.timelines.map(|t| {
            if t.contains(':') {
                t
            } else {
                format!("minecraft:{}", t)
            }
        });

        let minecraft_name = if name.contains(':') {
            name.clone()
        } else {
            format!("minecraft:{name}")
        };

        let sky_color_literal = if let Some(c) = sky_color {
            quote! { Some(#c) }
        } else {
            quote! { None }
        };
        let fog_color_literal = if let Some(c) = fog_color {
            quote! { Some(#c) }
        } else {
            quote! { None }
        };
        let cloud_color_literal = if let Some(c) = cloud_color {
            quote! { Some(#c) }
        } else {
            quote! { None }
        };
        let ambient_light_color_literal = if let Some(c) = ambient_light_color {
            quote! { Some(#c) }
        } else {
            quote! { None }
        };
        let sky_light_color_literal = if let Some(c) = sky_light_color {
            quote! { Some(#c) }
        } else {
            quote! { None }
        };
        let sky_light_factor_literal = if let Some(f) = sky_light_factor {
            quote! { Some(#f) }
        } else {
            quote! { None }
        };
        let fog_start_distance_literal = if let Some(f) = fog_start_distance {
            quote! { Some(#f) }
        } else {
            quote! { None }
        };
        let fog_end_distance_literal = if let Some(f) = fog_end_distance {
            quote! { Some(#f) }
        } else {
            quote! { None }
        };
        let cloud_height_literal = if let Some(f) = cloud_height {
            quote! { Some(#f) }
        } else {
            quote! { None }
        };
        let sky_light_level_literal = if let Some(f) = sky_light_level {
            quote! { Some(#f) }
        } else {
            quote! { None }
        };
        let timelines_literal = if let Some(t) = timelines.clone() {
            quote! { Some(#t) }
        } else {
            quote! { None }
        };

        variants.extend(quote! {
            pub const #format_name: Self = Self {
                id: #id,
                minecraft_name: #minecraft_name,
                fixed_time: #fixed_time,
                has_fixed_time: #has_fixed_time,
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
                sky_color: #sky_color_literal,
                fog_color: #fog_color_literal,
                cloud_color: #cloud_color_literal,
                ambient_light_color: #ambient_light_color_literal,
                sky_light_color: #sky_light_color_literal,
                sky_light_factor: #sky_light_factor_literal,
                fog_start_distance: #fog_start_distance_literal,
                fog_end_distance: #fog_end_distance_literal,
                cloud_height: #cloud_height_literal,
                sky_light_level: #sky_light_level_literal,
                water_evaporates: #water_evaporates,
                fast_lava: #fast_lava,
                respawn_anchor_works: #respawn_anchor_works,
                piglins_zombify: #piglins_zombify,
                snow_golem_melts: #snow_golem_melts,
                can_start_raid: #can_start_raid,
                nether_portal_spawns_piglin: #nether_portal_spawns_piglin,
                bed_rule: BedRule {
                    can_sleep: #can_sleep_token,
                    can_set_spawn: #can_set_spawn_token,
                    explodes: #explodes,
                },
                timelines: #timelines_literal,
            };
        });

        name_to_type.extend(quote! {
            #minecraft_name => Some(&Self::#format_name),
        });
    }

    quote!(
        use pumpkin_util::math::int_provider::{
            BiasedToBottomIntProvider, ClampedIntProvider, TrapezoidIntProvider, ClampedNormalIntProvider,
            ConstantIntProvider, IntProvider, NormalIntProvider, UniformIntProvider,
            WeightedEntry, WeightedListIntProvider,
        };

        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum BedRuleOption {
            Always,
            WhenDark,
            Never,
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct BedRule {
            pub can_sleep: BedRuleOption,
            pub can_set_spawn: BedRuleOption,
            pub explodes: bool,
        }

        impl BedRule {
            #[inline]
            #[must_use]
            pub const fn can_sleep(&self, is_dark_outside: bool) -> bool {
                match self.can_sleep {
                    BedRuleOption::Always => true,
                    BedRuleOption::WhenDark => is_dark_outside,
                    BedRuleOption::Never => false,
                }
            }

            #[inline]
            #[must_use]
            pub const fn can_set_spawn(&self, is_dark_outside: bool) -> bool {
                match self.can_set_spawn {
                    BedRuleOption::Always => true,
                    BedRuleOption::WhenDark => is_dark_outside,
                    BedRuleOption::Never => false,
                }
            }
        }

        #[derive(Debug, Clone)]
        pub struct Dimension {
            pub id: u8,
            pub minecraft_name: &'static str,
            pub fixed_time: Option<i64>,
            pub has_fixed_time: bool,
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
            pub sky_color: Option<i32>,
            pub fog_color: Option<i32>,
            pub cloud_color: Option<i32>,
            pub ambient_light_color: Option<i32>,
            pub sky_light_color: Option<i32>,
            pub sky_light_factor: Option<f32>,
            pub fog_start_distance: Option<f32>,
            pub fog_end_distance: Option<f32>,
            pub cloud_height: Option<f32>,
            pub sky_light_level: Option<f32>,
            pub water_evaporates: bool,
            pub fast_lava: bool,
            pub respawn_anchor_works: bool,
            pub piglins_zombify: bool,
            pub snow_golem_melts: bool,
            pub can_start_raid: bool,
            pub nether_portal_spawns_piglin: bool,
            pub bed_rule: BedRule,
            pub timelines: Option<&'static str>,
        }

        impl Dimension {
            #variants

            pub fn from_name(name: &str) -> Option<&'static Self> {
                match name {
                    #name_to_type
                    _ => None
                }
            }

            #[inline]
            #[must_use]
            pub const fn effective_sky_light_level(&self) -> f32 {
                if let Some(level) = self.sky_light_level {
                    level
                } else if self.has_skylight {
                    15.0
                } else {
                    0.0
                }
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
