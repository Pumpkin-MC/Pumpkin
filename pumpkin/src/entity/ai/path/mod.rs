//! Pathfinding and navigation system for entity AI.
//!
//! This module provides A* pathfinding implementation for Minecraft entities.

mod amphibious_node_evaluator;
mod binary_heap;
mod fly_node_evaluator;
mod node;
mod node_evaluator;
mod path;
mod path_finder;
mod path_type;
mod swim_node_evaluator;
mod target;
mod walk_node_evaluator;

// Re-export core types
pub use amphibious_node_evaluator::AmphibiousNodeEvaluator;
pub use binary_heap::BinaryHeap;
pub use fly_node_evaluator::FlyNodeEvaluator;
pub use node::Node as PathNode;
pub use node_evaluator::{NodeEvaluator, PathfindingContext, PathfindingMob};
pub use path::Path;
pub use path_finder::PathFinder;
pub use path_type::PathType;
pub use swim_node_evaluator::SwimNodeEvaluator;
pub use target::Target;
pub use walk_node_evaluator::WalkNodeEvaluator;

use pumpkin_util::math::{position::BlockPos, vector3::Vector3};

use crate::entity::living::LivingEntity;

#[derive(Default)]
pub struct Navigator {
    current_goal: Option<NavigatorGoal>,
}

pub struct NavigatorGoal {
    pub current_progress: Vector3<f64>,
    pub destination: Vector3<f64>,
    pub speed: f64,
}

impl Navigator {
    pub const fn set_progress(&mut self, goal: NavigatorGoal) {
        self.current_goal = Some(goal);
    }

    pub const fn cancel(&mut self) {
        self.current_goal = None;
    }

    pub async fn tick(&mut self, entity: &LivingEntity) {
        if let Some(goal) = &mut self.current_goal {
            // First, let's check if we have reached the destination
            if goal.current_progress == goal.destination {
                // If yes, we are done here.
                self.current_goal = None;
                return;
            }

            // A star algorithm
            let mut best_move = Vector3::new(0.0, 0.0, 0.0);
            let mut lowest_cost = f64::MAX;

            let world = entity.entity.world.load();

            for x in -1..=1 {
                for z in -1..=1 {
                    let x = f64::from(x);
                    let z = f64::from(z);
                    let potential_pos = Vector3::new(
                        goal.current_progress.x + x,
                        goal.current_progress.y,
                        goal.current_progress.z + z,
                    );
                    let state = world
                        .get_block_state(&BlockPos(potential_pos.to_i32()))
                        .await;
                    if !state.collision_shapes.is_empty() {
                        continue;
                    }

                    // Simple distance calculation for greedy navigation
                    let cost = potential_pos.squared_distance_to_vec(&goal.destination).sqrt();

                    if cost < lowest_cost {
                        lowest_cost = cost;
                        best_move = Vector3::new(x, 0.0, z);
                    }
                }
            }

            // This is important. Firstly, this saves us many packets when we don't actually move. Secondly, this prevents division using zero
            // when normalize
            if best_move.x == 0.0 && best_move.z == 0.0 {
                return;
            }

            // Update current progress based on the best move
            goal.current_progress += best_move.normalize() * goal.speed;

            // Now let's move
            entity.entity.set_pos(goal.current_progress);
            //entity.entity.send_pos().await;
        }
    }

    #[must_use]
    pub const fn is_idle(&self) -> bool {
        // TODO: implement
        false
    }
}
