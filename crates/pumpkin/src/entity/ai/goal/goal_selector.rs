use crate::entity::ai::goal::{Controls, Goal, PrioritizedGoal};
use crate::entity::mob::Mob;
use std::any::TypeId;

/// `GoalSelector` manages a set of goals and decides which ones can run.
///
/// Important: `GoalSelector` is intentionally not `Send`/`Sync`.
/// Once the outer mutex is locked, no other thread can access it,
/// so we don't need extra thread-safe wrappers here.
/// don't touch this if you dont know how this works!
pub struct GoalSelector {
    /// Indieces into self.goals
    /// `usize::max` means no goal
    goals_by_control: [usize; 4],
    goals: Vec<PrioritizedGoal>,
    disabled_controls: Controls,
}

impl GoalSelector {
    pub fn add_goal<G: Goal + 'static>(&mut self, priority: u8, goal: Box<G>) {
        self.goals
            .push(PrioritizedGoal::new(TypeId::of::<G>(), priority, goal));
    }

    pub async fn remove_goal<G: Goal + 'static>(&mut self, mob: &dyn Mob) {
        for prioritized_goal in &mut self.goals {
            if TypeId::of::<G>() == prioritized_goal.type_id && prioritized_goal.running {
                prioritized_goal.stop(mob).await;
            }
        }

        self.remove_goals_of_type(TypeId::of::<G>());
    }

    /// Removes every goal matching `type_id` and fixes up `goals_by_control` to still point at
    /// the correct surviving goals. Split out from `remove_goal` (which additionally has to
    /// `stop` running goals, requiring a `&dyn Mob`) so this index-remapping logic can be
    /// exercised without one.
    ///
    /// `Vec::retain` preserves the relative order of the surviving goals, so a goal's new index
    /// is simply the count of surviving goals before it -- unlike `swap_remove`, this can't move
    /// an unrelated goal into a slot that `goals_by_control` still references.
    fn remove_goals_of_type(&mut self, type_id: TypeId) {
        let mut new_index = vec![usize::MAX; self.goals.len()];
        let mut next = 0usize;
        for (i, goal) in self.goals.iter().enumerate() {
            if goal.type_id != type_id {
                new_index[i] = next;
                next += 1;
            }
        }

        self.goals.retain(|goal| goal.type_id != type_id);

        for slot in &mut self.goals_by_control {
            if *slot != usize::MAX {
                *slot = new_index[*slot];
            }
        }
    }

    pub fn clear(&mut self) -> Vec<PrioritizedGoal> {
        let mut running = Vec::new();
        for goal in self.goals.drain(..) {
            if goal.running {
                running.push(goal);
            }
        }
        self.goals_by_control = [usize::MAX; 4];
        running
    }

    fn uses_any(prioritized_goal: &PrioritizedGoal, controls: Controls) -> bool {
        let goal_controls = prioritized_goal.controls();
        for control in Controls::ITER {
            if controls.get(control) && goal_controls.get(control) {
                return true;
            }
        }

        false
    }

    fn can_replace_all(&self, goal: &PrioritizedGoal) -> bool {
        let controls = goal.controls();
        for control in Controls::ITER {
            if controls.get(control) {
                let goal_idx = self.goals_by_control[control.idx()];

                if goal_idx != usize::MAX && !self.goals[goal_idx].can_be_replaced_by(goal) {
                    return false;
                }
            }
        }
        true
    }

    pub async fn tick(&mut self, mob: &dyn Mob) {
        for prioritized_goal in &mut self.goals {
            if prioritized_goal.running
                && (Self::uses_any(prioritized_goal, self.disabled_controls)
                    || !prioritized_goal.should_continue(mob).await)
            {
                prioritized_goal.stop(mob).await;
            }
        }

        self.goals_by_control.iter_mut().for_each(|goal| {
            if *goal != usize::MAX && !self.goals[*goal].running {
                *goal = usize::MAX;
            }
        });

        for i in 0..self.goals.len() {
            if !self.goals[i].running
                && !Self::uses_any(&self.goals[i], self.disabled_controls)
                && self.can_replace_all(&self.goals[i])
                && self.goals[i].can_start(mob).await
            {
                let controls = self.goals[i].controls();
                for control in Controls::ITER {
                    if controls.get(control) {
                        if let Some(goal) = self.get_goal_by_control(control) {
                            goal.stop(mob).await;
                        }
                        self.goals_by_control[control.idx()] = i;
                    }
                }
                self.goals[i].start(mob).await;
            }
        }

        self.tick_goals(mob, true).await;
    }

    pub async fn tick_goals(&mut self, mob: &dyn Mob, tick_all: bool) {
        for prioritized_goal in &mut self.goals {
            if prioritized_goal.running && (tick_all || prioritized_goal.should_run_every_tick()) {
                prioritized_goal.tick(mob).await;
            }
        }
    }

    pub const fn disable_control(&mut self, control: Controls) {
        self.disabled_controls.set(control, true);
    }

    pub const fn enable_control(&mut self, control: Controls) {
        self.disabled_controls.set(control, false);
    }

    pub const fn set_control_enabled(&mut self, control: Controls, enabled: bool) {
        self.disabled_controls.set(control, !enabled);
    }

    fn get_goal_by_control(&mut self, control: Controls) -> Option<&mut PrioritizedGoal> {
        let i = self.goals_by_control[control.idx()];
        self.goals.get_mut(i)
    }
}

impl Default for GoalSelector {
    fn default() -> Self {
        Self {
            goals_by_control: [usize::MAX; 4],
            goals: Vec::default(),
            disabled_controls: Controls::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct RemoveMeGoal;
    impl Goal for RemoveMeGoal {}

    struct KeepMeGoal;
    impl Goal for KeepMeGoal {}

    #[test]
    fn remove_goals_of_type_remaps_goals_by_control_after_multi_remove() {
        let mut selector = GoalSelector::default();
        selector.add_goal(0, Box::new(RemoveMeGoal)); // index 0
        selector.add_goal(0, Box::new(RemoveMeGoal)); // index 1
        selector.add_goal(0, Box::new(KeepMeGoal)); // index 2, should survive
        selector.add_goal(0, Box::new(RemoveMeGoal)); // index 3

        // Simulate `KeepMeGoal` (index 2) being the currently-running goal for MOVE, and a
        // `RemoveMeGoal` (index 3) being tracked for LOOK.
        selector.goals_by_control[Controls::MOVE.idx()] = 2;
        selector.goals_by_control[Controls::LOOK.idx()] = 3;

        // Removes 3 `RemoveMeGoal`s (indices 0, 1, 3) in a single call. The old `swap_remove`-in-
        // a-loop code, run against this exact fixture, first misdirects the MOVE slot (its
        // uniform `*slot -= 1` fixup doesn't account for `swap_remove` moving the *last* element
        // into the removed slot, not shifting everything down by one) and then panics with an
        // out-of-bounds `swap_remove` once the vector has shrunk past a stale collected index.
        // The retain-based remap must instead still find `KeepMeGoal` and clear the LOOK slot.
        selector.remove_goals_of_type(TypeId::of::<RemoveMeGoal>());

        assert_eq!(selector.goals.len(), 1);
        let kept_idx = selector.goals_by_control[Controls::MOVE.idx()];
        assert_ne!(kept_idx, usize::MAX);
        assert_eq!(selector.goals[kept_idx].type_id, TypeId::of::<KeepMeGoal>());
        assert_eq!(selector.goals_by_control[Controls::LOOK.idx()], usize::MAX);
    }
}
