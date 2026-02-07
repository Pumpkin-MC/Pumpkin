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
use pumpkin_util::math::wrap_degrees;
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
    // Stuck detection
    ticks_on_current_node: u32,
    last_node_index: usize,
    total_ticks: u32,
    path_start_pos: Option<Vector3<f64>>,
}

// If I counted correctly this should be equal to the number of iters that vanilla does for
// a zombie (yes, vanilla does a different number of iterations based on the mob and some
// other things)
// TODO: Calculate from mob attributes like in vanilla
const MAX_ITERS: usize = 560;

const TARGET_DISTANCE_MULTIPLIER: f32 = 1.5;

// TODO: Read from mob attributes
const FOLLOW_RANGE: f32 = 35.0;

// TODO: Calculate from mob attributes
const SPEED_PER_TICK: f64 = 0.1;

const NODE_REACH_XZ: f64 = 0.5;
const NODE_REACH_Y: f64 = 1.0;

// TODO: Read from mob attributes
const MOB_STEP_HEIGHT: f64 = 1.0;

const MAX_YAW_TURN_PER_TICK: f32 = 90.0;

impl Navigator {
    pub fn set_progress(&mut self, goal: NavigatorGoal) {
        self.current_goal = Some(goal);
        self.current_path = None;
    }

    pub fn stop(&mut self) {
        self.current_goal = None;
        self.current_path = None;
        self.ticks_on_current_node = 0;
        self.total_ticks = 0;
        self.path_start_pos = None;
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
        let mob_data = MobData::new_zombie(
            start_pos_f,
            entity.entity.on_ground.load(Ordering::Relaxed),
        );

        self.evaluator.prepare(context, mob_data);

        let mut start_node = self.evaluator.get_start().await?;

        let mut target = self.evaluator.get_target(destination.to_block_pos());

        let mut open_set = BinaryHeap::new();

        start_node.g = 0.0;
        let start_dist = start_node.distance(&target);
        target.update_best(start_dist, &start_node);
        // Start node uses raw distance (no 1.5x multiplier - that's only for neighbors)
        start_node.h = start_dist;
        start_node.f = start_node.h;
        start_node.walked_dist = 0.0;
        start_node.came_from = None;

        let start_pos = start_node.pos.0;

        open_set.insert(start_node.clone());

        let mut iterations = 0usize;
        let mut reached = false;

        while !open_set.is_empty() {
            iterations += 1;
            if iterations >= MAX_ITERS {
                break;
            }

            let Some(current) = open_set.pop() else {
                break;
            };
            if current.distance_manhattan(&target) <= 1.0 {
                target.reached = true;
                reached = true;
                target.update_best(0.0, &current);
                break;
            }

            let euclidean_from_start = {
                let dx = (current.pos.0.x - start_pos.x) as f32;
                let dy = (current.pos.0.y - start_pos.y) as f32;
                let dz = (current.pos.0.z - start_pos.z) as f32;
                (dx * dx + dy * dy + dz * dz).sqrt()
            };
            if euclidean_from_start >= FOLLOW_RANGE {
                continue;
            }

            let neighbors_vec = self.evaluator.get_neighbors(&current).await;

            for mut neighbor in neighbors_vec {
                let step_cost = current.distance(&neighbor);
                neighbor.walked_dist = current.walked_dist + step_cost;
                let tentative_g = current.g + step_cost + neighbor.cost_malus;

                let in_heap = open_set.contains(&neighbor);
                if neighbor.walked_dist < FOLLOW_RANGE
                    && (!in_heap
                        || open_set
                            .get_node(&neighbor)
                            .is_some_and(|existing| tentative_g < existing.g))
                {
                    neighbor.came_from = Some(Box::new(current.clone()));
                    neighbor.g = tentative_g;
                    let dist_to_target = neighbor.distance(&target);
                    target.update_best(dist_to_target, &neighbor);
                    neighbor.h = dist_to_target * TARGET_DISTANCE_MULTIPLIER;
                    neighbor.f = neighbor.g + neighbor.h;

                    if in_heap {
                        open_set.update_node(&neighbor, neighbor.clone());
                    } else {
                        open_set.insert(neighbor);
                    }
                }
            }
        }

        if let Some(best_node) = target.best_node {
            let mut path_nodes: Vec<Node> = Vec::new();
            let mut node_here = *best_node;
            path_nodes.push(node_here.clone());
            while let Some(prev_box) = node_here.came_from.take() {
                node_here = *prev_box;
                path_nodes.push(node_here.clone());
            }
            path_nodes.reverse();

            let path_target = target.node.pos.0;
            return Some(Path::new(path_nodes, path_target, reached));
        }

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

    #[allow(clippy::too_many_lines)]
    pub async fn tick(&mut self, entity: &LivingEntity) {
        let Some(goal) = self.current_goal.take() else {
            return;
        };

        if goal.current_progress == goal.destination {
            self.current_path = None;
            return;
        }

        self.total_ticks += 1;

        if self.needs_new_path(&goal) {
            self.current_path = self.compute_path(entity, goal.destination).await;
            self.ticks_on_current_node = 0;
            self.last_node_index = 0;
            self.path_start_pos = Some(entity.entity.pos.load());
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

            let current_node_index = path.get_next_node_index();
            if current_node_index == self.last_node_index {
                self.ticks_on_current_node += 1;
            } else {
                self.ticks_on_current_node = 0;
                self.last_node_index = current_node_index;
            }

            if self.ticks_on_current_node > 100 {
                self.current_path = None;
                self.ticks_on_current_node = 0;
                self.current_goal = Some(goal);
                return;
            }

            if self.total_ticks.is_multiple_of(100) {
                if let Some(start_pos) = self.path_start_pos {
                    let current_pos = entity.entity.pos.load();
                    let dx = current_pos.x - start_pos.x;
                    let dy = current_pos.y - start_pos.y;
                    let dz = current_pos.z - start_pos.z;
                    let dist_sq = dx * dx + dy * dy + dz * dz;
                    if dist_sq < 2.0 * 2.0 {
                        self.current_path = None;
                        self.ticks_on_current_node = 0;
                        self.current_goal = Some(goal);
                        return;
                    }
                }
                self.path_start_pos = Some(entity.entity.pos.load());
            }

            if let Some(next_block) = path.get_next_node_pos() {
                let target_pos = Vector3::new(
                    f64::from(next_block.x) + 0.5,
                    f64::from(next_block.y),
                    f64::from(next_block.z) + 0.5,
                );

                let current_pos = entity.entity.pos.load();
                let dx = target_pos.x - current_pos.x;
                let dy = target_pos.y - current_pos.y;
                let dz = target_pos.z - current_pos.z;

                let horizontal_dist_sq = dx * dx + dz * dz;
                let horizontal_dist = horizontal_dist_sq.sqrt();

                if horizontal_dist < NODE_REACH_XZ && dy.abs() < NODE_REACH_Y {
                    path.advance();
                    self.current_goal = Some(goal);
                    return;
                }

                let desired_yaw =
                    wrap_degrees((dz.atan2(dx) as f32).to_degrees() - 90.0);
                let current_yaw = entity.entity.yaw.load();
                let yaw_diff = wrap_degrees(desired_yaw - current_yaw);
                let target_yaw =
                    current_yaw + yaw_diff.clamp(-MAX_YAW_TURN_PER_TICK, MAX_YAW_TURN_PER_TICK);
                entity.entity.yaw.store(target_yaw);
                entity.entity.head_yaw.store(target_yaw);
                entity.entity.body_yaw.store(target_yaw);

                let speed = goal.speed * SPEED_PER_TICK;
                let (move_x, move_z) = if horizontal_dist > 1e-6 {
                    let nx = dx / horizontal_dist;
                    let nz = dz / horizontal_dist;
                    let step = speed.min(horizontal_dist);
                    (nx * step, nz * step)
                } else {
                    (0.0, 0.0)
                };

                // Step-ups within MOB_STEP_HEIGHT snap Y directly (no physics-based collision yet)
                let move_y = if dy > MOB_STEP_HEIGHT {
                    entity
                        .jumping
                        .store(true, std::sync::atomic::Ordering::SeqCst);
                    speed.min(dy)
                } else if dy.abs() > 0.01 {
                    dy
                } else {
                    0.0
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
                        target_yaw,
                        entity.entity.pitch.load(),
                        entity.entity.on_ground.load(Ordering::Relaxed),
                    ))
                    .await;
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
