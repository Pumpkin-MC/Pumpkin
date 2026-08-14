/* This file is generated. Do not edit manually. */
use pumpkin_registry::{Registry, RegistryBuilder, bootstrap::RegistryEntry, bootstrap_provider};
use pumpkin_util::{
    identifier::Identifier,
    math::int_provider::{
        BiasedToBottomIntProvider, ClampedIntProvider, ClampedNormalIntProvider,
        ConstantIntProvider, IntProvider, NormalIntProvider, TrapezoidIntProvider,
        UniformIntProvider, WeightedEntry, WeightedListIntProvider,
    },
};
use std::sync::Arc;
#[derive(Debug, Clone)]
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
    pub monster_spawn_light_level: IntProvider,
    pub monster_spawn_block_light_limit: u8,
    pub sky_color: Option<i32>,
    pub fog_color: Option<i32>,
    pub cloud_color: Option<i32>,
    pub timelines: Option<&'static str>,
}
impl Dimension {
    pub const OVERWORLD: Self = Self {
        id: 0u8,
        minecraft_name: "minecraft:overworld",
        fixed_time: None,
        has_skylight: true,
        has_ceiling: false,
        coordinate_scale: 1f64,
        min_y: -64i32,
        height: 384i32,
        logical_height: 384i32,
        infiniburn: "#minecraft:infiniburn_overworld",
        ambient_light: 0f32,
        monster_spawn_light_level: IntProvider::Object(NormalIntProvider::Uniform(
            UniformIntProvider {
                min_inclusive: 0i32,
                max_inclusive: 7i32,
            },
        )),
        monster_spawn_block_light_limit: 0u8,
        sky_color: Some(7907327i32),
        fog_color: Some(12638463i32),
        cloud_color: None,
        timelines: Some("#minecraft:in_overworld"),
    };
    pub const OVERWORLD_CAVES: Self = Self {
        id: 1u8,
        minecraft_name: "minecraft:overworld_caves",
        fixed_time: None,
        has_skylight: true,
        has_ceiling: true,
        coordinate_scale: 1f64,
        min_y: -64i32,
        height: 384i32,
        logical_height: 384i32,
        infiniburn: "#minecraft:infiniburn_overworld",
        ambient_light: 0f32,
        monster_spawn_light_level: IntProvider::Object(NormalIntProvider::Uniform(
            UniformIntProvider {
                min_inclusive: 0i32,
                max_inclusive: 7i32,
            },
        )),
        monster_spawn_block_light_limit: 0u8,
        sky_color: Some(7907327i32),
        fog_color: Some(12638463i32),
        cloud_color: None,
        timelines: Some("#minecraft:in_overworld"),
    };
    pub const THE_END: Self = Self {
        id: 2u8,
        minecraft_name: "minecraft:the_end",
        fixed_time: None,
        has_skylight: true,
        has_ceiling: false,
        coordinate_scale: 1f64,
        min_y: 0i32,
        height: 256i32,
        logical_height: 256i32,
        infiniburn: "#minecraft:infiniburn_end",
        ambient_light: 0.25f32,
        monster_spawn_light_level: IntProvider::Constant(15i32),
        monster_spawn_block_light_limit: 0u8,
        sky_color: Some(0i32),
        fog_color: Some(1577752i32),
        cloud_color: None,
        timelines: Some("#minecraft:in_end"),
    };
    pub const THE_NETHER: Self = Self {
        id: 3u8,
        minecraft_name: "minecraft:the_nether",
        fixed_time: None,
        has_skylight: false,
        has_ceiling: true,
        coordinate_scale: 8f64,
        min_y: 0i32,
        height: 256i32,
        logical_height: 128i32,
        infiniburn: "#minecraft:infiniburn_nether",
        ambient_light: 0.1f32,
        monster_spawn_light_level: IntProvider::Constant(7i32),
        monster_spawn_block_light_limit: 15u8,
        sky_color: None,
        fog_color: None,
        cloud_color: None,
        timelines: Some("#minecraft:in_nether"),
    };
}
const STATIC_ENTRIES: [Dimension; 4usize] = [
    Dimension::OVERWORLD,
    Dimension::OVERWORLD_CAVES,
    Dimension::THE_END,
    Dimension::THE_NETHER,
];
const STATIC_IDENTIFIERS: [Identifier; 4usize] = [
    Identifier::parse_static("minecraft:overworld"),
    Identifier::parse_static("minecraft:overworld_caves"),
    Identifier::parse_static("minecraft:the_end"),
    Identifier::parse_static("minecraft:the_nether"),
];
bootstrap_provider! { DIMENSION_TYPE_REGISTRY : Arc < dyn Registry > => "minecraft:root" , || { vec ! [RegistryEntry :: new (Identifier :: vanilla_static ("dimension_type") , RegistryBuilder :: < Dimension > :: new_static (& Identifier :: vanilla_static ("dimension_type") , & STATIC_ENTRIES , & STATIC_IDENTIFIERS ,) . unwrap () . arc_dyn () ,)] } }
impl PartialEq for Dimension {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for Dimension {}
