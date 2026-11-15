/* This file is generated. Do not edit manually. */
use pumpkin_util::math::int_provider::{
    BiasedToBottomIntProvider, ClampedIntProvider, ClampedNormalIntProvider, ConstantIntProvider,
    IntProvider, NormalIntProvider, TrapezoidIntProvider, UniformIntProvider, WeightedEntry,
    WeightedListIntProvider,
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
    pub const OVERWORLD: Self = Self {
        id: 0u8,
        minecraft_name: "minecraft:overworld",
        fixed_time: None,
        has_fixed_time: false,
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
        cloud_color: Some(-855638017i32),
        ambient_light_color: Some(657930i32),
        sky_light_color: None,
        sky_light_factor: None,
        fog_start_distance: None,
        fog_end_distance: None,
        cloud_height: Some(192.33f32),
        sky_light_level: None,
        water_evaporates: false,
        fast_lava: false,
        respawn_anchor_works: false,
        piglins_zombify: true,
        snow_golem_melts: false,
        can_start_raid: true,
        nether_portal_spawns_piglin: true,
        bed_rule: BedRule {
            can_sleep: BedRuleOption::WhenDark,
            can_set_spawn: BedRuleOption::Always,
            explodes: false,
        },
        timelines: Some("#minecraft:in_overworld"),
    };
    pub const OVERWORLD_CAVES: Self = Self {
        id: 1u8,
        minecraft_name: "minecraft:overworld_caves",
        fixed_time: None,
        has_fixed_time: false,
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
        cloud_color: Some(-855638017i32),
        ambient_light_color: Some(657930i32),
        sky_light_color: None,
        sky_light_factor: None,
        fog_start_distance: None,
        fog_end_distance: None,
        cloud_height: Some(192.33f32),
        sky_light_level: None,
        water_evaporates: false,
        fast_lava: false,
        respawn_anchor_works: false,
        piglins_zombify: true,
        snow_golem_melts: false,
        can_start_raid: true,
        nether_portal_spawns_piglin: true,
        bed_rule: BedRule {
            can_sleep: BedRuleOption::WhenDark,
            can_set_spawn: BedRuleOption::Always,
            explodes: false,
        },
        timelines: Some("#minecraft:in_overworld"),
    };
    pub const THE_END: Self = Self {
        id: 2u8,
        minecraft_name: "minecraft:the_end",
        fixed_time: None,
        has_fixed_time: true,
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
        ambient_light_color: Some(4147007i32),
        sky_light_color: Some(11296973i32),
        sky_light_factor: Some(0f32),
        fog_start_distance: None,
        fog_end_distance: None,
        cloud_height: None,
        sky_light_level: None,
        water_evaporates: false,
        fast_lava: false,
        respawn_anchor_works: false,
        piglins_zombify: true,
        snow_golem_melts: false,
        can_start_raid: true,
        nether_portal_spawns_piglin: false,
        bed_rule: BedRule {
            can_sleep: BedRuleOption::Never,
            can_set_spawn: BedRuleOption::Never,
            explodes: true,
        },
        timelines: Some("#minecraft:in_end"),
    };
    pub const THE_NETHER: Self = Self {
        id: 3u8,
        minecraft_name: "minecraft:the_nether",
        fixed_time: None,
        has_fixed_time: true,
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
        ambient_light_color: Some(3156001i32),
        sky_light_color: Some(8026879i32),
        sky_light_factor: Some(0f32),
        fog_start_distance: Some(10f32),
        fog_end_distance: Some(96f32),
        cloud_height: None,
        sky_light_level: Some(4f32),
        water_evaporates: true,
        fast_lava: true,
        respawn_anchor_works: true,
        piglins_zombify: false,
        snow_golem_melts: true,
        can_start_raid: false,
        nether_portal_spawns_piglin: false,
        bed_rule: BedRule {
            can_sleep: BedRuleOption::Never,
            can_set_spawn: BedRuleOption::Never,
            explodes: true,
        },
        timelines: Some("#minecraft:in_nether"),
    };
    pub fn from_name(name: &str) -> Option<&'static Self> {
        match name {
            "minecraft:overworld" => Some(&Self::OVERWORLD),
            "minecraft:overworld_caves" => Some(&Self::OVERWORLD_CAVES),
            "minecraft:the_end" => Some(&Self::THE_END),
            "minecraft:the_nether" => Some(&Self::THE_NETHER),
            _ => None,
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
