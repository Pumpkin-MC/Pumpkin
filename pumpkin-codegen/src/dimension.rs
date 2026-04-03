use std::{collections::BTreeMap, fs};

use heck::ToShoutySnakeCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;

/// Parses a CSS-style hex color string (e.g. `"#78a7ff"`) into a signed 32-bit integer.
///
/// # Returns
/// The color as an `i32`, or `None` if the input is not a valid hex color.
fn parse_hex_color(s: &str) -> Option<i32> {
    if let Some(stripped) = s.strip_prefix('#') {
        i32::from_str_radix(stripped, 16).ok()
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
    /// Fixed day-time value in this dimension, or `None` if time progresses normally.
    #[serde(rename = "fixed_time")]
    pub fixed_time: Option<i64>,
    /// Optional bedrock-style visual attributes map (sky color, fog color, etc.).
    #[serde(default)]
    pub attributes: Option<serde_json::Value>,
    /// Optional timeline resource key controlling day/night progression.
    #[serde(default)]
    pub timelines: Option<String>,
}

// #[derive(Clone, PartialEq, Deserialize)]
// #[serde(untagged)]
// pub enum MonsterSpawnLightLevel {
//     Int(i32),
//     Tagged(MonsterSpawnLightLevelTagged),
// }

// #[derive(Clone, PartialEq, Deserialize)]
// pub struct MonsterSpawnLightLevelTagged {
//     min_inclusive: i32,
//     max_inclusive: i32,
//     r#type: String,
// }

// impl From<i32> for MonsterSpawnLightLevel {
//     fn from(value: i32) -> Self {
//         Self::Int(value)
//     }
// }

/// Generates the `TokenStream` for the `Dimension` struct, its constants, and `from_name` lookup.
pub fn build() -> TokenStream {
    let dimensions: BTreeMap<String, Dimension> = serde_json::from_str(
        &fs::read_to_string("../assets/dimension.json").expect("Missing dimension.json"),
    )
    .expect("Failed to parse dimension.json");

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

        // convert optional hex colors from attributes into ints
        let sky_color = dim
            .attributes
            .as_ref()
            .and_then(|a| a.get("minecraft:visual/sky_color"))
            .and_then(|v| v.as_str())
            .and_then(parse_hex_color);
        let fog_color = dim
            .attributes
            .as_ref()
            .and_then(|a| a.get("minecraft:visual/fog_color"))
            .and_then(|v| v.as_str())
            .and_then(parse_hex_color);
        let cloud_color = dim
            .attributes
            .as_ref()
            .and_then(|a| a.get("minecraft:visual/cloud_color"))
            .and_then(|v| v.as_str())
            .and_then(parse_hex_color);

        let fixed_time = if let Some(t) = dim.fixed_time {
            quote! { Some(#t) }
        } else {
            quote! { None }
        };

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
                has_skylight: #has_skylight,
                has_ceiling: #has_ceiling,
                coordinate_scale: #coordinate_scale,
                min_y: #min_y,
                height: #height,
                logical_height: #logical_height,
                infiniburn: #infiniburn,
                ambient_light: #ambient_light,
                sky_color: #sky_color_literal,
                fog_color: #fog_color_literal,
                cloud_color: #cloud_color_literal,
                timelines: #timelines_literal,
            };
        });

        name_to_type.extend(quote! {
            #minecraft_name => Some(&Self::#format_name),
        });
    }

    quote!(
        #[derive(Debug, Clone, Copy)]
        pub struct Dimension {
            pub id: u8,
            pub minecraft_name: &'static str,
            pub fixed_time: Option<i64>,
            pub has_skylight: bool,
            pub has_ceiling: bool,
            pub coordinate_scale: f64,
            pub min_y: i32,
            pub height: i32,
            pub logical_height: i32,
            pub infiniburn: &'static str,
            pub ambient_light: f32,
            pub sky_color: Option<i32>,
            pub fog_color: Option<i32>,
            pub cloud_color: Option<i32>,
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
        }
        impl PartialEq for Dimension {
            fn eq(&self, other: &Self) -> bool {
                 self.id == other.id
            }
       }
        impl Eq for Dimension {}
    )
}
