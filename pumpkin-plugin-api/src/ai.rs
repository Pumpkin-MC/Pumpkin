use crate::wit::pumpkin::plugin::context::Server;
use crate::wit::pumpkin::plugin::world::Entity;
use std::collections::BTreeMap;
use std::sync::Mutex;

#[allow(unused_variables)]
pub trait AiGoal: Send + Sync {
    fn can_start(&mut self, server: Server, entity: Entity) -> bool {
        false
    }
    fn should_continue(&mut self, server: Server, entity: Entity) -> bool {
        false
    }
    fn start(&mut self, server: Server, entity: Entity) {}
    fn tick(&mut self, server: Server, entity: Entity) {}
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
