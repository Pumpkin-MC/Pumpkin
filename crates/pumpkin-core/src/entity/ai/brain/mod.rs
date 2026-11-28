use std::collections::BTreeMap;
use std::fmt;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use pumpkin_data::attributes::Attributes;
use pumpkin_data::entity::EntityPose;
use pumpkin_data::environment_attribute::Activity;
use pumpkin_util::math::position::BlockPos;

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

    pub fn set_pose(&mut self, pose: EntityPose) {
        self.defer(move |mob| mob.get_entity().set_pose(pose));
    }

    /// Runs the action outside the brain lock before the next Brain callback.
    pub fn defer(&mut self, action: impl FnOnce(&dyn Mob) + Send + 'static) {
        self.state.effects.push(Box::new(move |mob| {
            action(mob);
            None
        }));
    }

    /// Queues an event and updates the originating Brain's target on approval.
    pub(crate) fn defer_target_change(
        &mut self,
        target: Option<Arc<dyn crate::entity::EntityBase>>,
        reason: crate::plugin::api::events::entity::entity_target::TargetReason,
        apply: impl FnOnce(&mut BrainTick<'_>, Option<Arc<dyn crate::entity::EntityBase>>)
        + Send
        + 'static,
    ) {
        let previous_id = self
            .state
            .get(memory::types::ATTACK_TARGET)
            .map(|target| target.get_entity().entity_id);
        self.defer_with_update(
            move |mob| crate::entity::mob::resolve_target_change(mob, previous_id, target, reason),
            move |tick, decision| {
                if let Ok(target) = decision {
                    apply(tick, target);
                }
            },
        );
    }

    /// Applies the memory update after the action and before the next Brain callback.
    pub(crate) fn defer_with_update<T: Send + 'static>(
        &mut self,
        action: impl FnOnce(&dyn Mob) -> T + Send + 'static,
        update: impl FnOnce(&mut BrainTick<'_>, T) + Send + 'static,
    ) {
        self.state.effects.push(Box::new(move |mob| {
            let result = action(mob);
            Some(Box::new(move |state| update(state, result)))
        }));
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
pub(crate) type BrainEffectUpdate = Box<dyn FnOnce(&mut BrainTick<'_>) + Send>;
pub(crate) type BrainEffect = Box<dyn FnOnce(&dyn Mob) -> Option<BrainEffectUpdate> + Send>;

#[derive(Clone, Copy)]
enum BrainCursor {
    Sensor(usize),
    Start(usize),
    Tick(usize),
    Stop(usize),
}

pub(crate) fn run_brain_effects(
    effects: Vec<BrainEffect>,
    mob: &dyn Mob,
    world: &Arc<World>,
    is_current: impl Fn() -> bool,
) -> Option<Vec<BrainEffectUpdate>> {
    let still_present = || {
        let entity = mob.get_entity();
        !entity.is_removed() && Arc::ptr_eq(&entity.world.load(), world)
    };

    let mut updates = Vec::new();
    for effect in effects {
        if !still_present() || !is_current() {
            return None;
        }
        updates.extend(effect(mob));
    }
    still_present().then_some(updates)
}

#[derive(Default)]
pub struct Brain {
    identity: Option<Arc<()>>,
    state: BrainState,
    sensors: Vec<SensorEntry>,
    behaviors: BTreeMap<i32, Vec<ActivityBehaviors>>,
    next_callback: Option<(BrainCursor, Arc<()>)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Schedule {
    Villager,
    BabyVillager,
}

pub struct BrainState {
    memories: MemoryStore,
    effects: Vec<BrainEffect>,
    activity_requirements: Vec<Option<ActivityConditions>>,
    activity_memories_to_erase: Vec<Option<Box<[MemoryModuleId]>>>,
    core_activities: ActivitySet,
    active_activities: ActivitySet,
    default_activity: Activity,
    schedule: Option<Schedule>,
    last_schedule_update: i64,
}

impl Default for BrainState {
    fn default() -> Self {
        let mut state = Self {
            memories: MemoryStore::default(),
            effects: Vec::new(),
            activity_requirements: Vec::new(),
            activity_memories_to_erase: Vec::new(),
            core_activities: ActivitySet::single(Activity::Core),
            active_activities: ActivitySet::default(),
            default_activity: Activity::Idle,
            schedule: None,
            last_schedule_update: -9999,
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

    pub const fn set_schedule(&mut self, schedule: Option<Schedule>) {
        self.schedule = schedule;
    }

    pub fn use_default_activity(&mut self) {
        self.set_active_activity(self.default_activity);
    }

    pub fn update_activity_from_schedule(&mut self, world: &World, game_time: i64, pos: &BlockPos) {
        if game_time - self.last_schedule_update > 20 {
            self.last_schedule_update = game_time;
            let scheduled_activity = self.schedule.map_or(Activity::Idle, |schedule| {
                world
                    .environment_attributes()
                    .get_value_activity(schedule == Schedule::BabyVillager, pos)
            });
            if !self.active_activities.contains(scheduled_activity) {
                self.set_active_activity_if_possible(scheduled_activity);
            }
        }
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

    /// Ticks a directly owned brain. A mob's stored brain must use `MobEntity::tick_brain`.
    pub fn tick(&mut self, world: &Arc<World>, mob: &dyn Mob, time: i64) {
        let effects = self.begin_tick(world, mob, time);
        self.complete_callbacks(world, mob, time, effects);
    }

    pub fn stop_all(&mut self, world: &Arc<World>, mob: &dyn Mob, time: i64) {
        let effects = self.begin_stop(world, mob, time);
        self.complete_callbacks(world, mob, time, effects);
    }

    fn complete_callbacks(
        &mut self,
        world: &Arc<World>,
        mob: &dyn Mob,
        time: i64,
        mut effects: Vec<BrainEffect>,
    ) {
        while !effects.is_empty() {
            let Some(updates) = run_brain_effects(effects, mob, world, || true) else {
                self.cancel_callbacks();
                return;
            };
            let Some(next) = self.resume_callbacks(world, mob, time, updates) else {
                return;
            };
            effects = next;
        }
    }

    /// Identity used when a deferred effect depends on another mob's installed Brain.
    pub(crate) fn identity(&mut self) -> Arc<()> {
        self.identity.get_or_insert_with(|| Arc::new(())).clone()
    }

    pub(crate) fn has_identity(&self, identity: &Arc<()>) -> bool {
        self.identity
            .as_ref()
            .is_some_and(|current| Arc::ptr_eq(current, identity))
    }

    /// Identity of the suspended callback that owns the current effect batch.
    pub(crate) fn effect_owner(&self) -> Option<&Arc<()>> {
        self.next_callback.as_ref().map(|(_, owner)| owner)
    }

    pub(crate) fn cancel_callbacks(&mut self) -> bool {
        self.next_callback.take().is_some()
    }

    pub(crate) fn begin_tick(
        &mut self,
        world: &Arc<World>,
        mob: &dyn Mob,
        time: i64,
    ) -> Vec<BrainEffect> {
        self.next_callback = None;
        Self::apply_pending_memory_clear(&mut self.state, mob);
        self.state.memories.tick();
        self.tick_sensors(world, mob, time, 0)
    }

    pub(crate) fn begin_stop(
        &mut self,
        world: &Arc<World>,
        mob: &dyn Mob,
        time: i64,
    ) -> Vec<BrainEffect> {
        self.next_callback = None;
        self.advance_running_behaviors(world, mob, time, 0, true)
    }

    /// A callback may rebuild the brain while its lock is released.
    pub(crate) fn resume_callbacks(
        &mut self,
        world: &Arc<World>,
        mob: &dyn Mob,
        time: i64,
        updates: Vec<BrainEffectUpdate>,
    ) -> Option<Vec<BrainEffect>> {
        let (cursor, _) = self.next_callback.take()?;
        for update in updates {
            update(&mut BrainTick {
                state: &mut self.state,
                world,
                mob,
                time,
            });
        }
        Self::apply_pending_memory_clear(&mut self.state, mob);
        let effects = match cursor {
            BrainCursor::Sensor(first) => self.tick_sensors(world, mob, time, first),
            BrainCursor::Start(first) => self.start_behaviors(world, mob, time, first),
            BrainCursor::Tick(first) => {
                self.advance_running_behaviors(world, mob, time, first, false)
            }
            BrainCursor::Stop(first) => {
                self.advance_running_behaviors(world, mob, time, first, true)
            }
        };
        Some(effects)
    }

    fn take_effects_at(&mut self, cursor: BrainCursor) -> Vec<BrainEffect> {
        self.next_callback = Some((cursor, Arc::new(())));
        std::mem::take(&mut self.state.effects)
    }

    fn tick_sensors(
        &mut self,
        world: &Arc<World>,
        mob: &dyn Mob,
        time: i64,
        first: usize,
    ) -> Vec<BrainEffect> {
        for (index, sensor) in self.sensors.iter_mut().enumerate().skip(first) {
            sensor.tick(&mut BrainTick {
                state: &mut self.state,
                world,
                mob,
                time,
            });
            if !self.state.effects.is_empty() {
                return self.take_effects_at(BrainCursor::Sensor(index + 1));
            }
            Self::apply_pending_memory_clear(&mut self.state, mob);
        }
        self.start_behaviors(world, mob, time, 0)
    }

    fn start_behaviors(
        &mut self,
        world: &Arc<World>,
        mob: &dyn Mob,
        time: i64,
        first: usize,
    ) -> Vec<BrainEffect> {
        let mut offset = 0;
        for group in self.behaviors.values_mut().flatten() {
            let start = offset;
            offset += group.behaviors.len();
            if offset <= first || (first <= start && !self.state.is_active(group.activity)) {
                continue;
            }
            let skip = first.saturating_sub(start);
            for (index, behavior) in group.behaviors.iter_mut().enumerate().skip(skip) {
                if behavior.status() != Status::Stopped {
                    continue;
                }
                behavior.try_start(&mut BrainTick {
                    state: &mut self.state,
                    world,
                    mob,
                    time,
                });
                if !self.state.effects.is_empty() {
                    return self.take_effects_at(BrainCursor::Start(start + index + 1));
                }
                Self::apply_pending_memory_clear(&mut self.state, mob);
            }
        }
        self.advance_running_behaviors(world, mob, time, 0, false)
    }

    fn advance_running_behaviors(
        &mut self,
        world: &Arc<World>,
        mob: &dyn Mob,
        time: i64,
        first: usize,
        stop: bool,
    ) -> Vec<BrainEffect> {
        let behaviors = self
            .behaviors
            .values_mut()
            .flatten()
            .flat_map(|group| &mut group.behaviors);
        for (index, behavior) in behaviors.enumerate().skip(first) {
            if behavior.status() != Status::Running {
                continue;
            }
            let mut tick = BrainTick {
                state: &mut self.state,
                world,
                mob,
                time,
            };
            if stop {
                behavior.do_stop(&mut tick);
            } else {
                behavior.tick_or_stop(&mut tick);
            }
            if !self.state.effects.is_empty() {
                let next = index + 1;
                return self.take_effects_at(if stop {
                    BrainCursor::Stop(next)
                } else {
                    BrainCursor::Tick(next)
                });
            }
            Self::apply_pending_memory_clear(&mut self.state, mob);
        }
        Vec::new()
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
            identity: None,
            state: BrainState {
                core_activities: ActivitySet::default(),
                active_activities: ActivitySet::default(),
                ..BrainState::default()
            },
            sensors: Vec::new(),
            behaviors: BTreeMap::new(),
            next_callback: None,
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

        fn stop(&mut self, tick: &mut BrainTick<'_>, _timed_out: bool) {
            let stops = self.0.clone();
            tick.defer_with_update(
                move |_| {
                    stops.fetch_add(1, Ordering::Relaxed);
                },
                |tick, ()| tick.state.set(types::PACIFIED, true),
            );
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
                        tick.defer(|_| {});
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
                    BlockPos::new(0, 64, 0),
                ),
            );
            brain.add_activity(ActivityData::with_priority_start(
                Activity::Idle,
                0,
                vec![
                    Box::new(Timed::new(StopProbe(self.1.clone()))),
                    Box::new(Timed::new(StopProbe(self.1.clone()))),
                ],
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
        assert_eq!(running, ["StopProbe", "StopProbe"]);
        assert_eq!(stops.load(Ordering::Relaxed), 2);
        assert!(kept_home);
        assert!(running_after.is_empty());
        Ok(())
    }

    struct ChangeWorldOnStop(Arc<World>);

    impl Behavior for ChangeWorldOnStop {
        fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
            &[]
        }

        fn can_still_use(&mut self, _tick: &mut BrainTick<'_>) -> bool {
            true
        }

        fn stop(&mut self, tick: &mut BrainTick<'_>, _timed_out: bool) {
            let destination = self.0.clone();
            tick.defer(move |mob| mob.get_entity().world.store(destination));
        }

        fn debug_name(&self) -> &'static str {
            "ChangeWorldOnStop"
        }
    }

    #[tokio::test]
    async fn interrupted_brain_refresh_retries_after_a_world_change()
    -> Result<(), Box<dyn std::error::Error>> {
        let origin_dir = tempfile::tempdir()?;
        let destination_dir = tempfile::tempdir()?;
        let origin = world(&origin_dir);
        let destination = world(&destination_dir);
        for direct in [false, true] {
            let mob = ProviderMob(
                MobEntity::new(Entity::new(
                    origin.clone(),
                    Vector3::new(0.0, 64.0, 0.0),
                    &EntityType::PIGLIN,
                )),
                Arc::default(),
            );
            let rebuilt = || {
                mob.0
                    .with_brain(&mob, |brain| brain.get(types::HOME).is_some())
            };
            let mut brain = Brain::default();
            brain.add_activity(ActivityData::with_priority_start(
                Activity::Idle,
                0,
                vec![Box::new(Timed::new(ChangeWorldOnStop(destination.clone())))],
            ));
            mob.0.set_brain(brain);
            mob.0.tick_brain(&mob);
            assert!(!rebuilt());
            if direct {
                mob.0.refresh_brain(&mob);
            } else {
                mob.0.request_refresh_brain();
                mob.0.tick_brain(&mob);
            }
            assert!(Arc::ptr_eq(&mob.get_entity().world.load(), &destination));
            assert!(!rebuilt());
            mob.0.tick_brain(&mob);
            assert!(rebuilt(), "refresh should retry in the destination world");
        }
        origin.level.shutdown().await;
        destination.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn effect_update_does_not_write_into_a_replacement_brain()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let (world, mob) = probe(&folder);
        let mut brain = Brain::default();
        brain.add_activity(ActivityData::with_priority_start(
            Activity::Core,
            0,
            vec![Box::new(OneShot::new("ReplaceBrain", Vec::new(), |tick| {
                tick.defer_with_update(
                    |mob| {
                        let mut replacement = Brain::default();
                        replacement.register_memory(types::PACIFIED.id());
                        replacement.set(types::PACIFIED, false);
                        mob.get_mob_entity().set_brain(replacement);
                    },
                    |tick, ()| tick.state.set(types::PACIFIED, true),
                );
                true
            }))],
        ));
        mob.0.set_brain(brain);
        mob.0.tick_brain(&mob);
        assert!(
            mob.0
                .with_brain(&mob, |brain| brain.get(types::PACIFIED) == Some(&false))
        );
        world.level.shutdown().await;
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

    #[tokio::test]
    async fn schedule_updates_the_activity_at_most_once_per_twenty_ticks()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let pos = BlockPos::new(0, 64, 0);
        world.set_time_of_day(2000);
        let mut brain = Brain::default();
        brain.add_activity(ActivityData::with_priority_start(
            Activity::Work,
            0,
            Vec::new(),
        ));
        brain.add_activity(ActivityData::with_priority_start(
            Activity::Rest,
            0,
            Vec::new(),
        ));
        brain.set_schedule(Some(Schedule::Villager));

        brain.update_activity_from_schedule(&world, 100, &pos);
        assert!(brain.is_active(Activity::Work));

        world.set_time_of_day(12000);
        brain.update_activity_from_schedule(&world, 110, &pos);
        assert!(brain.is_active(Activity::Work));

        brain.update_activity_from_schedule(&world, 121, &pos);
        world.level.shutdown().await;
        assert!(brain.is_active(Activity::Rest));
        Ok(())
    }

    #[tokio::test]
    async fn without_a_schedule_the_activity_falls_back_to_idle()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let pos = BlockPos::new(0, 64, 0);
        let mut brain = Brain::default();
        brain.add_activity(ActivityData::with_priority_start(
            Activity::Work,
            0,
            Vec::new(),
        ));
        brain.add_activity(ActivityData::with_priority_start(
            Activity::Idle,
            0,
            Vec::new(),
        ));
        brain.set_active_activity_if_possible(Activity::Work);
        assert!(brain.is_active(Activity::Work));

        brain.update_activity_from_schedule(&world, 100, &pos);
        world.level.shutdown().await;
        assert!(brain.is_active(Activity::Idle));
        Ok(())
    }
}
