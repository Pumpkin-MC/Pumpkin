use serde::{Deserialize, Serialize};

/// Vanilla sends a player's movement to the others every this many ticks.
const PLAYER_UPDATE_INTERVAL: u32 = 2;

/// How often the entity tracker sends player movement. Off by default, which is vanilla.
#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(default)]
pub struct EntityTrackingConfig {
    /// Distance in blocks from which a player's movement is sent to another player only every
    /// `far_player_update_interval` ticks. `0` is off.
    pub far_player_distance: u32,
    /// Ticks between movement updates of far players, up to 20. `2` is vanilla, e.g. `4`
    /// halves them. Values below 2 count as 2.
    pub far_player_update_interval: u32,
}

impl Default for EntityTrackingConfig {
    fn default() -> Self {
        Self {
            far_player_distance: 0,
            far_player_update_interval: PLAYER_UPDATE_INTERVAL,
        }
    }
}

impl EntityTrackingConfig {
    /// Whether players farther than `far_player_distance` get movement less often than
    /// vanilla sends it.
    #[must_use]
    pub const fn far_players_enabled(&self) -> bool {
        self.far_player_distance > 0 && self.far_player_update_interval > PLAYER_UPDATE_INTERVAL
    }

    /// Whether every option is off, which is vanilla behaviour.
    #[must_use]
    pub const fn is_vanilla(&self) -> bool {
        !self.far_players_enabled()
    }

    /// Why `far_player_update_interval` is not used as written, if it is not.
    #[must_use]
    pub fn far_player_update_interval_warning(&self) -> Option<String> {
        (self.far_player_update_interval < PLAYER_UPDATE_INTERVAL).then(|| {
            format!(
                "Entity tracking far_player_update_interval {} is below 2, the vanilla interval, so 2 is used",
                self.far_player_update_interval
            )
        })
    }

    /// Why a `far_player_distance` that is set changes nothing, if it does not.
    #[must_use]
    pub const fn far_player_distance_note(&self) -> Option<&'static str> {
        if self.far_player_distance > 0 && !self.far_players_enabled() {
            Some(
                "Entity tracking far_player_distance changes nothing while far_player_update_interval is 2, the vanilla interval",
            )
        } else {
            None
        }
    }

    /// Validates the value ranges.
    ///
    /// # Panics
    /// Panics on a value outside its range.
    pub fn validate(&self) {
        assert!(
            self.far_player_update_interval <= 20,
            "Entity tracking far_player_update_interval must be at most 20"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_file_without_the_section_is_vanilla() {
        let config: crate::AdvancedConfiguration = toml::from_str("").unwrap();
        config.validate();
        let entity_tracking = config.entity_tracking;
        assert!(entity_tracking.is_vanilla());
        assert_eq!(entity_tracking.far_player_update_interval, 2);
        assert_eq!(entity_tracking.far_player_update_interval_warning(), None);
        assert_eq!(entity_tracking.far_player_distance_note(), None);
    }

    #[test]
    fn the_options_are_read_from_the_section() {
        let config: crate::AdvancedConfiguration = toml::from_str(
            "[entity_tracking]\nfar_player_distance = 48\nfar_player_update_interval = 6",
        )
        .unwrap();
        config.validate();
        assert!(config.entity_tracking.far_players_enabled());
        assert_eq!(config.entity_tracking.far_player_distance_note(), None);
        assert_eq!(
            config.entity_tracking,
            EntityTrackingConfig {
                far_player_distance: 48,
                far_player_update_interval: 6,
            }
        );
    }

    #[test]
    fn an_interval_below_vanillas_counts_as_vanillas() {
        for interval in 0..=1 {
            let config = EntityTrackingConfig {
                far_player_distance: 48,
                far_player_update_interval: interval,
            };
            assert!(config.far_player_update_interval_warning().is_some());
            assert!(config.is_vanilla(), "interval {interval}");
        }
        let config = EntityTrackingConfig {
            far_player_distance: 48,
            ..Default::default()
        };
        assert_eq!(config.far_player_update_interval_warning(), None);
        assert!(config.is_vanilla());
        assert!(config.far_player_distance_note().is_some());
    }

    #[test]
    #[should_panic(expected = "far_player_update_interval must be at most 20")]
    fn an_interval_out_of_range_stops_the_server() {
        let config: EntityTrackingConfig =
            toml::from_str("far_player_update_interval = 21").unwrap();
        config.validate();
    }
}
