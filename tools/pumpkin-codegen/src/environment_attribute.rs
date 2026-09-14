use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

struct AttributeDef {
    id: u8,
    name: &'static str,
    attr_type: &'static str,
    syncable: bool,
    positional: bool,
    spatially_interpolated: bool,
}

const STATIC_ATTRIBUTES: &[AttributeDef] = &[
    AttributeDef {
        id: 0,
        name: "visual/fog_color",
        attr_type: "RgbColor",
        syncable: true,
        positional: true,
        spatially_interpolated: true,
    },
    AttributeDef {
        id: 1,
        name: "visual/fog_start_distance",
        attr_type: "Float",
        syncable: true,
        positional: true,
        spatially_interpolated: true,
    },
    AttributeDef {
        id: 2,
        name: "visual/fog_end_distance",
        attr_type: "Float",
        syncable: true,
        positional: true,
        spatially_interpolated: true,
    },
    AttributeDef {
        id: 3,
        name: "visual/sky_fog_end_distance",
        attr_type: "Float",
        syncable: true,
        positional: true,
        spatially_interpolated: true,
    },
    AttributeDef {
        id: 4,
        name: "visual/cloud_fog_end_distance",
        attr_type: "Float",
        syncable: true,
        positional: true,
        spatially_interpolated: true,
    },
    AttributeDef {
        id: 5,
        name: "visual/water_fog_color",
        attr_type: "RgbColor",
        syncable: true,
        positional: true,
        spatially_interpolated: true,
    },
    AttributeDef {
        id: 6,
        name: "visual/water_fog_start_distance",
        attr_type: "Float",
        syncable: true,
        positional: true,
        spatially_interpolated: true,
    },
    AttributeDef {
        id: 7,
        name: "visual/water_fog_end_distance",
        attr_type: "Float",
        syncable: true,
        positional: true,
        spatially_interpolated: true,
    },
    AttributeDef {
        id: 8,
        name: "visual/sky_color",
        attr_type: "RgbColor",
        syncable: true,
        positional: true,
        spatially_interpolated: true,
    },
    AttributeDef {
        id: 9,
        name: "visual/sunrise_sunset_color",
        attr_type: "ArgbColor",
        syncable: true,
        positional: true,
        spatially_interpolated: true,
    },
    AttributeDef {
        id: 10,
        name: "visual/cloud_color",
        attr_type: "ArgbColor",
        syncable: true,
        positional: true,
        spatially_interpolated: true,
    },
    AttributeDef {
        id: 11,
        name: "visual/cloud_height",
        attr_type: "Float",
        syncable: true,
        positional: true,
        spatially_interpolated: true,
    },
    AttributeDef {
        id: 12,
        name: "visual/sun_angle",
        attr_type: "AngleDegrees",
        syncable: true,
        positional: true,
        spatially_interpolated: true,
    },
    AttributeDef {
        id: 13,
        name: "visual/moon_angle",
        attr_type: "AngleDegrees",
        syncable: true,
        positional: true,
        spatially_interpolated: true,
    },
    AttributeDef {
        id: 14,
        name: "visual/star_angle",
        attr_type: "AngleDegrees",
        syncable: true,
        positional: true,
        spatially_interpolated: true,
    },
    AttributeDef {
        id: 15,
        name: "visual/moon_phase",
        attr_type: "MoonPhase",
        syncable: true,
        positional: true,
        spatially_interpolated: false,
    },
    AttributeDef {
        id: 16,
        name: "visual/star_brightness",
        attr_type: "Float",
        syncable: true,
        positional: true,
        spatially_interpolated: true,
    },
    AttributeDef {
        id: 17,
        name: "visual/block_light_tint",
        attr_type: "RgbColor",
        syncable: true,
        positional: true,
        spatially_interpolated: true,
    },
    AttributeDef {
        id: 18,
        name: "visual/sky_light_color",
        attr_type: "RgbColor",
        syncable: true,
        positional: true,
        spatially_interpolated: true,
    },
    AttributeDef {
        id: 19,
        name: "visual/sky_light_factor",
        attr_type: "Float",
        syncable: true,
        positional: true,
        spatially_interpolated: true,
    },
    AttributeDef {
        id: 20,
        name: "visual/night_vision_color",
        attr_type: "RgbColor",
        syncable: true,
        positional: true,
        spatially_interpolated: true,
    },
    AttributeDef {
        id: 21,
        name: "visual/ambient_light_color",
        attr_type: "RgbColor",
        syncable: true,
        positional: true,
        spatially_interpolated: true,
    },
    AttributeDef {
        id: 22,
        name: "visual/default_dripstone_particle",
        attr_type: "Particle",
        syncable: true,
        positional: true,
        spatially_interpolated: false,
    },
    AttributeDef {
        id: 23,
        name: "visual/ambient_particles",
        attr_type: "AmbientParticles",
        syncable: true,
        positional: true,
        spatially_interpolated: false,
    },
    AttributeDef {
        id: 24,
        name: "audio/background_music",
        attr_type: "BackgroundMusic",
        syncable: true,
        positional: true,
        spatially_interpolated: false,
    },
    AttributeDef {
        id: 25,
        name: "audio/music_volume",
        attr_type: "Float",
        syncable: true,
        positional: true,
        spatially_interpolated: false,
    },
    AttributeDef {
        id: 26,
        name: "audio/ambient_sounds",
        attr_type: "AmbientSounds",
        syncable: true,
        positional: true,
        spatially_interpolated: false,
    },
    AttributeDef {
        id: 27,
        name: "audio/firefly_bush_sounds",
        attr_type: "Boolean",
        syncable: true,
        positional: true,
        spatially_interpolated: false,
    },
    AttributeDef {
        id: 28,
        name: "gameplay/sky_light_level",
        attr_type: "Float",
        syncable: true,
        positional: false,
        spatially_interpolated: false,
    },
    AttributeDef {
        id: 29,
        name: "gameplay/can_start_raid",
        attr_type: "Boolean",
        syncable: false,
        positional: true,
        spatially_interpolated: false,
    },
    AttributeDef {
        id: 30,
        name: "gameplay/water_evaporates",
        attr_type: "Boolean",
        syncable: true,
        positional: true,
        spatially_interpolated: false,
    },
    AttributeDef {
        id: 31,
        name: "gameplay/bed_rule",
        attr_type: "BedRule",
        syncable: false,
        positional: true,
        spatially_interpolated: false,
    },
    AttributeDef {
        id: 32,
        name: "gameplay/respawn_anchor_works",
        attr_type: "Boolean",
        syncable: false,
        positional: true,
        spatially_interpolated: false,
    },
    AttributeDef {
        id: 33,
        name: "gameplay/nether_portal_spawns_piglin",
        attr_type: "Boolean",
        syncable: false,
        positional: true,
        spatially_interpolated: false,
    },
    AttributeDef {
        id: 34,
        name: "gameplay/fast_lava",
        attr_type: "Boolean",
        syncable: true,
        positional: false,
        spatially_interpolated: false,
    },
    AttributeDef {
        id: 35,
        name: "gameplay/increased_fire_burnout",
        attr_type: "Boolean",
        syncable: false,
        positional: true,
        spatially_interpolated: false,
    },
    AttributeDef {
        id: 36,
        name: "gameplay/eyeblossom_open",
        attr_type: "TriState",
        syncable: false,
        positional: true,
        spatially_interpolated: false,
    },
    AttributeDef {
        id: 37,
        name: "gameplay/turtle_egg_hatch_chance",
        attr_type: "Float",
        syncable: false,
        positional: true,
        spatially_interpolated: false,
    },
    AttributeDef {
        id: 38,
        name: "gameplay/piglins_zombify",
        attr_type: "Boolean",
        syncable: true,
        positional: true,
        spatially_interpolated: false,
    },
    AttributeDef {
        id: 39,
        name: "gameplay/snow_golem_melts",
        attr_type: "Boolean",
        syncable: false,
        positional: true,
        spatially_interpolated: false,
    },
    AttributeDef {
        id: 40,
        name: "gameplay/creaking_active",
        attr_type: "Boolean",
        syncable: true,
        positional: true,
        spatially_interpolated: false,
    },
    AttributeDef {
        id: 41,
        name: "gameplay/surface_slime_spawn_chance",
        attr_type: "Float",
        syncable: false,
        positional: true,
        spatially_interpolated: false,
    },
    AttributeDef {
        id: 42,
        name: "gameplay/cat_waking_up_gift_chance",
        attr_type: "Float",
        syncable: false,
        positional: true,
        spatially_interpolated: false,
    },
    AttributeDef {
        id: 43,
        name: "gameplay/bees_stay_in_hive",
        attr_type: "Boolean",
        syncable: false,
        positional: true,
        spatially_interpolated: false,
    },
    AttributeDef {
        id: 44,
        name: "gameplay/monsters_burn",
        attr_type: "Boolean",
        syncable: false,
        positional: true,
        spatially_interpolated: false,
    },
    AttributeDef {
        id: 45,
        name: "gameplay/can_pillager_patrol_spawn",
        attr_type: "Boolean",
        syncable: false,
        positional: true,
        spatially_interpolated: false,
    },
    AttributeDef {
        id: 46,
        name: "gameplay/villager_activity",
        attr_type: "Activity",
        syncable: false,
        positional: true,
        spatially_interpolated: false,
    },
    AttributeDef {
        id: 47,
        name: "gameplay/baby_villager_activity",
        attr_type: "Activity",
        syncable: false,
        positional: true,
        spatially_interpolated: false,
    },
];

#[derive(serde::Deserialize)]
struct TimelineJson {
    #[serde(default)]
    period_ticks: Option<i32>,
    #[serde(default)]
    tracks: std::collections::BTreeMap<String, TimelineTrackJson>,
}

#[derive(serde::Deserialize)]
struct TimelineTrackJson {
    #[serde(default)]
    keyframes: Vec<KeyframeJson>,
}

#[derive(serde::Deserialize)]
struct KeyframeJson {
    ticks: i32,
    value: serde_json::Value,
}

pub fn build() -> TokenStream {
    let timeline_path =
        std::path::Path::new("../../assets/datapacks/26_2/data/minecraft/timeline/day.json");
    let (
        period_ticks,
        sky_light_level_keyframes,
        monsters_burn_keyframes,
        bees_stay_in_hive_keyframes,
        creaking_active_keyframes,
        eyeblossom_open_keyframes,
    ) = if let Ok(content) = std::fs::read_to_string(timeline_path) {
        if let Ok(timeline) = serde_json::from_str::<TimelineJson>(&content) {
            let period = timeline.period_ticks.unwrap_or(24000);
            let mut sky_kfs = Vec::new();
            let mut monsters_kfs = Vec::new();
            let mut bees_kfs = Vec::new();
            let mut creaking_kfs = Vec::new();
            let mut eyeblossom_kfs = Vec::new();

            if let Some(track) = timeline.tracks.get("minecraft:gameplay/sky_light_level") {
                for kf in &track.keyframes {
                    let ticks = kf.ticks;
                    let val = kf.value.as_f64().unwrap_or(1.0) as f32;
                    sky_kfs.push(quote! { FloatKeyframe::new(#ticks, #val) });
                }
            }
            if let Some(track) = timeline.tracks.get("minecraft:gameplay/monsters_burn") {
                for kf in &track.keyframes {
                    let ticks = kf.ticks;
                    let val = kf.value.as_bool().unwrap_or(false);
                    monsters_kfs.push(quote! { BoolKeyframe::new(#ticks, #val) });
                }
            }
            if let Some(track) = timeline.tracks.get("minecraft:gameplay/bees_stay_in_hive") {
                for kf in &track.keyframes {
                    let ticks = kf.ticks;
                    let val = kf.value.as_bool().unwrap_or(false);
                    bees_kfs.push(quote! { BoolKeyframe::new(#ticks, #val) });
                }
            }
            if let Some(track) = timeline.tracks.get("minecraft:gameplay/creaking_active") {
                for kf in &track.keyframes {
                    let ticks = kf.ticks;
                    let val = kf.value.as_bool().unwrap_or(false);
                    creaking_kfs.push(quote! { BoolKeyframe::new(#ticks, #val) });
                }
            }
            if let Some(track) = timeline.tracks.get("minecraft:gameplay/eyeblossom_open") {
                for kf in &track.keyframes {
                    let ticks = kf.ticks;
                    let val = kf.value.as_bool().unwrap_or(false);
                    eyeblossom_kfs.push(quote! { BoolKeyframe::new(#ticks, #val) });
                }
            }
            (
                period,
                sky_kfs,
                monsters_kfs,
                bees_kfs,
                creaking_kfs,
                eyeblossom_kfs,
            )
        } else {
            (
                24000,
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            )
        }
    } else {
        (
            24000,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        )
    };

    let moon_path =
        std::path::Path::new("../../assets/datapacks/26_2/data/minecraft/timeline/moon.json");
    let (moon_period_ticks, surface_slime_spawn_chance_keyframes, moon_phase_keyframes) =
        if let Ok(content) = std::fs::read_to_string(moon_path) {
            if let Ok(timeline) = serde_json::from_str::<TimelineJson>(&content) {
                let period = timeline.period_ticks.unwrap_or(192000);
                let mut slime_kfs = Vec::new();
                let mut moon_kfs = Vec::new();

                if let Some(track) = timeline
                    .tracks
                    .get("minecraft:gameplay/surface_slime_spawn_chance")
                {
                    for kf in &track.keyframes {
                        let ticks = kf.ticks;
                        let val = kf.value.as_f64().unwrap_or(0.0) as f32;
                        slime_kfs.push(quote! { FloatKeyframe::new(#ticks, #val) });
                    }
                }
                if let Some(track) = timeline.tracks.get("minecraft:visual/moon_phase") {
                    for kf in &track.keyframes {
                        let ticks = kf.ticks;
                        let phase_str = kf.value.as_str().unwrap_or("full_moon");
                        let phase_ident = format_ident!("{}", phase_str.to_pascal_case());
                        moon_kfs.push(
                            quote! { MoonPhaseKeyframe::new(#ticks, MoonPhase::#phase_ident) },
                        );
                    }
                }
                (period, slime_kfs, moon_kfs)
            } else {
                (192000, Vec::new(), Vec::new())
            }
        } else {
            (192000, Vec::new(), Vec::new())
        };

    let early_game_path =
        std::path::Path::new("../../assets/datapacks/26_2/data/minecraft/timeline/early_game.json");
    let can_pillager_patrol_spawn_keyframes =
        if let Ok(content) = std::fs::read_to_string(early_game_path) {
            if let Ok(timeline) = serde_json::from_str::<TimelineJson>(&content) {
                let mut patrol_kfs = Vec::new();
                if let Some(track) = timeline
                    .tracks
                    .get("minecraft:gameplay/can_pillager_patrol_spawn")
                {
                    for kf in &track.keyframes {
                        let ticks = kf.ticks;
                        let val = kf.value.as_bool().unwrap_or(false);
                        patrol_kfs.push(quote! { BoolKeyframe::new(#ticks, #val) });
                    }
                }
                patrol_kfs
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

    let villager_schedule_path = std::path::Path::new(
        "../../assets/datapacks/26_2/data/minecraft/timeline/villager_schedule.json",
    );
    let (schedule_period_ticks, villager_activity_keyframes, baby_villager_activity_keyframes) =
        if let Ok(content) = std::fs::read_to_string(villager_schedule_path) {
            if let Ok(timeline) = serde_json::from_str::<TimelineJson>(&content) {
                let period = timeline.period_ticks.unwrap_or(24000);
                let mut villager_kfs = Vec::new();
                let mut baby_kfs = Vec::new();

                if let Some(track) = timeline.tracks.get("minecraft:gameplay/villager_activity") {
                    for kf in &track.keyframes {
                        let ticks = kf.ticks;
                        let val_str = kf.value.as_str().unwrap_or("idle");
                        let clean_str = val_str.strip_prefix("minecraft:").unwrap_or(val_str);
                        let ident = format_ident!("{}", clean_str.to_pascal_case());
                        villager_kfs
                            .push(quote! { ActivityKeyframe::new(#ticks, Activity::#ident) });
                    }
                }
                if let Some(track) = timeline
                    .tracks
                    .get("minecraft:gameplay/baby_villager_activity")
                {
                    for kf in &track.keyframes {
                        let ticks = kf.ticks;
                        let val_str = kf.value.as_str().unwrap_or("idle");
                        let clean_str = val_str.strip_prefix("minecraft:").unwrap_or(val_str);
                        let ident = format_ident!("{}", clean_str.to_pascal_case());
                        baby_kfs.push(quote! { ActivityKeyframe::new(#ticks, Activity::#ident) });
                    }
                }
                (period, villager_kfs, baby_kfs)
            } else {
                (24000, Vec::new(), Vec::new())
            }
        } else {
            (24000, Vec::new(), Vec::new())
        };

    let mut variants = TokenStream::new();
    let mut to_name_match = TokenStream::new();
    let mut to_resource_match = TokenStream::new();
    let mut to_id_match = TokenStream::new();
    let mut from_id_match = TokenStream::new();
    let mut from_name_match = TokenStream::new();
    let mut to_type_match = TokenStream::new();
    let mut syncable_match = TokenStream::new();
    let mut positional_match = TokenStream::new();
    let mut spatially_interpolated_match = TokenStream::new();

    for def in STATIC_ATTRIBUTES {
        let ident = format_ident!("{}", def.name.to_pascal_case());
        let id_lit = def.id;
        let name_str = def.name;
        let resource_str = format!("minecraft:{name_str}");
        let type_ident = format_ident!("{}", def.attr_type);
        let syncable = def.syncable;
        let positional = def.positional;
        let spatially_interpolated = def.spatially_interpolated;

        variants.extend(quote! {
            #ident,
        });

        to_id_match.extend(quote! {
            Self::#ident => #id_lit,
        });

        from_id_match.extend(quote! {
            #id_lit => Some(Self::#ident),
        });

        to_name_match.extend(quote! {
            Self::#ident => #name_str,
        });

        to_resource_match.extend(quote! {
            Self::#ident => #resource_str,
        });

        from_name_match.extend(quote! {
            #name_str | #resource_str => Some(Self::#ident),
        });

        to_type_match.extend(quote! {
            Self::#ident => EnvironmentAttributeType::#type_ident,
        });

        syncable_match.extend(quote! {
            Self::#ident => #syncable,
        });

        positional_match.extend(quote! {
            Self::#ident => #positional,
        });

        spatially_interpolated_match.extend(quote! {
            Self::#ident => #spatially_interpolated,
        });
    }

    quote! {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum EnvironmentAttributeType {
            RgbColor,
            ArgbColor,
            Float,
            AngleDegrees,
            Boolean,
            TriState,
            MoonPhase,
            Activity,
            BedRule,
            Particle,
            AmbientParticles,
            BackgroundMusic,
            AmbientSounds,
            Integer,
        }

        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum EnvironmentAttribute {
            #variants
        }

        impl EnvironmentAttribute {
            #[must_use]
            pub const fn id(&self) -> u8 {
                match self {
                    #to_id_match
                }
            }

            #[must_use]
            pub const fn name(&self) -> &'static str {
                match self {
                    #to_name_match
                }
            }

            #[must_use]
            pub const fn resource_location(&self) -> &'static str {
                match self {
                    #to_resource_match
                }
            }

            #[must_use]
            pub const fn attribute_type(&self) -> EnvironmentAttributeType {
                match self {
                    #to_type_match
                }
            }

            #[must_use]
            pub const fn is_syncable(&self) -> bool {
                match self {
                    #syncable_match
                }
            }

            #[must_use]
            pub const fn is_positional(&self) -> bool {
                match self {
                    #positional_match
                }
            }

            #[must_use]
            pub const fn is_spatially_interpolated(&self) -> bool {
                match self {
                    #spatially_interpolated_match
                }
            }

            #[must_use]
            pub fn from_name(name: &str) -> Option<Self> {
                match name {
                    #from_name_match
                    _ => None,
                }
            }

            #[must_use]
            pub const fn from_id(id: u8) -> Option<Self> {
                match id {
                    #from_id_match
                    _ => None,
                }
            }
        }

        #[derive(Debug, Clone, Copy, PartialEq)]
        pub struct FloatKeyframe {
            pub ticks: i32,
            pub value: f32,
        }

        impl FloatKeyframe {
            pub const fn new(ticks: i32, value: f32) -> Self {
                Self { ticks, value }
            }
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct BoolKeyframe {
            pub ticks: i32,
            pub value: bool,
        }

        impl BoolKeyframe {
            pub const fn new(ticks: i32, value: bool) -> Self {
                Self { ticks, value }
            }
        }

        pub struct DayTimeline;

        impl DayTimeline {
            pub const PERIOD_TICKS: i32 = #period_ticks;
            pub const SKY_LIGHT_LEVEL_KEYFRAMES: &'static [FloatKeyframe] = &[
                #(#sky_light_level_keyframes),*
            ];
            pub const MONSTERS_BURN_KEYFRAMES: &'static [BoolKeyframe] = &[
                #(#monsters_burn_keyframes),*
            ];
            pub const BEES_STAY_IN_HIVE_KEYFRAMES: &'static [BoolKeyframe] = &[
                #(#bees_stay_in_hive_keyframes),*
            ];
            pub const CREAKING_ACTIVE_KEYFRAMES: &'static [BoolKeyframe] = &[
                #(#creaking_active_keyframes),*
            ];
            pub const EYEBLOSSOM_OPEN_KEYFRAMES: &'static [BoolKeyframe] = &[
                #(#eyeblossom_open_keyframes),*
            ];
        }

        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum MoonPhase {
            FullMoon,
            WaningGibbous,
            ThirdQuarter,
            WaningCrescent,
            NewMoon,
            WaxingCrescent,
            FirstQuarter,
            WaxingGibbous,
        }

        impl MoonPhase {
            #[must_use]
            pub const fn index(&self) -> u8 {
                match self {
                    Self::FullMoon => 0,
                    Self::WaningGibbous => 1,
                    Self::ThirdQuarter => 2,
                    Self::WaningCrescent => 3,
                    Self::NewMoon => 4,
                    Self::WaxingCrescent => 5,
                    Self::FirstQuarter => 6,
                    Self::WaxingGibbous => 7,
                }
            }

            #[must_use]
            pub const fn from_index(index: u8) -> Option<Self> {
                match index {
                    0 => Some(Self::FullMoon),
                    1 => Some(Self::WaningGibbous),
                    2 => Some(Self::ThirdQuarter),
                    3 => Some(Self::WaningCrescent),
                    4 => Some(Self::NewMoon),
                    5 => Some(Self::WaxingCrescent),
                    6 => Some(Self::FirstQuarter),
                    7 => Some(Self::WaxingGibbous),
                    _ => None,
                }
            }

            #[must_use]
            pub const fn name(&self) -> &'static str {
                match self {
                    Self::FullMoon => "full_moon",
                    Self::WaningGibbous => "waning_gibbous",
                    Self::ThirdQuarter => "third_quarter",
                    Self::WaningCrescent => "waning_crescent",
                    Self::NewMoon => "new_moon",
                    Self::WaxingCrescent => "waxing_crescent",
                    Self::FirstQuarter => "first_quarter",
                    Self::WaxingGibbous => "waxing_gibbous",
                }
            }

            #[must_use]
            pub fn from_name(name: &str) -> Option<Self> {
                match name {
                    "full_moon" => Some(Self::FullMoon),
                    "waning_gibbous" => Some(Self::WaningGibbous),
                    "third_quarter" => Some(Self::ThirdQuarter),
                    "waning_crescent" => Some(Self::WaningCrescent),
                    "new_moon" => Some(Self::NewMoon),
                    "waxing_crescent" => Some(Self::WaxingCrescent),
                    "first_quarter" => Some(Self::FirstQuarter),
                    "waxing_gibbous" => Some(Self::WaxingGibbous),
                    _ => None,
                }
            }

            #[must_use]
            pub const fn start_tick(&self) -> i32 {
                self.index() as i32 * 24000
            }
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct MoonPhaseKeyframe {
            pub ticks: i32,
            pub value: MoonPhase,
        }

        impl MoonPhaseKeyframe {
            pub const fn new(ticks: i32, value: MoonPhase) -> Self {
                Self { ticks, value }
            }
        }

        pub struct MoonTimeline;

        impl MoonTimeline {
            pub const PERIOD_TICKS: i32 = #moon_period_ticks;
            pub const SURFACE_SLIME_SPAWN_CHANCE_KEYFRAMES: &'static [FloatKeyframe] = &[
                #(#surface_slime_spawn_chance_keyframes),*
            ];
            pub const MOON_PHASE_KEYFRAMES: &'static [MoonPhaseKeyframe] = &[
                #(#moon_phase_keyframes),*
            ];
        }

        pub struct EarlyGameTimeline;

        impl EarlyGameTimeline {
            pub const CAN_PILLAGER_PATROL_SPAWN_KEYFRAMES: &'static [BoolKeyframe] = &[
                #(#can_pillager_patrol_spawn_keyframes),*
            ];
        }

        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum Activity {
            Core,
            Idle,
            Work,
            Play,
            Rest,
            Meet,
            Panic,
            Raid,
            PreRaid,
            Hide,
            Fight,
            Celebrate,
            AdmireItem,
            Avoid,
            Ride,
            PlayDead,
            LongJump,
            Ram,
            Tongue,
            Swim,
            LaySpawn,
            Sniff,
            Investigate,
            Roar,
            Emerge,
            Dig,
        }

        impl Activity {
            #[must_use]
            pub const fn name(&self) -> &'static str {
                match self {
                    Self::Core => "core",
                    Self::Idle => "idle",
                    Self::Work => "work",
                    Self::Play => "play",
                    Self::Rest => "rest",
                    Self::Meet => "meet",
                    Self::Panic => "panic",
                    Self::Raid => "raid",
                    Self::PreRaid => "pre_raid",
                    Self::Hide => "hide",
                    Self::Fight => "fight",
                    Self::Celebrate => "celebrate",
                    Self::AdmireItem => "admire_item",
                    Self::Avoid => "avoid",
                    Self::Ride => "ride",
                    Self::PlayDead => "play_dead",
                    Self::LongJump => "long_jump",
                    Self::Ram => "ram",
                    Self::Tongue => "tongue",
                    Self::Swim => "swim",
                    Self::LaySpawn => "lay_spawn",
                    Self::Sniff => "sniff",
                    Self::Investigate => "investigate",
                    Self::Roar => "roar",
                    Self::Emerge => "emerge",
                    Self::Dig => "dig",
                }
            }

            #[must_use]
            pub fn from_name(name: &str) -> Option<Self> {
                let name = name.strip_prefix("minecraft:").unwrap_or(name);
                match name {
                    "core" => Some(Self::Core),
                    "idle" => Some(Self::Idle),
                    "work" => Some(Self::Work),
                    "play" => Some(Self::Play),
                    "rest" => Some(Self::Rest),
                    "meet" => Some(Self::Meet),
                    "panic" => Some(Self::Panic),
                    "raid" => Some(Self::Raid),
                    "pre_raid" => Some(Self::PreRaid),
                    "hide" => Some(Self::Hide),
                    "fight" => Some(Self::Fight),
                    "celebrate" => Some(Self::Celebrate),
                    "admire_item" => Some(Self::AdmireItem),
                    "avoid" => Some(Self::Avoid),
                    "ride" => Some(Self::Ride),
                    "play_dead" => Some(Self::PlayDead),
                    "long_jump" => Some(Self::LongJump),
                    "ram" => Some(Self::Ram),
                    "tongue" => Some(Self::Tongue),
                    "swim" => Some(Self::Swim),
                    "lay_spawn" => Some(Self::LaySpawn),
                    "sniff" => Some(Self::Sniff),
                    "investigate" => Some(Self::Investigate),
                    "roar" => Some(Self::Roar),
                    "emerge" => Some(Self::Emerge),
                    "dig" => Some(Self::Dig),
                    _ => None,
                }
            }
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct ActivityKeyframe {
            pub ticks: i32,
            pub value: Activity,
        }

        impl ActivityKeyframe {
            pub const fn new(ticks: i32, value: Activity) -> Self {
                Self { ticks, value }
            }
        }

        pub struct VillagerScheduleTimeline;

        impl VillagerScheduleTimeline {
            pub const PERIOD_TICKS: i32 = #schedule_period_ticks;
            pub const VILLAGER_ACTIVITY_KEYFRAMES: &'static [ActivityKeyframe] = &[
                #(#villager_activity_keyframes),*
            ];
            pub const BABY_VILLAGER_ACTIVITY_KEYFRAMES: &'static [ActivityKeyframe] = &[
                #(#baby_villager_activity_keyframes),*
            ];
        }

        /// Samples a cyclic float keyframe track with linear interpolation across `period_ticks`.
        #[must_use]
        pub fn sample_float_track(keyframes: &[FloatKeyframe], period_ticks: i32, ticks: i64) -> f32 {
            if keyframes.is_empty() {
                return 1.0;
            }
            if keyframes.len() == 1 {
                return keyframes[0].value;
            }
            let t = ticks.rem_euclid(period_ticks as i64) as f32;
            let n = keyframes.len();
            let first = &keyframes[0];
            let last = &keyframes[n - 1];

            if t <= first.ticks as f32 {
                let seg_len = (first.ticks + period_ticks - last.ticks) as f32;
                if seg_len <= 0.0 {
                    return first.value;
                }
                let alpha = (t + period_ticks as f32 - last.ticks as f32) / seg_len;
                return last.value + alpha * (first.value - last.value);
            }
            if t >= last.ticks as f32 {
                let seg_len = (first.ticks + period_ticks - last.ticks) as f32;
                if seg_len <= 0.0 {
                    return last.value;
                }
                let alpha = (t - last.ticks as f32) / seg_len;
                return last.value + alpha * (first.value - last.value);
            }

            for i in 0..n - 1 {
                let k1 = &keyframes[i];
                let k2 = &keyframes[i + 1];
                if t >= k1.ticks as f32 && t <= k2.ticks as f32 {
                    let seg_len = (k2.ticks - k1.ticks) as f32;
                    if seg_len <= 0.0 {
                        return k1.value;
                    }
                    let alpha = (t - k1.ticks as f32) / seg_len;
                    return k1.value + alpha * (k2.value - k1.value);
                }
            }

            last.value
        }

        /// Samples a cyclic float keyframe track with step interpolation (constant easing) across `period_ticks`.
        #[must_use]
        pub fn sample_step_float_track(keyframes: &[FloatKeyframe], period_ticks: i32, ticks: i64) -> f32 {
            if keyframes.is_empty() {
                return 0.0;
            }
            let t = ticks.rem_euclid(period_ticks as i64) as i32;
            let mut current = keyframes[keyframes.len() - 1].value;
            for kf in keyframes {
                if t >= kf.ticks {
                    current = kf.value;
                } else {
                    break;
                }
            }
            current
        }

        /// Samples a cyclic boolean keyframe track (step function) across `period_ticks`.
        #[must_use]
        pub fn sample_bool_track(keyframes: &[BoolKeyframe], period_ticks: i32, ticks: i64) -> bool {
            if keyframes.is_empty() {
                return false;
            }
            let t = ticks.rem_euclid(period_ticks as i64) as i32;
            let mut current = keyframes[keyframes.len() - 1].value;
            for kf in keyframes {
                if t >= kf.ticks {
                    current = kf.value;
                } else {
                    break;
                }
            }
            current
        }

        /// Samples an unbounded (non-cyclic) boolean keyframe track (step function).
        #[must_use]
        pub fn sample_unbounded_bool_track(keyframes: &[BoolKeyframe], ticks: i64) -> bool {
            if keyframes.is_empty() {
                return false;
            }
            let mut current = keyframes[0].value;
            for kf in keyframes {
                if ticks >= kf.ticks as i64 {
                    current = kf.value;
                } else {
                    break;
                }
            }
            current
        }

        /// Samples a cyclic moon phase keyframe track across `period_ticks`.
        #[must_use]
        pub fn sample_moon_phase_track(keyframes: &[MoonPhaseKeyframe], period_ticks: i32, ticks: i64) -> MoonPhase {
            if keyframes.is_empty() {
                return MoonPhase::FullMoon;
            }
            let t = ticks.rem_euclid(period_ticks as i64) as i32;
            let mut current = keyframes[keyframes.len() - 1].value;
            for kf in keyframes {
                if t >= kf.ticks {
                    current = kf.value;
                } else {
                    break;
                }
            }
            current
        }

        /// Samples a cyclic activity keyframe track across `period_ticks`.
        #[must_use]
        pub fn sample_activity_track(keyframes: &[ActivityKeyframe], period_ticks: i32, ticks: i64) -> Activity {
            if keyframes.is_empty() {
                return Activity::Idle;
            }
            let t = ticks.rem_euclid(period_ticks as i64) as i32;
            let mut current = keyframes[keyframes.len() - 1].value;
            for kf in keyframes {
                if t >= kf.ticks {
                    current = kf.value;
                } else {
                    break;
                }
            }
            current
        }
    }
}
