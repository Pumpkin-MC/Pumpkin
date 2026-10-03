use std::fmt;
use std::sync::Arc;

use pumpkin_data::attributes::Attributes;
use pumpkin_data::environment_attribute::Activity;

use crate::entity::mob::Mob;
use crate::world::World;

use activity::{ActivityData, ActivitySet};
use behavior::{BehaviorEntry, Status};
use memory::{
    MemoryModuleId, MemoryModuleType, MemoryStatus, MemoryStore, MemoryValue, PackedMemories,
};
use sensing::{SensorEntry, SensorType};

pub mod activity;
pub mod behavior;
pub mod memory;
pub mod sensing;

/// The brain of the mob being ticked, reachable only through this context.
///
/// Behaviors and sensors must never lock `MobEntity::brain` themselves; the tick already
/// holds that lock. While a tick runs the behavior and sensor lists are detached from the
/// brain, so `stop_all`, `debug_dump` and `is_brain_dead` only answer correctly outside one.
pub struct BrainTick<'a> {
    pub brain: &'a mut Brain,
    pub world: &'a Arc<World>,
    pub mob: &'a dyn Mob,
    pub time: i64,
}

impl BrainTick<'_> {
    #[must_use]
    pub fn visibility(&self) -> VisibilityContext<'_> {
        VisibilityContext {
            brain: self.brain,
            world: self.world,
            mob: self.mob,
            follow_range: self
                .mob
                .get_mob_entity()
                .living_entity
                .get_attribute_value(&Attributes::FOLLOW_RANGE),
        }
    }
}

/// Read once per sensor scan, where vanilla re-ranges its shared `TargetingConditions`.
pub struct VisibilityContext<'a> {
    pub brain: &'a Brain,
    pub world: &'a Arc<World>,
    pub mob: &'a dyn Mob,
    pub follow_range: f64,
}

type ActivityConditions = Box<[(MemoryModuleId, MemoryStatus)]>;

pub struct Brain {
    memories: MemoryStore,
    sensors: Vec<SensorEntry>,
    behaviors: Vec<BehaviorEntry>,
    activity_requirements: Vec<Option<ActivityConditions>>,
    activity_memories_to_erase: Vec<Option<Box<[MemoryModuleId]>>>,
    core_activities: ActivitySet,
    active_activities: ActivitySet,
    default_activity: Activity,
}

impl Default for Brain {
    fn default() -> Self {
        let mut brain = Self {
            memories: MemoryStore::default(),
            sensors: Vec::new(),
            behaviors: Vec::new(),
            activity_requirements: Vec::new(),
            activity_memories_to_erase: Vec::new(),
            core_activities: ActivitySet::single(Activity::Core),
            active_activities: ActivitySet::default(),
            default_activity: Activity::Idle,
        };
        brain.use_default_activity();
        brain
    }
}

impl Brain {
    pub fn register_memory(&mut self, id: MemoryModuleId) {
        self.memories.register(id);
    }

    #[must_use]
    pub fn check(&self, id: MemoryModuleId, status: MemoryStatus) -> bool {
        self.memories.check(id, status)
    }

    #[must_use]
    pub fn check_all(&self, conditions: &[(MemoryModuleId, MemoryStatus)]) -> bool {
        conditions
            .iter()
            .all(|(id, status)| self.memories.check(*id, *status))
    }

    #[must_use]
    pub fn has_memory_value(&self, id: MemoryModuleId) -> bool {
        self.memories.has_value(id)
    }

    #[must_use]
    pub fn get<T: MemoryValue>(&self, key: MemoryModuleType<T>) -> Option<&T> {
        self.memories.get(key)
    }

    pub fn set<T: MemoryValue>(&mut self, key: MemoryModuleType<T>, value: T) {
        self.memories.set(key, value);
    }

    pub fn set_optional<T: MemoryValue>(&mut self, key: MemoryModuleType<T>, value: Option<T>) {
        self.memories.set_optional(key, value);
    }

    pub fn set_with_expiry<T: MemoryValue>(
        &mut self,
        key: MemoryModuleType<T>,
        value: T,
        ttl: i64,
    ) {
        self.memories.set_with_expiry(key, value, ttl);
    }

    pub fn erase(&mut self, id: MemoryModuleId) {
        self.memories.erase(id);
    }

    pub fn clear_memories(&mut self) {
        self.memories.clear_all();
    }

    #[must_use]
    pub fn is_value<T: MemoryValue + PartialEq>(
        &self,
        key: MemoryModuleType<T>,
        value: &T,
    ) -> bool {
        self.memories.is_value(key, value)
    }

    #[must_use]
    pub fn is_entity_value(
        &self,
        key: MemoryModuleType<Arc<dyn crate::entity::EntityBase>>,
        entity: &dyn crate::entity::EntityBase,
    ) -> bool {
        self.memories
            .get(key)
            .is_some_and(|stored| stored.get_entity().entity_id == entity.get_entity().entity_id)
    }

    #[must_use]
    pub fn time_until_expiry(&self, id: MemoryModuleId) -> Option<i64> {
        self.memories.time_until_expiry(id)
    }

    pub const fn set_core_activities(&mut self, activities: ActivitySet) {
        self.core_activities = activities;
    }

    pub const fn set_default_activity(&mut self, activity: Activity) {
        self.default_activity = activity;
    }

    pub fn use_default_activity(&mut self) {
        self.set_active_activity(self.default_activity);
    }

    #[must_use]
    pub const fn is_active(&self, activity: Activity) -> bool {
        self.active_activities.contains(activity)
    }

    #[must_use]
    pub const fn active_activities(&self) -> ActivitySet {
        self.active_activities
    }

    #[must_use]
    pub fn get_active_non_core_activity(&self) -> Option<Activity> {
        self.active_activities
            .iter()
            .find(|activity| !self.core_activities.contains(*activity))
    }

    pub fn set_active_activity_if_possible(&mut self, activity: Activity) {
        if self.activity_requirements_are_met(activity) {
            self.set_active_activity(activity);
        } else {
            self.use_default_activity();
        }
    }

    pub fn set_active_activity_to_first_valid(&mut self, activities: &[Activity]) {
        for activity in activities {
            if self.activity_requirements_are_met(*activity) {
                self.set_active_activity(*activity);
                break;
            }
        }
    }

    fn set_active_activity(&mut self, activity: Activity) {
        if self.is_active(activity) {
            return;
        }
        self.erase_memories_for_other_activities_than(activity);
        self.active_activities.clear();
        self.active_activities.insert_all(self.core_activities);
        self.active_activities.insert(activity);
    }

    fn erase_memories_for_other_activities_than(&mut self, activity: Activity) {
        for old in self.active_activities.iter() {
            if old == activity {
                continue;
            }
            if let Some(Some(memories)) = self.activity_memories_to_erase.get(old as usize) {
                for id in memories {
                    self.memories.erase(*id);
                }
            }
        }
    }

    #[must_use]
    pub fn activity_requirements_are_met(&self, activity: Activity) -> bool {
        let Some(Some(conditions)) = self.activity_requirements.get(activity as usize) else {
            return false;
        };
        self.check_all(conditions)
    }

    pub fn add_activity(&mut self, data: ActivityData) {
        let index = data.activity as usize;
        let slots = activity::ALL_ACTIVITIES.len();
        if self.activity_requirements.len() < slots {
            self.activity_requirements.resize_with(slots, || None);
            self.activity_memories_to_erase.resize_with(slots, || None);
        }
        self.activity_requirements[index] = Some(data.conditions.into_boxed_slice());
        if !data.memories_to_erase_when_stopped.is_empty() {
            self.activity_memories_to_erase[index] =
                Some(data.memories_to_erase_when_stopped.into_boxed_slice());
        }
        for (priority, behavior) in data.behaviors {
            for id in behavior.required_memories() {
                self.memories.register(*id);
            }
            self.behaviors.push(BehaviorEntry {
                priority,
                activity: data.activity,
                behavior,
            });
        }
        self.behaviors.sort_by_key(|entry| entry.priority);
    }

    pub fn tick(&mut self, world: &Arc<World>, mob: &dyn Mob, time: i64) {
        self.memories.tick();

        let mut sensors = std::mem::take(&mut self.sensors);
        for sensor in &mut sensors {
            sensor.tick(&mut BrainTick {
                brain: self,
                world,
                mob,
                time,
            });
        }
        debug_assert!(self.sensors.is_empty());
        self.sensors = sensors;

        let mut behaviors = std::mem::take(&mut self.behaviors);
        for entry in &mut behaviors {
            if self.active_activities.contains(entry.activity)
                && entry.behavior.status() == Status::Stopped
            {
                entry.behavior.try_start(&mut BrainTick {
                    brain: self,
                    world,
                    mob,
                    time,
                });
            }
        }
        for entry in &mut behaviors {
            if entry.behavior.status() == Status::Running {
                entry.behavior.tick_or_stop(&mut BrainTick {
                    brain: self,
                    world,
                    mob,
                    time,
                });
            }
        }
        debug_assert!(self.behaviors.is_empty());
        self.behaviors = behaviors;
    }

    pub fn stop_all(&mut self, world: &Arc<World>, mob: &dyn Mob, time: i64) {
        let mut behaviors = std::mem::take(&mut self.behaviors);
        for entry in &mut behaviors {
            if entry.behavior.status() == Status::Running {
                entry.behavior.do_stop(&mut BrainTick {
                    brain: self,
                    world,
                    mob,
                    time,
                });
            }
        }
        self.behaviors = behaviors;
    }

    #[must_use]
    pub const fn is_brain_dead(&self) -> bool {
        self.memories.is_empty() && self.sensors.is_empty() && self.behaviors.is_empty()
    }

    #[must_use]
    pub fn pack(&self) -> PackedMemories {
        self.memories.pack()
    }

    pub fn load_packed(&mut self, packed: &PackedMemories) {
        self.memories.load_packed(packed);
    }

    #[must_use]
    pub fn debug_dump(&self) -> BrainDebugDump {
        BrainDebugDump {
            activities: self
                .active_activities
                .iter()
                .map(|activity| activity.name())
                .collect(),
            behaviors: self
                .behaviors
                .iter()
                .filter(|entry| entry.behavior.status() == Status::Running)
                .map(|entry| entry.behavior.debug_string())
                .collect(),
            memories: self.memories.debug_lines(),
        }
    }
}

impl fmt::Debug for Brain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.debug_dump(), f)
    }
}

/// The brain's share of vanilla's `DebugBrainDump`: active activities, running behaviors and
/// one line per registered memory, in the formats the debug subscription sends to the client.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BrainDebugDump {
    pub activities: Vec<&'static str>,
    pub behaviors: Vec<String>,
    pub memories: Vec<String>,
}

impl fmt::Display for BrainDebugDump {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "active: [{}]", self.activities.join(", "))?;
        writeln!(f, "running: [{}]", self.behaviors.join(", "))?;
        for memory in &self.memories {
            writeln!(f, "  {memory}")?;
        }
        Ok(())
    }
}

pub struct BrainProvider {
    pub memory_types: &'static [MemoryModuleId],
    pub sensor_types: &'static [SensorType],
    pub activities: fn(&dyn Mob) -> Vec<ActivityData>,
}

impl BrainProvider {
    #[must_use]
    pub fn make_brain(&self, mob: &dyn Mob, packed: &PackedMemories) -> Brain {
        let mut brain = Self::empty_brain();
        for id in self.memory_types {
            brain.memories.register(*id);
        }
        let mut rng = mob.get_random();
        for ty in self.sensor_types {
            let sensor = SensorEntry::new(*ty, &mut rng);
            for id in sensor.requires() {
                brain.memories.register(*id);
            }
            brain.sensors.push(sensor);
        }
        for data in (self.activities)(mob) {
            brain.add_activity(data);
        }
        brain.load_packed(packed);
        brain.set_core_activities(ActivitySet::single(Activity::Core));
        brain.use_default_activity();
        brain
    }

    fn empty_brain() -> Brain {
        Brain {
            memories: MemoryStore::default(),
            sensors: Vec::new(),
            behaviors: Vec::new(),
            activity_requirements: Vec::new(),
            activity_memories_to_erase: Vec::new(),
            core_activities: ActivitySet::default(),
            active_activities: ActivitySet::default(),
            default_activity: Activity::Idle,
        }
    }
}

#[cfg(test)]
mod tests {
    use behavior::{BehaviorControl, DoNothing, OneShot};
    use memory::types;

    use super::*;

    fn named(name: &'static str) -> Box<dyn BehaviorControl> {
        Box::new(OneShot::new(name, Vec::new(), |_| true))
    }

    fn order(brain: &Brain) -> Vec<String> {
        brain
            .behaviors
            .iter()
            .map(|entry| entry.behavior.debug_string())
            .collect()
    }

    #[test]
    fn a_default_brain_is_brain_dead_and_idle() {
        let brain = Brain::default();
        assert!(brain.is_brain_dead());
        assert!(brain.is_active(Activity::Core));
        assert!(brain.is_active(Activity::Idle));
        assert_eq!(brain.get_active_non_core_activity(), Some(Activity::Idle));
        assert!(brain.pack().into_nbt().get_compound("memories").is_some());
    }

    #[test]
    fn add_activity_registers_the_required_memories_of_its_behaviors() {
        let mut brain = Brain::default();
        let behavior = OneShot::new(
            "Admire",
            vec![(types::ADMIRING_ITEM.id(), MemoryStatus::ValuePresent)],
            |_| true,
        );
        brain.add_activity(ActivityData::with_priority_start(
            Activity::AdmireItem,
            0,
            vec![Box::new(behavior)],
        ));
        assert!(brain.check(types::ADMIRING_ITEM.id(), MemoryStatus::Registered));
        assert!(!brain.is_brain_dead());
    }

    #[test]
    fn behaviors_sort_by_priority_and_keep_insertion_order_within_one() {
        let mut brain = Brain::default();
        brain.add_activity(ActivityData::with_pairs(
            Activity::Idle,
            vec![(5, named("late")), (1, named("early")), (5, named("later"))],
        ));
        brain.add_activity(ActivityData::with_pairs(
            Activity::Fight,
            vec![(5, named("latest")), (0, named("earliest"))],
        ));
        assert_eq!(
            order(&brain),
            vec!["earliest", "early", "late", "later", "latest"]
        );
    }

    #[test]
    fn an_activity_with_unmet_requirements_falls_back_to_the_default() {
        let mut brain = Brain::default();
        brain.add_activity(ActivityData::with_gate_memory(
            Activity::AdmireItem,
            0,
            Vec::new(),
            types::ADMIRING_ITEM.id(),
        ));
        brain.register_memory(types::ADMIRING_ITEM.id());
        brain.set_active_activity_if_possible(Activity::AdmireItem);
        assert!(brain.is_active(Activity::Idle));
        assert!(!brain.is_active(Activity::AdmireItem));

        brain.set(types::ADMIRING_ITEM, true);
        brain.set_active_activity_if_possible(Activity::AdmireItem);
        assert!(brain.is_active(Activity::AdmireItem));
        assert!(brain.is_active(Activity::Core));
        assert!(!brain.is_active(Activity::Idle));
    }

    #[test]
    fn switching_activity_erases_the_previous_activitys_erase_set() {
        let mut brain = Brain::default();
        brain.add_activity(ActivityData::with_gate_memory(
            Activity::AdmireItem,
            0,
            Vec::new(),
            types::ADMIRING_ITEM.id(),
        ));
        brain.add_activity(ActivityData::with_priority_start(
            Activity::Fight,
            0,
            Vec::new(),
        ));
        brain.register_memory(types::ADMIRING_ITEM.id());
        brain.set(types::ADMIRING_ITEM, true);
        brain.set_active_activity_if_possible(Activity::AdmireItem);
        assert!(brain.has_memory_value(types::ADMIRING_ITEM.id()));

        brain.set_active_activity_if_possible(Activity::Fight);
        assert!(brain.is_active(Activity::Fight));
        assert!(!brain.has_memory_value(types::ADMIRING_ITEM.id()));
    }

    #[test]
    fn first_valid_picks_the_first_satisfiable_activity() {
        let mut brain = Brain::default();
        brain.add_activity(ActivityData::with_gate_memory(
            Activity::AdmireItem,
            0,
            Vec::new(),
            types::ADMIRING_ITEM.id(),
        ));
        brain.add_activity(ActivityData::with_priority_start(
            Activity::Fight,
            0,
            Vec::new(),
        ));
        brain.set_active_activity_to_first_valid(&[
            Activity::AdmireItem,
            Activity::Fight,
            Activity::Idle,
        ]);
        assert!(brain.is_active(Activity::Fight));
    }

    #[test]
    fn requirements_of_an_activity_that_was_never_added_are_never_met() {
        let brain = Brain::default();
        assert!(!brain.activity_requirements_are_met(Activity::Fight));
    }

    #[test]
    fn debug_dump_reports_active_activities_and_expiring_memories() {
        let mut brain = Brain::default();
        brain.add_activity(ActivityData::with_priority_start(
            Activity::Idle,
            0,
            vec![Box::new(DoNothing::new(30, 60))],
        ));
        brain.register_memory(types::HUNTED_RECENTLY.id());
        brain.set_with_expiry(types::HUNTED_RECENTLY, true, 119);
        let dump = brain.debug_dump();
        assert_eq!(dump.activities, vec!["core", "idle"]);
        assert!(dump.behaviors.is_empty());
        assert_eq!(dump.memories, vec!["hunted_recently: true (ttl: 119)"]);
        assert!(dump.to_string().starts_with("active: [core, idle]\n"));
    }
}
