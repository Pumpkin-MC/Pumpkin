/* This file is generated. Do not edit manually. */
use crate::effect::StatusEffect;
use std::hash::Hash;
pub struct Potion {
    pub id: u8,
    pub name: &'static str,
    pub effects: &'static [Effect],
}
#[derive(Clone)]
pub struct Effect {
    pub effect_type: &'static StatusEffect,
    pub duration: i32,
    pub amplifier: u8,
    pub ambient: bool,
    pub show_particles: bool,
    pub show_icon: bool,
    pub blend: bool,
}
impl Potion {
    pub const AWKWARD: Self = Self {
        name: "awkward",
        id: 3u8,
        effects: &[],
    };
    pub const FIRE_RESISTANCE: Self = Self {
        name: "fire_resistance",
        id: 11u8,
        effects: &[Effect {
            effect_type: &StatusEffect::FIRE_RESISTANCE,
            duration: 3600i32,
            amplifier: 0u8,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        }],
    };
    pub const HARMING: Self = Self {
        name: "harming",
        id: 26u8,
        effects: &[Effect {
            effect_type: &StatusEffect::INSTANT_DAMAGE,
            duration: 1i32,
            amplifier: 0u8,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        }],
    };
    pub const HEALING: Self = Self {
        name: "healing",
        id: 24u8,
        effects: &[Effect {
            effect_type: &StatusEffect::INSTANT_HEALTH,
            duration: 1i32,
            amplifier: 0u8,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        }],
    };
    pub const INFESTED: Self = Self {
        name: "infested",
        id: 45u8,
        effects: &[Effect {
            effect_type: &StatusEffect::INFESTED,
            duration: 3600i32,
            amplifier: 0u8,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        }],
    };
    pub const INVISIBILITY: Self = Self {
        name: "invisibility",
        id: 6u8,
        effects: &[Effect {
            effect_type: &StatusEffect::INVISIBILITY,
            duration: 3600i32,
            amplifier: 0u8,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        }],
    };
    pub const LEAPING: Self = Self {
        name: "leaping",
        id: 8u8,
        effects: &[Effect {
            effect_type: &StatusEffect::JUMP_BOOST,
            duration: 3600i32,
            amplifier: 0u8,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        }],
    };
    pub const LONG_FIRE_RESISTANCE: Self = Self {
        name: "long_fire_resistance",
        id: 12u8,
        effects: &[Effect {
            effect_type: &StatusEffect::FIRE_RESISTANCE,
            duration: 9600i32,
            amplifier: 0u8,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        }],
    };
    pub const LONG_INVISIBILITY: Self = Self {
        name: "long_invisibility",
        id: 7u8,
        effects: &[Effect {
            effect_type: &StatusEffect::INVISIBILITY,
            duration: 9600i32,
            amplifier: 0u8,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        }],
    };
    pub const LONG_LEAPING: Self = Self {
        name: "long_leaping",
        id: 9u8,
        effects: &[Effect {
            effect_type: &StatusEffect::JUMP_BOOST,
            duration: 9600i32,
            amplifier: 0u8,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        }],
    };
    pub const LONG_NIGHT_VISION: Self = Self {
        name: "long_night_vision",
        id: 5u8,
        effects: &[Effect {
            effect_type: &StatusEffect::NIGHT_VISION,
            duration: 9600i32,
            amplifier: 0u8,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        }],
    };
    pub const LONG_POISON: Self = Self {
        name: "long_poison",
        id: 29u8,
        effects: &[Effect {
            effect_type: &StatusEffect::POISON,
            duration: 1800i32,
            amplifier: 0u8,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        }],
    };
    pub const LONG_REGENERATION: Self = Self {
        name: "long_regeneration",
        id: 32u8,
        effects: &[Effect {
            effect_type: &StatusEffect::REGENERATION,
            duration: 1800i32,
            amplifier: 0u8,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        }],
    };
    pub const LONG_SLOW_FALLING: Self = Self {
        name: "long_slow_falling",
        id: 41u8,
        effects: &[Effect {
            effect_type: &StatusEffect::SLOW_FALLING,
            duration: 4800i32,
            amplifier: 0u8,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        }],
    };
    pub const LONG_SLOWNESS: Self = Self {
        name: "long_slowness",
        id: 17u8,
        effects: &[Effect {
            effect_type: &StatusEffect::SLOWNESS,
            duration: 4800i32,
            amplifier: 0u8,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        }],
    };
    pub const LONG_STRENGTH: Self = Self {
        name: "long_strength",
        id: 35u8,
        effects: &[Effect {
            effect_type: &StatusEffect::STRENGTH,
            duration: 9600i32,
            amplifier: 0u8,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        }],
    };
    pub const LONG_SWIFTNESS: Self = Self {
        name: "long_swiftness",
        id: 14u8,
        effects: &[Effect {
            effect_type: &StatusEffect::SPEED,
            duration: 9600i32,
            amplifier: 0u8,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        }],
    };
    pub const LONG_TURTLE_MASTER: Self = Self {
        name: "long_turtle_master",
        id: 20u8,
        effects: &[
            Effect {
                effect_type: &StatusEffect::SLOWNESS,
                duration: 800i32,
                amplifier: 3u8,
                ambient: false,
                show_particles: true,
                show_icon: true,
                blend: false,
            },
            Effect {
                effect_type: &StatusEffect::RESISTANCE,
                duration: 800i32,
                amplifier: 2u8,
                ambient: false,
                show_particles: true,
                show_icon: true,
                blend: false,
            },
        ],
    };
    pub const LONG_WATER_BREATHING: Self = Self {
        name: "long_water_breathing",
        id: 23u8,
        effects: &[Effect {
            effect_type: &StatusEffect::WATER_BREATHING,
            duration: 9600i32,
            amplifier: 0u8,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        }],
    };
    pub const LONG_WEAKNESS: Self = Self {
        name: "long_weakness",
        id: 38u8,
        effects: &[Effect {
            effect_type: &StatusEffect::WEAKNESS,
            duration: 4800i32,
            amplifier: 0u8,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        }],
    };
    pub const LUCK: Self = Self {
        name: "luck",
        id: 39u8,
        effects: &[Effect {
            effect_type: &StatusEffect::LUCK,
            duration: 6000i32,
            amplifier: 0u8,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        }],
    };
    pub const MUNDANE: Self = Self {
        name: "mundane",
        id: 1u8,
        effects: &[],
    };
    pub const NIGHT_VISION: Self = Self {
        name: "night_vision",
        id: 4u8,
        effects: &[Effect {
            effect_type: &StatusEffect::NIGHT_VISION,
            duration: 3600i32,
            amplifier: 0u8,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        }],
    };
    pub const OOZING: Self = Self {
        name: "oozing",
        id: 44u8,
        effects: &[Effect {
            effect_type: &StatusEffect::OOZING,
            duration: 3600i32,
            amplifier: 0u8,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        }],
    };
    pub const POISON: Self = Self {
        name: "poison",
        id: 28u8,
        effects: &[Effect {
            effect_type: &StatusEffect::POISON,
            duration: 900i32,
            amplifier: 0u8,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        }],
    };
    pub const REGENERATION: Self = Self {
        name: "regeneration",
        id: 31u8,
        effects: &[Effect {
            effect_type: &StatusEffect::REGENERATION,
            duration: 900i32,
            amplifier: 0u8,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        }],
    };
    pub const SLOW_FALLING: Self = Self {
        name: "slow_falling",
        id: 40u8,
        effects: &[Effect {
            effect_type: &StatusEffect::SLOW_FALLING,
            duration: 1800i32,
            amplifier: 0u8,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        }],
    };
    pub const SLOWNESS: Self = Self {
        name: "slowness",
        id: 16u8,
        effects: &[Effect {
            effect_type: &StatusEffect::SLOWNESS,
            duration: 1800i32,
            amplifier: 0u8,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        }],
    };
    pub const STRENGTH: Self = Self {
        name: "strength",
        id: 34u8,
        effects: &[Effect {
            effect_type: &StatusEffect::STRENGTH,
            duration: 3600i32,
            amplifier: 0u8,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        }],
    };
    pub const STRONG_HARMING: Self = Self {
        name: "strong_harming",
        id: 27u8,
        effects: &[Effect {
            effect_type: &StatusEffect::INSTANT_DAMAGE,
            duration: 1i32,
            amplifier: 1u8,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        }],
    };
    pub const STRONG_HEALING: Self = Self {
        name: "strong_healing",
        id: 25u8,
        effects: &[Effect {
            effect_type: &StatusEffect::INSTANT_HEALTH,
            duration: 1i32,
            amplifier: 1u8,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        }],
    };
    pub const STRONG_LEAPING: Self = Self {
        name: "strong_leaping",
        id: 10u8,
        effects: &[Effect {
            effect_type: &StatusEffect::JUMP_BOOST,
            duration: 1800i32,
            amplifier: 1u8,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        }],
    };
    pub const STRONG_POISON: Self = Self {
        name: "strong_poison",
        id: 30u8,
        effects: &[Effect {
            effect_type: &StatusEffect::POISON,
            duration: 432i32,
            amplifier: 1u8,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        }],
    };
    pub const STRONG_REGENERATION: Self = Self {
        name: "strong_regeneration",
        id: 33u8,
        effects: &[Effect {
            effect_type: &StatusEffect::REGENERATION,
            duration: 450i32,
            amplifier: 1u8,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        }],
    };
    pub const STRONG_SLOWNESS: Self = Self {
        name: "strong_slowness",
        id: 18u8,
        effects: &[Effect {
            effect_type: &StatusEffect::SLOWNESS,
            duration: 400i32,
            amplifier: 3u8,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        }],
    };
    pub const STRONG_STRENGTH: Self = Self {
        name: "strong_strength",
        id: 36u8,
        effects: &[Effect {
            effect_type: &StatusEffect::STRENGTH,
            duration: 1800i32,
            amplifier: 1u8,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        }],
    };
    pub const STRONG_SWIFTNESS: Self = Self {
        name: "strong_swiftness",
        id: 15u8,
        effects: &[Effect {
            effect_type: &StatusEffect::SPEED,
            duration: 1800i32,
            amplifier: 1u8,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        }],
    };
    pub const STRONG_TURTLE_MASTER: Self = Self {
        name: "strong_turtle_master",
        id: 21u8,
        effects: &[
            Effect {
                effect_type: &StatusEffect::SLOWNESS,
                duration: 400i32,
                amplifier: 5u8,
                ambient: false,
                show_particles: true,
                show_icon: true,
                blend: false,
            },
            Effect {
                effect_type: &StatusEffect::RESISTANCE,
                duration: 400i32,
                amplifier: 3u8,
                ambient: false,
                show_particles: true,
                show_icon: true,
                blend: false,
            },
        ],
    };
    pub const SWIFTNESS: Self = Self {
        name: "swiftness",
        id: 13u8,
        effects: &[Effect {
            effect_type: &StatusEffect::SPEED,
            duration: 3600i32,
            amplifier: 0u8,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        }],
    };
    pub const THICK: Self = Self {
        name: "thick",
        id: 2u8,
        effects: &[],
    };
    pub const TURTLE_MASTER: Self = Self {
        name: "turtle_master",
        id: 19u8,
        effects: &[
            Effect {
                effect_type: &StatusEffect::SLOWNESS,
                duration: 400i32,
                amplifier: 3u8,
                ambient: false,
                show_particles: true,
                show_icon: true,
                blend: false,
            },
            Effect {
                effect_type: &StatusEffect::RESISTANCE,
                duration: 400i32,
                amplifier: 2u8,
                ambient: false,
                show_particles: true,
                show_icon: true,
                blend: false,
            },
        ],
    };
    pub const WATER: Self = Self {
        name: "water",
        id: 0u8,
        effects: &[],
    };
    pub const WATER_BREATHING: Self = Self {
        name: "water_breathing",
        id: 22u8,
        effects: &[Effect {
            effect_type: &StatusEffect::WATER_BREATHING,
            duration: 3600i32,
            amplifier: 0u8,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        }],
    };
    pub const WEAKNESS: Self = Self {
        name: "weakness",
        id: 37u8,
        effects: &[Effect {
            effect_type: &StatusEffect::WEAKNESS,
            duration: 1800i32,
            amplifier: 0u8,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        }],
    };
    pub const WEAVING: Self = Self {
        name: "weaving",
        id: 43u8,
        effects: &[Effect {
            effect_type: &StatusEffect::WEAVING,
            duration: 3600i32,
            amplifier: 0u8,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        }],
    };
    pub const WIND_CHARGED: Self = Self {
        name: "wind_charged",
        id: 42u8,
        effects: &[Effect {
            effect_type: &StatusEffect::WIND_CHARGED,
            duration: 3600i32,
            amplifier: 0u8,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        }],
    };
    pub fn from_name(name: &str) -> Option<&'static Self> {
        match name {
            "awkward" => Some(&Self::AWKWARD),
            "fire_resistance" => Some(&Self::FIRE_RESISTANCE),
            "harming" => Some(&Self::HARMING),
            "healing" => Some(&Self::HEALING),
            "infested" => Some(&Self::INFESTED),
            "invisibility" => Some(&Self::INVISIBILITY),
            "leaping" => Some(&Self::LEAPING),
            "long_fire_resistance" => Some(&Self::LONG_FIRE_RESISTANCE),
            "long_invisibility" => Some(&Self::LONG_INVISIBILITY),
            "long_leaping" => Some(&Self::LONG_LEAPING),
            "long_night_vision" => Some(&Self::LONG_NIGHT_VISION),
            "long_poison" => Some(&Self::LONG_POISON),
            "long_regeneration" => Some(&Self::LONG_REGENERATION),
            "long_slow_falling" => Some(&Self::LONG_SLOW_FALLING),
            "long_slowness" => Some(&Self::LONG_SLOWNESS),
            "long_strength" => Some(&Self::LONG_STRENGTH),
            "long_swiftness" => Some(&Self::LONG_SWIFTNESS),
            "long_turtle_master" => Some(&Self::LONG_TURTLE_MASTER),
            "long_water_breathing" => Some(&Self::LONG_WATER_BREATHING),
            "long_weakness" => Some(&Self::LONG_WEAKNESS),
            "luck" => Some(&Self::LUCK),
            "mundane" => Some(&Self::MUNDANE),
            "night_vision" => Some(&Self::NIGHT_VISION),
            "oozing" => Some(&Self::OOZING),
            "poison" => Some(&Self::POISON),
            "regeneration" => Some(&Self::REGENERATION),
            "slow_falling" => Some(&Self::SLOW_FALLING),
            "slowness" => Some(&Self::SLOWNESS),
            "strength" => Some(&Self::STRENGTH),
            "strong_harming" => Some(&Self::STRONG_HARMING),
            "strong_healing" => Some(&Self::STRONG_HEALING),
            "strong_leaping" => Some(&Self::STRONG_LEAPING),
            "strong_poison" => Some(&Self::STRONG_POISON),
            "strong_regeneration" => Some(&Self::STRONG_REGENERATION),
            "strong_slowness" => Some(&Self::STRONG_SLOWNESS),
            "strong_strength" => Some(&Self::STRONG_STRENGTH),
            "strong_swiftness" => Some(&Self::STRONG_SWIFTNESS),
            "strong_turtle_master" => Some(&Self::STRONG_TURTLE_MASTER),
            "swiftness" => Some(&Self::SWIFTNESS),
            "thick" => Some(&Self::THICK),
            "turtle_master" => Some(&Self::TURTLE_MASTER),
            "water" => Some(&Self::WATER),
            "water_breathing" => Some(&Self::WATER_BREATHING),
            "weakness" => Some(&Self::WEAKNESS),
            "weaving" => Some(&Self::WEAVING),
            "wind_charged" => Some(&Self::WIND_CHARGED),
            _ => None,
        }
    }
}
