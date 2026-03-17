use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Error returned when parsing a string into a [`GameMode`] fails.
#[derive(Debug, PartialEq, Eq)]
pub struct ParseGameModeError;

/// Represents the various game modes a player can be in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameMode {
    /// Standard survival mode: players take damage, gather resources, and interact normally.
    Survival = 0,
    /// Creative mode: players have unlimited resources, can fly, and cannot take damage.
    Creative = 1,
    /// Adventure mode: players cannot break blocks without the proper tools.
    Adventure = 2,
    /// Spectator mode: players can fly through blocks and observe without interacting.
    Spectator = 3,
}

impl GameMode {
    /// Returns the display string for this game mode.
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

    /// Attempts to convert an `i8` value into a [`GameMode`].
    ///
    /// # Parameters
    /// - `value`: The numeric representation of a game mode.
    ///
    /// # Returns
    /// - `Ok(GameMode)` if the value corresponds to a valid game mode:
    ///   - `0` → `Survival`
    ///   - `1` → `Creative`
    ///   - `2` → `Adventure`
    ///   - `3` → `Spectator`
    /// - `Err(())` if the value does not correspond to any valid game mode.
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
