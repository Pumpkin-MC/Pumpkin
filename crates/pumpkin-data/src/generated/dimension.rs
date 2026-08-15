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
#[derive(Debug, Clone, Copy)]
pub struct EnvironmentAttribute {
    pub identifier: &'static str,
    #[doc = r" JSON representation of the Environment Attribute value or modifier."]
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
    pub const OVERWORLD: Self = Self {
        id: 0u8,
        minecraft_name: "minecraft:overworld",
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
        attributes: &[
            EnvironmentAttribute {
                identifier: "minecraft:audio/ambient_sounds",
                value: "{\"mood\":{\"sound\":\"minecraft:ambient.cave\",\"tick_delay\":6000,\"block_search_extent\":8,\"offset\":2.0}}",
            },
            EnvironmentAttribute {
                identifier: "minecraft:audio/background_music",
                value: "{\"default\":{\"sound\":\"minecraft:music.game\",\"min_delay\":12000,\"max_delay\":24000},\"creative\":{\"sound\":\"minecraft:music.creative\",\"min_delay\":12000,\"max_delay\":24000}}",
            },
            EnvironmentAttribute {
                identifier: "minecraft:gameplay/bed_rule",
                value: "{\"can_sleep\":\"when_dark\",\"can_set_spawn\":\"always\",\"error_message\":{\"translate\":\"block.minecraft.bed.no_sleep\"}}",
            },
            EnvironmentAttribute {
                identifier: "minecraft:gameplay/nether_portal_spawns_piglin",
                value: "true",
            },
            EnvironmentAttribute {
                identifier: "minecraft:gameplay/respawn_anchor_works",
                value: "false",
            },
            EnvironmentAttribute {
                identifier: "minecraft:visual/ambient_light_color",
                value: "\"#0a0a0a\"",
            },
            EnvironmentAttribute {
                identifier: "minecraft:visual/cloud_color",
                value: "\"#ccffffff\"",
            },
            EnvironmentAttribute {
                identifier: "minecraft:visual/cloud_height",
                value: "192.33",
            },
            EnvironmentAttribute {
                identifier: "minecraft:visual/fog_color",
                value: "\"#c0d8ff\"",
            },
            EnvironmentAttribute {
                identifier: "minecraft:visual/sky_color",
                value: "\"#78a7ff\"",
            },
        ],
        has_fixed_time: false,
        has_ender_dragon_fight: false,
        skybox: None,
        cardinal_light: None,
        timelines: &["#minecraft:in_overworld"],
        default_clock: Some("minecraft:overworld"),
    };
    pub const OVERWORLD_CAVES: Self = Self {
        id: 1u8,
        minecraft_name: "minecraft:overworld_caves",
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
        attributes: &[
            EnvironmentAttribute {
                identifier: "minecraft:audio/ambient_sounds",
                value: "{\"mood\":{\"sound\":\"minecraft:ambient.cave\",\"tick_delay\":6000,\"block_search_extent\":8,\"offset\":2.0}}",
            },
            EnvironmentAttribute {
                identifier: "minecraft:audio/background_music",
                value: "{\"default\":{\"sound\":\"minecraft:music.game\",\"min_delay\":12000,\"max_delay\":24000},\"creative\":{\"sound\":\"minecraft:music.creative\",\"min_delay\":12000,\"max_delay\":24000}}",
            },
            EnvironmentAttribute {
                identifier: "minecraft:gameplay/bed_rule",
                value: "{\"can_sleep\":\"when_dark\",\"can_set_spawn\":\"always\",\"error_message\":{\"translate\":\"block.minecraft.bed.no_sleep\"}}",
            },
            EnvironmentAttribute {
                identifier: "minecraft:gameplay/nether_portal_spawns_piglin",
                value: "true",
            },
            EnvironmentAttribute {
                identifier: "minecraft:gameplay/respawn_anchor_works",
                value: "false",
            },
            EnvironmentAttribute {
                identifier: "minecraft:visual/ambient_light_color",
                value: "\"#0a0a0a\"",
            },
            EnvironmentAttribute {
                identifier: "minecraft:visual/cloud_color",
                value: "\"#ccffffff\"",
            },
            EnvironmentAttribute {
                identifier: "minecraft:visual/cloud_height",
                value: "192.33",
            },
            EnvironmentAttribute {
                identifier: "minecraft:visual/fog_color",
                value: "\"#c0d8ff\"",
            },
            EnvironmentAttribute {
                identifier: "minecraft:visual/sky_color",
                value: "\"#78a7ff\"",
            },
        ],
        has_fixed_time: false,
        has_ender_dragon_fight: false,
        skybox: None,
        cardinal_light: None,
        timelines: &["#minecraft:in_overworld"],
        default_clock: Some("minecraft:overworld"),
    };
    pub const THE_END: Self = Self {
        id: 2u8,
        minecraft_name: "minecraft:the_end",
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
        attributes: &[
            EnvironmentAttribute {
                identifier: "minecraft:audio/ambient_sounds",
                value: "{\"mood\":{\"sound\":\"minecraft:ambient.cave\",\"tick_delay\":6000,\"block_search_extent\":8,\"offset\":2.0}}",
            },
            EnvironmentAttribute {
                identifier: "minecraft:audio/background_music",
                value: "{\"default\":{\"sound\":\"minecraft:music.end\",\"min_delay\":6000,\"max_delay\":24000,\"replace_current_music\":true}}",
            },
            EnvironmentAttribute {
                identifier: "minecraft:gameplay/bed_rule",
                value: "{\"can_sleep\":\"never\",\"can_set_spawn\":\"never\",\"explodes\":true}",
            },
            EnvironmentAttribute {
                identifier: "minecraft:gameplay/respawn_anchor_works",
                value: "false",
            },
            EnvironmentAttribute {
                identifier: "minecraft:visual/ambient_light_color",
                value: "\"#3f473f\"",
            },
            EnvironmentAttribute {
                identifier: "minecraft:visual/fog_color",
                value: "\"#181318\"",
            },
            EnvironmentAttribute {
                identifier: "minecraft:visual/sky_color",
                value: "\"#000000\"",
            },
            EnvironmentAttribute {
                identifier: "minecraft:visual/sky_light_color",
                value: "\"#ac60cd\"",
            },
            EnvironmentAttribute {
                identifier: "minecraft:visual/sky_light_factor",
                value: "0.0",
            },
        ],
        has_fixed_time: true,
        has_ender_dragon_fight: true,
        skybox: Some("end"),
        cardinal_light: None,
        timelines: &["#minecraft:in_end"],
        default_clock: Some("minecraft:the_end"),
    };
    pub const THE_NETHER: Self = Self {
        id: 3u8,
        minecraft_name: "minecraft:the_nether",
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
        attributes: &[
            EnvironmentAttribute {
                identifier: "minecraft:gameplay/bed_rule",
                value: "{\"can_sleep\":\"never\",\"can_set_spawn\":\"never\",\"explodes\":true}",
            },
            EnvironmentAttribute {
                identifier: "minecraft:gameplay/can_start_raid",
                value: "false",
            },
            EnvironmentAttribute {
                identifier: "minecraft:gameplay/fast_lava",
                value: "true",
            },
            EnvironmentAttribute {
                identifier: "minecraft:gameplay/piglins_zombify",
                value: "false",
            },
            EnvironmentAttribute {
                identifier: "minecraft:gameplay/respawn_anchor_works",
                value: "true",
            },
            EnvironmentAttribute {
                identifier: "minecraft:gameplay/sky_light_level",
                value: "4.0",
            },
            EnvironmentAttribute {
                identifier: "minecraft:gameplay/snow_golem_melts",
                value: "true",
            },
            EnvironmentAttribute {
                identifier: "minecraft:gameplay/water_evaporates",
                value: "true",
            },
            EnvironmentAttribute {
                identifier: "minecraft:visual/ambient_light_color",
                value: "\"#302821\"",
            },
            EnvironmentAttribute {
                identifier: "minecraft:visual/default_dripstone_particle",
                value: "{\"type\":\"minecraft:dripping_dripstone_lava\"}",
            },
            EnvironmentAttribute {
                identifier: "minecraft:visual/fog_end_distance",
                value: "96.0",
            },
            EnvironmentAttribute {
                identifier: "minecraft:visual/fog_start_distance",
                value: "10.0",
            },
            EnvironmentAttribute {
                identifier: "minecraft:visual/sky_light_color",
                value: "\"#7a7aff\"",
            },
            EnvironmentAttribute {
                identifier: "minecraft:visual/sky_light_factor",
                value: "0.0",
            },
        ],
        has_fixed_time: true,
        has_ender_dragon_fight: false,
        skybox: Some("none"),
        cardinal_light: Some("nether"),
        timelines: &["#minecraft:in_nether"],
        default_clock: None,
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
