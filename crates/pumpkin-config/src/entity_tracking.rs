use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Configuration for the distance at which entities are sent to clients.
#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct EntityTrackingConfig {
    /// Tracking range, in chunks, for entity types without an explicit override.
    pub default_range: u32,
    /// Per-entity tracking range overrides, in chunks.
    pub ranges: BTreeMap<String, u32>,
}

impl Default for EntityTrackingConfig {
    fn default() -> Self {
        let ranges = [
            ("marker", 0),
            ("arrow", 4),
            ("breeze_wind_charge", 4),
            ("cod", 4),
            ("dragon_fireball", 4),
            ("egg", 4),
            ("ender_pearl", 4),
            ("experience_bottle", 4),
            ("eye_of_ender", 4),
            ("fireball", 4),
            ("firework_rocket", 4),
            ("fishing_bobber", 4),
            ("lingering_potion", 4),
            ("llama_spit", 4),
            ("pufferfish", 4),
            ("salmon", 4),
            ("small_fireball", 4),
            ("snowball", 4),
            ("spectral_arrow", 4),
            ("splash_potion", 4),
            ("trident", 4),
            ("tropical_fish", 4),
            ("wind_charge", 4),
            ("wither_skull", 4),
            ("bat", 5),
            ("dolphin", 5),
            ("evoker_fangs", 6),
            ("experience_orb", 6),
            ("item", 6),
            ("allay", 8),
            ("bee", 8),
            ("blaze", 8),
            ("bogged", 8),
            ("cat", 8),
            ("cave_spider", 8),
            ("chest_minecart", 8),
            ("command_block_minecart", 8),
            ("creaking", 8),
            ("creeper", 8),
            ("drowned", 8),
            ("enderman", 8),
            ("endermite", 8),
            ("evoker", 8),
            ("fox", 8),
            ("furnace_minecart", 8),
            ("guardian", 8),
            ("hoglin", 8),
            ("hopper_minecart", 8),
            ("husk", 8),
            ("illusioner", 8),
            ("magma_cube", 8),
            ("minecart", 8),
            ("mule", 8),
            ("ominous_item_spawner", 8),
            ("parched", 8),
            ("parrot", 8),
            ("phantom", 8),
            ("piglin", 8),
            ("piglin_brute", 8),
            ("pillager", 8),
            ("rabbit", 8),
            ("shulker_bullet", 8),
            ("silverfish", 8),
            ("skeleton", 8),
            ("snow_golem", 8),
            ("spawner_minecart", 8),
            ("spider", 8),
            ("squid", 8),
            ("stray", 8),
            ("tnt_minecart", 8),
            ("vex", 8),
            ("vindicator", 8),
            ("witch", 8),
            ("wither_skeleton", 8),
            ("zoglin", 8),
            ("zombie", 8),
            ("zombie_villager", 8),
            ("zombified_piglin", 8),
            ("end_crystal", 16),
            ("lightning_bolt", 16),
            ("warden", 16),
            ("mannequin", 32),
            ("player", 32),
        ]
        .into_iter()
        .map(|(entity, range)| (entity.to_owned(), range))
        .collect();

        Self {
            default_range: 10,
            ranges,
        }
    }
}

impl EntityTrackingConfig {
    /// Returns the configured tracking range for an entity type, or the default range.
    #[must_use]
    pub fn range_for(&self, entity: &str) -> u32 {
        self.ranges
            .get(entity)
            .copied()
            .unwrap_or(self.default_range)
    }
}

#[cfg(test)]
mod tests {
    use super::EntityTrackingConfig;

    #[test]
    fn uses_override_or_default_range() {
        let config = EntityTrackingConfig::default();

        assert_eq!(config.range_for("marker"), 0);
        assert_eq!(config.range_for("arrow"), 4);
        assert_eq!(config.range_for("player"), 32);
        assert_eq!(config.range_for("pig"), 10);
    }
}
