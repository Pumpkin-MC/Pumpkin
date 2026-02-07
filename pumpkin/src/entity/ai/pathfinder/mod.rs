use std::collections::HashSet;

use pumpkin_util::math::vector3::Vector3;

use crate::entity::living::LivingEntity;

use crate::entity::ai::pathfinder::binary_heap::BinaryHeap;
use crate::entity::ai::pathfinder::node::Coordinate;
use crate::entity::ai::pathfinder::node::Node;
use crate::entity::ai::pathfinder::node_evaluator::{MobData, NodeEvaluator};
use crate::entity::ai::pathfinder::path::Path;
use crate::entity::ai::pathfinder::pathfinding_context::PathfindingContext;
use crate::entity::ai::pathfinder::walk_node_evaluator::WalkNodeEvaluator;
use pumpkin_protocol::java::client::play::CEntityPositionSync;
use std::sync::atomic::Ordering;

pub mod binary_heap;
pub mod node;
pub mod node_evaluator;
pub mod path;
pub mod path_type_cache;
pub mod pathfinding_context;
pub mod walk_node_evaluator;

pub struct NavigatorGoal {
    pub current_progress: Vector3<f64>,
    pub destination: Vector3<f64>,
    pub speed: f64,
}

impl NavigatorGoal {
    #[must_use]
    pub const fn new(
        current_progress: Vector3<f64>,
        destination: Vector3<f64>,
        speed: f64,
    ) -> Self {
        Self {
            current_progress,
            destination,
            speed,
        }
    }
}

#[derive(Default)]
pub struct Navigator {
    current_goal: Option<NavigatorGoal>,
    evaluator: WalkNodeEvaluator,
    current_path: Option<Path>,
}

// If I counted correctly this should be equal to the number of iters that vanilla does for
// a zombie (yes, vanilla does a different number of iterations based on the mob and some
// other things)
// TODO: Calculate from mob attributes like in vanilla
const MAX_ITERS: usize = 560;

impl Navigator {
    pub fn set_progress(&mut self, goal: NavigatorGoal) {
        self.current_goal = Some(goal);
        self.current_path = None;
    }

    pub fn stop(&mut self) {
        self.current_goal = None;
        self.current_path = None;
    }

    async fn compute_path(
        &mut self,
        entity: &LivingEntity,
        destination: Vector3<f64>,
    ) -> Option<Path> {
        let start_pos_f = entity.entity.pos.load();
        let start_block_vec = start_pos_f.to_i32();
        let mob_position = Vector3::new(start_block_vec.x, start_block_vec.y, start_block_vec.z);

        let context = PathfindingContext::new(mob_position, entity.entity.world.load_full());
        // TODO: Assign based on mob type, or load from mob/entity once implemented
        let mob_data = MobData::new_zombie(start_pos_f);

        self.evaluator.prepare(context, mob_data);

        let mut start_node = self.evaluator.get_start().await?;

        let target = self.evaluator.get_target(destination.to_block_pos());

        let mut open_set = BinaryHeap::new();
        let mut closed: HashSet<Vector3<i32>> = HashSet::new();

        start_node.g = 0.0;
        start_node.h = start_node.distance(&target);
        start_node.f = start_node.g + start_node.h;
        start_node.came_from = None;

        open_set.insert(start_node.clone());

        let mut iterations = 0usize;

        while !open_set.is_empty() && iterations < MAX_ITERS {
            iterations += 1;

            let Some(current) = open_set.pop() else {
                break;
            };
            closed.insert(current.pos.0);

            if current.pos.0 == target.node.pos.0 {
                // Reconstruct path by following came_from
                let mut path_nodes: Vec<Node> = Vec::new();
                let mut node_here = current.clone();
                path_nodes.push(node_here.clone());
                while let Some(prev_box) = node_here.came_from.take() {
                    node_here = *prev_box;
                    path_nodes.push(node_here.clone());
                }
                path_nodes.reverse();

                let path_target = target.node.pos.0;
                let path = Path::new(path_nodes, path_target, true);
                return Some(path);
            }

            let neighbors_vec = self.evaluator.get_neighbors(&current).await;

            for mut neighbor in neighbors_vec {
                let neighbor_pos = neighbor.pos.0;
                if closed.contains(&neighbor_pos) {
                    continue;
                }

                let step_cost = current.distance(&neighbor);
                // Include neighbor's malus as extra cost
                let malus = neighbor.cost_malus;
                let tentative_g = current.g + step_cost + malus;

                neighbor.came_from = Some(Box::new(current.clone()));
                neighbor.g = tentative_g;
                neighbor.h = neighbor.distance(&target);
                neighbor.f = neighbor.g + neighbor.h;

                open_set.insert(neighbor);
            }
        }

        // No path found
        None
    }

    fn needs_new_path(&self, goal: &NavigatorGoal) -> bool {
        self.current_path.is_none()
            || self.current_path.as_ref().is_some_and(|p| {
                let path_target = p.get_target();
                let goal_target = goal.destination.to_i32();
                let dx = f64::from(path_target.x - goal_target.x);
                let dy = f64::from(path_target.y - goal_target.y);
                let dz = f64::from(path_target.z - goal_target.z);
                let distance = (dx * dx + dy * dy + dz * dz).sqrt();
                distance > 2.0
            })
    }

    pub async fn tick(&mut self, entity: &LivingEntity) {
        // Take the goal out so we can mutably borrow self inside compute_path
        let Some(goal) = self.current_goal.take() else {
            return;
        };

        if goal.current_progress == goal.destination {
            self.current_path = None;
            return;
        }

        if self.needs_new_path(&goal) {
            // compute_path borrows &mut self, so it's important that we don't hold a borrow to `self.current_goal` here
            self.current_path = self.compute_path(entity, goal.destination).await;
        }

        if self.current_path.is_none() {
            self.current_goal = Some(goal);
            return;
        }

        if let Some(path) = &mut self.current_path {
            if path.is_done() || !path.is_valid() {
                self.current_goal = Some(goal);
                return;
            }

            if let Some(next_block) = path.get_next_node_pos() {
                let target_pos = Vector3::new(
                    f64::from(next_block.x) + 0.5,
                    f64::from(next_block.y),
                    f64::from(next_block.z) + 0.5,
                );

                let current_pos = entity.entity.pos.load();
                let dx = target_pos.x - current_pos.x;
                let dz = target_pos.z - current_pos.z;
                let dy = target_pos.y - current_pos.y;
                let d = Vector3::new(dx, dy, dz);

                let horizontal_dist = d.horizontal_length_squared();

                if d.length_squared() < (goal.speed * 1.1) || horizontal_dist < 0.1 {
                    entity.entity.set_pos(target_pos);
                    entity
                        .entity
                        .world
                        .load()
                        .broadcast_packet_all(&CEntityPositionSync::new(
                            entity.entity.entity_id.into(),
                            target_pos,
                            d,
                            entity.entity.yaw.load(),
                            entity.entity.pitch.load(),
                            entity.entity.on_ground.load(Ordering::Relaxed),
                        ))
                        .await;

                    path.advance();
                } else {
                    let (move_x, move_y, move_z) = if horizontal_dist > 1e-6 {
                        let nx = dx / horizontal_dist;
                        let nz = dz / horizontal_dist;
                        (
                            nx * goal.speed,
                            if dy.abs() > 0.01 {
                                dy.signum() * (goal.speed * 0.5)
                            } else {
                                0.0
                            },
                            nz * goal.speed,
                        )
                    } else {
                        (
                            0.0,
                            if dy.abs() > 0.01 {
                                dy.signum() * (goal.speed * 0.5)
                            } else {
                                0.0
                            },
                            0.0,
                        )
                    };

                    let new_pos = Vector3::new(
                        current_pos.x + move_x,
                        current_pos.y + move_y,
                        current_pos.z + move_z,
                    );

                    entity.entity.set_pos(new_pos);

                    let delta = Vector3::new(move_x, move_y, move_z);
                    entity
                        .entity
                        .world
                        .load()
                        .broadcast_packet_all(&CEntityPositionSync::new(
                            entity.entity.entity_id.into(),
                            new_pos,
                            delta,
                            entity.entity.yaw.load(),
                            entity.entity.pitch.load(),
                            entity.entity.on_ground.load(Ordering::Relaxed),
                        ))
                        .await;
                }
            } else {
                self.current_path = None;
            }
        }

        self.current_goal = Some(goal);
    }

    #[must_use]
    pub const fn is_idle(&self) -> bool {
        self.current_goal.is_none()
    }
}
