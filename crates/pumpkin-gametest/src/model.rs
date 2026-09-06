use pumpkin_data::block_rotation::Rotation;
use serde::Deserialize;
use serde_json::Value;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
pub enum TestType {
    #[serde(rename = "minecraft:block_based")]
    BlockBased,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq)]
pub enum GameTestRotation {
    #[default]
    #[serde(rename = "none")]
    None,
    #[serde(rename = "clockwise_90")]
    Clockwise90,
    #[serde(rename = "180")]
    Clockwise180,
    #[serde(rename = "counterclockwise_90")]
    Counterclockwise90,
}

impl GameTestRotation {
    #[must_use]
    pub const fn serialized_name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Clockwise90 => "clockwise_90",
            Self::Clockwise180 => "180",
            Self::Counterclockwise90 => "counterclockwise_90",
        }
    }

    #[must_use]
    pub const fn as_block_rotation(self) -> Rotation {
        match self {
            Self::None => Rotation::None,
            Self::Clockwise90 => Rotation::Clockwise90,
            Self::Clockwise180 => Rotation::Rotate180,
            Self::Counterclockwise90 => Rotation::CounterClockwise90,
        }
    }

    /// Combines the datapack's base rotation with an additional controller rotation.
    /// This mirrors vanilla `Rotation::getRotated`/`GameTestInfo` extra rotation.
    #[must_use]
    pub const fn then(self, extra: Self) -> Self {
        match self.as_block_rotation().then(extra.as_block_rotation()) {
            Rotation::None => Self::None,
            Rotation::Clockwise90 => Self::Clockwise90,
            Rotation::Rotate180 => Self::Clockwise180,
            Rotation::CounterClockwise90 => Self::Counterclockwise90,
        }
    }

    #[must_use]
    pub const fn from_steps(steps: i32) -> Self {
        match steps.rem_euclid(4) {
            0 => Self::None,
            1 => Self::Clockwise90,
            2 => Self::Clockwise180,
            _ => Self::Counterclockwise90,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct GameTestDefinition {
    #[serde(rename = "type")]
    pub instance_type: TestType,
    pub environment: Value,
    pub structure: String,
    pub max_ticks: i32,
    #[serde(default)]
    pub setup_ticks: i32,
    #[serde(default = "default_true")]
    pub required: bool,
    #[serde(default)]
    pub rotation: GameTestRotation,
    #[serde(default)]
    pub manual_only: bool,
    #[serde(default = "default_one")]
    pub max_attempts: i32,
    #[serde(default = "default_one")]
    pub required_successes: i32,
    #[serde(default)]
    pub sky_access: bool,
    #[serde(default)]
    pub padding: i32,
}

impl GameTestDefinition {
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.max_ticks > 0
            && self.setup_ticks >= 0
            && self.max_attempts > 0
            && self.required_successes > 0
            && (0..=128).contains(&self.padding)
    }
}

const fn default_true() -> bool {
    true
}

const fn default_one() -> i32 {
    1
}
