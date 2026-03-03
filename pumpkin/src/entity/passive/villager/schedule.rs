/// Villager daily schedule based on world time (0-24000 ticks per day).
///
/// Schedule phases:
/// - 0..2000: Idle (waking up)
/// - 2000..9000: Work (go to workstation)
/// - 9000..11000: Meet (gather at bell)
/// - 11000..12000: Idle
/// - 12000..24000: Rest (sleep in bed)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VillagerActivity {
    Idle,
    Work,
    Meet,
    Rest,
}

impl VillagerActivity {
    /// Get the current activity based on world time (0-24000).
    #[must_use]
    pub const fn from_time(time_of_day: i64) -> Self {
        let t = time_of_day.rem_euclid(24000); // Normalize to 0-23999
        match t {
            0..2000 | 11000..12000 => Self::Idle,
            2000..9000 => Self::Work,
            9000..11000 => Self::Meet,
            _ => Self::Rest,
        }
    }

    /// Whether the villager should be sleeping during this activity.
    #[must_use]
    pub const fn is_sleeping(&self) -> bool {
        matches!(self, Self::Rest)
    }

    /// Whether the villager should go to their workstation.
    #[must_use]
    pub const fn is_working(&self) -> bool {
        matches!(self, Self::Work)
    }

    /// Whether the villager should gather at the village bell.
    #[must_use]
    pub const fn is_meeting(&self) -> bool {
        matches!(self, Self::Meet)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schedule_phases() {
        assert_eq!(VillagerActivity::from_time(0), VillagerActivity::Idle);
        assert_eq!(VillagerActivity::from_time(1000), VillagerActivity::Idle);
        assert_eq!(VillagerActivity::from_time(2000), VillagerActivity::Work);
        assert_eq!(VillagerActivity::from_time(5000), VillagerActivity::Work);
        assert_eq!(VillagerActivity::from_time(9000), VillagerActivity::Meet);
        assert_eq!(VillagerActivity::from_time(10000), VillagerActivity::Meet);
        assert_eq!(VillagerActivity::from_time(11000), VillagerActivity::Idle);
        assert_eq!(VillagerActivity::from_time(12000), VillagerActivity::Rest);
        assert_eq!(VillagerActivity::from_time(18000), VillagerActivity::Rest);
        assert_eq!(VillagerActivity::from_time(23999), VillagerActivity::Rest);
    }

    #[test]
    fn schedule_wrapping() {
        assert_eq!(VillagerActivity::from_time(24000), VillagerActivity::Idle);
        assert_eq!(VillagerActivity::from_time(48000), VillagerActivity::Idle);
        assert_eq!(VillagerActivity::from_time(-1), VillagerActivity::Rest);
    }
}
