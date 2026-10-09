//! Custom entity AI goals for the async v0.2 world
//!
//! The host drives a goal the way it drives a built-in one: it asks [`AiGoal::can_start`], runs
//! [`AiGoal::start`] once, then calls [`AiGoal::tick`] and [`AiGoal::should_continue`] while the
//! goal is active and [`AiGoal::stop`] when it ends
//!
//! Every one of those is its own async export, so a goal can await host imports at any step

use std::sync::Arc;

use crate::{
    Server,
    host::world::Entity,
    registry::{LocalBoxFuture, Registry},
};

static AI_GOALS: Registry<dyn ErasedAiGoal> = Registry::new();

/// A custom AI goal for mobs
///
/// Register an implementation with [`register_ai_goal`]
///
/// Every default does nothing and the two predicates answer `false`, so a goal only runs once you
/// implement [`can_start`](Self::can_start); an empty goal is inert instead of stuck
///
/// Each callback receives the server and the entity by value, which means nothing is borrowed
/// across an await and a goal never has to hold on to an entity between calls
///
/// # Reentry
///
/// Callbacks take `&self` because the host can call into a goal again while an earlier call is
/// suspended, so mutable goal state needs interior mutability
///
/// # Examples
///
/// ```rust,ignore
/// struct Wander;
///
/// impl AiGoal for Wander {
///     async fn can_start(&self, _server: Server, entity: Entity) -> bool {
///         // is_on_ground is a plain import, so nothing suspends in this particular check
///         entity.is_on_ground()
///     }
/// }
/// ```
#[allow(
    async_fn_in_trait,
    unused_variables,
    reason = "guest futures are intentionally not Send and defaults ignore their arguments"
)]
pub trait AiGoal: Send + Sync + 'static {
    /// Returns whether the goal should start
    async fn can_start(&self, server: Server, entity: Entity) -> bool {
        false
    }

    /// Returns whether the goal should keep running on the next tick
    async fn should_continue(&self, server: Server, entity: Entity) -> bool {
        false
    }

    /// Runs once when the goal starts
    async fn start(&self, server: Server, entity: Entity) {}

    /// Runs on every tick while the goal is active
    async fn tick(&self, server: Server, entity: Entity) {}

    /// Runs once when the goal stops
    async fn stop(&self, server: Server, entity: Entity) {}
}

trait ErasedAiGoal: Send + Sync {
    fn can_start(&self, server: Server, entity: Entity) -> LocalBoxFuture<'_, bool>;
    fn should_continue(&self, server: Server, entity: Entity) -> LocalBoxFuture<'_, bool>;
    fn start(&self, server: Server, entity: Entity) -> LocalBoxFuture<'_, ()>;
    fn tick(&self, server: Server, entity: Entity) -> LocalBoxFuture<'_, ()>;
    fn stop(&self, server: Server, entity: Entity) -> LocalBoxFuture<'_, ()>;
}

impl<G: AiGoal> ErasedAiGoal for G {
    fn can_start(&self, server: Server, entity: Entity) -> LocalBoxFuture<'_, bool> {
        Box::pin(AiGoal::can_start(self, server, entity))
    }

    fn should_continue(&self, server: Server, entity: Entity) -> LocalBoxFuture<'_, bool> {
        Box::pin(AiGoal::should_continue(self, server, entity))
    }

    fn start(&self, server: Server, entity: Entity) -> LocalBoxFuture<'_, ()> {
        Box::pin(AiGoal::start(self, server, entity))
    }

    fn tick(&self, server: Server, entity: Entity) -> LocalBoxFuture<'_, ()> {
        Box::pin(AiGoal::tick(self, server, entity))
    }

    fn stop(&self, server: Server, entity: Entity) -> LocalBoxFuture<'_, ()> {
        Box::pin(AiGoal::stop(self, server, entity))
    }
}

/// Registers a goal and returns the id to pass to a mob's `add_custom_ai_goal`
///
/// # Examples
///
/// ```rust,ignore
/// let goal_id = ai::register_ai_goal(Wander);
/// mob.add_custom_ai_goal(4, goal_id);
/// ```
pub fn register_ai_goal<G: AiGoal>(goal: G) -> u32 {
    AI_GOALS.register(Arc::new(goal))
}

// An unknown goal id answers `false` and does nothing, so the goal simply never runs
pub(crate) async fn can_start(goal_id: u32, server: Server, entity: Entity) -> bool {
    // The registry lock is released before the goal runs
    match AI_GOALS.get(goal_id) {
        Some(goal) => goal.can_start(server, entity).await,
        None => false,
    }
}

pub(crate) async fn should_continue(goal_id: u32, server: Server, entity: Entity) -> bool {
    match AI_GOALS.get(goal_id) {
        Some(goal) => goal.should_continue(server, entity).await,
        None => false,
    }
}

pub(crate) async fn start(goal_id: u32, server: Server, entity: Entity) {
    if let Some(goal) = AI_GOALS.get(goal_id) {
        goal.start(server, entity).await;
    }
}

pub(crate) async fn tick(goal_id: u32, server: Server, entity: Entity) {
    if let Some(goal) = AI_GOALS.get(goal_id) {
        goal.tick(server, entity).await;
    }
}

pub(crate) async fn stop(goal_id: u32, server: Server, entity: Entity) {
    if let Some(goal) = AI_GOALS.get(goal_id) {
        goal.stop(server, entity).await;
    }
}
