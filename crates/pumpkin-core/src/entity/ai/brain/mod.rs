use std::collections::BTreeMap;
use std::fmt;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use pumpkin_data::attributes::Attributes;
use pumpkin_data::environment_attribute::Activity;

use crate::entity::mob::Mob;
use crate::world::World;

use activity::{ActivityData, ActivitySet, activity_slot};
use behavior::{ActivityBehaviors, Status};
use memory::{
    MemoryModuleId, MemoryModuleType, MemoryStatus, MemoryStore, MemoryValue, PackedMemories,
};
use sensing::{SensorEntry, SensorType};

pub mod activity;
pub mod behavior;
pub mod memory;
pub mod sensing;

/// Memory and activity state borrowed by the current sensor or behavior.
/// Use this state rather than calling `MobEntity::with_brain` from a callback.
pub struct BrainTick<'a> {
    pub state: &'a mut BrainState,
    pub world: &'a Arc<World>,
    pub mob: &'a dyn Mob,
    pub time: i64,
}

impl BrainTick<'_> {
    #[must_use]
    pub fn visibility(&self) -> VisibilityContext<'_> {
        VisibilityContext {
            state: self.state,
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
    pub state: &'a BrainState,
    pub world: &'a Arc<World>,
    pub mob: &'a dyn Mob,
    pub follow_range: f64,
}

type ActivityConditions = Box<[(MemoryModuleId, MemoryStatus)]>;

#[derive(Default)]
pub struct Brain {
    state: BrainState,
    sensors: Vec<SensorEntry>,
    behaviors: BTreeMap<i32, Vec<ActivityBehaviors>>,
}

pub struct BrainState {
    memories: MemoryStore,
    activity_requirements: Vec<Option<ActivityConditions>>,
    activity_memories_to_erase: Vec<Option<Box<[MemoryModuleId]>>>,
    core_activities: ActivitySet,
    active_activities: ActivitySet,
    default_activity: Activity,
}

impl Default for BrainState {
    fn default() -> Self {
        let mut state = Self {
            memories: MemoryStore::default(),
            activity_requirements: Vec::new(),
            activity_memories_to_erase: Vec::new(),
            core_activities: ActivitySet::single(Activity::Core),
            active_activities: ActivitySet::default(),
            default_activity: Activity::Idle,
        };
        state.use_default_activity();
        state
    }
}

impl Deref for Brain {
    type Target = BrainState;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl DerefMut for Brain {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}

impl BrainState {
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
}

impl Brain {
    pub fn add_activity(&mut self, data: ActivityData) {
        let index = data.activity as usize;
        let slots = activity::ALL_ACTIVITIES.len();
        if self.state.activity_requirements.len() < slots {
            self.state.activity_requirements.resize_with(slots, || None);
            self.state
                .activity_memories_to_erase
                .resize_with(slots, || None);
        }
        self.state.activity_requirements[index] = Some(data.conditions.into_boxed_slice());
        if !data.memories_to_erase_when_stopped.is_empty() {
            self.state.activity_memories_to_erase[index] =
                Some(data.memories_to_erase_when_stopped.into_boxed_slice());
        }
        for (priority, behavior) in data.behaviors {
            for id in behavior.required_memories() {
                self.state.memories.register(*id);
            }
            let activities = self.behaviors.entry(priority).or_default();
            let group = activities
                .iter()
                .position(|group| group.activity == data.activity)
                .unwrap_or_else(|| {
                    activities.push(ActivityBehaviors {
                        activity: data.activity,
                        behaviors: Vec::new(),
                    });
                    activities.len() - 1
                });
            activities[group].behaviors.push(behavior);
            let count = activities.len();
            activities.sort_by_key(|group| activity_slot(group.activity, count));
        }
    }

    fn apply_pending_memory_clear(state: &mut BrainState, mob: &dyn Mob) {
        if mob.get_mob_entity().take_brain_memory_clear_request() {
            state.clear_memories();
        }
    }

    pub fn tick(&mut self, world: &Arc<World>, mob: &dyn Mob, time: i64) {
        Self::apply_pending_memory_clear(&mut self.state, mob);
        self.state.memories.tick();

        for sensor in &mut self.sensors {
            sensor.tick(&mut BrainTick {
                state: &mut self.state,
                world,
                mob,
                time,
            });
            Self::apply_pending_memory_clear(&mut self.state, mob);
        }

        for group in self.behaviors.values_mut().flatten() {
            if !self.state.is_active(group.activity) {
                continue;
            }
            for behavior in &mut group.behaviors {
                if behavior.status() != Status::Stopped {
                    continue;
                }
                behavior.try_start(&mut BrainTick {
                    state: &mut self.state,
                    world,
                    mob,
                    time,
                });
                Self::apply_pending_memory_clear(&mut self.state, mob);
            }
        }
        for behavior in self
            .behaviors
            .values_mut()
            .flatten()
            .flat_map(|group| &mut group.behaviors)
        {
            if behavior.status() == Status::Running {
                behavior.tick_or_stop(&mut BrainTick {
                    state: &mut self.state,
                    world,
                    mob,
                    time,
                });
                Self::apply_pending_memory_clear(&mut self.state, mob);
            }
        }
    }

    pub fn stop_all(&mut self, world: &Arc<World>, mob: &dyn Mob, time: i64) {
        for behavior in self
            .behaviors
            .values_mut()
            .flatten()
            .flat_map(|group| &mut group.behaviors)
        {
            if behavior.status() == Status::Running {
                behavior.do_stop(&mut BrainTick {
                    state: &mut self.state,
                    world,
                    mob,
                    time,
                });
                Self::apply_pending_memory_clear(&mut self.state, mob);
            }
        }
    }

    #[must_use]
    pub fn is_brain_dead(&self) -> bool {
        self.state.memories.is_empty() && self.sensors.is_empty() && self.behaviors.is_empty()
    }

    #[must_use]
    pub fn pack(&self) -> PackedMemories {
        self.state.memories.pack()
    }

    pub fn load_packed(&mut self, packed: &PackedMemories) {
        self.state.memories.load_packed(packed);
    }

    #[must_use]
    pub fn debug_dump(&self) -> BrainDebugDump {
        BrainDebugDump {
            activities: self
                .state
                .active_activities
                .iter()
                .map(|activity| activity.name())
                .collect(),
            behaviors: self
                .behaviors
                .values()
                .flatten()
                .flat_map(|group| &group.behaviors)
                .filter(|behavior| behavior.status() == Status::Running)
                .map(|behavior| behavior.debug_string())
                .collect(),
            memories: self.state.memories.debug_lines(),
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
            state: BrainState {
                core_activities: ActivitySet::default(),
                active_activities: ActivitySet::default(),
                ..BrainState::default()
            },
            sensors: Vec::new(),
            behaviors: BTreeMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use behavior::{Behavior, BehaviorControl, DoNothing, OneShot, Timed};
    use memory::types;

    use crate::entity::{Entity, EntityBase, mob::MobEntity};

    use super::*;
    use crate::entity::ai::test_support::world;
    use pumpkin_data::entity::EntityType;
    use pumpkin_util::math::vector3::Vector3;

    struct ProbeMob(MobEntity);

    impl Mob for ProbeMob {
        fn get_mob_entity(&self) -> &MobEntity {
            &self.0
        }
    }

    fn probe(folder: &tempfile::TempDir) -> (Arc<World>, ProbeMob) {
        let world = world(folder);
        let mob = ProbeMob(MobEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.0, 64.0, 0.0),
            &EntityType::PIGLIN,
        )));
        (world, mob)
    }

    struct StopProbe(Arc<AtomicUsize>);

    impl Behavior for StopProbe {
        fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
            &[]
        }

        fn can_still_use(&mut self, _tick: &mut BrainTick<'_>) -> bool {
            true
        }

        fn stop(&mut self, tick: &mut BrainTick<'_>) {
            self.0.fetch_add(1, Ordering::Relaxed);
            tick.state.set(types::PACIFIED, true);
        }

        fn debug_name(&self) -> &'static str {
            "StopProbe"
        }
    }

    fn named(name: &'static str) -> Box<dyn BehaviorControl> {
        Box::new(OneShot::new(name, Vec::new(), |_| true))
    }

    fn order(brain: &Brain) -> Vec<String> {
        brain
            .behaviors
            .values()
            .flatten()
            .flat_map(|group| &group.behaviors)
            .map(|behavior| behavior.debug_string())
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

    #[tokio::test]
    async fn callbacks_observe_expiry_and_same_tick_activity_changes()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let (world, mob) = probe(&folder);
        let events = Arc::new(Mutex::new(Vec::new()));
        let mut brain = Brain::default();
        brain.register_memory(types::PACIFIED.id());
        brain.register_memory(types::HUNTED_RECENTLY.id());
        brain.set_with_expiry(types::HUNTED_RECENTLY, true, 0);
        let switch_events = events.clone();
        let same_group_events = events.clone();
        brain.add_activity(ActivityData::with_pairs(
            Activity::Idle,
            vec![
                (
                    0,
                    Box::new(OneShot::new("Switch", Vec::new(), move |tick| {
                        switch_events
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner)
                            .push("switch");
                        tick.state.set(types::PACIFIED, true);
                        tick.state.set_active_activity_if_possible(Activity::Fight);
                        true
                    })),
                ),
                (
                    0,
                    Box::new(OneShot::new("SameGroup", Vec::new(), move |_| {
                        same_group_events
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner)
                            .push("same_group");
                        true
                    })),
                ),
            ],
        ));
        let fight_events = events.clone();
        brain.add_activity(ActivityData::with_priority_start(
            Activity::Fight,
            1,
            vec![Box::new(OneShot::new(
                "Fight",
                vec![
                    (types::PACIFIED.id(), MemoryStatus::ValuePresent),
                    (types::HUNTED_RECENTLY.id(), MemoryStatus::ValueAbsent),
                ],
                move |_| {
                    fight_events
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .push("fight");
                    true
                },
            ))],
        ));
        let stale_events = events.clone();
        brain.add_activity(ActivityData::with_priority_start(
            Activity::Idle,
            2,
            vec![Box::new(OneShot::new("Stale", Vec::new(), move |_| {
                stale_events
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .push("stale");
                true
            }))],
        ));

        brain.tick(&world, &mob, 1);
        world.level.shutdown().await;

        assert_eq!(
            *events
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
            ["switch", "same_group", "fight"]
        );
        assert!(brain.is_active(Activity::Fight));
        assert!(!brain.has_memory_value(types::HUNTED_RECENTLY.id()));
        Ok(())
    }

    #[tokio::test]
    async fn stop_all_stops_running_behaviors_once_and_keeps_their_state_writes()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let (world, mob) = probe(&folder);
        let stops = Arc::new(AtomicUsize::new(0));
        let mut brain = Brain::default();
        brain.register_memory(types::PACIFIED.id());
        brain.add_activity(ActivityData::with_priority_start(
            Activity::Idle,
            0,
            vec![Box::new(Timed::new(StopProbe(stops.clone())))],
        ));

        brain.tick(&world, &mob, 1);
        let running = brain.debug_dump().behaviors;
        brain.stop_all(&world, &mob, 2);
        brain.stop_all(&world, &mob, 2);
        world.level.shutdown().await;

        assert_eq!(running, ["StopProbe"]);
        assert_eq!(stops.load(Ordering::Relaxed), 1);
        assert_eq!(brain.get(types::PACIFIED), Some(&true));
        assert!(brain.debug_dump().behaviors.is_empty());
        Ok(())
    }
    #[tokio::test]
    async fn activities_of_one_priority_run_in_vanilla_hash_map_order()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let (world, mob) = probe(&folder);
        let started = Arc::new(Mutex::new(Vec::new()));
        let mut brain = Brain::default();
        for (activity, name) in [
            (Activity::Fight, "fight"),
            (Activity::Core, "core"),
            (Activity::Idle, "idle"),
        ] {
            let started = started.clone();
            brain.add_activity(ActivityData::with_priority_start(
                activity,
                0,
                vec![Box::new(OneShot::new(name, Vec::new(), move |_| {
                    started
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .push(name);
                    true
                }))],
            ));
        }
        let mut core = ActivitySet::single(Activity::Core);
        core.insert(Activity::Idle);
        brain.set_core_activities(core);
        brain.set_active_activity_if_possible(Activity::Fight);
        brain.tick(&world, &mob, 1);
        world.level.shutdown().await;
        assert_eq!(
            *started
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
            ["core", "idle", "fight"]
        );
        Ok(())
    }

    struct ProviderMob(MobEntity, Arc<AtomicUsize>);

    impl Mob for ProviderMob {
        fn get_mob_entity(&self) -> &MobEntity {
            &self.0
        }

        fn make_brain(&self, packed: &PackedMemories) -> Brain {
            let mut brain = Brain::default();
            brain.register_memory(types::HOME.id());
            brain.register_memory(types::PACIFIED.id());
            brain.set(
                types::HOME,
                memory::GlobalPos::new(
                    &pumpkin_data::dimension::Dimension::OVERWORLD,
                    pumpkin_util::math::position::BlockPos::new(0, 64, 0),
                ),
            );
            brain.add_activity(ActivityData::with_priority_start(
                Activity::Idle,
                0,
                vec![Box::new(Timed::new(StopProbe(self.1.clone())))],
            ));
            brain.load_packed(packed);
            brain
        }
    }

    #[tokio::test]
    async fn serialization_initializes_the_species_brain() -> Result<(), Box<dyn std::error::Error>>
    {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let mob = ProviderMob(
            MobEntity::new(Entity::new(
                world.clone(),
                Vector3::new(0.0, 64.0, 0.0),
                &EntityType::PIGLIN,
            )),
            Arc::default(),
        );
        let mut nbt = pumpkin_nbt::compound::NbtCompound::new();
        mob.write_custom_nbt(&mut nbt);
        let saved = nbt
            .get_compound("Brain")
            .and_then(|brain| brain.get_compound("memories"))
            .is_some_and(|memories| memories.get(types::HOME.id().name()).is_some());
        world.level.shutdown().await;
        assert!(saved);
        Ok(())
    }

    #[tokio::test]
    async fn brains_come_from_the_provider_and_refresh_keeps_saved_memories()
    -> Result<(), Box<dyn std::error::Error>> {
        use pumpkin_data::dimension::Dimension;
        use pumpkin_util::math::position::BlockPos;
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let stops = Arc::new(AtomicUsize::new(0));
        let mob = ProviderMob(
            MobEntity::new(Entity::new(
                world.clone(),
                Vector3::new(0.0, 64.0, 0.0),
                &EntityType::PIGLIN,
            )),
            stops.clone(),
        );
        let home = memory::GlobalPos::new(&Dimension::OVERWORLD, BlockPos::new(1, 64, 2));
        let registered = mob.0.with_brain(&mob, |brain| {
            brain.check(types::HOME.id(), MemoryStatus::Registered)
        });
        mob.0.with_brain(&mob, |brain| brain.set(types::HOME, home));
        mob.0.tick_brain(&mob);
        let running = mob.0.with_brain(&mob, |brain| brain.debug_dump().behaviors);
        mob.0.refresh_brain(&mob);
        let (kept_home, running_after) = mob.0.with_brain(&mob, |brain| {
            (
                brain.get(types::HOME) == Some(&home),
                brain.debug_dump().behaviors,
            )
        });
        world.level.shutdown().await;
        assert!(registered);
        assert_eq!(running, ["StopProbe"]);
        assert_eq!(stops.load(Ordering::Relaxed), 1);
        assert!(kept_home);
        assert!(running_after.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn self_removal_clears_memories_before_the_next_brain_callback()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let (world, mob) = probe(&folder);
        let mob = Arc::new(mob);
        world.entities.store(Arc::new(vec![mob.clone()]));
        let observed = Arc::new(AtomicUsize::new(0));
        let mut brain = Brain::default();
        let count = observed.clone();
        brain.add_activity(ActivityData::with_priority_start(
            Activity::Core,
            0,
            vec![
                Box::new(OneShot::new("Remove", Vec::new(), |tick| {
                    tick.mob.get_entity().remove();
                    true
                })),
                Box::new(OneShot::new(
                    "Observe",
                    vec![(types::PACIFIED.id(), MemoryStatus::ValuePresent)],
                    move |_| {
                        count.fetch_add(1, Ordering::Relaxed);
                        true
                    },
                )),
            ],
        ));
        brain.set(types::PACIFIED, true);
        mob.0.set_brain(brain);
        mob.0.tick_brain(&*mob);
        let has_memory = mob
            .0
            .with_brain(&*mob, |brain| brain.has_memory_value(types::PACIFIED.id()));
        world.level.shutdown().await;
        assert!(mob.get_entity().is_removed());
        assert!(!has_memory);
        assert_eq!(observed.load(Ordering::Relaxed), 0);
        Ok(())
    }
}
