use std::str::FromStr;

use num_derive::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};

pub struct ParseDifficultyError;

#[derive(Serialize, Deserialize, FromPrimitive, ToPrimitive, PartialEq, Clone, Copy, Debug)]
pub enum Difficulty {
    Peaceful = 0,
    Easy = 1,
    Normal = 2,
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
