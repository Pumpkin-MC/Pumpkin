//! Port of `net.minecraft.world.entity.ai.Brain` / `ActivityData` /
//! `net.minecraft.world.entity.schedule.Activity` (26.2 decompile).
//!
//! # Why this is not the Goal system
//!
//! Modern vanilla AI (villager, piglin, allay, warden, frog, camel, ...) is Brain-driven, not
//! Goal-driven. The two arbitrate completely differently and MUST NOT be conflated:
//!
//! - `GoalSelector` (`crate::entity::ai::goal::goal_selector`) gives goals mutual exclusion by
//!   control bit (`goals_by_control: [usize; 4]`): at most one goal owns MOVE at a time.
//! - `Brain` has no such thing. `Brain.startEachNonRunningBehavior` (`Brain.java:409-424`)
//!   walks the whole priority `TreeMap` and calls `tryStart` on *every* stopped behavior in
//!   *every* active activity whose memory conditions hold. Several behaviors run concurrently
//!   in the same tick. Contention is resolved by **memory ownership** instead: many behaviors
//!   write `WALK_TARGET`, and exactly one terminal sink (`MoveToTargetSink`) reads it and
//!   drains it into the navigator. The priority map only orders the `tryStart` attempts within
//!   a tick; it is not exclusivity.
//!
//! Do not "fix" the behavior loop below by adding `Controls`-style slots. That would silently
//! change arbitration semantics away from vanilla.
//!
//! # Concurrency
//!
//! A `Brain` is deliberately split into two independently locked halves:
//!
//! - [`MemoryStore`] stays live behind its own short-held mutex. Memories are written from
//!   outside the owning mob's AI tick -- `LivingEntity::damage_with_context` writes `HURT_BY`,
//!   game-event listeners write note-block memories -- so taking the memory map out of its
//!   mutex with `mem::take` for the duration of the tick would drop those writes onto a
//!   throwaway `Default` and lose them silently.
//! - [`BrainRuntime`] (sensors, behavior tables, active activities) IS taken out with
//!   `std::mem::take` for the tick, exactly like `GoalSelector` in `Mob::tick`
//!   (`entity/mob/mod.rs:967-985`), because it genuinely has a single owner during a tick.
//!
//! Lock order, leaf-last: `runtime` -> `memory`, and `memory` is never held while taking the
//! mob's `navigator` / `look_control` / `move_control` locks. The runtime take is guarded so
//! an interrupted sensor await restores the live runtime instead of losing it.

pub mod behavior;
pub mod memory;
pub mod sensor;

use std::collections::BTreeMap;
use std::ops::{Deref, DerefMut};
use std::sync::Mutex;

use crate::entity::mob::Mob;

use behavior::{Behavior, BehaviorStatus};
use memory::{MemoryKey, MemoryKeyId, MemoryStatus, MemoryStore};
use sensor::Sensor;

/// `net.minecraft.world.entity.schedule.Activity`. Vanilla registers 27 constants; only the
/// two Allay uses are declared here. Adding an activity is additive and does not affect the
/// runtime loop.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Activity {
    Core,
    Idle,
}

/// `ActivityData<E>` (`ActivityData.java`): the immutable per-mob-type description of one
/// activity, built once at brain construction the way vanilla's `*Ai.getActivities()` static
/// methods do.
pub struct ActivityData {
    pub activity: Activity,
    pub priority: u32,
    pub behaviors: Vec<Box<dyn Behavior>>,
    /// `Brain.activityRequirements` entry, checked by `activityRequirementsAreMet`.
    pub conditions: Vec<(MemoryKeyId, MemoryStatus)>,
    /// `Brain.activityMemoriesToEraseWhenStopped` entry.
    pub memories_to_erase_when_stopped: Vec<MemoryKeyId>,
}

impl ActivityData {
    #[must_use]
    pub fn create(activity: Activity, priority: u32, behaviors: Vec<Box<dyn Behavior>>) -> Self {
        Self {
            activity,
            priority,
            behaviors,
            conditions: Vec::new(),
            memories_to_erase_when_stopped: Vec::new(),
        }
    }
}

/// `availableBehaviorsByPriority: TreeMap<Integer, Map<Activity, Set<BehaviorControl>>>`
/// (`Brain.java:42`). `BTreeMap` reproduces the ascending-priority iteration order that
/// `startEachNonRunningBehavior` relies on.
type BehaviorsByPriority = BTreeMap<u32, Vec<(Activity, Box<dyn Behavior>)>>;

/// Runtime half of a `Brain`: `Brain.java:41-42,47` minus the memory map.
///
/// Must implement `Default` so `std::mem::take` can lift it out of its mutex for the tick.
#[derive(Default)]
pub struct BrainRuntime {
    sensors: Vec<Box<dyn Sensor>>,
    behaviors_by_priority: BehaviorsByPriority,
    /// `Brain.activeActivities` (`Brain.java:47`).
    active_activities: Vec<Activity>,
}

/// `Brain<E>` (`Brain.java:40-49`), split per the module comment.
pub struct Brain {
    /// Always live. Short-held lock only; never `mem::take`n.
    pub memory: Mutex<MemoryStore>,
    /// Taken out for the duration of the tick.
    runtime: Mutex<BrainRuntime>,
    /// Immutable after construction, so activity switching never has to nest the two locks.
    core_activities: Vec<Activity>,
    activity_requirements: Vec<(Activity, Vec<(MemoryKeyId, MemoryStatus)>)>,
    activity_memories_to_erase: Vec<(Activity, Vec<MemoryKeyId>)>,
    default_activity: Activity,
}

struct RuntimeTakeGuard<'a> {
    mutex: &'a Mutex<BrainRuntime>,
    runtime: Option<BrainRuntime>,
}

impl RuntimeTakeGuard<'_> {
    fn new(mutex: &Mutex<BrainRuntime>) -> RuntimeTakeGuard<'_> {
        let runtime = std::mem::take(&mut *mutex.lock().unwrap());
        RuntimeTakeGuard {
            mutex,
            runtime: Some(runtime),
        }
    }
}

impl Deref for RuntimeTakeGuard<'_> {
    type Target = BrainRuntime;

    fn deref(&self) -> &Self::Target {
        self.runtime.as_ref().unwrap()
    }
}

impl DerefMut for RuntimeTakeGuard<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.runtime.as_mut().unwrap()
    }
}

impl Drop for RuntimeTakeGuard<'_> {
    fn drop(&mut self) {
        if let Some(runtime) = self.runtime.take() {
            *self.mutex.lock().unwrap() = runtime;
        }
    }
}

impl Brain {
    /// `Brain(memoryTypes, sensorTypes, activities, ...)` (`Brain.java:71-100`): registers the
    /// memories each sensor writes and each behavior requires, installs the activity tables,
    /// then sets `CORE` as the core activity set and runs the default activity.
    #[must_use]
    pub fn new(sensors: Vec<Box<dyn Sensor>>, activities: Vec<ActivityData>) -> Self {
        let mut store = MemoryStore::new();
        for sensor in &sensors {
            for id in sensor.requires() {
                store.register(*id);
            }
        }

        let mut behaviors_by_priority: BehaviorsByPriority = BTreeMap::new();
        let mut activity_requirements = Vec::new();
        let mut activity_memories_to_erase = Vec::new();

        for activity_data in activities {
            let ActivityData {
                activity,
                priority,
                behaviors,
                conditions,
                memories_to_erase_when_stopped,
            } = activity_data;

            for (id, _) in &conditions {
                store.register(*id);
            }
            activity_requirements.push((activity, conditions));
            if !memories_to_erase_when_stopped.is_empty() {
                activity_memories_to_erase.push((activity, memories_to_erase_when_stopped));
            }

            for behavior in behaviors {
                // Brain.addActivity registers every memory a behavior declares as required,
                // which is what makes MemoryStatus::Registered meaningful (Brain.java:363-366).
                for (id, _) in behavior.required_memories() {
                    store.register(*id);
                }
                behaviors_by_priority
                    .entry(priority)
                    .or_default()
                    .push((activity, behavior));
            }
        }

        let brain = Self {
            memory: Mutex::new(store),
            runtime: Mutex::new(BrainRuntime {
                sensors,
                behaviors_by_priority,
                active_activities: Vec::new(),
            }),
            core_activities: vec![Activity::Core],
            activity_requirements,
            activity_memories_to_erase,
            default_activity: Activity::Idle,
        };
        brain.use_default_activity();
        brain
    }

    // --- memory accessors -------------------------------------------------------------
    //
    // Each of these takes the memory lock for the duration of exactly one field access. This
    // is the only sanctioned way for behaviors and external (non-AI-tick) code to touch
    // memory; it makes "never hold the memory lock across an await, never hold it while
    // taking a controller lock" structurally hard to get wrong.

    pub fn register<K: MemoryKey>(&self) {
        self.memory.lock().unwrap().register(K::ID);
    }

    /// Cloning read. Vanilla returns the live object; every memory value in this stage is
    /// cheap to clone (`WalkTarget`, `PositionTracker`, `Weak`, primitives).
    #[must_use]
    pub fn get<K: MemoryKey>(&self) -> Option<K::Value>
    where
        K::Value: Clone,
    {
        self.memory.lock().unwrap().get::<K>().cloned()
    }

    pub fn set<K: MemoryKey>(&self, value: K::Value) {
        self.memory.lock().unwrap().set::<K>(value);
    }

    pub fn set_with_expiry<K: MemoryKey>(&self, value: K::Value, time_to_live: i64) {
        self.memory
            .lock()
            .unwrap()
            .set_with_expiry::<K>(value, time_to_live);
    }

    pub fn erase<K: MemoryKey>(&self) {
        self.memory.lock().unwrap().erase::<K>();
    }

    #[must_use]
    pub fn has_value<K: MemoryKey>(&self) -> bool {
        self.memory.lock().unwrap().has_value::<K>()
    }

    /// `Behavior.hasRequiredMemories` (`behavior/Behavior.java:97-107`): every entry condition
    /// must hold. Checked under a single lock acquisition.
    #[must_use]
    pub fn check_memories(&self, conditions: &[(MemoryKeyId, MemoryStatus)]) -> bool {
        let store = self.memory.lock().unwrap();
        conditions
            .iter()
            .all(|(id, status)| store.check(*id, *status))
    }

    // --- activities -------------------------------------------------------------------

    fn activity_requirements_are_met(&self, activity: Activity) -> bool {
        // Brain.activityRequirementsAreMet (Brain.java:434+): an activity with no registered
        // requirement entry at all returns false.
        let Some((_, conditions)) = self
            .activity_requirements
            .iter()
            .find(|(candidate, _)| *candidate == activity)
        else {
            return false;
        };
        self.check_memories(conditions)
    }

    #[must_use]
    pub fn is_active(&self, activity: Activity) -> bool {
        self.runtime
            .lock()
            .unwrap()
            .active_activities
            .contains(&activity)
    }

    /// `Brain.setActiveActivity` (`Brain.java:305-312`) plus
    /// `eraseMemoriesForOtherActivitesThan` (`:314-325`). CORE stays active alongside exactly
    /// one non-core activity.
    ///
    /// The two locks are taken in sequence, never nested: the runtime lock is released before
    /// the memory erasures run.
    fn set_active_activity(&self, activity: Activity) {
        let to_erase: Vec<MemoryKeyId> = {
            let mut runtime = self.runtime.lock().unwrap();
            if runtime.active_activities.contains(&activity) {
                return;
            }
            let to_erase = runtime
                .active_activities
                .iter()
                .filter(|old| **old != activity)
                .filter_map(|old| {
                    self.activity_memories_to_erase
                        .iter()
                        .find(|(candidate, _)| candidate == old)
                })
                .flat_map(|(_, ids)| ids.iter().copied())
                .collect();

            runtime.active_activities.clear();
            runtime
                .active_activities
                .extend_from_slice(&self.core_activities);
            runtime.active_activities.push(activity);
            to_erase
        };

        if !to_erase.is_empty() {
            let mut store = self.memory.lock().unwrap();
            for id in to_erase {
                store.erase_by_id(id);
            }
        }
    }

    /// `Brain.useDefaultActivity` (`Brain.java:284-286`).
    pub fn use_default_activity(&self) {
        self.set_active_activity(self.default_activity);
    }

    /// `Brain.setActiveActivityIfPossible` (`Brain.java:297-303`).
    pub fn set_active_activity_if_possible(&self, activity: Activity) {
        if self.activity_requirements_are_met(activity) {
            self.set_active_activity(activity);
        } else {
            self.use_default_activity();
        }
    }

    /// `Brain.setActiveActivityToFirstValid` (`Brain.java:337-344`). This is what
    /// `AllayAi.updateActivity` calls; no schedule subsystem is involved.
    pub fn set_active_activity_to_first_valid(&self, activities: &[Activity]) {
        for activity in activities {
            if self.activity_requirements_are_met(*activity) {
                self.set_active_activity(*activity);
                break;
            }
        }
    }

    // --- tick -------------------------------------------------------------------------

    /// `Brain.tick` (`Brain.java:384-389`): expire memories, tick sensors, start every
    /// non-running behavior whose activity is active and whose memory gate holds, then tick
    /// every running behavior.
    ///
    /// `game_time` is the caller's tick timestamp; see `Mob::tick` for which clock is used and
    /// why it is not vanilla's `level.getGameTime()`.
    pub async fn tick(&self, mob: &dyn Mob, game_time: i64) {
        self.memory.lock().unwrap().tick_expiry();

        // Take the runtime out of its mutex so sensor `.await`s hold no lock. The memory half
        // deliberately stays behind, live, accepting writes from damage/game-event code that
        // lands during this window.
        let mut runtime = RuntimeTakeGuard::new(&self.runtime);

        for sensor in &mut runtime.sensors {
            sensor.tick(mob, self).await;
        }
        runtime.start_each_non_running_behavior(mob, self, game_time);
        runtime.tick_each_running_behavior(mob, self, game_time);
    }

    /// `Brain.stopAll` (`Brain.java:401-407`).
    pub fn stop_all(&self, mob: &dyn Mob, game_time: i64) {
        let mut runtime = RuntimeTakeGuard::new(&self.runtime);
        for behaviors in runtime.behaviors_by_priority.values_mut() {
            for (_, behavior) in behaviors.iter_mut() {
                if behavior.status() == BehaviorStatus::Running {
                    behavior.do_stop(mob, self, game_time);
                }
            }
        }
    }
}

impl BrainRuntime {
    /// `Brain.startEachNonRunningBehavior` (`Brain.java:409-424`).
    ///
    /// Every stopped behavior in every active activity gets a `try_start`, in ascending
    /// priority order. There is no early exit and no per-control exclusion: two behaviors can
    /// both start in the same tick.
    fn start_each_non_running_behavior(&mut self, mob: &dyn Mob, brain: &Brain, game_time: i64) {
        for behaviors in self.behaviors_by_priority.values_mut() {
            for (activity, behavior) in behaviors.iter_mut() {
                if !self.active_activities.contains(activity) {
                    continue;
                }
                if behavior.status() == BehaviorStatus::Stopped {
                    behavior.try_start(mob, brain, game_time);
                }
            }
        }
    }

    /// `Brain.tickEachRunningBehavior` (`Brain.java:426-432`).
    fn tick_each_running_behavior(&mut self, mob: &dyn Mob, brain: &Brain, game_time: i64) {
        for behaviors in self.behaviors_by_priority.values_mut() {
            for (_, behavior) in behaviors.iter_mut() {
                if behavior.status() == BehaviorStatus::Running {
                    behavior.tick_or_stop(mob, brain, game_time);
                }
            }
        }
    }
}
