use serde::{Deserialize, Deserializer, Serialize};

use crate::{chunk::ChunkConfig, lighting::LightingEngineConfig};

/// Configuration for world and level-specific settings.
///
/// Currently, it includes chunk-related options; more settings may be added later.
#[derive(Deserialize, Serialize, Clone, toml_spanner::Toml)]
#[toml(ignore_unknown_fields, recoverable)]
pub struct LevelConfig {
    /// Configuration for chunk behaviour and management.
    #[toml(default = Self::default().chunk)]
    pub chunk: ChunkConfig,
    /// Configuration for lighting engine propagation mode.
    #[serde(default)]
    #[toml(default = Self::default().lighting)]
    pub lighting: LightingEngineConfig,
    /// Number of ticks between autosave checks. If 0, autosave is disabled.
    #[serde(default = "default_autosave_ticks")]
    #[toml(default = Self::default().autosave_ticks)]
    pub autosave_ticks: u64,
    /// Margins for ticking entities in parallel islands.
    #[serde(default)]
    #[toml(default = Self::default().entity_islands)]
    pub entity_islands: EntityIslandsConfig,
    // TODO: More options
}

/// Safety margins for ticking entities in parallel islands. Lower values tick more entities in
/// parallel but can let interactions run out of tick order.
#[derive(Deserialize, Serialize, Clone, Copy, toml_spanner::Toml)]
#[serde(default)]
#[toml(ignore_unknown_fields, recoverable)]
pub struct EntityIslandsConfig {
    /// Blocks an entity whose tick is not audited can reach beyond its follow range.
    #[serde(deserialize_with = "EntityIslandsConfig::deserialize_unaudited_reach")]
    #[toml(default = Self::default().unaudited_reach, with = unaudited_reach)]
    pub unaudited_reach: f64,
    /// Blocks a hit from such an entity can throw an audited one in a tick. Raise it for TNT
    /// cannons.
    #[serde(deserialize_with = "EntityIslandsConfig::deserialize_knockback_reach")]
    #[toml(default = Self::default().knockback_reach, with = knockback_reach)]
    pub knockback_reach: f64,
}

impl EntityIslandsConfig {
    fn deserialize_unaudited_reach<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<f64, D::Error> {
        deserialize_margin(deserializer, Self::default().unaudited_reach)
    }

    fn deserialize_knockback_reach<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<f64, D::Error> {
        deserialize_margin(deserializer, Self::default().knockback_reach)
    }
}

fn deserialize_margin<'de, D: Deserializer<'de>>(
    deserializer: D,
    fallback: f64,
) -> Result<f64, D::Error> {
    let value = f64::deserialize(deserializer)?;
    Ok(if value.is_finite() {
        value.max(0.0)
    } else {
        fallback
    })
}

impl Default for EntityIslandsConfig {
    fn default() -> Self {
        Self {
            unaudited_reach: 16.0,
            knockback_reach: 4.0,
        }
    }
}

const fn default_autosave_ticks() -> u64 {
    6000 // Default to 5 minutes at 20 TPS
}

impl Default for LevelConfig {
    fn default() -> Self {
        Self {
            chunk: ChunkConfig::default(),
            lighting: LightingEngineConfig::default(),
            autosave_ticks: default_autosave_ticks(),
            entity_islands: EntityIslandsConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_level_config_enables_autosave() {
        assert_eq!(LevelConfig::default().autosave_ticks, 6000);
    }

    #[test]
    fn generated_config_round_trip_keeps_autosave_enabled() {
        let generated = toml::to_string(&LevelConfig::default()).unwrap();
        let reloaded: LevelConfig = toml::from_str(&generated).unwrap();
        assert_eq!(reloaded.autosave_ticks, 6000);
    }

    #[test]
    fn an_explicit_zero_still_disables_autosave() {
        let disabled = LevelConfig {
            autosave_ticks: 0,
            ..LevelConfig::default()
        };
        let generated = toml::to_string(&disabled).unwrap();
        let reloaded: LevelConfig = toml::from_str(&generated).unwrap();
        assert_eq!(reloaded.autosave_ticks, 0);
    }
}

mod unaudited_reach {
    pub fn from_toml<'de>(
        ctx: &mut toml_spanner::Context<'de>,
        item: &toml_spanner::Item<'de>,
    ) -> Result<f64, toml_spanner::Failed> {
        super::toml_margin(
            ctx,
            item,
            super::EntityIslandsConfig::default().unaudited_reach,
        )
    }
}

mod knockback_reach {
    pub fn from_toml<'de>(
        ctx: &mut toml_spanner::Context<'de>,
        item: &toml_spanner::Item<'de>,
    ) -> Result<f64, toml_spanner::Failed> {
        super::toml_margin(
            ctx,
            item,
            super::EntityIslandsConfig::default().knockback_reach,
        )
    }
}

fn toml_margin<'de>(
    ctx: &mut toml_spanner::Context<'de>,
    item: &toml_spanner::Item<'de>,
    fallback: f64,
) -> Result<f64, toml_spanner::Failed> {
    let value = <f64 as toml_spanner::FromToml>::from_toml(ctx, item)?;
    Ok(if value.is_finite() {
        value.max(0.0)
    } else {
        fallback
    })
}
