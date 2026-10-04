use pumpkin_data::environment_attribute::Activity;

use super::behavior::BehaviorControl;
use super::memory::{MemoryModuleId, MemoryStatus};

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct ActivitySet(u32);

impl ActivitySet {
    #[must_use]
    pub const fn single(activity: Activity) -> Self {
        Self(1 << activity as u32)
    }

    #[must_use]
    pub const fn contains(self, activity: Activity) -> bool {
        self.0 & (1 << activity as u32) != 0
    }

    pub const fn insert(&mut self, activity: Activity) {
        self.0 |= 1 << activity as u32;
    }

    pub const fn remove(&mut self, activity: Activity) {
        self.0 &= !(1 << activity as u32);
    }

    pub const fn insert_all(&mut self, other: Self) {
        self.0 |= other.0;
    }

    pub const fn clear(&mut self) {
        self.0 = 0;
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub fn iter(self) -> impl Iterator<Item = Activity> {
        ALL_ACTIVITIES
            .iter()
            .copied()
            .filter(move |activity| self.contains(*activity))
    }
}

pub const ALL_ACTIVITIES: [Activity; 26] = [
    Activity::Core,
    Activity::Idle,
    Activity::Work,
    Activity::Play,
    Activity::Rest,
    Activity::Meet,
    Activity::Panic,
    Activity::Raid,
    Activity::PreRaid,
    Activity::Hide,
    Activity::Fight,
    Activity::Celebrate,
    Activity::AdmireItem,
    Activity::Avoid,
    Activity::Ride,
    Activity::PlayDead,
    Activity::LongJump,
    Activity::Ram,
    Activity::Tongue,
    Activity::Swim,
    Activity::LaySpawn,
    Activity::Sniff,
    Activity::Investigate,
    Activity::Roar,
    Activity::Emerge,
    Activity::Dig,
];

pub struct ActivityData {
    pub activity: Activity,
    pub behaviors: Vec<(i32, Box<dyn BehaviorControl>)>,
    pub conditions: Vec<(MemoryModuleId, MemoryStatus)>,
    pub memories_to_erase_when_stopped: Vec<MemoryModuleId>,
}

impl ActivityData {
    #[must_use]
    pub fn new(
        activity: Activity,
        behaviors: Vec<(i32, Box<dyn BehaviorControl>)>,
        conditions: Vec<(MemoryModuleId, MemoryStatus)>,
        memories_to_erase_when_stopped: Vec<MemoryModuleId>,
    ) -> Self {
        Self {
            activity,
            behaviors,
            conditions,
            memories_to_erase_when_stopped,
        }
    }

    #[must_use]
    pub fn with_pairs(activity: Activity, behaviors: Vec<(i32, Box<dyn BehaviorControl>)>) -> Self {
        Self::new(activity, behaviors, Vec::new(), Vec::new())
    }

    #[must_use]
    pub fn with_pairs_and_conditions(
        activity: Activity,
        behaviors: Vec<(i32, Box<dyn BehaviorControl>)>,
        conditions: Vec<(MemoryModuleId, MemoryStatus)>,
    ) -> Self {
        Self::new(activity, behaviors, conditions, Vec::new())
    }

    #[must_use]
    pub fn with_priority_start(
        activity: Activity,
        first_priority: i32,
        behaviors: Vec<Box<dyn BehaviorControl>>,
    ) -> Self {
        Self::with_pairs(activity, create_priority_pairs(first_priority, behaviors))
    }

    #[must_use]
    pub fn with_priority_start_and_conditions(
        activity: Activity,
        first_priority: i32,
        behaviors: Vec<Box<dyn BehaviorControl>>,
        conditions: Vec<(MemoryModuleId, MemoryStatus)>,
    ) -> Self {
        Self::with_pairs_and_conditions(
            activity,
            create_priority_pairs(first_priority, behaviors),
            conditions,
        )
    }

    #[must_use]
    pub fn with_gate_memory(
        activity: Activity,
        first_priority: i32,
        behaviors: Vec<Box<dyn BehaviorControl>>,
        memory: MemoryModuleId,
    ) -> Self {
        Self::new(
            activity,
            create_priority_pairs(first_priority, behaviors),
            vec![(memory, MemoryStatus::ValuePresent)],
            vec![memory],
        )
    }
}

#[must_use]
pub fn create_priority_pairs(
    first_priority: i32,
    behaviors: Vec<Box<dyn BehaviorControl>>,
) -> Vec<(i32, Box<dyn BehaviorControl>)> {
    behaviors
        .into_iter()
        .enumerate()
        .map(|(offset, behavior)| (first_priority + offset as i32, behavior))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::super::memory::types;
    use super::*;

    #[test]
    fn activity_indices_fit_the_bitset() {
        assert_eq!(ALL_ACTIVITIES.len(), 26);
        assert_eq!(Activity::Dig as u32, 25);
        assert!(ALL_ACTIVITIES.len() <= u32::BITS as usize);
    }

    #[test]
    fn every_activity_round_trips_through_the_set() {
        for activity in ALL_ACTIVITIES {
            let mut set = ActivitySet::default();
            assert!(!set.contains(activity));
            set.insert(activity);
            assert!(set.contains(activity));
            assert_eq!(set.iter().collect::<Vec<_>>(), vec![activity]);
            set.remove(activity);
            assert!(set.is_empty());
        }
    }

    #[test]
    fn set_operations_combine_and_clear() {
        let mut set = ActivitySet::single(Activity::Core);
        set.insert_all(ActivitySet::single(Activity::Idle));
        assert!(set.contains(Activity::Core));
        assert!(set.contains(Activity::Idle));
        assert!(!set.contains(Activity::Fight));
        set.clear();
        assert!(set.is_empty());
    }

    #[test]
    fn gate_memory_sets_both_the_condition_and_the_erase_set() {
        let data = ActivityData::with_gate_memory(
            Activity::AdmireItem,
            10,
            Vec::new(),
            types::ADMIRING_ITEM.id(),
        );
        assert_eq!(
            data.conditions,
            vec![(types::ADMIRING_ITEM.id(), MemoryStatus::ValuePresent)]
        );
        assert_eq!(
            data.memories_to_erase_when_stopped,
            vec![types::ADMIRING_ITEM.id()]
        );
    }

    #[test]
    fn priority_pairs_count_up_from_the_first_priority() {
        let behaviors: Vec<Box<dyn BehaviorControl>> = vec![
            Box::new(super::super::behavior::DoNothing::new(1, 1)),
            Box::new(super::super::behavior::DoNothing::new(1, 1)),
            Box::new(super::super::behavior::DoNothing::new(1, 1)),
        ];
        let pairs = create_priority_pairs(5, behaviors);
        assert_eq!(
            pairs.iter().map(|(p, _)| *p).collect::<Vec<_>>(),
            vec![5, 6, 7]
        );
    }
}
