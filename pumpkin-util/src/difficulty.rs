use std::str::FromStr;

use num_derive::{FromPrimitive, ToPrimitive};
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
#[derive(Serialize, Deserialize, FromPrimitive, ToPrimitive, PartialEq, Eq, Clone, Copy, Debug)]
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
