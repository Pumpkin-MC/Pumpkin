use crate::entity::ai::goal::{Control, Goal, GoalControl, PrioritizedGoal};
use crate::entity::mob::Mob;
use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::Ordering::Relaxed;
use std::sync::{Arc, LazyLock};
use tokio::sync::Mutex;

static REPLACEABLE_GOAL: LazyLock<Arc<PrioritizedGoal>> = LazyLock::new(|| {
    Arc::new(PrioritizedGoal::new(
        u8::MAX,
        Arc::new(DummyGoal {
            goal_control: GoalControl::default(),
        }),
    ))
});

pub struct GoalSelector {
    goals_by_control: Mutex<HashMap<Control, Arc<PrioritizedGoal>>>,
    goals: Mutex<Vec<Arc<PrioritizedGoal>>>,
    disabled_controls: Mutex<HashSet<Control>>,
}

impl GoalSelector {
    pub fn new() -> Self {
        Self {
            goals_by_control: Mutex::new(HashMap::new()),
            goals: Mutex::new(Vec::new()),
            disabled_controls: Mutex::new(HashSet::new()),
        }
    }

    pub async fn add_goal(&self, priority: u8, goal: Arc<dyn Goal>) {
        let mut goals = self.goals.lock().await;
        goals.push(Arc::new(PrioritizedGoal::new(priority, goal)));
    }

    pub async fn remove_goal(&self, goal: Arc<dyn Goal>, mob: &dyn Mob) {
        let mut goals = self.goals.lock().await;
        for prioritized_goal in goals.iter() {
            if Arc::ptr_eq(&prioritized_goal.goal, &goal) && prioritized_goal.running.load(Relaxed)
            {
                prioritized_goal.stop(mob).await;
            }
        }

        goals.retain(|prioritized_goal| !Arc::ptr_eq(&prioritized_goal.goal, &goal));
    }

    pub async fn uses_any(
        prioritized_goal: Arc<PrioritizedGoal>,
        controls: HashSet<Control>,
    ) -> bool {
        let goal_control = prioritized_goal.get_goal_control();
        let goal_controls = goal_control.controls.read().await;
        for control in goal_controls.iter() {
            if controls.contains(control) {
                return true;
            }
        }

        false
    }

    pub async fn can_replace_all(
        goal: Arc<PrioritizedGoal>,
        goals_by_control: &HashMap<Control, Arc<PrioritizedGoal>>,
    ) -> bool {
        let controls_lock = goal.get_goal_control().controls.read().await;
        for control in controls_lock.iter() {
            let existing: &Arc<PrioritizedGoal> =
                goals_by_control.get(control).unwrap_or(&*REPLACEABLE_GOAL);

            if !existing.can_be_replaced_by(goal.clone()) {
                return false;
            }
        }
        true
    }

    pub async fn tick(&self, mob: &dyn Mob) {
        let goals = self.goals.lock().await;
        let disabled_controls = self.disabled_controls.lock().await;
        for prioritized_goal in goals.iter() {
            if prioritized_goal.running.load(Relaxed)
                && (Self::uses_any(prioritized_goal.clone(), disabled_controls.clone()).await
                    || !prioritized_goal.should_continue(mob).await)
            {
                prioritized_goal.stop(mob).await;
            }
        }

        let mut goals_by_control = self.goals_by_control.lock().await;
        goals_by_control.retain(|_, prioritized_goal| prioritized_goal.running.load(Relaxed));

        for prioritized_goal in goals.iter() {
            if !prioritized_goal.running.load(Relaxed)
                && !Self::uses_any(prioritized_goal.clone(), disabled_controls.clone()).await
                && Self::can_replace_all(prioritized_goal.clone(), &goals_by_control).await
                && prioritized_goal.can_start(mob).await
            {
                let controls = prioritized_goal.get_goal_control().controls.read().await;
                for control in controls.iter() {
                    let goal = goals_by_control.get(control).unwrap_or(&*REPLACEABLE_GOAL);
                    goal.stop(mob).await;
                    goals_by_control.insert(control.clone(), prioritized_goal.clone());
                }
                drop(controls); // Drop lock
                prioritized_goal.start(mob).await;
            }
        }
        // Drop locks
        drop(goals);
        drop(disabled_controls);
        drop(goals_by_control);

        self.tick_goals(mob, true).await;
    }

    pub async fn tick_goals(&self, mob: &dyn Mob, tick_all: bool) {
        for prioritized_goal in self.goals.lock().await.iter() {
            if prioritized_goal.running.load(Relaxed)
                && (tick_all || prioritized_goal.should_run_every_tick())
            {
                prioritized_goal.tick(mob).await;
            }
        }
    }

    pub async fn disable_control(&self, control: Control) {
        self.disabled_controls.lock().await.insert(control);
    }

    pub async fn enable_control(&self, control: Control) {
        self.disabled_controls.lock().await.remove(&control);
    }

    pub async fn set_control_enabled(&self, control: Control, enabled: bool) {
        if enabled {
            self.enable_control(control).await;
        } else {
            self.disable_control(control).await;
        }
    }
}

pub struct DummyGoal {
    goal_control: GoalControl,
}

#[async_trait]
impl Goal for DummyGoal {
    async fn can_start(&self, _mob: &dyn Mob) -> bool {
        false
    }

    async fn should_continue(&self, _mob: &dyn Mob) -> bool {
        false
    }

    async fn start(&self, _mob: &dyn Mob) {}

    async fn stop(&self, _mob: &dyn Mob) {}

    async fn tick(&self, _mob: &dyn Mob) {}

    fn get_goal_control(&self) -> &GoalControl {
        &self.goal_control
    }
}
