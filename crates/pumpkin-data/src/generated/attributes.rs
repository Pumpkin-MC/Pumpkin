/* This file is generated. Do not edit manually. */
use std::hash::Hash;
#[derive(Clone, Debug)]
pub struct Attributes {
    pub id: u8,
    pub default_value: f64,
    pub name: &'static str,
}
impl PartialEq for Attributes {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for Attributes {}
impl Hash for Attributes {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
impl Attributes {
    pub const AIR_DRAG_MODIFIER: Self = Self {
        id: 0,
        default_value: 1f64,
        name: "minecraft:air_drag_modifier",
    };
    pub const ARMOR: Self = Self {
        id: 1,
        default_value: 0f64,
        name: "minecraft:armor",
    };
    pub const ARMOR_TOUGHNESS: Self = Self {
        id: 2,
        default_value: 0f64,
        name: "minecraft:armor_toughness",
    };
    pub const ATTACK_DAMAGE: Self = Self {
        id: 3,
        default_value: 2f64,
        name: "minecraft:attack_damage",
    };
    pub const ATTACK_KNOCKBACK: Self = Self {
        id: 4,
        default_value: 0f64,
        name: "minecraft:attack_knockback",
    };
    pub const ATTACK_SPEED: Self = Self {
        id: 5,
        default_value: 4f64,
        name: "minecraft:attack_speed",
    };
    pub const BELOW_NAME_DISTANCE: Self = Self {
        id: 6,
        default_value: 10f64,
        name: "minecraft:below_name_distance",
    };
    pub const BLOCK_BREAK_SPEED: Self = Self {
        id: 7,
        default_value: 1f64,
        name: "minecraft:block_break_speed",
    };
    pub const BLOCK_INTERACTION_RANGE: Self = Self {
        id: 8,
        default_value: 4.5f64,
        name: "minecraft:block_interaction_range",
    };
    pub const BOUNCINESS: Self = Self {
        id: 9,
        default_value: 0f64,
        name: "minecraft:bounciness",
    };
    pub const BURNING_TIME: Self = Self {
        id: 10,
        default_value: 1f64,
        name: "minecraft:burning_time",
    };
    pub const CAMERA_DISTANCE: Self = Self {
        id: 11,
        default_value: 4f64,
        name: "minecraft:camera_distance",
    };
    pub const EXPLOSION_KNOCKBACK_RESISTANCE: Self = Self {
        id: 12,
        default_value: 0f64,
        name: "minecraft:explosion_knockback_resistance",
    };
    pub const ENTITY_INTERACTION_RANGE: Self = Self {
        id: 13,
        default_value: 3f64,
        name: "minecraft:entity_interaction_range",
    };
    pub const FALL_DAMAGE_MULTIPLIER: Self = Self {
        id: 14,
        default_value: 1f64,
        name: "minecraft:fall_damage_multiplier",
    };
    pub const FLYING_SPEED: Self = Self {
        id: 15,
        default_value: 0.4f64,
        name: "minecraft:flying_speed",
    };
    pub const FOLLOW_RANGE: Self = Self {
        id: 16,
        default_value: 32f64,
        name: "minecraft:follow_range",
    };
    pub const FRICTION_MODIFIER: Self = Self {
        id: 17,
        default_value: 1f64,
        name: "minecraft:friction_modifier",
    };
    pub const GRAVITY: Self = Self {
        id: 18,
        default_value: 0.08f64,
        name: "minecraft:gravity",
    };
    pub const JUMP_STRENGTH: Self = Self {
        id: 19,
        default_value: 0.41999998688697815f64,
        name: "minecraft:jump_strength",
    };
    pub const KNOCKBACK_RESISTANCE: Self = Self {
        id: 20,
        default_value: 0f64,
        name: "minecraft:knockback_resistance",
    };
    pub const LUCK: Self = Self {
        id: 21,
        default_value: 0f64,
        name: "minecraft:luck",
    };
    pub const MAX_ABSORPTION: Self = Self {
        id: 22,
        default_value: 0f64,
        name: "minecraft:max_absorption",
    };
    pub const MAX_HEALTH: Self = Self {
        id: 23,
        default_value: 20f64,
        name: "minecraft:max_health",
    };
    pub const MINING_EFFICIENCY: Self = Self {
        id: 24,
        default_value: 0f64,
        name: "minecraft:mining_efficiency",
    };
    pub const MOVEMENT_EFFICIENCY: Self = Self {
        id: 25,
        default_value: 0f64,
        name: "minecraft:movement_efficiency",
    };
    pub const MOVEMENT_SPEED: Self = Self {
        id: 26,
        default_value: 0.7f64,
        name: "minecraft:movement_speed",
    };
    pub const NAME_TAG_DISTANCE: Self = Self {
        id: 27,
        default_value: 64f64,
        name: "minecraft:name_tag_distance",
    };
    pub const OXYGEN_BONUS: Self = Self {
        id: 28,
        default_value: 0f64,
        name: "minecraft:oxygen_bonus",
    };
    pub const SAFE_FALL_DISTANCE: Self = Self {
        id: 29,
        default_value: 3f64,
        name: "minecraft:safe_fall_distance",
    };
    pub const SCALE: Self = Self {
        id: 30,
        default_value: 1f64,
        name: "minecraft:scale",
    };
    pub const SNEAKING_SPEED: Self = Self {
        id: 31,
        default_value: 0.3f64,
        name: "minecraft:sneaking_speed",
    };
    pub const SPAWN_REINFORCEMENTS: Self = Self {
        id: 32,
        default_value: 0f64,
        name: "minecraft:spawn_reinforcements",
    };
    pub const STEP_HEIGHT: Self = Self {
        id: 33,
        default_value: 0.6f64,
        name: "minecraft:step_height",
    };
    pub const SUBMERGED_MINING_SPEED: Self = Self {
        id: 34,
        default_value: 0.2f64,
        name: "minecraft:submerged_mining_speed",
    };
    pub const SWEEPING_DAMAGE_RATIO: Self = Self {
        id: 35,
        default_value: 0f64,
        name: "minecraft:sweeping_damage_ratio",
    };
    pub const TEMPT_RANGE: Self = Self {
        id: 36,
        default_value: 10f64,
        name: "minecraft:tempt_range",
    };
    pub const WATER_MOVEMENT_EFFICIENCY: Self = Self {
        id: 37,
        default_value: 0f64,
        name: "minecraft:water_movement_efficiency",
    };
    pub const WAYPOINT_TRANSMIT_RANGE: Self = Self {
        id: 38,
        default_value: 0f64,
        name: "minecraft:waypoint_transmit_range",
    };
    pub const WAYPOINT_RECEIVE_RANGE: Self = Self {
        id: 39,
        default_value: 0f64,
        name: "minecraft:waypoint_receive_range",
    };
    pub const ALL: &'static [Self] = &[
        Self::AIR_DRAG_MODIFIER,
        Self::ARMOR,
        Self::ARMOR_TOUGHNESS,
        Self::ATTACK_DAMAGE,
        Self::ATTACK_KNOCKBACK,
        Self::ATTACK_SPEED,
        Self::BELOW_NAME_DISTANCE,
        Self::BLOCK_BREAK_SPEED,
        Self::BLOCK_INTERACTION_RANGE,
        Self::BOUNCINESS,
        Self::BURNING_TIME,
        Self::CAMERA_DISTANCE,
        Self::EXPLOSION_KNOCKBACK_RESISTANCE,
        Self::ENTITY_INTERACTION_RANGE,
        Self::FALL_DAMAGE_MULTIPLIER,
        Self::FLYING_SPEED,
        Self::FOLLOW_RANGE,
        Self::FRICTION_MODIFIER,
        Self::GRAVITY,
        Self::JUMP_STRENGTH,
        Self::KNOCKBACK_RESISTANCE,
        Self::LUCK,
        Self::MAX_ABSORPTION,
        Self::MAX_HEALTH,
        Self::MINING_EFFICIENCY,
        Self::MOVEMENT_EFFICIENCY,
        Self::MOVEMENT_SPEED,
        Self::NAME_TAG_DISTANCE,
        Self::OXYGEN_BONUS,
        Self::SAFE_FALL_DISTANCE,
        Self::SCALE,
        Self::SNEAKING_SPEED,
        Self::SPAWN_REINFORCEMENTS,
        Self::STEP_HEIGHT,
        Self::SUBMERGED_MINING_SPEED,
        Self::SWEEPING_DAMAGE_RATIO,
        Self::TEMPT_RANGE,
        Self::WATER_MOVEMENT_EFFICIENCY,
        Self::WAYPOINT_TRANSMIT_RANGE,
        Self::WAYPOINT_RECEIVE_RANGE,
    ];
}
