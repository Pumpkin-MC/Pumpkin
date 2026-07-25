use pumpkin_util::math::vector3::Vector3;

use crate::entity::living::LivingEntity;

use crate::entity::ai::pathfinder::binary_heap::BinaryHeap;
use crate::entity::ai::pathfinder::node::Coordinate;
use crate::entity::ai::pathfinder::node::Node;
use crate::entity::ai::pathfinder::node::PathType;
use crate::entity::ai::pathfinder::node_evaluator::{MobData, NodeEvaluator};
use crate::entity::ai::pathfinder::path::Path;
use crate::entity::ai::pathfinder::pathfinding_context::PathfindingContext;
use crate::entity::ai::pathfinder::walk_node_evaluator::WalkNodeEvaluator;
use pumpkin_data::attributes::Attributes;
use pumpkin_util::math::wrap_degrees;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};

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

pub struct Navigator {
    current_goal: Option<NavigatorGoal>,
    evaluator: WalkNodeEvaluator,
    current_path: Option<Path>,
    // Stuck detection
    ticks_on_current_node: u32,
    last_node_index: usize,
    total_ticks: u32,
    path_start_pos: Option<Vector3<f64>>,
    path_type_overrides: HashMap<PathType, f32>,
    mob_width: f32,
    mob_height: f32,
    // Smart re-pathing cooldown
    repath_cooldown: u32,
    // Reusable allocations to avoid per-pathfind heap allocations
    open_set: BinaryHeap,
    neighbors_buf: Vec<Node>,
    /// Thread-safe status check to avoid deadlocks when components (like `LookControl`) need to
    /// check navigation status.
    pub is_idle: AtomicBool,
}

impl Default for Navigator {
    fn default() -> Self {
        Self {
            current_goal: None,
            evaluator: WalkNodeEvaluator::default(),
            current_path: None,
            ticks_on_current_node: 0,
            last_node_index: 0,
            total_ticks: 0,
            path_start_pos: None,
            path_type_overrides: HashMap::new(),
            mob_width: 0.6,
            mob_height: 1.95,
            repath_cooldown: 0,
            open_set: BinaryHeap::new(),
            neighbors_buf: Vec::new(),
            is_idle: AtomicBool::new(true),
        }
    }
}

// If I counted correctly this should be equal to the number of iters that vanilla does for
// a zombie (yes, vanilla does a different number of iterations based on the mob and some
// other things)
// TODO: Calculate from mob attributes like in vanilla
const MAX_ITERS: usize = 560;
const TARGET_DISTANCE_MULTIPLIER: f32 = 1.5;
const NODE_REACH_XZ: f64 = 0.5;
const NODE_REACH_Y: f64 = 1.0;
const MAX_YAW_TURN_PER_TICK: f32 = 90.0;

fn should_relax_node(tentative_g: f32, known_g: Option<f32>) -> bool {
    known_g.is_none_or(|known_g| tentative_g < known_g)
}

fn known_node_g(
    open_set: &BinaryHeap,
    closed_set: &HashMap<Vector3<i32>, Node>,
    candidate: &Node,
) -> Option<f32> {
    open_set
        .get_node(candidate)
        .map(|node| node.g)
        .or_else(|| closed_set.get(&candidate.pos.0).map(|node| node.g))
}

fn node_center(node: &Node) -> Vector3<f64> {
    Vector3::new(
        f64::from(node.pos.0.x) + 0.5,
        f64::from(node.pos.0.y),
        f64::from(node.pos.0.z) + 0.5,
    )
}

fn should_target_next_node_in_direction(path: &Path, mob_pos: Vector3<f64>) -> bool {
    let index = path.get_next_node_index();
    let (Some(current), Some(next)) = (path.get_node(index), path.get_node(index + 1)) else {
        return false;
    };

    if matches!(
        current.path_type,
        PathType::DangerFire | PathType::DangerOther | PathType::WalkableDoor
    ) {
        return false;
    }

    let to_current = node_center(current).sub(&mob_pos);
    let to_next = node_center(next).sub(&mob_pos);
    let current_distance_sq =
        to_current.x * to_current.x + to_current.y * to_current.y + to_current.z * to_current.z;
    let next_distance_sq = to_next.x * to_next.x + to_next.y * to_next.y + to_next.z * to_next.z;

    current_distance_sq < 4.0
        && (next_distance_sq < current_distance_sq || current_distance_sq < 0.5)
        && to_current.x * to_next.x + to_current.y * to_next.y + to_current.z * to_next.z < 0.0
}

impl Navigator {
    pub fn set_progress(&mut self, goal: NavigatorGoal) {
        self.is_idle.store(false, Ordering::Relaxed);
        self.current_goal = Some(goal);
        self.current_path = None;
    }

    pub const fn set_speed(&mut self, speed: f64) {
        if let Some(goal) = &mut self.current_goal {
            goal.speed = speed;
        }
    }

    pub fn stop(&mut self) {
        self.is_idle.store(true, Ordering::Relaxed);
        self.current_goal = None;
        self.current_path = None;
        self.ticks_on_current_node = 0;
        self.total_ticks = 0;
        self.path_start_pos = None;
    }

    pub fn set_pathfinding_malus(&mut self, path_type: PathType, malus: f32) {
        self.path_type_overrides.insert(path_type, malus);
    }

    pub const fn set_mob_dimensions(&mut self, width: f32, height: f32) {
        self.mob_width = width;
        self.mob_height = height;
    }

    #[allow(clippy::too_many_lines)]
    async fn compute_path(
        &mut self,
        entity: &LivingEntity,
        destination: Vector3<f64>,
    ) -> Option<Path> {
        let start_pos_f = entity.entity.pos.load();
        let start_block_vec = start_pos_f.to_i32();
        let mob_position = Vector3::new(start_block_vec.x, start_block_vec.y, start_block_vec.z);

        let context = PathfindingContext::new(mob_position, entity.entity.world.load_full());
        let mut mob_data = MobData::new(start_pos_f, self.mob_width, self.mob_height, 1.0);
        mob_data.on_ground = entity.entity.on_ground.load(Ordering::Relaxed);
        mob_data.set_pathfinding_malus(PathType::DangerFire, 16.0);
        mob_data.set_pathfinding_malus(PathType::DamageFire, -1.0);
        mob_data.set_pathfinding_malus(PathType::Water, 8.0);
        mob_data.set_pathfinding_malus(PathType::Lava, -1.0);
        mob_data.set_pathfinding_malus(PathType::DangerOther, 8.0);

        // Apply per-mob pathfinding malus overrides
        for (&path_type, &malus) in &self.path_type_overrides {
            mob_data.set_pathfinding_malus(path_type, malus);
        }

        self.evaluator.prepare(context, mob_data);

        let mut start_node = self.evaluator.get_start().await?;

        let mut target = self.evaluator.get_target(destination.to_block_pos());

        start_node.g = 0.0;
        let start_dist = start_node.distance(&target);
        target.update_best(start_dist, &start_node);
        // Start node uses raw distance (no 1.5x multiplier - that's only for neighbors)
        start_node.h = start_dist;
        start_node.f = start_node.h;
        start_node.walked_dist = 0.0;
        start_node.came_from = None;

        let start_pos = start_node.pos.0;

        // Map to store closed nodes for path reconstruction
        let mut closed_set: HashMap<Vector3<i32>, Node> = HashMap::new();

        // Reuse the navigator's open_set and neighbors_buf
        self.open_set.clear();
        self.open_set.insert(start_node);

        let mut iterations = 0usize;
        let mut reached = false;

        while !self.open_set.is_empty() {
            iterations += 1;
            if iterations >= MAX_ITERS {
                break;
            }

            let Some(current) = self.open_set.pop() else {
                break;
            };
            if current.distance_manhattan(&target) < 1.0 {
                target.reached = true;
                reached = true;
                target.update_best(0.0, &current);
                closed_set.insert(current.pos.0, current);
                break;
            }

            let euclidean_from_start = {
                let dx = (current.pos.0.x - start_pos.x) as f32;
                let dy = (current.pos.0.y - start_pos.y) as f32;
                let dz = (current.pos.0.z - start_pos.z) as f32;
                (dx * dx + dy * dy + dz * dz).sqrt()
            };

            let follow_range = entity.get_attribute_value(&Attributes::FOLLOW_RANGE) as f32;
            if euclidean_from_start >= follow_range {
                closed_set.insert(current.pos.0, current);
                continue;
            }

            self.neighbors_buf.clear();
            self.evaluator
                .get_neighbors(&current, &mut self.neighbors_buf)
                .await;

            for mut neighbor in self.neighbors_buf.drain(..) {
                let step_cost = current.distance(&neighbor);
                neighbor.walked_dist = current.walked_dist + step_cost;
                let tentative_g = current.g + step_cost + neighbor.cost_malus;

                let in_heap = self.open_set.contains(&neighbor);
                let known_g = known_node_g(&self.open_set, &closed_set, &neighbor);
                if neighbor.walked_dist < follow_range && should_relax_node(tentative_g, known_g) {
                    // A lower-cost route may reopen a closed node. Higher-cost routes must not
                    // overwrite its predecessor, or reconstruction can form a cycle.
                    closed_set.remove(&neighbor.pos.0);
                    neighbor.came_from = Some(current.pos.0);
                    neighbor.g = tentative_g;
                    let dist_to_target = neighbor.distance(&target);
                    target.update_best(dist_to_target, &neighbor);
                    neighbor.h = dist_to_target * TARGET_DISTANCE_MULTIPLIER;
                    neighbor.f = neighbor.g + neighbor.h;

                    if in_heap {
                        self.open_set.update_node(&neighbor, neighbor);
                    } else {
                        self.open_set.insert(neighbor);
                    }
                }
            }

            closed_set.insert(current.pos.0, current);
        }

        // Also store any remaining open set nodes for path reconstruction
        for node in self.open_set.drain() {
            closed_set.entry(node.pos.0).or_insert(node);
        }

        if let Some(best_node) = target.best_node {
            let mut path_nodes: Vec<Node> = Vec::new();
            let mut current_pos = best_node.pos.0;
            path_nodes.push(best_node);
            let mut visited: std::collections::HashSet<Vector3<i32>> =
                std::collections::HashSet::new();
            visited.insert(current_pos);
            while let Some(node) = closed_set.get(&current_pos) {
                if let Some(prev_pos) = node.came_from {
                    if prev_pos == current_pos || !visited.insert(prev_pos) {
                        break; // Self-reference or cycle detected
                    }
                    if let Some(&prev_node) = closed_set.get(&prev_pos) {
                        path_nodes.push(prev_node);
                        current_pos = prev_pos;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
            path_nodes.reverse();

            let path_target = target.node.pos.0;
            return Some(Path::new(path_nodes, path_target, reached));
        }

        None
    }

    fn needs_new_path(&self, goal: &NavigatorGoal) -> bool {
        if self.current_path.is_none() {
            return true;
        }
        if self.repath_cooldown > 0 {
            return false;
        }
        self.current_path.as_ref().is_some_and(|p| {
            let path_target = p.get_target();
            let goal_target = goal.destination.to_i32();
            let dx = f64::from(path_target.x - goal_target.x);
            let dy = f64::from(path_target.y - goal_target.y);
            let dz = f64::from(path_target.z - goal_target.z);
            let distance_sq = dx * dx + dy * dy + dz * dz;
            // Adaptive threshold based on remaining distance
            let remaining = p.get_remaining_distance().clamp(4.0, 16.0);
            let threshold = remaining * 0.5;
            distance_sq > f64::from(threshold * threshold)
        })
    }

    #[allow(clippy::too_many_lines)]
    pub async fn tick(&mut self, entity: &LivingEntity) {
        let Some(goal) = self.current_goal.take() else {
            // Idle: stop the mob
            self.is_idle.store(true, Ordering::Relaxed);
            entity.movement_input.store(Vector3::new(0.0, 0.0, 0.0));
            return;
        };

        if goal.current_progress == goal.destination {
            self.is_idle.store(true, Ordering::Relaxed);
            self.current_path = None;
            entity.movement_input.store(Vector3::new(0.0, 0.0, 0.0));
            return;
        }

        self.total_ticks += 1;
        if self.repath_cooldown > 0 {
            self.repath_cooldown -= 1;
        }

        if self.needs_new_path(&goal) {
            self.current_path = self.compute_path(entity, goal.destination).await;
            self.ticks_on_current_node = 0;
            self.last_node_index = 0;
            self.path_start_pos = Some(entity.entity.pos.load());
            self.repath_cooldown = 15; // ~0.75 seconds cooldown before recomputing
        }

        if self.current_path.is_none() {
            entity.movement_input.store(Vector3::new(0.0, 0.0, 0.0));
            self.current_goal = Some(goal);
            return;
        }

        if let Some(path) = &mut self.current_path {
            if path.is_done() || !path.is_valid() {
                entity.movement_input.store(Vector3::new(0.0, 0.0, 0.0));
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
                entity.movement_input.store(Vector3::new(0.0, 0.0, 0.0));
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
                        entity.movement_input.store(Vector3::new(0.0, 0.0, 0.0));
                        self.current_goal = Some(goal);
                        return;
                    }
                }
                self.path_start_pos = Some(entity.entity.pos.load());
            }

            let current_pos = entity.entity.pos.load();
            let navigation_pos =
                Vector3::new(current_pos.x, (current_pos.y + 0.5).floor(), current_pos.z);
            if should_target_next_node_in_direction(path, navigation_pos) {
                path.advance();
                self.current_goal = Some(goal);
                return;
            }

            let on_ground = entity.entity.on_ground.load(Ordering::Relaxed);

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

                // Skip node if we're above it on the same XZ column and airborne (falling toward it)
                if !on_ground && horizontal_dist < NODE_REACH_XZ && dy < -0.5 {
                    path.advance();
                    self.current_goal = Some(goal);
                    return;
                }

                if horizontal_dist < NODE_REACH_XZ && dy.abs() < NODE_REACH_Y {
                    path.advance();
                    self.current_goal = Some(goal);
                    return;
                }

                let desired_yaw = wrap_degrees((dz.atan2(dx) as f32).to_degrees() - 90.0);
                let current_yaw = entity.entity.yaw.load();
                let yaw_diff = wrap_degrees(desired_yaw - current_yaw);
                let target_yaw =
                    current_yaw + yaw_diff.clamp(-MAX_YAW_TURN_PER_TICK, MAX_YAW_TURN_PER_TICK);
                entity.entity.yaw.store(target_yaw);
                entity.entity.head_yaw.store(target_yaw);
                entity.entity.body_yaw.store(target_yaw);

                // Get movement speed from goal and mob attributes
                let mob_speed =
                    goal.speed * entity.get_attribute_value(&Attributes::MOVEMENT_SPEED);

                entity
                    .movement_input
                    .store(Vector3::new(0.0, 0.0, mob_speed));

                let bbox = entity.entity.bounding_box.load();
                let width = bbox.max.x - bbox.min.x;
                let jump_distance = 1.0f64.max(width);

                // Jump when the next node is above step height and we're close enough horizontally
                if dy > entity.get_attribute_value(&Attributes::STEP_HEIGHT)
                    && horizontal_dist_sq < jump_distance
                {
                    entity
                        .jumping
                        .store(true, std::sync::atomic::Ordering::SeqCst);
                } else {
                    entity
                        .jumping
                        .store(false, std::sync::atomic::Ordering::SeqCst);
                }
            } else {
                self.is_idle.store(true, Ordering::Relaxed);
                self.current_path = None;
                entity.movement_input.store(Vector3::new(0.0, 0.0, 0.0));
            }
        }

        self.current_goal = Some(goal);
    }

    #[must_use]
    pub fn is_idle(&self) -> bool {
        self.is_idle.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BinaryHeap, HashMap, Node, Path, Vector3, known_node_g, should_relax_node,
        should_target_next_node_in_direction,
    };
    use crate::entity::ai::pathfinder::node::PathType;
    use pumpkin_util::math::position::BlockPos;

    #[test]
    fn only_relaxes_unseen_or_lower_cost_nodes() {
        assert!(should_relax_node(2.0, None));
        assert!(should_relax_node(2.0, Some(3.0)));
        assert!(!should_relax_node(2.0, Some(2.0)));
        assert!(!should_relax_node(2.0, Some(1.0)));
    }

    #[test]
    fn does_not_reopen_closed_node_with_worse_cost() {
        let closed_node = Node {
            g: 2.0,
            ..Default::default()
        };

        let mut closed_set = HashMap::new();
        closed_set.insert(closed_node.pos.0, closed_node);

        let known_g = known_node_g(&BinaryHeap::new(), &closed_set, &closed_node);
        assert_eq!(known_g, Some(2.0));
        assert!(!should_relax_node(4.0, known_g));
    }

    #[test]
    fn skips_a_waypoint_behind_the_mob() {
        let path = Path::new(
            vec![
                Node::new(BlockPos::new(0, 64, 0)),
                Node::new(BlockPos::new(1, 64, 0)),
            ],
            Vector3::new(1, 64, 0),
            true,
        );

        assert!(should_target_next_node_in_direction(
            &path,
            Vector3::new(0.9, 64.0, 0.5)
        ));
    }

    #[test]
    fn skips_a_later_waypoint_behind_the_mob() {
        let mut path = Path::new(
            vec![
                Node::new(BlockPos::new(0, 64, 0)),
                Node::new(BlockPos::new(1, 64, 0)),
                Node::new(BlockPos::new(2, 64, 0)),
            ],
            Vector3::new(2, 64, 0),
            true,
        );
        path.set_next_node_index(1);

        assert!(should_target_next_node_in_direction(
            &path,
            Vector3::new(2.1, 64.0, 0.5)
        ));
    }

    #[test]
    fn does_not_skip_a_dangerous_waypoint() {
        let path = Path::new(
            vec![
                Node {
                    path_type: PathType::DangerFire,
                    ..Node::new(BlockPos::new(0, 64, 0))
                },
                Node::new(BlockPos::new(1, 64, 0)),
            ],
            Vector3::new(1, 64, 0),
            true,
        );

        assert!(!should_target_next_node_in_direction(
            &path,
            Vector3::new(0.9, 64.0, 0.5)
        ));
    }
}
