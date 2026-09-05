use crate::wit::pumpkin::plugin::context::Server;
use crate::wit::pumpkin::plugin::world::Entity;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

/// Represents a custom entity AI goal for mobs.
///
/// Goal callbacks use shared references so they can be re-entered. Mutable goal state must use
/// thread-safe interior mutability.
#[allow(unused_variables)]
pub trait AiGoal: Send + Sync {
    /// Returns `true` if the goal should start executing.
    fn can_start(&self, server: Server, entity: Entity) -> bool {
        false
    }
    /// Returns `true` if the goal should continue executing on subsequent ticks.
    fn should_continue(&self, server: Server, entity: Entity) -> bool {
        false
    }
    /// Executed when the goal starts.
    fn start(&self, server: Server, entity: Entity) {}
    /// Executed on every server tick while the goal is active.
    fn tick(&self, server: Server, entity: Entity) {}
    /// Executed when the goal stops executing.
    fn stop(&self, server: Server, entity: Entity) {}
}

pub(crate) static AI_GOAL_HANDLERS: Mutex<LazyAiGoalHandlers> = Mutex::new(LazyAiGoalHandlers {
    handlers: BTreeMap::new(),
    next_id: 0,
});

#[allow(dead_code)]
pub(crate) struct LazyAiGoalHandlers {
    pub handlers: BTreeMap<u32, Arc<dyn AiGoal>>,
    pub next_id: u32,
}

#[allow(dead_code)]
impl LazyAiGoalHandlers {
    #[must_use]
    pub fn register(&mut self, goal: Box<dyn AiGoal>) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.handlers.insert(id, Arc::from(goal));
        id
    }

    #[must_use]
    pub fn get(&self, id: u32) -> Option<Arc<dyn AiGoal>> {
        self.handlers.get(&id).map(Arc::clone)
    }
}
