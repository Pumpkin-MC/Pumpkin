use pumpkin_util::math::vector3::Vector3;
use uuid::Uuid;

// Mirrors net.minecraft.world.level.gameevent.vibrations.VibrationInfo, except the
// game event itself is replaced by its precomputed vibration frequency: the generated
// pumpkin_data::GameEvent enum (pumpkin-data/src/generated/game_event.rs) derives
// nothing (no Copy/Clone/PartialEq), and that file may not be edited, so it cannot be
// stored and compared across ticks the way vanilla's Holder<GameEvent> is. Selection
// only ever needs the frequency (see `shouldReplaceVibration`), so storing it directly
// loses nothing for this purpose.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VibrationInfo {
    pub frequency: i32,
    pub distance: f32,
    pub pos: Vector3<f64>,
    pub source_entity: Option<Uuid>,
}

// Port of VibrationSelector.java (66 lines). Picks, per game tick, the single
// candidate vibration a listener will actually act on: closer distance wins, and on
// a distance tie the higher-frequency event wins (VibrationSelector.shouldReplaceVibration).
//
// Not yet wired into the live emission path: vanilla resolves the chosen candidate on
// a later tick via VibrationSystem.Ticker (travel-time simulation), which this Phase 1
// does not implement (see mod.rs doc comment). This type is provided, tested, standalone,
// for a future phase's tick-based listener to use.
#[derive(Default)]
pub struct VibrationSelector {
    current: Option<(VibrationInfo, u64)>,
}

impl VibrationSelector {
    #[must_use]
    pub const fn new() -> Self {
        Self { current: None }
    }

    pub fn add_candidate(&mut self, candidate: VibrationInfo, tick_time: u64) {
        if self.should_replace_vibration(&candidate, tick_time) {
            self.current = Some((candidate, tick_time));
        }
    }

    fn should_replace_vibration(&self, candidate: &VibrationInfo, tick_time: u64) -> bool {
        let Some((previous, previous_tick)) = &self.current else {
            return true;
        };
        if tick_time != *previous_tick {
            return false;
        }
        if candidate.distance < previous.distance {
            return true;
        }
        if candidate.distance > previous.distance {
            return false;
        }
        candidate.frequency > previous.frequency
    }

    #[must_use]
    pub fn chosen_candidate(&self, time: u64) -> Option<VibrationInfo> {
        let (info, tick) = self.current.as_ref()?;
        (*tick < time).then_some(*info)
    }

    pub const fn start_over(&mut self) {
        self.current = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info(frequency: i32, distance: f32) -> VibrationInfo {
        VibrationInfo {
            frequency,
            distance,
            pos: Vector3::new(0.0, 0.0, 0.0),
            source_entity: None,
        }
    }

    #[test]
    fn first_candidate_is_always_accepted() {
        let mut selector = VibrationSelector::new();
        selector.add_candidate(info(1, 5.0), 10);
        assert_eq!(selector.chosen_candidate(11), Some(info(1, 5.0)));
    }

    #[test]
    fn not_chosen_before_the_tick_after_selection() {
        let mut selector = VibrationSelector::new();
        selector.add_candidate(info(1, 5.0), 10);
        assert_eq!(selector.chosen_candidate(10), None);
        assert_eq!(selector.chosen_candidate(9), None);
    }

    #[test]
    fn closer_candidate_replaces_further_one_on_the_same_tick() {
        let mut selector = VibrationSelector::new();
        selector.add_candidate(info(1, 10.0), 10);
        selector.add_candidate(info(1, 2.0), 10);
        assert_eq!(selector.chosen_candidate(11), Some(info(1, 2.0)));
    }

    #[test]
    fn further_candidate_does_not_replace_closer_one_on_the_same_tick() {
        let mut selector = VibrationSelector::new();
        selector.add_candidate(info(1, 2.0), 10);
        selector.add_candidate(info(1, 10.0), 10);
        assert_eq!(selector.chosen_candidate(11), Some(info(1, 2.0)));
    }

    #[test]
    fn equal_distance_prefers_higher_frequency() {
        let mut selector = VibrationSelector::new();
        selector.add_candidate(info(3, 5.0), 10);
        selector.add_candidate(info(9, 5.0), 10);
        assert_eq!(selector.chosen_candidate(11), Some(info(9, 5.0)));

        let mut selector = VibrationSelector::new();
        selector.add_candidate(info(9, 5.0), 10);
        selector.add_candidate(info(3, 5.0), 10);
        assert_eq!(selector.chosen_candidate(11), Some(info(9, 5.0)));
    }

    #[test]
    fn candidate_on_a_different_tick_does_not_replace_the_current_one() {
        let mut selector = VibrationSelector::new();
        selector.add_candidate(info(1, 5.0), 10);
        selector.add_candidate(info(15, 0.1), 11);
        assert_eq!(selector.chosen_candidate(11), Some(info(1, 5.0)));
    }

    #[test]
    fn start_over_clears_the_selection() {
        let mut selector = VibrationSelector::new();
        selector.add_candidate(info(1, 5.0), 10);
        selector.start_over();
        assert_eq!(selector.chosen_candidate(11), None);
    }
}
