use std::str::FromStr;

use num_derive::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};

pub struct ParseDifficultyError;

#[derive(Serialize, Deserialize, FromPrimitive, ToPrimitive, PartialEq, Clone, Copy, Debug)]
pub enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}

impl Difficulty {
    pub fn to_int(&self) -> u8 {
        match self {
            Self::Peaceful => 0,
            Self::Easy => 1,
            Self::Normal => 2,
            Self::Hard => 3,
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
