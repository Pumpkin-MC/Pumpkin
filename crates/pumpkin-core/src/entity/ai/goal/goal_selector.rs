use crate::entity::ai::goal::{Controls, Goal, GoalReach, PrioritizedGoal};
use crate::entity::mob::Mob;
use crossbeam::atomic::AtomicCell;
use std::any::TypeId;
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, TryLockError};

#[derive(Clone, Copy, PartialEq, Eq)]
enum BorrowKind {
    Callback,
    Reach,
}

struct GoalSlot {
    value: Option<PrioritizedGoal>,
    borrowed: Option<BorrowKind>,
}

struct RegisteredGoal {
    type_id: TypeId,
    priority: u8,
    registered: AtomicBool,
    running: AtomicBool,
    controls: AtomicCell<Controls>,
    goal: Mutex<GoalSlot>,
    available: Condvar,
}

struct BorrowedGoal<'a> {
    entry: &'a RegisteredGoal,
    value: Option<PrioritizedGoal>,
}

impl Drop for BorrowedGoal<'_> {
    fn drop(&mut self) {
        let mut slot = self
            .entry
            .goal
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        slot.value = self.value.take();
        slot.borrowed = None;
        self.entry.available.notify_all();
    }
}

impl RegisteredGoal {
    fn is_running(&self) -> bool {
        self.registered.load(Relaxed) && self.running.load(Relaxed)
    }

    fn borrow(&self, kind: BorrowKind) -> Option<BorrowedGoal<'_>> {
        let mut slot = if kind == BorrowKind::Reach {
            match self.goal.try_lock() {
                Ok(slot) => slot,
                Err(TryLockError::Poisoned(error)) => error.into_inner(),
                Err(TryLockError::WouldBlock) => return None,
            }
        } else {
            self.goal
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
        };
        while slot.borrowed.is_some() {
            if kind == BorrowKind::Reach {
                return None;
            }
            slot = self
                .available
                .wait(slot)
                .unwrap_or_else(std::sync::PoisonError::into_inner);
        }
        if !self.registered.load(Relaxed) {
            return None;
        }
        let value = slot.value.take()?;
        slot.borrowed = Some(kind);
        Some(BorrowedGoal {
            entry: self,
            value: Some(value),
        })
    }

    fn finish_retirement(&self, mob: &dyn Mob) {
        let mut slot = self
            .goal
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        loop {
            match slot.borrowed {
                // The gameplay callback completes its own stop after returning.
                Some(BorrowKind::Callback) => return,
                // Reporting must not invoke gameplay callbacks, including stop.
                Some(BorrowKind::Reach) => {
                    slot = self
                        .available
                        .wait(slot)
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                }
                None => break,
            }
        }
        let retired = slot.value.take();
        drop(slot);
        if let Some(mut goal) = retired {
            goal.stop(mob);
        }
    }
}

#[derive(Default)]
struct GoalRegistry {
    goals_by_control: [Option<Arc<RegisteredGoal>>; 4],
    #[expect(
        clippy::rc_buffer,
        reason = "Arc::make_mut grows and retains registrations in place."
    )]
    goals: Arc<Vec<Arc<RegisteredGoal>>>,
    disabled_controls: Arc<AtomicCell<Controls>>,
}

impl GoalRegistry {
    fn release_controls(&mut self, keep: impl Fn(&RegisteredGoal) -> bool) {
        for owner in &mut self.goals_by_control {
            if owner.as_ref().is_some_and(|entry| !keep(entry)) {
                *owner = None;
            }
        }
    }
}

/// Owns registrations and completes lifecycle callbacks outside the registry lock.
#[derive(Default)]
pub struct GoalSelector {
    registry: Mutex<GoalRegistry>,
    execution: Mutex<()>,
}

impl GoalSelector {
    pub fn add_goal<G: Goal + 'static>(&self, priority: u8, goal: Box<G>) {
        let goal = PrioritizedGoal::new(TypeId::of::<G>(), priority, goal);
        Arc::make_mut(&mut Self::lock(self).goals).push(Arc::new(RegisteredGoal {
            type_id: goal.type_id,
            priority,
            registered: AtomicBool::new(true),
            running: AtomicBool::new(false),
            controls: AtomicCell::new(goal.controls()),
            goal: Mutex::new(GoalSlot {
                value: Some(goal),
                borrowed: None,
            }),
            available: Condvar::new(),
        }));
    }

    pub fn remove_goal<G: Goal + 'static>(&self, mob: &dyn Mob) {
        self.remove_goal_by_type_id(TypeId::of::<G>(), mob);
    }

    pub fn remove_goal_by_type_id(&self, type_id: TypeId, mob: &dyn Mob) {
        for entry in self.retire_matching(|entry| entry.type_id == type_id) {
            entry.finish_retirement(mob);
        }
    }

    pub fn clear(&self, mob: &dyn Mob) {
        for entry in self.retire_matching(|_| true) {
            entry.finish_retirement(mob);
        }
    }

    fn retire_matching(
        &self,
        mut matches: impl FnMut(&RegisteredGoal) -> bool,
    ) -> Vec<Arc<RegisteredGoal>> {
        let mut registry = Self::lock(self);
        let mut retired = Vec::new();
        Arc::make_mut(&mut registry.goals).retain(|entry| {
            if !matches(entry) {
                return true;
            }
            entry.registered.store(false, Relaxed);
            entry.running.store(false, Relaxed);
            retired.push(entry.clone());
            false
        });
        registry.release_controls(|entry| entry.registered.load(Relaxed));
        retired
    }

    fn uses_any(controls: Controls, disabled: Controls) -> bool {
        Controls::ITER
            .into_iter()
            .any(|control| controls.get(control) && disabled.get(control))
    }

    fn lock(selector: &Self) -> MutexGuard<'_, GoalRegistry> {
        selector
            .registry
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    #[expect(
        clippy::rc_buffer,
        reason = "Snapshots share the growable registration list."
    )]
    fn snapshot(selector: &Self) -> Arc<Vec<Arc<RegisteredGoal>>> {
        Self::lock(selector).goals.clone()
    }

    /// Adds the reach of every registered goal; `None` if one is unaudited or busy.
    pub(in crate::entity) fn tick_reach(
        selector: &Self,
        mob: &dyn Mob,
        reach: &mut GoalReach,
    ) -> Option<()> {
        for entry in Self::snapshot(selector).iter() {
            if !entry.registered.load(Relaxed) {
                continue;
            }
            let borrowed = entry.borrow(BorrowKind::Reach)?;
            let goal = borrowed.value.as_ref()?;
            let report = goal.tick_reach(mob, goal.running);
            drop(borrowed);
            if !entry.registered.load(Relaxed) {
                return None;
            }
            reach.add(report?);
        }
        Some(())
    }

    fn candidate(entry: &RegisteredGoal, disabled: Controls) -> bool {
        entry.registered.load(Relaxed)
            && !entry.running.load(Relaxed)
            && !Self::uses_any(entry.controls.load(), disabled)
    }

    // Retiring a borrowed goal removes its registration immediately. Only that
    // goal's stop waits for its current callback; other stops remain synchronous.
    fn call<R>(
        selector: &Self,
        entry: &RegisteredGoal,
        mob: &dyn Mob,
        running: Option<bool>,
        callback: impl FnOnce(&mut PrioritizedGoal) -> R,
    ) -> Option<R> {
        let Some(mut borrowed) = entry.borrow(BorrowKind::Callback) else {
            if !entry.registered.load(Relaxed) {
                entry.finish_retirement(mob);
            }
            return None;
        };
        let eligible = entry.registered.load(Relaxed)
            && match running {
                Some(true) => {
                    let state = Self::lock(selector);
                    Self::candidate(entry, state.disabled_controls.load())
                        && Controls::ITER.into_iter().all(|control| {
                            !entry.controls.load().get(control)
                                || state.goals_by_control[control.idx()]
                                    .as_ref()
                                    .is_some_and(|owner| std::ptr::eq(owner.as_ref(), entry))
                        })
                }
                Some(false) => entry.running.load(Relaxed),
                None => true,
            };
        let result = if eligible {
            borrowed.value.as_mut().map(|goal| {
                if let Some(running) = running {
                    entry.running.store(running, Relaxed);
                }
                let result = callback(goal);
                entry.controls.store(goal.controls());
                entry.running.store(goal.running, Relaxed);
                result
            })
        } else {
            None
        };
        drop(borrowed);
        mob.get_mob_entity().process_navigation_updates();
        if !entry.registered.load(Relaxed) {
            entry.finish_retirement(mob);
            return None;
        }
        result
    }

    fn can_replace_all(selector: &Self, entry: &RegisteredGoal, mob: &dyn Mob) -> bool {
        for control in Controls::ITER {
            let owner = {
                let state = Self::lock(selector);
                if !Self::candidate(entry, state.disabled_controls.load()) {
                    return false;
                }
                if !entry.controls.load().get(control) {
                    continue;
                }
                state.goals_by_control[control.idx()].clone()
            };
            if let Some(owner) = owner {
                if owner.priority <= entry.priority {
                    return false;
                }
                let can_stop = Self::call(selector, &owner, mob, None, |goal| goal.can_stop());
                let state = Self::lock(selector);
                if !Self::candidate(entry, state.disabled_controls.load()) {
                    return false;
                }
                if state.goals_by_control[control.idx()]
                    .as_ref()
                    .is_some_and(|current| Arc::ptr_eq(current, &owner))
                    && can_stop != Some(true)
                {
                    return false;
                }
            }
        }
        true
    }

    /// Runs one selector pass. Scheduler entry points must not be called from callbacks.
    pub(in crate::entity) fn tick(selector: &Self, mob: &dyn Mob) {
        let disabled = Self::lock(selector).disabled_controls.clone();
        let _execution = selector
            .execution
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let entries = Self::snapshot(selector);
        for entry in entries.iter() {
            if entry.is_running()
                && (Self::uses_any(entry.controls.load(), disabled.load())
                    || Self::call(selector, entry, mob, None, |goal| goal.should_continue(mob))
                        == Some(false))
            {
                Self::call(selector, entry, mob, Some(false), |goal| goal.stop(mob));
            }
        }
        Self::lock(selector).release_controls(RegisteredGoal::is_running);
        let entries = Self::snapshot(selector);
        for entry in entries.iter() {
            if !entry.registered.load(Relaxed) || entry.running.load(Relaxed) {
                continue;
            }
            if !Self::can_replace_all(selector, entry, mob)
                || Self::call(selector, entry, mob, None, |goal| goal.can_start(mob)) != Some(true)
                || !Self::can_replace_all(selector, entry, mob)
            {
                continue;
            }
            for control in Controls::ITER {
                let owner = {
                    let state = Self::lock(selector);
                    if !Self::candidate(entry, state.disabled_controls.load()) {
                        break;
                    }
                    if !entry.controls.load().get(control) {
                        continue;
                    }
                    state.goals_by_control[control.idx()].clone()
                };
                if let Some(owner) = owner {
                    Self::call(selector, &owner, mob, Some(false), |goal| goal.stop(mob));
                }
            }
            let can_start = {
                let mut state = Self::lock(selector);
                if Self::candidate(entry, state.disabled_controls.load()) {
                    for control in Controls::ITER {
                        if entry.controls.load().get(control) {
                            state.goals_by_control[control.idx()] = Some(entry.clone());
                        }
                    }
                    true
                } else {
                    false
                }
            };
            if can_start {
                Self::call(selector, entry, mob, Some(true), |goal| goal.start(mob));
            }
        }
        Self::tick_running(selector, mob, true);
    }

    pub(in crate::entity) fn tick_goals(selector: &Self, mob: &dyn Mob, tick_all: bool) {
        let _execution = selector
            .execution
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Self::tick_running(selector, mob, tick_all);
    }

    fn tick_running(selector: &Self, mob: &dyn Mob, tick_all: bool) {
        let entries = Self::snapshot(selector);
        for entry in entries.iter() {
            if entry.is_running() {
                Self::call(selector, entry, mob, None, |goal| {
                    if goal.running && (tick_all || goal.should_run_every_tick()) {
                        goal.tick(mob);
                    }
                });
            }
        }
    }

    pub fn disable_control(&self, control: Controls) {
        self.set_control_enabled(control, false);
    }

    pub fn enable_control(&self, control: Controls) {
        self.set_control_enabled(control, true);
    }

    pub fn set_control_enabled(&self, control: Controls, enabled: bool) {
        let registry = Self::lock(self);
        let mut disabled = registry.disabled_controls.load();
        disabled.set(control, !enabled);
        registry.disabled_controls.store(disabled);
    }
}
#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    use pumpkin_data::entity::EntityType;
    use pumpkin_util::math::vector3::Vector3;

    use super::*;
    use crate::entity::{Entity, ai::test_support::world, mob::MobEntity};
    use crate::world::World;

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
            &EntityType::ZOMBIE,
        )));
        (world, mob)
    }

    #[derive(Default)]
    struct Counts {
        starts: AtomicUsize,
        ticks: AtomicUsize,
        stops: AtomicUsize,
    }

    struct CountGoal {
        counts: Arc<Counts>,
        enabled: Option<Arc<AtomicBool>>,
        controls: Controls,
        clear_on_stop: bool,
    }

    fn count_goal(counts: Arc<Counts>) -> CountGoal {
        CountGoal {
            counts,
            enabled: None,
            controls: Controls::empty(),
            clear_on_stop: false,
        }
    }

    impl Goal for CountGoal {
        fn can_start(&mut self, _mob: &dyn Mob) -> bool {
            self.enabled
                .as_ref()
                .is_none_or(|enabled| enabled.load(Ordering::Relaxed))
        }
        fn start(&mut self, _mob: &dyn Mob) {
            self.counts.starts.fetch_add(1, Ordering::Relaxed);
        }
        fn tick(&mut self, _mob: &dyn Mob) {
            self.counts.ticks.fetch_add(1, Ordering::Relaxed);
        }
        fn stop(&mut self, mob: &dyn Mob) {
            self.counts.stops.fetch_add(1, Ordering::Relaxed);
            if self.clear_on_stop {
                mob.get_mob_entity().clear_ai_goals(mob);
            }
        }
        fn controls(&self) -> Controls {
            self.controls
        }
    }

    struct ClearGoal {
        events: Arc<Mutex<Vec<&'static str>>>,
        replacement: Option<Arc<Counts>>,
    }

    impl Goal for ClearGoal {
        fn can_start(&mut self, _mob: &dyn Mob) -> bool {
            true
        }
        fn tick(&mut self, mob: &dyn Mob) {
            self.events
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push("tick");
            mob.get_mob_entity().clear_ai_goals(mob);
            if let Some(replacement) = &self.replacement {
                mob.get_mob_entity()
                    .add_goal(0, count_goal(replacement.clone()));
            }
            self.events
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push("return");
        }
        fn stop(&mut self, _mob: &dyn Mob) {
            self.events
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push("stop");
        }
    }

    struct RemoveWhileChecking(Arc<Counts>);

    impl Goal for RemoveWhileChecking {
        fn can_start(&mut self, mob: &dyn Mob) -> bool {
            mob.get_mob_entity().goals_selector.remove_goal::<Self>(mob);
            true
        }
        fn start(&mut self, _mob: &dyn Mob) {
            self.0.starts.fetch_add(1, Ordering::Relaxed);
        }
        fn tick(&mut self, _mob: &dyn Mob) {
            self.0.ticks.fetch_add(1, Ordering::Relaxed);
        }
    }

    struct DisableWhileChecking(Arc<Counts>);

    impl Goal for DisableWhileChecking {
        fn can_start(&mut self, mob: &dyn Mob) -> bool {
            mob.get_mob_entity()
                .goals_selector
                .disable_control(Controls::MOVE);
            true
        }
        fn start(&mut self, _mob: &dyn Mob) {
            self.0.starts.fetch_add(1, Ordering::Relaxed);
        }
        fn controls(&self) -> Controls {
            Controls::MOVE
        }
    }

    #[tokio::test]
    async fn clear_from_a_running_callback_stops_each_goal_once_without_restoring_them()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let (world, mob) = probe(&folder);
        let events = Arc::new(Mutex::new(Vec::new()));
        let other = Arc::new(Counts::default());
        mob.0.add_goal(
            0,
            ClearGoal {
                events: events.clone(),
                replacement: None,
            },
        );
        mob.0.add_goal(1, count_goal(other.clone()));

        mob.0.server_ai_step(&mob, &mob);
        mob.0.server_ai_step(&mob, &mob);
        world.level.shutdown().await;

        assert_eq!(
            *events
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
            ["tick", "return", "stop"]
        );
        assert_eq!(other.starts.load(Ordering::Relaxed), 1);
        assert_eq!(other.ticks.load(Ordering::Relaxed), 0);
        assert_eq!(other.stops.load(Ordering::Relaxed), 1);
        assert!(GoalSelector::lock(&mob.0.goals_selector).goals.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn replacement_registered_by_a_callback_survives_and_starts_next_pass()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let (world, mob) = probe(&folder);
        let events = Arc::new(Mutex::new(Vec::new()));
        let replacement = Arc::new(Counts::default());
        mob.0.add_goal(
            0,
            ClearGoal {
                events: events.clone(),
                replacement: Some(replacement.clone()),
            },
        );

        mob.0.server_ai_step(&mob, &mob);
        let first_starts = replacement.starts.load(Ordering::Relaxed);
        mob.0.server_ai_step(&mob, &mob);
        world.level.shutdown().await;

        assert_eq!(first_starts, 0);
        assert_eq!(replacement.starts.load(Ordering::Relaxed), 1);
        assert_eq!(replacement.ticks.load(Ordering::Relaxed), 1);
        assert_eq!(
            *events
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
            ["tick", "return", "stop"]
        );
        assert_eq!(GoalSelector::lock(&mob.0.goals_selector).goals.len(), 1);
        Ok(())
    }

    #[tokio::test]
    async fn removal_during_can_start_never_starts_the_removed_goal()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let (world, mob) = probe(&folder);
        let removed = Arc::new(Counts::default());
        let other = Arc::new(Counts::default());
        mob.0.add_goal(0, RemoveWhileChecking(removed.clone()));
        mob.0.add_goal(1, count_goal(other.clone()));

        mob.0.server_ai_step(&mob, &mob);
        world.level.shutdown().await;

        assert_eq!(removed.starts.load(Ordering::Relaxed), 0);
        assert_eq!(removed.ticks.load(Ordering::Relaxed), 0);
        assert_eq!(other.ticks.load(Ordering::Relaxed), 1);
        assert_eq!(GoalSelector::lock(&mob.0.goals_selector).goals.len(), 1);
        Ok(())
    }

    #[tokio::test]
    async fn displaced_stop_callback_cannot_restore_a_cleared_candidate()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let (world, mob) = probe(&folder);
        let old = Arc::new(Counts::default());
        let candidate = Arc::new(Counts::default());
        let enabled = Arc::new(AtomicBool::new(false));
        mob.0.add_goal(
            5,
            CountGoal {
                counts: old.clone(),
                enabled: None,
                controls: Controls::MOVE,
                clear_on_stop: true,
            },
        );
        mob.0.add_goal(
            0,
            CountGoal {
                counts: candidate.clone(),
                enabled: Some(enabled.clone()),
                controls: Controls::MOVE,
                clear_on_stop: false,
            },
        );

        mob.0.server_ai_step(&mob, &mob);
        enabled.store(true, Ordering::Relaxed);
        mob.0.server_ai_step(&mob, &mob);
        world.level.shutdown().await;

        assert_eq!(old.stops.load(Ordering::Relaxed), 1);
        assert_eq!(candidate.starts.load(Ordering::Relaxed), 0);
        assert_eq!(candidate.ticks.load(Ordering::Relaxed), 0);
        assert!(GoalSelector::lock(&mob.0.goals_selector).goals.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn controls_disabled_by_can_start_are_rechecked_before_start()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let (world, mob) = probe(&folder);
        let candidate = Arc::new(Counts::default());
        mob.0.add_goal(0, DisableWhileChecking(candidate.clone()));

        mob.0.server_ai_step(&mob, &mob);
        world.level.shutdown().await;

        assert_eq!(candidate.starts.load(Ordering::Relaxed), 0);
        assert_eq!(GoalSelector::lock(&mob.0.goals_selector).goals.len(), 1);
        Ok(())
    }

    #[tokio::test]
    async fn goal_displaced_this_pass_keeps_its_other_controls_locked()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let (world, mob) = probe(&folder);
        let displaced = Arc::new(Counts::default());
        let mover = Arc::new(Counts::default());
        let looker = Arc::new(Counts::default());
        let enabled = Arc::new(AtomicBool::new(false));
        let goal = |priority, counts: &Arc<Counts>, enabled: Option<&Arc<AtomicBool>>, controls| {
            mob.0.add_goal(
                priority,
                CountGoal {
                    counts: counts.clone(),
                    enabled: enabled.cloned(),
                    controls,
                    clear_on_stop: false,
                },
            );
        };
        goal(2, &displaced, None, Controls::MOVE | Controls::LOOK);
        goal(1, &mover, Some(&enabled), Controls::MOVE);
        goal(3, &looker, Some(&enabled), Controls::LOOK);

        mob.0.server_ai_step(&mob, &mob);
        enabled.store(true, Ordering::Relaxed);
        mob.0.server_ai_step(&mob, &mob);
        let looker_starts = looker.starts.load(Ordering::Relaxed);
        mob.0.server_ai_step(&mob, &mob);
        world.level.shutdown().await;

        assert_eq!(displaced.stops.load(Ordering::Relaxed), 1);
        assert_eq!(mover.starts.load(Ordering::Relaxed), 1);
        assert_eq!(looker_starts, 0);
        assert_eq!(looker.starts.load(Ordering::Relaxed), 1);
        Ok(())
    }

    #[tokio::test]
    async fn target_callback_clear_prevents_regular_goals_from_starting()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let (world, mob) = probe(&folder);
        let regular = Arc::new(Counts::default());
        mob.0.add_target_goal(
            0,
            ClearGoal {
                events: Arc::default(),
                replacement: None,
            },
        );
        mob.0.add_goal(0, count_goal(regular.clone()));

        mob.0.server_ai_step(&mob, &mob);
        world.level.shutdown().await;

        assert_eq!(regular.starts.load(Ordering::Relaxed), 0);
        assert_eq!(regular.ticks.load(Ordering::Relaxed), 0);
        Ok(())
    }

    #[tokio::test]
    async fn rejected_callback_finishes_retirement_once() -> Result<(), Box<dyn std::error::Error>>
    {
        let folder = tempfile::tempdir()?;
        let (world, mob) = probe(&folder);
        for running in [None, Some(false), Some(true)] {
            let counts = Arc::new(Counts::default());
            mob.0.add_goal(0, count_goal(counts.clone()));
            mob.0.server_ai_step(&mob, &mob);
            let entry = GoalSelector::lock(&mob.0.goals_selector).goals[0].clone();
            let borrowed = entry
                .borrow(BorrowKind::Callback)
                .ok_or("goal was not available")?;
            // Retire while the payload is borrowed, as when clear races callback entry.
            mob.0.clear_ai_goals(&mob);
            drop(borrowed);

            let mut called = false;
            GoalSelector::call(&mob.0.goals_selector, &entry, &mob, running, |_| {
                called = true;
            });
            GoalSelector::call(&mob.0.goals_selector, &entry, &mob, running, |_| {
                called = true;
            });
            assert!(!called);
            assert_eq!(counts.stops.load(Ordering::Relaxed), 1);
        }
        world.level.shutdown().await;
        Ok(())
    }

    struct BlockingGoal {
        entered: std::sync::mpsc::Sender<()>,
        resume: Mutex<std::sync::mpsc::Receiver<()>>,
        counts: Arc<Counts>,
    }

    struct BlockingReachGoal {
        entered: std::sync::mpsc::Sender<()>,
        resume: Mutex<std::sync::mpsc::Receiver<()>>,
        stopped_on: Arc<Mutex<Vec<std::thread::ThreadId>>>,
    }

    impl Goal for BlockingReachGoal {
        fn can_start(&mut self, _mob: &dyn Mob) -> bool {
            true
        }

        fn tick_reach(&self, _mob: &dyn Mob, _running: bool) -> Option<GoalReach> {
            self.entered.send(()).ok()?;
            self.resume
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .recv_timeout(std::time::Duration::from_secs(5))
                .ok()?;
            Some(GoalReach::default())
        }

        fn stop(&mut self, _mob: &dyn Mob) {
            self.stopped_on
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(std::thread::current().id());
        }
    }

    #[tokio::test]
    async fn clearing_a_reported_goal_stops_it_on_the_removing_thread()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let (world, mob) = probe(&folder);
        let stopped_on = Arc::new(Mutex::new(Vec::new()));
        let (entered_tx, entered_rx) = std::sync::mpsc::channel();
        let (resume_tx, resume_rx) = std::sync::mpsc::channel();
        mob.0.add_goal(
            0,
            BlockingReachGoal {
                entered: entered_tx,
                resume: Mutex::new(resume_rx),
                stopped_on: stopped_on.clone(),
            },
        );
        mob.0.server_ai_step(&mob, &mob);
        std::thread::scope(|scope| -> Result<(), Box<dyn std::error::Error>> {
            let report = scope.spawn(|| {
                GoalSelector::tick_reach(&mob.0.goals_selector, &mob, &mut GoalReach::default())
            });
            entered_rx.recv_timeout(std::time::Duration::from_secs(5))?;
            let clear = scope.spawn(|| mob.0.clear_ai_goals(&mob));
            let clear_thread = clear.thread().id();
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
            while !GoalSelector::lock(&mob.0.goals_selector).goals.is_empty() {
                assert!(std::time::Instant::now() < deadline, "goal was not retired");
                std::thread::yield_now();
            }
            resume_tx.send(())?;
            assert!(report.join().is_ok());
            assert!(clear.join().is_ok());
            assert_eq!(
                *stopped_on
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner),
                [clear_thread]
            );
            Ok(())
        })?;
        world.level.shutdown().await;
        Ok(())
    }

    impl Goal for BlockingGoal {
        fn can_start(&mut self, _mob: &dyn Mob) -> bool {
            true
        }
        fn tick(&mut self, _mob: &dyn Mob) {
            assert!(self.entered.send(()).is_ok());
            assert!(
                self.resume
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .recv_timeout(std::time::Duration::from_secs(5))
                    .is_ok()
            );
        }
        fn stop(&mut self, _mob: &dyn Mob) {
            self.counts.stops.fetch_add(1, Ordering::Relaxed);
        }
    }

    #[tokio::test]
    async fn concurrent_clear_stops_inflight_goal_on_return_and_keeps_replacement()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let (world, mob) = probe(&folder);
        let counts = Arc::new(Counts::default());
        let replacement = Arc::new(Counts::default());
        let (entered_tx, entered_rx) = std::sync::mpsc::channel();
        let (resume_tx, resume_rx) = std::sync::mpsc::channel();
        mob.0.add_goal(
            0,
            BlockingGoal {
                entered: entered_tx,
                resume: Mutex::new(resume_rx),
                counts: counts.clone(),
            },
        );
        std::thread::scope(|scope| -> Result<(), Box<dyn std::error::Error>> {
            let step = scope.spawn(|| mob.0.server_ai_step(&mob, &mob));
            entered_rx.recv_timeout(std::time::Duration::from_secs(5))?;
            mob.0.clear_ai_goals(&mob);
            mob.0.add_goal(0, count_goal(replacement.clone()));
            let stops_before_return = counts.stops.load(Ordering::Relaxed);
            resume_tx.send(())?;
            assert!(step.join().is_ok());
            assert_eq!(stops_before_return, 0);
            Ok(())
        })?;
        mob.0.server_ai_step(&mob, &mob);
        world.level.shutdown().await;

        assert_eq!(counts.stops.load(Ordering::Relaxed), 1);
        assert_eq!(replacement.starts.load(Ordering::Relaxed), 1);
        assert_eq!(replacement.ticks.load(Ordering::Relaxed), 1);
        Ok(())
    }
}
