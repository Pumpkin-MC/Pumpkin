use proc_macro2::TokenStream;
use quote::quote;
use serde_json::Value;
use std::fs;

pub fn build() -> TokenStream {
    let json_content =
        fs::read_to_string("../assets/placed_feature.json").expect("Failed to read placed_feature.json");
    let json: Value = serde_json::from_str(&json_content).expect("Failed to parse placed_feature.json");

    let entries: Vec<TokenStream> = json
        .as_object()
        .unwrap()
        .iter()
        .map(|(name, value)| {
            let pf = value_to_placed_feature(value);
            quote! {
                map.insert(#name.to_string(), #pf);
            }
        })
        .collect();

    quote! {
        #[allow(clippy::all, unused_imports, dead_code)]
        fn build_placed_features() -> std::collections::HashMap<String, PlacedFeature> {
            use crate::generation::block_predicate::{
                AllOfBlockPredicate, AnyOfBlockPredicate, BlockPredicate,
                HasSturdyFacePredicate, InsideWorldBoundsBlockPredicate,
                MatchingBlockTagPredicate, MatchingBlocksBlockPredicate, MatchingBlocksWrapper,
                MatchingFluidsBlockPredicate, NotBlockPredicate, OffsetBlocksBlockPredicate,
                ReplaceableBlockPredicate, SolidBlockPredicate, WouldSurviveBlockPredicate,
            };
            use crate::generation::height_provider::{
                HeightProvider, TrapezoidHeightProvider, UniformHeightProvider,
                VeryBiasedToBottomHeightProvider,
            };
            use pumpkin_util::y_offset::{AboveBottom, Absolute, BelowTop, YOffset};
            use pumpkin_util::math::int_provider::{
                BiasedToBottomIntProvider, ClampedIntProvider, ClampedNormalIntProvider,
                ConstantIntProvider, IntProvider, NormalIntProvider, UniformIntProvider,
                WeightedEntry, WeightedListIntProvider,
            };
            use crate::block::BlockStateCodec;
            use pumpkin_data::{Block, BlockDirection};
            use pumpkin_util::math::vector3::Vector3;
            use pumpkin_util::HeightMap;
            let mut map = std::collections::HashMap::new();
            #(#entries)*
            map
        }
    }
}

fn value_to_placed_feature(v: &Value) -> TokenStream {
    let feature = value_to_feature(&v["feature"]);
    let placement_arr = v["placement"].as_array().map(|a| a.as_slice()).unwrap_or(&[]);
    let placement: Vec<TokenStream> = placement_arr.iter().map(value_to_placement_modifier).collect();
    quote! {
        PlacedFeature {
            feature: #feature,
            placement: vec![#(#placement),*],
        }
    }
}

fn value_to_feature(v: &Value) -> TokenStream {
    match v {
        Value::String(s) => {
            let name = s.strip_prefix("minecraft:").unwrap_or(s);
            quote! { Feature::Named(#name.to_string()) }
        }
        Value::Object(_) => {
            let cf = value_to_inline_configured_feature(v);
            quote! { Feature::Inlined(Box::new(#cf)) }
        }
        _ => quote! { Feature::Named(String::new()) },
    }
}

fn value_to_placement_modifier(v: &Value) -> TokenStream {
    let type_str = v["type"].as_str().unwrap_or("");
    match type_str {
        "minecraft:biome" => quote! { PlacementModifier::Biome(BiomePlacementModifier) },
        "minecraft:in_square" => quote! { PlacementModifier::InSquare(SquarePlacementModifier) },
        "minecraft:fixed_placement" => quote! { PlacementModifier::FixedPlacement },
        "minecraft:heightmap" => {
            let heightmap = value_to_height_map(v["heightmap"].as_str().unwrap_or("MOTION_BLOCKING"));
            quote! {
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: #heightmap,
                })
            }
        }
        "minecraft:height_range" => {
            let height = value_to_height_provider(&v["height"]);
            quote! {
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: #height,
                })
            }
        }
        "minecraft:count" => {
            let count = value_to_int_provider(&v["count"]);
            quote! {
                PlacementModifier::Count(CountPlacementModifier {
                    count: #count,
                })
            }
        }
        "minecraft:count_on_every_layer" => {
            let count = value_to_int_provider(&v["count"]);
            quote! {
                PlacementModifier::CountOnEveryLayer(CountOnEveryLayerPlacementModifier {
                    count: #count,
                })
            }
        }
        "minecraft:rarity_filter" => {
            let chance = v["chance"].as_u64().unwrap_or(1) as u32;
            quote! {
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier {
                    chance: #chance,
                })
            }
        }
        "minecraft:block_predicate_filter" => {
            let predicate = value_to_block_predicate(&v["predicate"]);
            quote! {
                PlacementModifier::BlockPredicateFilter(BlockFilterPlacementModifier {
                    predicate: #predicate,
                })
            }
        }
        "minecraft:surface_relative_threshold_filter" => {
            let heightmap =
                value_to_height_map(v["heightmap"].as_str().unwrap_or("MOTION_BLOCKING"));
            let min_inc = value_to_option_i32(&v["min_inclusive"]);
            let max_inc = value_to_option_i32(&v["max_inclusive"]);
            quote! {
                PlacementModifier::SurfaceRelativeThresholdFilter(SurfaceThresholdFilterPlacementModifier {
                    heightmap: #heightmap,
                    min_inclusive: #min_inc,
                    max_inclusive: #max_inc,
                })
            }
        }
        "minecraft:surface_water_depth_filter" => {
            let depth = v["max_water_depth"].as_i64().unwrap_or(0) as i32;
            quote! {
                PlacementModifier::SurfaceWaterDepthFilter(SurfaceWaterDepthFilterPlacementModifier {
                    max_water_depth: #depth,
                })
            }
        }
        "minecraft:noise_based_count" => {
            let ratio = v["noise_to_count_ratio"].as_i64().unwrap_or(0) as i32;
            let factor = v["noise_factor"].as_f64().unwrap_or(1.0);
            let offset = v["noise_offset"].as_f64().unwrap_or(0.0);
            quote! {
                PlacementModifier::NoiseBasedCount(NoiseBasedCountPlacementModifier {
                    noise_to_count_ratio: #ratio,
                    noise_factor: #factor,
                    noise_offset: #offset,
                })
            }
        }
        "minecraft:noise_threshold_count" => {
            let level = v["noise_level"].as_f64().unwrap_or(0.0);
            let below = v["below_noise"].as_i64().unwrap_or(0) as i32;
            let above = v["above_noise"].as_i64().unwrap_or(0) as i32;
            quote! {
                PlacementModifier::NoiseThresholdCount(NoiseThresholdCountPlacementModifier {
                    noise_level: #level,
                    below_noise: #below,
                    above_noise: #above,
                })
            }
        }
        "minecraft:environment_scan" => {
            let dir = value_to_block_direction(v["direction_of_search"].as_str().unwrap_or("down"));
            let target = value_to_block_predicate(&v["target_condition"]);
            let allowed = if v["allowed_search_condition"].is_null() || v["allowed_search_condition"].is_object() {
                if v["allowed_search_condition"].is_object() {
                    let p = value_to_block_predicate(&v["allowed_search_condition"]);
                    quote! { Some(#p) }
                } else {
                    quote! { None }
                }
            } else {
                quote! { None }
            };
            let steps = v["max_steps"].as_i64().unwrap_or(1) as i32;
            quote! {
                PlacementModifier::EnvironmentScan(EnvironmentScanPlacementModifier {
                    direction_of_search: #dir,
                    target_condition: #target,
                    allowed_search_condition: #allowed,
                    max_steps: #steps,
                })
            }
        }
        "minecraft:random_offset" => {
            let xz = value_to_int_provider(&v["xz_spread"]);
            let y = value_to_int_provider(&v["y_spread"]);
            quote! {
                PlacementModifier::RandomOffset(RandomOffsetPlacementModifier {
                    xz_spread: #xz,
                    y_spread: #y,
                })
            }
        }
        other => {
            let msg = format!("unknown placement modifier: {other}");
            quote! { compile_error!(#msg) }
        }
    }
}

pub fn value_to_block_predicate(v: &Value) -> TokenStream {
    // Handle bare string values: "#minecraft:some_tag" is a block-tag predicate,
    // "true" (or any non-# string) is AlwaysTrue.
    if let Some(s) = v.as_str() {
        if let Some(tag) = s.strip_prefix('#') {
            return quote! {
                BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                    offset: OffsetBlocksBlockPredicate { offset: None },
                    tag: #tag.to_string(),
                })
            };
        }
        return quote! { BlockPredicate::AlwaysTrue };
    }

    let type_str = v["type"].as_str().unwrap_or("");
    match type_str {
        "minecraft:true" | "" => quote! { BlockPredicate::AlwaysTrue },
        "minecraft:matching_blocks" => {
            let offset = value_to_offset_predicate(&v["offset"]);
            let blocks = value_to_matching_blocks_wrapper(&v["blocks"]);
            quote! {
                BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                    offset: #offset,
                    blocks: #blocks,
                })
            }
        }
        "minecraft:matching_block_tag" => {
            let offset = value_to_offset_predicate(&v["offset"]);
            let tag = v["tag"].as_str().unwrap_or("");
            quote! {
                BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                    offset: #offset,
                    tag: #tag.to_string(),
                })
            }
        }
        "minecraft:matching_fluids" => {
            let offset = value_to_offset_predicate(&v["offset"]);
            let fluids = value_to_matching_blocks_wrapper(&v["fluids"]);
            quote! {
                BlockPredicate::MatchingFluids(MatchingFluidsBlockPredicate {
                    offset: #offset,
                    fluids: #fluids,
                })
            }
        }
        "minecraft:has_sturdy_face" => {
            let offset = value_to_offset_predicate(&v["offset"]);
            let dir = value_to_block_direction(v["direction"].as_str().unwrap_or("down"));
            quote! {
                BlockPredicate::HasSturdyFace(HasSturdyFacePredicate {
                    offset: #offset,
                    direction: #dir,
                })
            }
        }
        "minecraft:solid" => {
            let offset = value_to_offset_predicate(&v["offset"]);
            quote! {
                BlockPredicate::Solid(SolidBlockPredicate { offset: #offset })
            }
        }
        "minecraft:replaceable" => {
            let offset = value_to_offset_predicate(&v["offset"]);
            quote! {
                BlockPredicate::Replaceable(ReplaceableBlockPredicate { offset: #offset })
            }
        }
        "minecraft:would_survive" => {
            let offset = value_to_offset_predicate(&v["offset"]);
            let state = value_to_block_state_codec(&v["state"]);
            quote! {
                BlockPredicate::WouldSurvive(WouldSurviveBlockPredicate {
                    offset: #offset,
                    state: #state,
                })
            }
        }
        "minecraft:inside_world_bounds" => {
            let offset_v = &v["offset"];
            let offset = if offset_v.is_array() {
                let x = offset_v[0].as_i64().unwrap_or(0) as i32;
                let y = offset_v[1].as_i64().unwrap_or(0) as i32;
                let z = offset_v[2].as_i64().unwrap_or(0) as i32;
                quote! { Vector3::new(#x, #y, #z) }
            } else {
                quote! { Vector3::new(0, 0, 0) }
            };
            quote! {
                BlockPredicate::InsideWorldBounds(InsideWorldBoundsBlockPredicate {
                    offset: #offset,
                })
            }
        }
        "minecraft:any_of" => {
            let predicates: Vec<TokenStream> = v["predicates"]
                .as_array()
                .map(|a| a.iter().map(value_to_block_predicate).collect())
                .unwrap_or_default();
            quote! {
                BlockPredicate::AnyOf(AnyOfBlockPredicate {
                    predicates: vec![#(#predicates),*],
                })
            }
        }
        "minecraft:all_of" => {
            let predicates: Vec<TokenStream> = v["predicates"]
                .as_array()
                .map(|a| a.iter().map(value_to_block_predicate).collect())
                .unwrap_or_default();
            quote! {
                BlockPredicate::AllOf(AllOfBlockPredicate {
                    predicates: vec![#(#predicates),*],
                })
            }
        }
        "minecraft:not" => {
            let inner = value_to_block_predicate(&v["predicate"]);
            quote! {
                BlockPredicate::Not(NotBlockPredicate {
                    predicate: Box::new(#inner),
                })
            }
        }
        other => {
            let msg = format!("unknown block predicate: {other}");
            quote! { compile_error!(#msg) }
        }
    }
}

pub fn value_to_height_provider(v: &Value) -> TokenStream {
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
            let plateau = value_to_option_i32(&v["plateau"]);
            quote! {
                HeightProvider::Trapezoid(TrapezoidHeightProvider {
                    min_inclusive: #min,
                    max_inclusive: #max,
                    plateau: #plateau,
                })
            }
        }
        "minecraft:very_biased_to_bottom" => {
            let min = value_to_y_offset(&v["min_inclusive"]);
            let max = value_to_y_offset(&v["max_inclusive"]);
            let inner = if v["inner"].is_null() || !v["inner"].is_number() {
                quote! { None }
            } else {
                let n = v["inner"].as_u64().unwrap_or(1) as u32;
                quote! { std::num::NonZeroU32::new(#n) }
            };
            quote! {
                HeightProvider::VeryBiasedToBottom(VeryBiasedToBottomHeightProvider {
                    min_inclusive: #min,
                    max_inclusive: #max,
                    inner: #inner,
                })
            }
        }
        "minecraft:constant" => {
            // constant height provider wraps a y_offset
            let y = value_to_y_offset(&v["value"]);
            // Map to uniform with same min/max
            quote! {
                HeightProvider::Uniform(UniformHeightProvider {
                    min_inclusive: #y,
                    max_inclusive: #y,
                })
            }
        }
        other => {
            let msg = format!("unknown height provider: {other}");
            quote! { compile_error!(#msg) }
        }
    }
}

pub fn value_to_y_offset(v: &Value) -> TokenStream {
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
}

pub fn value_to_int_provider(v: &Value) -> TokenStream {
    match v {
        Value::Number(n) => {
            let val = n.as_i64().unwrap_or(0) as i32;
            quote! { IntProvider::Constant(#val) }
        }
        Value::Object(_) => {
            let type_str = v["type"].as_str().unwrap_or("");
            match type_str {
                "minecraft:constant" => {
                    let val = v["value"].as_i64().unwrap_or(0) as i32;
                    quote! { IntProvider::Object(NormalIntProvider::Constant(ConstantIntProvider { value: #val })) }
                }
                "minecraft:uniform" => {
                    let min = v["min_inclusive"].as_i64().unwrap_or(0) as i32;
                    let max = v["max_inclusive"].as_i64().unwrap_or(0) as i32;
                    quote! { IntProvider::Object(NormalIntProvider::Uniform(UniformIntProvider { min_inclusive: #min, max_inclusive: #max })) }
                }
                "minecraft:biased_to_bottom" => {
                    let min = v["min_inclusive"].as_i64().unwrap_or(0) as i32;
                    let max = v["max_inclusive"].as_i64().unwrap_or(0) as i32;
                    quote! { IntProvider::Object(NormalIntProvider::BiasedToBottom(BiasedToBottomIntProvider { min_inclusive: #min, max_inclusive: #max })) }
                }
                "minecraft:clamped" => {
                    let min = v["min_inclusive"].as_i64().unwrap_or(0) as i32;
                    let max = v["max_inclusive"].as_i64().unwrap_or(0) as i32;
                    let src = value_to_int_provider(&v["source"]);
                    quote! { IntProvider::Object(NormalIntProvider::Clamped(ClampedIntProvider { source: Box::new(#src), min_inclusive: #min, max_inclusive: #max })) }
                }
                "minecraft:clamped_normal" => {
                    let mean = v["mean"].as_f64().unwrap_or(0.0) as f32;
                    let dev = v["deviation"].as_f64().unwrap_or(1.0) as f32;
                    let min = v["min_inclusive"].as_i64().unwrap_or(0) as i32;
                    let max = v["max_inclusive"].as_i64().unwrap_or(0) as i32;
                    quote! { IntProvider::Object(NormalIntProvider::ClampedNormal(ClampedNormalIntProvider { mean: #mean, deviation: #dev, min_inclusive: #min, max_inclusive: #max })) }
                }
                "minecraft:weighted_list" => {
                    let entries: Vec<TokenStream> = v["distribution"]
                        .as_array()
                        .map(|arr| {
                            arr.iter().map(|e| {
                                let data = value_to_int_provider(&e["data"]);
                                let weight = e["weight"].as_i64().unwrap_or(1) as i32;
                                quote! { WeightedEntry { data: #data, weight: #weight } }
                            }).collect()
                        })
                        .unwrap_or_default();
                    quote! { IntProvider::Object(NormalIntProvider::WeightedList(WeightedListIntProvider { distribution: vec![#(#entries),*] })) }
                }
                _ => {
                    let val = v["value"].as_i64().unwrap_or(0) as i32;
                    quote! { IntProvider::Constant(#val) }
                }
            }
        }
        _ => quote! { IntProvider::Constant(0) },
    }
}

pub fn value_to_height_map(s: &str) -> TokenStream {
    match s {
        "WORLD_SURFACE_WG" => quote! { HeightMap::WorldSurfaceWg },
        "WORLD_SURFACE" => quote! { HeightMap::WorldSurface },
        "OCEAN_FLOOR_WG" => quote! { HeightMap::OceanFloorWg },
        "OCEAN_FLOOR" => quote! { HeightMap::OceanFloor },
        "MOTION_BLOCKING" => quote! { HeightMap::MotionBlocking },
        "MOTION_BLOCKING_NO_LEAVES" => quote! { HeightMap::MotionBlockingNoLeaves },
        _ => quote! { HeightMap::MotionBlocking },
    }
}

pub fn value_to_block_direction(s: &str) -> TokenStream {
    match s.to_lowercase().as_str() {
        "down" => quote! { BlockDirection::Down },
        "up" => quote! { BlockDirection::Up },
        "north" => quote! { BlockDirection::North },
        "south" => quote! { BlockDirection::South },
        "west" => quote! { BlockDirection::West },
        "east" => quote! { BlockDirection::East },
        _ => quote! { BlockDirection::Down },
    }
}

fn value_to_offset_predicate(v: &Value) -> TokenStream {
    if v.is_null() || v.is_object() && v.as_object().map_or(true, |o| o.is_empty()) {
        quote! { OffsetBlocksBlockPredicate { offset: None } }
    } else if v.is_array() {
        let x = v[0].as_i64().unwrap_or(0) as i32;
        let y = v[1].as_i64().unwrap_or(0) as i32;
        let z = v[2].as_i64().unwrap_or(0) as i32;
        quote! { OffsetBlocksBlockPredicate { offset: Some(Vector3::new(#x, #y, #z)) } }
    } else {
        quote! { OffsetBlocksBlockPredicate { offset: None } }
    }
}

fn value_to_matching_blocks_wrapper(v: &Value) -> TokenStream {
    match v {
        Value::String(s) => {
            quote! { MatchingBlocksWrapper::Single(#s.to_string()) }
        }
        Value::Array(arr) => {
            let items: Vec<TokenStream> = arr
                .iter()
                .filter_map(|s| s.as_str().map(|s| quote! { #s.to_string() }))
                .collect();
            quote! { MatchingBlocksWrapper::Multiple(vec![#(#items),*]) }
        }
        _ => quote! { MatchingBlocksWrapper::Single(String::new()) },
    }
}

pub fn value_to_block_state_codec(v: &Value) -> TokenStream {
    let name = v["Name"].as_str().unwrap_or("minecraft:air");
    let name_stripped = name.strip_prefix("minecraft:").unwrap_or(name);
    let block_ident = quote::format_ident!("{}", name_stripped.to_uppercase().replace(':', "_").replace('-', "_"));
    if let Some(props) = v["Properties"].as_object() {
        let keys: Vec<&str> = props.keys().map(|k| k.as_str()).collect();
        let vals: Vec<&str> = props.values().filter_map(|v| v.as_str()).collect();
        quote! {
            {
                let mut props = std::collections::HashMap::new();
                #(props.insert(#keys.to_string(), #vals.to_string());)*
                BlockStateCodec {
                    name: &pumpkin_data::Block::#block_ident,
                    properties: Some(props),
                }
            }
        }
    } else {
        quote! {
            BlockStateCodec {
                name: &pumpkin_data::Block::#block_ident,
                properties: None,
            }
        }
    }
}

fn value_to_option_i32(v: &Value) -> TokenStream {
    if v.is_null() {
        quote! { None }
    } else if let Some(n) = v.as_i64() {
        let val = n as i32;
        quote! { Some(#val) }
    } else {
        quote! { None }
    }
}

/// Stub for configured feature inline in placed feature
fn value_to_inline_configured_feature(_v: &Value) -> TokenStream {
    quote! { crate::generation::feature::configured_features::ConfiguredFeature::NoOp }
}
