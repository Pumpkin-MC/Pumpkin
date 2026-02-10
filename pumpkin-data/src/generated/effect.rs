/* This file is generated. Do not edit manually. */
use crate::attributes::Attributes;
use crate::data_component_impl::Operation;
use std::hash::{Hash, Hasher};
pub struct StatusEffect {
    pub minecraft_name: &'static str,
    pub id: u8,
    pub category: MobEffectCategory,
    pub color: i32,
    pub translation_key: &'static str,
    pub attribute_modifiers: &'static [Modifiers],
}
impl PartialEq for StatusEffect {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for StatusEffect {}
impl Hash for StatusEffect {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
#[derive(Debug, Clone, Hash)]
pub enum MobEffectCategory {
    Beneficial,
    Harmful,
    Neutral,
}
#[derive(Debug)]
pub struct Modifiers {
    pub attribute: &'static Attributes,
    pub id: &'static str,
    pub base_value: f64,
    pub operation: Operation,
}
impl StatusEffect {
    pub const ABSORPTION: Self = Self {
        minecraft_name: "minecraft:absorption",
        id: 21u8,
        category: MobEffectCategory::Beneficial,
        color: 2445989i32,
        translation_key: "effect.minecraft.absorption",
        attribute_modifiers: &[Modifiers {
            attribute: &Attributes::MAX_ABSORPTION,
            id: "minecraft:effect.absorption",
            base_value: 4f64,
            operation: Operation::AddValue,
        }],
    };
    pub const BAD_OMEN: Self = Self {
        minecraft_name: "minecraft:bad_omen",
        id: 30u8,
        category: MobEffectCategory::Neutral,
        color: 745784i32,
        translation_key: "effect.minecraft.bad_omen",
        attribute_modifiers: &[],
    };
    pub const BLINDNESS: Self = Self {
        minecraft_name: "minecraft:blindness",
        id: 14u8,
        category: MobEffectCategory::Harmful,
        color: 2039587i32,
        translation_key: "effect.minecraft.blindness",
        attribute_modifiers: &[],
    };
    pub const CONDUIT_POWER: Self = Self {
        minecraft_name: "minecraft:conduit_power",
        id: 28u8,
        category: MobEffectCategory::Beneficial,
        color: 1950417i32,
        translation_key: "effect.minecraft.conduit_power",
        attribute_modifiers: &[],
    };
    pub const DARKNESS: Self = Self {
        minecraft_name: "minecraft:darkness",
        id: 32u8,
        category: MobEffectCategory::Harmful,
        color: 2696993i32,
        translation_key: "effect.minecraft.darkness",
        attribute_modifiers: &[],
    };
    pub const DOLPHINS_GRACE: Self = Self {
        minecraft_name: "minecraft:dolphins_grace",
        id: 29u8,
        category: MobEffectCategory::Beneficial,
        color: 8954814i32,
        translation_key: "effect.minecraft.dolphins_grace",
        attribute_modifiers: &[],
    };
    pub const FIRE_RESISTANCE: Self = Self {
        minecraft_name: "minecraft:fire_resistance",
        id: 11u8,
        category: MobEffectCategory::Beneficial,
        color: 16750848i32,
        translation_key: "effect.minecraft.fire_resistance",
        attribute_modifiers: &[],
    };
    pub const GLOWING: Self = Self {
        minecraft_name: "minecraft:glowing",
        id: 23u8,
        category: MobEffectCategory::Neutral,
        color: 9740385i32,
        translation_key: "effect.minecraft.glowing",
        attribute_modifiers: &[],
    };
    pub const HASTE: Self = Self {
        minecraft_name: "minecraft:haste",
        id: 2u8,
        category: MobEffectCategory::Beneficial,
        color: 14270531i32,
        translation_key: "effect.minecraft.haste",
        attribute_modifiers: &[Modifiers {
            attribute: &Attributes::ATTACK_SPEED,
            id: "minecraft:effect.haste",
            base_value: 0.10000000149011612f64,
            operation: Operation::AddMultipliedTotal,
        }],
    };
    pub const HEALTH_BOOST: Self = Self {
        minecraft_name: "minecraft:health_boost",
        id: 20u8,
        category: MobEffectCategory::Beneficial,
        color: 16284963i32,
        translation_key: "effect.minecraft.health_boost",
        attribute_modifiers: &[Modifiers {
            attribute: &Attributes::MAX_HEALTH,
            id: "minecraft:effect.health_boost",
            base_value: 4f64,
            operation: Operation::AddValue,
        }],
    };
    pub const HERO_OF_THE_VILLAGE: Self = Self {
        minecraft_name: "minecraft:hero_of_the_village",
        id: 31u8,
        category: MobEffectCategory::Beneficial,
        color: 4521796i32,
        translation_key: "effect.minecraft.hero_of_the_village",
        attribute_modifiers: &[],
    };
    pub const HUNGER: Self = Self {
        minecraft_name: "minecraft:hunger",
        id: 16u8,
        category: MobEffectCategory::Harmful,
        color: 5797459i32,
        translation_key: "effect.minecraft.hunger",
        attribute_modifiers: &[],
    };
    pub const INFESTED: Self = Self {
        minecraft_name: "minecraft:infested",
        id: 38u8,
        category: MobEffectCategory::Harmful,
        color: 9214860i32,
        translation_key: "effect.minecraft.infested",
        attribute_modifiers: &[],
    };
    pub const INSTANT_DAMAGE: Self = Self {
        minecraft_name: "minecraft:instant_damage",
        id: 6u8,
        category: MobEffectCategory::Harmful,
        color: 11101546i32,
        translation_key: "effect.minecraft.instant_damage",
        attribute_modifiers: &[],
    };
    pub const INSTANT_HEALTH: Self = Self {
        minecraft_name: "minecraft:instant_health",
        id: 5u8,
        category: MobEffectCategory::Beneficial,
        color: 16262179i32,
        translation_key: "effect.minecraft.instant_health",
        attribute_modifiers: &[],
    };
    pub const INVISIBILITY: Self = Self {
        minecraft_name: "minecraft:invisibility",
        id: 13u8,
        category: MobEffectCategory::Beneficial,
        color: 16185078i32,
        translation_key: "effect.minecraft.invisibility",
        attribute_modifiers: &[Modifiers {
            attribute: &Attributes::WAYPOINT_TRANSMIT_RANGE,
            id: "minecraft:effect.waypoint_transmit_range_hide",
            base_value: -1f64,
            operation: Operation::AddMultipliedTotal,
        }],
    };
    pub const JUMP_BOOST: Self = Self {
        minecraft_name: "minecraft:jump_boost",
        id: 7u8,
        category: MobEffectCategory::Beneficial,
        color: 16646020i32,
        translation_key: "effect.minecraft.jump_boost",
        attribute_modifiers: &[Modifiers {
            attribute: &Attributes::SAFE_FALL_DISTANCE,
            id: "minecraft:effect.jump_boost",
            base_value: 1f64,
            operation: Operation::AddValue,
        }],
    };
    pub const LEVITATION: Self = Self {
        minecraft_name: "minecraft:levitation",
        id: 24u8,
        category: MobEffectCategory::Harmful,
        color: 13565951i32,
        translation_key: "effect.minecraft.levitation",
        attribute_modifiers: &[],
    };
    pub const LUCK: Self = Self {
        minecraft_name: "minecraft:luck",
        id: 25u8,
        category: MobEffectCategory::Beneficial,
        color: 5882118i32,
        translation_key: "effect.minecraft.luck",
        attribute_modifiers: &[Modifiers {
            attribute: &Attributes::LUCK,
            id: "minecraft:effect.luck",
            base_value: 1f64,
            operation: Operation::AddValue,
        }],
    };
    pub const MINING_FATIGUE: Self = Self {
        minecraft_name: "minecraft:mining_fatigue",
        id: 3u8,
        category: MobEffectCategory::Harmful,
        color: 4866583i32,
        translation_key: "effect.minecraft.mining_fatigue",
        attribute_modifiers: &[Modifiers {
            attribute: &Attributes::ATTACK_SPEED,
            id: "minecraft:effect.mining_fatigue",
            base_value: -0.10000000149011612f64,
            operation: Operation::AddMultipliedTotal,
        }],
    };
    pub const NAUSEA: Self = Self {
        minecraft_name: "minecraft:nausea",
        id: 8u8,
        category: MobEffectCategory::Harmful,
        color: 5578058i32,
        translation_key: "effect.minecraft.nausea",
        attribute_modifiers: &[],
    };
    pub const NIGHT_VISION: Self = Self {
        minecraft_name: "minecraft:night_vision",
        id: 15u8,
        category: MobEffectCategory::Beneficial,
        color: 12779366i32,
        translation_key: "effect.minecraft.night_vision",
        attribute_modifiers: &[],
    };
    pub const OOZING: Self = Self {
        minecraft_name: "minecraft:oozing",
        id: 37u8,
        category: MobEffectCategory::Harmful,
        color: 10092451i32,
        translation_key: "effect.minecraft.oozing",
        attribute_modifiers: &[],
    };
    pub const POISON: Self = Self {
        minecraft_name: "minecraft:poison",
        id: 18u8,
        category: MobEffectCategory::Harmful,
        color: 8889187i32,
        translation_key: "effect.minecraft.poison",
        attribute_modifiers: &[],
    };
    pub const RAID_OMEN: Self = Self {
        minecraft_name: "minecraft:raid_omen",
        id: 34u8,
        category: MobEffectCategory::Neutral,
        color: 14565464i32,
        translation_key: "effect.minecraft.raid_omen",
        attribute_modifiers: &[],
    };
    pub const REGENERATION: Self = Self {
        minecraft_name: "minecraft:regeneration",
        id: 9u8,
        category: MobEffectCategory::Beneficial,
        color: 13458603i32,
        translation_key: "effect.minecraft.regeneration",
        attribute_modifiers: &[],
    };
    pub const RESISTANCE: Self = Self {
        minecraft_name: "minecraft:resistance",
        id: 10u8,
        category: MobEffectCategory::Beneficial,
        color: 9520880i32,
        translation_key: "effect.minecraft.resistance",
        attribute_modifiers: &[],
    };
    pub const SATURATION: Self = Self {
        minecraft_name: "minecraft:saturation",
        id: 22u8,
        category: MobEffectCategory::Beneficial,
        color: 16262179i32,
        translation_key: "effect.minecraft.saturation",
        attribute_modifiers: &[],
    };
    pub const SLOW_FALLING: Self = Self {
        minecraft_name: "minecraft:slow_falling",
        id: 27u8,
        category: MobEffectCategory::Beneficial,
        color: 15978425i32,
        translation_key: "effect.minecraft.slow_falling",
        attribute_modifiers: &[],
    };
    pub const SLOWNESS: Self = Self {
        minecraft_name: "minecraft:slowness",
        id: 1u8,
        category: MobEffectCategory::Harmful,
        color: 9154528i32,
        translation_key: "effect.minecraft.slowness",
        attribute_modifiers: &[Modifiers {
            attribute: &Attributes::MOVEMENT_SPEED,
            id: "minecraft:effect.slowness",
            base_value: -0.15000000596046448f64,
            operation: Operation::AddMultipliedTotal,
        }],
    };
    pub const SPEED: Self = Self {
        minecraft_name: "minecraft:speed",
        id: 0u8,
        category: MobEffectCategory::Beneficial,
        color: 3402751i32,
        translation_key: "effect.minecraft.speed",
        attribute_modifiers: &[Modifiers {
            attribute: &Attributes::MOVEMENT_SPEED,
            id: "minecraft:effect.speed",
            base_value: 0.20000000298023224f64,
            operation: Operation::AddMultipliedTotal,
        }],
    };
    pub const STRENGTH: Self = Self {
        minecraft_name: "minecraft:strength",
        id: 4u8,
        category: MobEffectCategory::Beneficial,
        color: 16762624i32,
        translation_key: "effect.minecraft.strength",
        attribute_modifiers: &[Modifiers {
            attribute: &Attributes::ATTACK_DAMAGE,
            id: "minecraft:effect.strength",
            base_value: 3f64,
            operation: Operation::AddValue,
        }],
    };
    pub const TRIAL_OMEN: Self = Self {
        minecraft_name: "minecraft:trial_omen",
        id: 33u8,
        category: MobEffectCategory::Neutral,
        color: 1484454i32,
        translation_key: "effect.minecraft.trial_omen",
        attribute_modifiers: &[],
    };
    pub const UNLUCK: Self = Self {
        minecraft_name: "minecraft:unluck",
        id: 26u8,
        category: MobEffectCategory::Harmful,
        color: 12624973i32,
        translation_key: "effect.minecraft.unluck",
        attribute_modifiers: &[Modifiers {
            attribute: &Attributes::LUCK,
            id: "minecraft:effect.unluck",
            base_value: -1f64,
            operation: Operation::AddValue,
        }],
    };
    pub const WATER_BREATHING: Self = Self {
        minecraft_name: "minecraft:water_breathing",
        id: 12u8,
        category: MobEffectCategory::Beneficial,
        color: 10017472i32,
        translation_key: "effect.minecraft.water_breathing",
        attribute_modifiers: &[],
    };
    pub const WEAKNESS: Self = Self {
        minecraft_name: "minecraft:weakness",
        id: 17u8,
        category: MobEffectCategory::Harmful,
        color: 4738376i32,
        translation_key: "effect.minecraft.weakness",
        attribute_modifiers: &[Modifiers {
            attribute: &Attributes::ATTACK_DAMAGE,
            id: "minecraft:effect.weakness",
            base_value: -4f64,
            operation: Operation::AddValue,
        }],
    };
    pub const WEAVING: Self = Self {
        minecraft_name: "minecraft:weaving",
        id: 36u8,
        category: MobEffectCategory::Harmful,
        color: 7891290i32,
        translation_key: "effect.minecraft.weaving",
        attribute_modifiers: &[],
    };
    pub const WIND_CHARGED: Self = Self {
        minecraft_name: "minecraft:wind_charged",
        id: 35u8,
        category: MobEffectCategory::Harmful,
        color: 12438015i32,
        translation_key: "effect.minecraft.wind_charged",
        attribute_modifiers: &[],
    };
    pub const WITHER: Self = Self {
        minecraft_name: "minecraft:wither",
        id: 19u8,
        category: MobEffectCategory::Harmful,
        color: 7561558i32,
        translation_key: "effect.minecraft.wither",
        attribute_modifiers: &[],
    };
    pub fn from_name(name: &str) -> Option<&'static Self> {
        match name {
            "absorption" => Some(&Self::ABSORPTION),
            "bad_omen" => Some(&Self::BAD_OMEN),
            "blindness" => Some(&Self::BLINDNESS),
            "conduit_power" => Some(&Self::CONDUIT_POWER),
            "darkness" => Some(&Self::DARKNESS),
            "dolphins_grace" => Some(&Self::DOLPHINS_GRACE),
            "fire_resistance" => Some(&Self::FIRE_RESISTANCE),
            "glowing" => Some(&Self::GLOWING),
            "haste" => Some(&Self::HASTE),
            "health_boost" => Some(&Self::HEALTH_BOOST),
            "hero_of_the_village" => Some(&Self::HERO_OF_THE_VILLAGE),
            "hunger" => Some(&Self::HUNGER),
            "infested" => Some(&Self::INFESTED),
            "instant_damage" => Some(&Self::INSTANT_DAMAGE),
            "instant_health" => Some(&Self::INSTANT_HEALTH),
            "invisibility" => Some(&Self::INVISIBILITY),
            "jump_boost" => Some(&Self::JUMP_BOOST),
            "levitation" => Some(&Self::LEVITATION),
            "luck" => Some(&Self::LUCK),
            "mining_fatigue" => Some(&Self::MINING_FATIGUE),
            "nausea" => Some(&Self::NAUSEA),
            "night_vision" => Some(&Self::NIGHT_VISION),
            "oozing" => Some(&Self::OOZING),
            "poison" => Some(&Self::POISON),
            "raid_omen" => Some(&Self::RAID_OMEN),
            "regeneration" => Some(&Self::REGENERATION),
            "resistance" => Some(&Self::RESISTANCE),
            "saturation" => Some(&Self::SATURATION),
            "slow_falling" => Some(&Self::SLOW_FALLING),
            "slowness" => Some(&Self::SLOWNESS),
            "speed" => Some(&Self::SPEED),
            "strength" => Some(&Self::STRENGTH),
            "trial_omen" => Some(&Self::TRIAL_OMEN),
            "unluck" => Some(&Self::UNLUCK),
            "water_breathing" => Some(&Self::WATER_BREATHING),
            "weakness" => Some(&Self::WEAKNESS),
            "weaving" => Some(&Self::WEAVING),
            "wind_charged" => Some(&Self::WIND_CHARGED),
            "wither" => Some(&Self::WITHER),
            _ => None,
        }
    }
    pub fn from_minecraft_name(name: &str) -> Option<&'static Self> {
        match name {
            "absorption" => Some(&Self::ABSORPTION),
            "bad_omen" => Some(&Self::BAD_OMEN),
            "blindness" => Some(&Self::BLINDNESS),
            "conduit_power" => Some(&Self::CONDUIT_POWER),
            "darkness" => Some(&Self::DARKNESS),
            "dolphins_grace" => Some(&Self::DOLPHINS_GRACE),
            "fire_resistance" => Some(&Self::FIRE_RESISTANCE),
            "glowing" => Some(&Self::GLOWING),
            "haste" => Some(&Self::HASTE),
            "health_boost" => Some(&Self::HEALTH_BOOST),
            "hero_of_the_village" => Some(&Self::HERO_OF_THE_VILLAGE),
            "hunger" => Some(&Self::HUNGER),
            "infested" => Some(&Self::INFESTED),
            "instant_damage" => Some(&Self::INSTANT_DAMAGE),
            "instant_health" => Some(&Self::INSTANT_HEALTH),
            "invisibility" => Some(&Self::INVISIBILITY),
            "jump_boost" => Some(&Self::JUMP_BOOST),
            "levitation" => Some(&Self::LEVITATION),
            "luck" => Some(&Self::LUCK),
            "mining_fatigue" => Some(&Self::MINING_FATIGUE),
            "nausea" => Some(&Self::NAUSEA),
            "night_vision" => Some(&Self::NIGHT_VISION),
            "oozing" => Some(&Self::OOZING),
            "poison" => Some(&Self::POISON),
            "raid_omen" => Some(&Self::RAID_OMEN),
            "regeneration" => Some(&Self::REGENERATION),
            "resistance" => Some(&Self::RESISTANCE),
            "saturation" => Some(&Self::SATURATION),
            "slow_falling" => Some(&Self::SLOW_FALLING),
            "slowness" => Some(&Self::SLOWNESS),
            "speed" => Some(&Self::SPEED),
            "strength" => Some(&Self::STRENGTH),
            "trial_omen" => Some(&Self::TRIAL_OMEN),
            "unluck" => Some(&Self::UNLUCK),
            "water_breathing" => Some(&Self::WATER_BREATHING),
            "weakness" => Some(&Self::WEAKNESS),
            "weaving" => Some(&Self::WEAVING),
            "wind_charged" => Some(&Self::WIND_CHARGED),
            "wither" => Some(&Self::WITHER),
            _ => None,
        }
    }
}
