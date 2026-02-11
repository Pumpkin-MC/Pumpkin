/* This file is generated. Do not edit manually. */
use std::hash::Hash;
#[derive(Clone, Debug)]
pub struct Attributes {
    pub id: u8,
    pub default_value: f64,
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
    pub const ARMOR: Self = Self {
        id: 0,
        default_value: 0f64,
    };
    pub const ARMOR_TOUGHNESS: Self = Self {
        id: 1,
        default_value: 0f64,
    };
    pub const ATTACK_DAMAGE: Self = Self {
        id: 2,
        default_value: 2f64,
    };
    pub const ATTACK_KNOCKBACK: Self = Self {
        id: 3,
        default_value: 0f64,
    };
    pub const ATTACK_SPEED: Self = Self {
        id: 4,
        default_value: 4f64,
    };
    pub const BLOCK_BREAK_SPEED: Self = Self {
        id: 5,
        default_value: 1f64,
    };
    pub const BLOCK_INTERACTION_RANGE: Self = Self {
        id: 6,
        default_value: 4.5f64,
    };
    pub const BURNING_TIME: Self = Self {
        id: 7,
        default_value: 1f64,
    };
    pub const CAMERA_DISTANCE: Self = Self {
        id: 8,
        default_value: 4f64,
    };
    pub const EXPLOSION_KNOCKBACK_RESISTANCE: Self = Self {
        id: 9,
        default_value: 0f64,
    };
    pub const ENTITY_INTERACTION_RANGE: Self = Self {
        id: 10,
        default_value: 3f64,
    };
    pub const FALL_DAMAGE_MULTIPLIER: Self = Self {
        id: 11,
        default_value: 1f64,
    };
    pub const FLYING_SPEED: Self = Self {
        id: 12,
        default_value: 0.4f64,
    };
    pub const FOLLOW_RANGE: Self = Self {
        id: 13,
        default_value: 32f64,
    };
    pub const GRAVITY: Self = Self {
        id: 14,
        default_value: 0.08f64,
    };
    pub const JUMP_STRENGTH: Self = Self {
        id: 15,
        default_value: 0.41999998688697815f64,
    };
    pub const KNOCKBACK_RESISTANCE: Self = Self {
        id: 16,
        default_value: 0f64,
    };
    pub const LUCK: Self = Self {
        id: 17,
        default_value: 0f64,
    };
    pub const MAX_ABSORPTION: Self = Self {
        id: 18,
        default_value: 0f64,
    };
    pub const MAX_HEALTH: Self = Self {
        id: 19,
        default_value: 20f64,
    };
    pub const MINING_EFFICIENCY: Self = Self {
        id: 20,
        default_value: 0f64,
    };
    pub const MOVEMENT_EFFICIENCY: Self = Self {
        id: 21,
        default_value: 0f64,
    };
    pub const MOVEMENT_SPEED: Self = Self {
        id: 22,
        default_value: 0.7f64,
    };
    pub const OXYGEN_BONUS: Self = Self {
        id: 23,
        default_value: 0f64,
    };
    pub const SAFE_FALL_DISTANCE: Self = Self {
        id: 24,
        default_value: 3f64,
    };
    pub const SCALE: Self = Self {
        id: 25,
        default_value: 1f64,
    };
    pub const SNEAKING_SPEED: Self = Self {
        id: 26,
        default_value: 0.3f64,
    };
    pub const SPAWN_REINFORCEMENTS: Self = Self {
        id: 27,
        default_value: 0f64,
    };
    pub const STEP_HEIGHT: Self = Self {
        id: 28,
        default_value: 0.6f64,
    };
    pub const SUBMERGED_MINING_SPEED: Self = Self {
        id: 29,
        default_value: 0.2f64,
    };
    pub const SWEEPING_DAMAGE_RATIO: Self = Self {
        id: 30,
        default_value: 0f64,
    };
    pub const TEMPT_RANGE: Self = Self {
        id: 31,
        default_value: 10f64,
    };
    pub const WATER_MOVEMENT_EFFICIENCY: Self = Self {
        id: 32,
        default_value: 0f64,
    };
    pub const WAYPOINT_TRANSMIT_RANGE: Self = Self {
        id: 33,
        default_value: 0f64,
    };
    pub const WAYPOINT_RECEIVE_RANGE: Self = Self {
        id: 34,
        default_value: 0f64,
    };
}
