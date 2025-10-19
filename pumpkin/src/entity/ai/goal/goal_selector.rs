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
        let mut goals_to_remove = Vec::with_capacity(2);
        for (i, prioritized_goal) in &mut self.goals.iter_mut().enumerate() {
            if TypeId::of::<G>() == prioritized_goal.type_id {
                if prioritized_goal.running {
                    prioritized_goal.stop(mob).await;
                }
                goals_to_remove.push(i);
            }
        }

        for goal_idx in goals_to_remove {
            self.goals.swap_remove(goal_idx);

            // This is very fast because arrays are on the stack and the compiler knows the size
            for slot in &mut self.goals_by_control {
                if *slot == usize::MAX {
                    continue;
                }
                // Update the idx
                if *slot == goal_idx {
                    *slot = usize::MAX;
                } else if *slot > goal_idx {
                    *slot -= 1;
                }
            }
        }
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

    pub fn disable_control(&mut self, control: Controls) {
        self.disabled_controls.set(control, true);
    }

    pub fn enable_control(&mut self, control: Controls) {
        self.disabled_controls.set(control, false);
    }

    pub fn set_control_enabled(&mut self, control: Controls, enabled: bool) {
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
