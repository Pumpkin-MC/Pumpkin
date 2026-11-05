use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// Error returned when parsing a [`Difficulty`] from a string fails.
pub struct ParseDifficultyError;

/// Represents the difficulty level of the game.
///
/// Each numeric value corresponds to a specific difficulty:
/// - `Peaceful` (0): No hostile mobs spawn, health regenerates naturally.
/// - `Easy` (1): Standard gameplay, hostile mobs spawn with reduced damage.
/// - `Normal` (2): Standard difficulty with full damage from mobs.
/// - `Hard` (3): Hostile mobs deal extra damage and health regeneration is limited.
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Debug)]
pub enum Difficulty {
    /// No hostile mobs; natural health regeneration.
    Peaceful = 0,
    /// Easy difficulty; hostile mobs deal reduced damage.
    Easy = 1,
    /// Normal difficulty; standard mob damage and health.
    Normal = 2,
    /// Hard difficulty; increased mob damage and limited health regen.
    Hard = 3,
}

impl Difficulty {
    /// Gets the lowercase name of this difficulty.
    /// For example, [`Difficulty::Peaceful`] will yield `"peaceful"`.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Peaceful => "peaceful",
            Self::Easy => "easy",
            Self::Normal => "normal",
            Self::Hard => "hard",
        }
    }

    /// Gets the translation key of this difficulty.
    /// For example, [`Difficulty::Peaceful`] will yield `"options.difficulty.peaceful"`.
    #[must_use]
    pub const fn translation_key(self) -> &'static str {
        match self {
            Self::Peaceful => "options.difficulty.peaceful",
            Self::Easy => "options.difficulty.easy",
            Self::Normal => "options.difficulty.normal",
            Self::Hard => "options.difficulty.hard",
        }
    }
}

impl FromStr for Difficulty {
    type Err = ParseDifficultyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "peaceful" => Ok(Self::Peaceful),
            "easy" => Ok(Self::Easy),
            "normal" => Ok(Self::Normal),
            "hard" => Ok(Self::Hard),
            _ => Err(ParseDifficultyError),
        }
    }
}

impl num_traits::FromPrimitive for Difficulty {
    fn from_i64(n: i64) -> Option<Self> {
        match n {
            0 => Some(Self::Peaceful),
            1 => Some(Self::Easy),
            2 => Some(Self::Normal),
            3 => Some(Self::Hard),
            _ => None,
        }
    }

    fn from_u64(n: u64) -> Option<Self> {
        match n {
            0 => Some(Self::Peaceful),
            1 => Some(Self::Easy),
            2 => Some(Self::Normal),
            3 => Some(Self::Hard),
            _ => None,
        }
    }
}

impl num_traits::ToPrimitive for Difficulty {
    fn to_i64(&self) -> Option<i64> {
        Some(*self as i64)
    }

    fn to_u64(&self) -> Option<u64> {
        Some(*self as u64)
    }
}

impl TryFrom<u8> for Difficulty {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Peaceful),
            1 => Ok(Self::Easy),
            2 => Ok(Self::Normal),
            3 => Ok(Self::Hard),
            _ => Err(()),
        }
    }
}

impl From<Difficulty> for u8 {
    fn from(difficulty: Difficulty) -> Self {
        difficulty as Self
    }
}
