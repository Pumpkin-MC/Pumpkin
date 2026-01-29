use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq)]
pub struct ParseGameModeError;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameMode {
    Survival = 0,
    Creative = 1,
    Adventure = 2,
    Spectator = 3,
}

impl GameMode {
    #[must_use]
    pub const fn to_str(&self) -> &'static str {
        match self {
            Self::Survival => "Survival",
            Self::Creative => "Creative",
            Self::Adventure => "Adventure",
            Self::Spectator => "Spectator",
        }
    }
}

impl TryFrom<i8> for GameMode {
    type Error = ();

    fn try_from(value: i8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Survival),
            1 => Ok(Self::Creative),
            2 => Ok(Self::Adventure),
            3 => Ok(Self::Spectator),
            _ => Err(()),
        }
    }
}

impl FromStr for GameMode {
    type Err = ParseGameModeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "survival" => Ok(Self::Survival),
            "creative" => Ok(Self::Creative),
            "adventure" => Ok(Self::Adventure),
            "spectator" => Ok(Self::Spectator),
            _ => Err(ParseGameModeError),
        }
    }
}
