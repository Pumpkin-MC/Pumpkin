//! Data-driven weather
//! profiles describing how a weather cycle times itself, and the
//! per-world settings choosing which cycle a world follows.

use std::{collections::HashMap, path::Path, sync::Arc};

use rand::RngExt;
use serde::Deserialize;
use tracing::warn;

/// The profile id the server's shared weather cycle uses, if it is loaded.
pub const DEFAULT_WEATHER_PROFILE: &str = "default";

/// An inclusive tick range, written `[min, max]` in JSON.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
pub struct TickRange(pub i32, pub i32);

impl TickRange {
    #[must_use]
    pub fn sample(self, rng: &mut impl rand::Rng) -> i32 {
        rng.random_range(self.0..=self.1)
    }

    fn validate(self, what: &str) -> Result<(), String> {
        if self.0 < 0 || self.0 > self.1 {
            return Err(format!(
                "{what} must be [min, max] with 0 <= min <= max, got [{}, {}]",
                self.0, self.1
            ));
        }
        Ok(())
    }
}

/// How long a weather state waits to start and how long it lasts.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PhaseTiming {
    pub delay: TickRange,
    pub duration: TickRange,
}

impl PhaseTiming {
    const RAIN: Self = Self {
        delay: TickRange(12_000, 180_000),
        duration: TickRange(12_000, 24_000),
    };
    const THUNDER: Self = Self {
        delay: TickRange(12_000, 180_000),
        duration: TickRange(3_600, 15_600),
    };

    fn validate(self, what: &str) -> Result<(), String> {
        self.delay.validate(&format!("{what}.delay"))?;
        self.duration.validate(&format!("{what}.duration"))
    }
}

/// What clients see while a custom phase is active.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ClientEffect {
    #[default]
    None,
    Rain,
    Thunder,
}

/// A custom weather state with a cycle of its own, such as a blood moon.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PhaseDefinition {
    pub id: String,
    pub delay: TickRange,
    pub duration: TickRange,
    #[serde(default)]
    pub client_effect: ClientEffect,
}

impl PhaseDefinition {
    #[must_use]
    pub const fn timing(&self) -> PhaseTiming {
        PhaseTiming {
            delay: self.delay,
            duration: self.duration,
        }
    }
}

/// A weather cycle's timings. Everything left out of the JSON keeps vanilla's values.
///
/// ```json
/// {
///   "rain": { "delay": [12000, 180000], "duration": [12000, 24000] },
///   "phases": [
///     { "id": "blood_moon", "delay": [24000, 96000], "duration": [6000, 12000],
///       "client_effect": "thunder" }
///   ]
/// }
/// ```
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct WeatherProfile {
    pub rain: PhaseTiming,
    pub thunder: PhaseTiming,
    pub phases: Vec<PhaseDefinition>,
}

impl Default for WeatherProfile {
    fn default() -> Self {
        Self {
            rain: PhaseTiming::RAIN,
            thunder: PhaseTiming::THUNDER,
            phases: Vec::new(),
        }
    }
}

impl WeatherProfile {
    /// Parses and validates a profile.
    pub fn from_json(json: &str) -> Result<Self, String> {
        let profile: Self = serde_json::from_str(json).map_err(|e| e.to_string())?;
        profile.rain.validate("rain")?;
        profile.thunder.validate("thunder")?;
        for (i, phase) in profile.phases.iter().enumerate() {
            if phase.id.is_empty() || profile.phases[..i].iter().any(|p| p.id == phase.id) {
                return Err(format!("phase id '{}' is empty or repeated", phase.id));
            }
            phase.timing().validate(&format!("phase '{}'", phase.id))?;
        }
        Ok(profile)
    }
}

/// The weather profiles a server knows by id
#[derive(Default)]
pub struct WeatherProfiles {
    profiles: std::sync::RwLock<HashMap<String, Arc<WeatherProfile>>>,
}

impl WeatherProfiles {
    /// Loads every `<id>.json` of a folder, a broken file is skipped with a warning.
    pub fn load_folder(&self, folder: &Path) {
        let Ok(entries) = std::fs::read_dir(folder) else {
            return;
        };
        for path in entries.flatten().map(|e| e.path()) {
            if path.extension().is_none_or(|ext| ext != "json") {
                continue;
            }
            let Some(id) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            let result = std::fs::read_to_string(&path)
                .map_err(|e| e.to_string())
                .and_then(|json| self.register(id, &json));
            if let Err(e) = result {
                warn!("Skipping weather profile {}: {e}", path.display());
            }
        }
    }

    /// Adds or replaces a profile. Worlds already using it keep the old timings.
    pub fn register(&self, id: &str, json: &str) -> Result<(), String> {
        let profile = Arc::new(WeatherProfile::from_json(json)?);
        self.profiles
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(id.to_owned(), profile);
        Ok(())
    }

    #[must_use]
    pub fn get(&self, id: &str) -> Option<Arc<WeatherProfile>> {
        self.profiles
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(id)
            .cloned()
    }
}

/// Which weather cycle a world follows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WeatherCycle {
    #[default]
    Shared,
    Own,
    /// Never changes by itself.
    Fixed {
        raining: bool,
        thundering: bool,
    },
}

/// How a created world handles weather.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WorldWeatherSettings {
    pub cycle: WeatherCycle,
    /// A profile id for an own or fixed cycle.
    pub profile: Option<String>,
    /// Overrides whether the world has weather, the dimension decides if unset.
    pub enabled: Option<bool>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn missing_fields_keep_vanilla_timings() {
        let profile = WeatherProfile::from_json("{}").unwrap();
        assert_eq!(profile, WeatherProfile::default());
        assert_eq!(profile.rain.duration, TickRange(12_000, 24_000));
    }

    #[test]
    fn phases_are_parsed() {
        let profile = WeatherProfile::from_json(
            r#"{"phases":[{"id":"blood_moon","delay":[10,20],"duration":[5,6],"client_effect":"thunder"}]}"#,
        )
        .unwrap();
        assert_eq!(profile.phases[0].client_effect, ClientEffect::Thunder);
        assert_eq!(profile.phases[0].delay, TickRange(10, 20));
    }

    #[test]
    fn bad_profiles_are_rejected() {
        for json in [
            r#"{"rain":{"delay":[9,1],"duration":[1,2]}}"#,
            r#"{"phases":[{"id":"a","delay":[1,2],"duration":[1,2]},{"id":"a","delay":[1,2],"duration":[1,2]}]}"#,
            r#"{"unknown":1}"#,
            r#"{"phases":[{"id":"a","delay":[1,2],"duration":[1,2],"typo":true}]}"#,
            r#"{"rain":{"delay":[1,2],"duration":[1,2],"typo":1}}"#,
        ] {
            assert!(WeatherProfile::from_json(json).is_err(), "{json}");
        }
    }
}
