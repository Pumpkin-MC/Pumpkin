use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;

pub fn build() -> TokenStream {
    let carvers: BTreeMap<String, Value> =
        serde_json::from_str(&fs::read_to_string("../assets/carver.json").unwrap())
            .expect("Failed to parse carver.json");

    let mut carver_instances = Vec::new();

    for (name, data) in carvers {
        let variant_name = format_ident!("{}", name.to_uppercase());
        let config = &data["config"];
        let carver_type = data["type"].as_str().unwrap_or("");

        let prob = config["probability"].as_f64().unwrap_or(0.0) as f32;
        let y = value_to_height_provider(&config["y"]);
        let y_scale = value_to_float_provider(&config["yScale"]);
        let lava_level = value_to_y_offset(&config["lava_level"]);

        let replaceable_str = config["replaceable"].as_str().unwrap_or("");
        let replaceable = if let Some(tag_name) = replaceable_str.strip_prefix('#') {
            let tag_name = tag_name.to_uppercase().replace([':', '.', '-'], "_");
            let tag_ident = format_ident!("{}", tag_name);
            quote! { crate::tag::Block::#tag_ident }
        } else {
            // Handle non-tag replaceables if any
            quote! { crate::tag::Block::MINECRAFT_OVERWORLD_CARVER_REPLACEABLES }
        };

        let additional = match carver_type {
            "minecraft:cave" => {
                let horizontal_radius_multiplier =
                    value_to_float_provider(&config["horizontal_radius_multiplier"]);
                let vertical_radius_multiplier =
                    value_to_float_provider(&config["vertical_radius_multiplier"]);
                let floor_level = value_to_float_provider(&config["floor_level"]);
                quote! {
                    CarverAdditionalConfig::Cave(CaveCarverConfig {
                        horizontal_radius_multiplier: #horizontal_radius_multiplier,
                        vertical_radius_multiplier: #vertical_radius_multiplier,
                        floor_level: #floor_level,
                    })
                }
            }
            "minecraft:nether_cave" => {
                let horizontal_radius_multiplier =
                    value_to_float_provider(&config["horizontal_radius_multiplier"]);
                let vertical_radius_multiplier =
                    value_to_float_provider(&config["vertical_radius_multiplier"]);
                let floor_level = value_to_float_provider(&config["floor_level"]);
                quote! {
                    CarverAdditionalConfig::NetherCave(CaveCarverConfig {
                        horizontal_radius_multiplier: #horizontal_radius_multiplier,
                        vertical_radius_multiplier: #vertical_radius_multiplier,
                        floor_level: #floor_level,
                    })
                }
            }
            "minecraft:canyon" => {
                let vertical_rotation = value_to_float_provider(&config["vertical_rotation"]);
                let shape = &config["shape"];
                let distance_factor = value_to_float_provider(&shape["distance_factor"]);
                let thickness = value_to_float_provider(&shape["thickness"]);
                let width_smoothness = shape["width_smoothness"].as_i64().unwrap_or(0) as i32;
                let horizontal_radius_factor =
                    value_to_float_provider(&shape["horizontal_radius_factor"]);
                let vertical_radius_default_factor = shape["vertical_radius_default_factor"]
                    .as_f64()
                    .unwrap_or(0.0) as f32;
                let vertical_radius_center_factor = shape["vertical_radius_center_factor"]
                    .as_f64()
                    .unwrap_or(0.0) as f32;

                quote! {
                    CarverAdditionalConfig::Canyon(CanyonCarverConfig {
                        vertical_rotation: #vertical_rotation,
                        shape: CanyonShapeConfig {
                            distance_factor: #distance_factor,
                            thickness: #thickness,
                            width_smoothness: #width_smoothness,
                            horizontal_radius_factor: #horizontal_radius_factor,
                            vertical_radius_default_factor: #vertical_radius_default_factor,
                            vertical_radius_center_factor: #vertical_radius_center_factor,
                        }
                    })
                }
            }
            _ => quote! { CarverAdditionalConfig::Cave(CaveCarverConfig::default()) },
        };

        carver_instances.push(quote! {
            pub const #variant_name: CarverConfig = CarverConfig {
                probability: #prob,
                y: #y,
                y_scale: #y_scale,
                lava_level: #lava_level,
                replaceable: #replaceable,
                additional: #additional,
            };
        });
    }

    quote! {
        use pumpkin_util::math::float_provider::{FloatProvider, NormalFloatProvider, ConstantFloatProvider, UniformFloatProvider, TrapezoidFloatProvider, ClampedNormalFloatProvider};
        use pumpkin_util::y_offset::{YOffset, Absolute, AboveBottom, BelowTop};

        pub enum HeightProvider {
            Uniform(UniformHeightProvider),
            Trapezoid(TrapezoidHeightProvider),
            VeryBiasedToBottom(VeryBiasedToBottomHeightProvider),
        }

        pub struct UniformHeightProvider {
            pub min_inclusive: YOffset,
            pub max_inclusive: YOffset,
        }

        pub struct TrapezoidHeightProvider {
            pub min_inclusive: YOffset,
            pub max_inclusive: YOffset,
            pub plateau: Option<i32>,
        }

        pub struct VeryBiasedToBottomHeightProvider {
            pub min_inclusive: YOffset,
            pub max_inclusive: YOffset,
            pub inner: Option<std::num::NonZeroU32>,
        }

        pub struct CaveCarverConfig {
            pub horizontal_radius_multiplier: FloatProvider,
            pub vertical_radius_multiplier: FloatProvider,
            pub floor_level: FloatProvider,
        }

        impl CaveCarverConfig {
            pub const fn default() -> Self {
                Self {
                    horizontal_radius_multiplier: FloatProvider::Constant(1.0),
                    vertical_radius_multiplier: FloatProvider::Constant(1.0),
                    floor_level: FloatProvider::Constant(-0.7),
                }
            }
        }

        pub struct CanyonShapeConfig {
            pub distance_factor: FloatProvider,
            pub thickness: FloatProvider,
            pub width_smoothness: i32,
            pub horizontal_radius_factor: FloatProvider,
            pub vertical_radius_default_factor: f32,
            pub vertical_radius_center_factor: f32,
        }

        pub struct CanyonCarverConfig {
            pub vertical_rotation: FloatProvider,
            pub shape: CanyonShapeConfig,
        }

        pub enum CarverAdditionalConfig {
            Cave(CaveCarverConfig),
            NetherCave(CaveCarverConfig),
            Canyon(CanyonCarverConfig),
        }

        pub struct CarverConfig {
            pub probability: f32,
            pub y: HeightProvider,
            pub y_scale: FloatProvider,
            pub lava_level: YOffset,
            pub replaceable: crate::tag::Tag,
            pub additional: CarverAdditionalConfig,
        }

        use super::*;
        #(#carver_instances)*
    }
}

fn value_to_float_provider(v: &Value) -> TokenStream {
    if v.is_number() {
        let f = v.as_f64().unwrap_or(0.0) as f32;
        quote! { FloatProvider::Constant(#f) }
    } else {
        // This is complex because we need to match the structure of FloatProvider in pumpkin-util
        // For now, let's handle the common case of uniform
        let type_str = v["type"].as_str().unwrap_or("");
        match type_str {
            "minecraft:uniform" => {
                let min = v["min_inclusive"].as_f64().unwrap_or(0.0) as f32;
                let max = v["max_exclusive"].as_f64().unwrap_or(0.0) as f32;
                quote! { FloatProvider::Object(NormalFloatProvider::Uniform(UniformFloatProvider::new(#min, #max))) }
            }
            "minecraft:constant" => {
                let val = v["value"].as_f64().unwrap_or(0.0) as f32;
                quote! { FloatProvider::Constant(#val) }
            }
            "minecraft:trapezoid" => {
                let min = v["min"].as_f64().unwrap_or(0.0) as f32;
                let max = v["max"].as_f64().unwrap_or(0.0) as f32;
                let plateau = v["plateau"].as_f64().unwrap_or(0.0) as f32;
                quote! { FloatProvider::Object(NormalFloatProvider::Trapezoid(TrapezoidFloatProvider::new(#min, #max, #plateau))) }
            }
            _ => {
                // Fallback to constant 0 if unknown
                quote! { FloatProvider::Constant(0.0) }
            }
        }
    }
}

fn value_to_height_provider(v: &Value) -> TokenStream {
    let type_str = v["type"].as_str().unwrap_or("");
    match type_str {
        "minecraft:uniform" => {
            let min = value_to_y_offset(&v["min_inclusive"]);
            let max = value_to_y_offset(&v["max_inclusive"]);
            quote! {
                HeightProvider::Uniform(UniformHeightProvider {
                    min_inclusive: #min,
                    max_inclusive: #max,
                })
            }
        }
        "minecraft:trapezoid" => {
            let min = value_to_y_offset(&v["min_inclusive"]);
            let max = value_to_y_offset(&v["max_inclusive"]);
            let plateau = if v["plateau"].is_number() {
                let p = v["plateau"].as_i64().unwrap_or(0) as i32;
                quote! { Some(#p) }
            } else {
                quote! { None }
            };
            quote! {
                HeightProvider::Trapezoid(TrapezoidHeightProvider {
                    min_inclusive: #min,
                    max_inclusive: #max,
                    plateau: #plateau,
                })
            }
        }
        _ => {
            let zero = value_to_y_offset(&Value::Null);
            quote! {
                HeightProvider::Uniform(UniformHeightProvider {
                    min_inclusive: #zero,
                    max_inclusive: #zero,
                })
            }
        }
    }
}

fn value_to_y_offset(v: &Value) -> TokenStream {
    if v.is_object() {
        if let Some(abs) = v.get("absolute") {
            let val = abs.as_i64().unwrap_or(0) as i16;
            quote! { YOffset::Absolute(Absolute { absolute: #val }) }
        } else if let Some(ab) = v.get("above_bottom") {
            let val = ab.as_i64().unwrap_or(0) as i8;
            quote! { YOffset::AboveBottom(AboveBottom { above_bottom: #val }) }
        } else if let Some(bt) = v.get("below_top") {
            let val = bt.as_i64().unwrap_or(0) as i8;
            quote! { YOffset::BelowTop(BelowTop { below_top: #val }) }
        } else {
            quote! { YOffset::Absolute(Absolute { absolute: 0 }) }
        }
    } else {
        quote! { YOffset::Absolute(Absolute { absolute: 0 }) }
    }
}
