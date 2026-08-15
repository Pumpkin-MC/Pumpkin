use crate::wit::pumpkin::plugin::context::Server;
use crate::wit::pumpkin::plugin::world::Entity;
use std::collections::BTreeMap;
use std::sync::Mutex;

/// Represents a custom entity AI goal for mobs.
#[allow(unused_variables)]
pub trait AiGoal: Send + Sync {
    /// Returns `true` if the goal should start executing.
    fn can_start(&mut self, server: Server, entity: Entity) -> bool {
        false
    }
    /// Returns `true` if the goal should continue executing on subsequent ticks.
    fn should_continue(&mut self, server: Server, entity: Entity) -> bool {
        false
    }
    /// Executed when the goal starts.
    fn start(&mut self, server: Server, entity: Entity) {}
    /// Executed on every server tick while the goal is active.
    fn tick(&mut self, server: Server, entity: Entity) {}
    /// Executed when the goal stops executing.
    fn stop(&mut self, server: Server, entity: Entity) {}
}

pub(crate) static AI_GOAL_HANDLERS: Mutex<LazyAiGoalHandlers> = Mutex::new(LazyAiGoalHandlers {
    handlers: BTreeMap::new(),
    next_id: 0,
});

#[allow(dead_code)]
pub(crate) struct LazyAiGoalHandlers {
    pub handlers: BTreeMap<u32, Box<dyn AiGoal>>,
    pub next_id: u32,
}

#[allow(dead_code)]
impl LazyAiGoalHandlers {
    #[must_use]
    pub fn register(&mut self, goal: Box<dyn AiGoal>) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.handlers.insert(id, goal);
        id
    }
}
