use pumpkin_util::math::{position::BlockPos, vector3::Vector3};

use crate::entity::{ai::control::MoveControlTrait, living::LivingEntity};

use crate::entity::ai::pathfinder::binary_heap::BinaryHeap;
use crate::entity::ai::pathfinder::node::Coordinate;
use crate::entity::ai::pathfinder::node::Node;
use crate::entity::ai::pathfinder::node::PathType;
use crate::entity::ai::pathfinder::node_evaluator::{MobData, NodeEvaluator};
use crate::entity::ai::pathfinder::path::Path;
use crate::entity::ai::pathfinder::pathfinding_context::PathfindingContext;
use crate::entity::ai::pathfinder::walk_node_evaluator::WalkNodeEvaluator;
use pumpkin_data::attributes::Attributes;
use std::collections::HashMap;
use std::sync::Mutex;
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

// Vanilla PathFinder.java:36 `FUDGING = 1.5f`, applied to the heuristic of
// every expanded neighbor at PathFinder.java:105 (`h = getBestH(...) * 1.5f`).
const FUDGING: f32 = 1.5;
// Vanilla PathNavigation.java:64 `requiredPathLength = 16.0f`, the lower bound
// of `getMaxPathLength()` (PathNavigation.java:87-89).
const REQUIRED_PATH_LENGTH: f32 = 16.0;
// Vanilla PathNavigation.java:178-179: `moveTo(x, y, z, speed)` creates the
// path with `reachRange = 1`.
const DEFAULT_REACH_RANGE: i32 = 1;
// Vanilla GroundPathNavigation: abs(dy) < 1.0 for "on node". Exactly 1-block drops
// sit at dy≈1.0 and fail that check — we allow a hair more so mobs step down
// instead of orbiting the ledge and repathing the long way around.
const NODE_REACH_Y: f64 = 1.0;
const STEP_DOWN_REACH_Y: f64 = 1.05;
const MAX_FALL_FOLLOW_Y: f64 = 3.25;

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
        // Note: living clear_speed is applied on next idle tick via current_goal=None.
    }

    /// Vanilla `PathNavigation.getPath` — read-only access for goals that inspect
    /// upcoming nodes, such as the door interaction goals.
    #[must_use]
    pub const fn current_path(&self) -> Option<&Path> {
        self.current_path.as_ref()
    }

    /// Vanilla `PathNavigation.setCanOpenDoors`. Mobs that may open or break
    /// doors are allowed to path through closed wooden ones.
    pub fn set_can_open_doors(&mut self, can_open: bool) {
        self.evaluator.set_can_open_doors(can_open);
    }

    pub fn set_pathfinding_malus(&mut self, path_type: PathType, malus: f32) {
        self.path_type_overrides.insert(path_type, malus);
    }

    /// True when water is marked impassable (malus < 0), e.g. iron golems.
    #[must_use]
    pub fn avoids_water(&self) -> bool {
        self.path_type_overrides
            .get(&PathType::Water)
            .is_some_and(|&m| m < 0.0)
    }

    pub const fn set_mob_dimensions(&mut self, width: f32, height: f32) {
        self.mob_width = width;
        self.mob_height = height;
    }

    /// Vanilla `PathNavigation.createPath` — used by `MeleeAttackGoal.canUse`.
    /// Uses the `moveTo` default `reachRange = 1` (PathNavigation.java:178-187).
    pub async fn create_path_to(
        &mut self,
        entity: &LivingEntity,
        destination: Vector3<f64>,
    ) -> Option<Path> {
        self.compute_path(entity, destination, DEFAULT_REACH_RANGE)
            .await
    }

    /// Vanilla `PathNavigation.createPath(pos, reachRange)` — goals that only
    /// need to get within `reach_range` blocks (manhattan) may pass it here.
    pub async fn create_path_to_in_range(
        &mut self,
        entity: &LivingEntity,
        destination: Vector3<f64>,
        reach_range: i32,
    ) -> Option<Path> {
        self.compute_path(entity, destination, reach_range).await
    }

    #[allow(clippy::too_many_lines)]
    async fn compute_path(
        &mut self,
        entity: &LivingEntity,
        destination: Vector3<f64>,
        reach_range: i32,
    ) -> Option<Path> {
        let start_pos_f = entity.entity.pos.load();
        // Vanilla `Entity.blockPosition()` and `BlockPos.containing` floor world
        // coordinates. Rounding shifts every `.5` waypoint into the next block,
        // which makes a path miss the intended ledge or choose a diagonal route.
        let start_block_vec = start_pos_f.floor_to_i32();
        let mob_position = Vector3::new(start_block_vec.x, start_block_vec.y, start_block_vec.z);

        let context = PathfindingContext::new(mob_position, entity.entity.world.load_full());
        // Prefer live entity size + STEP_HEIGHT (golem 1.4×2.7 / step 1.0). Hardcoded
        // 0.6×1.95 + step 1.0 left wide mobs pathing as zombies and failing 1-block steps.
        let dim = entity.entity.entity_dimension.load();
        let width = if self.mob_width > 0.0 {
            self.mob_width.max(dim.width)
        } else {
            dim.width
        };
        let height = if self.mob_height > 0.0 {
            self.mob_height.max(dim.height)
        } else {
            dim.height
        };
        let step_height = entity
            .get_attribute_value(&Attributes::STEP_HEIGHT)
            .max(0.6) as f32;
        let mut mob_data = MobData::new(start_pos_f, width, height, step_height);
        mob_data.on_ground = entity.entity.on_ground.load(Ordering::Relaxed);
        mob_data.is_in_water = entity.entity.touching_water.load(Ordering::SeqCst);

        // Vanilla `Mob` sets no malus overrides in its constructor — the base
        // `getPathfindingMalus` falls through to `PathType.getMalus()`
        // (Mob.java:167, 204-210). Per-mob overrides (e.g. zombies avoiding
        // fire, golems avoiding water) belong in the mob constructors via
        // `Navigator::set_pathfinding_malus`, mirroring `setPathfindingMalus`
        // (Mob.java:212-214).
        for (&path_type, &malus) in &self.path_type_overrides {
            mob_data.set_pathfinding_malus(path_type, malus);
        }

        self.evaluator.prepare(context, mob_data);

        let mut start_node = self.evaluator.get_start().await?;

        let mut target = self.evaluator.get_target(BlockPos::floored_v(destination));

        // Vanilla PathNavigation.java:87-89: maxPathLength =
        // max(FOLLOW_RANGE attribute, requiredPathLength (16)).
        let follow_range = entity.get_attribute_value(&Attributes::FOLLOW_RANGE) as f32;
        let max_path_length = follow_range.max(REQUIRED_PATH_LENGTH);
        // Vanilla PathNavigation.java:69,77-80: maxVisitedNodes =
        // floor(maxPathLength * 16).
        let max_visited_nodes = (max_path_length * 16.0).floor() as usize;

        // Vanilla PathFinder.java:74-75: the start node's h has no FUDGING.
        start_node.g = 0.0;
        let start_dist = start_node.distance(&target);
        target.update_best(start_dist, &start_node);
        start_node.h = start_dist;
        start_node.f = start_node.h;
        start_node.walked_dist = 0.0;
        start_node.came_from = None;

        let start_pos = start_node.pos.0;

        // Popped ("closed", PathFinder.java:85) nodes, kept for path
        // reconstruction. Vanilla marks the shared Node object; our nodes are
        // Copy, so this map is the source of truth for the closed flag.
        let mut closed_set: HashMap<Vector3<i32>, Node> = HashMap::new();

        // Reuse the navigator's open_set and neighbors_buf
        self.open_set.clear();
        self.open_set.insert(start_node);

        let mut count = 0usize;
        let mut reached = false;

        // Vanilla PathFinder.java:83: `while (!openSet.isEmpty() && ++count < max)`.
        while !self.open_set.is_empty() {
            count += 1;
            if count >= max_visited_nodes {
                break;
            }

            let Some(mut current) = self.open_set.pop() else {
                break;
            };
            // Vanilla PathFinder.java:85: popped nodes are closed for good.
            current.closed = true;
            closed_set.insert(current.pos.0, current);

            // Vanilla PathFinder.java:86-91: reached when within reachRange
            // (manhattan) of the target.
            if current.distance_manhattan(&target) <= reach_range as f32 {
                target.reached = true;
                reached = true;
                break;
            }

            // Vanilla PathFinder.java:95: `distanceTo(from) >= maxPathLength`.
            let euclidean_from_start = {
                let dx = (current.pos.0.x - start_pos.x) as f32;
                let dy = (current.pos.0.y - start_pos.y) as f32;
                let dz = (current.pos.0.z - start_pos.z) as f32;
                (dx * dx + dy * dy + dz * dz).sqrt()
            };
            if euclidean_from_start >= max_path_length {
                continue;
            }

            self.neighbors_buf.clear();
            self.evaluator
                .get_neighbors(&current, &mut self.neighbors_buf)
                .await;

            for mut neighbor in self.neighbors_buf.drain(..) {
                // Closed nodes are never re-expanded. Vanilla checks the shared
                // node's `closed` flag inside isNeighborValid
                // (WalkNodeEvaluator.java:149-151); with Copy nodes the
                // closed_set map carries that flag.
                if closed_set.contains_key(&neighbor.pos.0) {
                    continue;
                }

                // Vanilla PathFinder.java:99,126-128: plain euclidean distance.
                let distance = current.distance(&neighbor);
                neighbor.walked_dist = current.walked_dist + distance;
                // Vanilla PathFinder.java:101
                let tentative_g = current.g + distance + neighbor.cost_malus;

                let in_heap = self.open_set.contains(&neighbor);
                // Vanilla PathFinder.java:102
                if neighbor.walked_dist < max_path_length
                    && (!in_heap
                        || self
                            .open_set
                            .get_node(&neighbor)
                            .is_some_and(|existing| tentative_g < existing.g))
                {
                    neighbor.came_from = Some(current.pos.0);
                    neighbor.g = tentative_g;
                    // Vanilla PathFinder.java:105,130-138 (getBestH + FUDGING).
                    let dist_to_target = neighbor.distance(&target);
                    target.update_best(dist_to_target, &neighbor);
                    neighbor.h = dist_to_target * FUDGING;
                    neighbor.f = neighbor.g + neighbor.h;

                    if in_heap {
                        self.open_set.update_node(&neighbor, neighbor);
                    } else {
                        self.open_set.insert(neighbor);
                    }
                }
            }
        }

        // Vanilla PathFinder.java:65: release per-search evaluator state.
        self.evaluator.done();

        // Vanilla PathFinder.java:114: whether the target was reached or not,
        // reconstruct from the best node seen (min distToTarget); with a single
        // target no further tie-breaking is needed. PathFinder.java:140-149.
        let best_node = target.best_node?;
        let mut path_nodes: Vec<Node> = vec![best_node];
        let mut visited: std::collections::HashSet<Vector3<i32>> = std::collections::HashSet::new();
        visited.insert(best_node.pos.0);
        let mut came_from = best_node.came_from;
        while let Some(prev_pos) = came_from {
            if !visited.insert(prev_pos) {
                break; // Cycle guard — should not happen, but never loop forever.
            }
            let Some(&prev_node) = closed_set.get(&prev_pos) else {
                break;
            };
            path_nodes.push(prev_node);
            came_from = prev_node.came_from;
        }
        path_nodes.reverse();

        Some(Path::new(path_nodes, target.node.pos.0, reached))
    }

    /// Advance past path nodes the mob is already standing on (vanilla
    /// `followThePath` consumes the start node immediately). Also skips a
    /// first waypoint that sits *behind* the mob relative to the path end
    /// (common cause of golem "walk back then charge").
    fn skip_nodes_already_reached(path: &mut Path, entity: &LivingEntity) {
        let current_pos = entity.entity.pos.load();
        let bbox = entity.entity.bounding_box.load();
        let width = (bbox.max.x - bbox.min.x).max(0.1);
        let max_waypoint = if width > 0.75 {
            width * 0.5
        } else {
            0.75 - width * 0.5
        };
        // Never skip the final node — need at least one waypoint to walk to.
        while path.get_next_node_index() + 1 < path.get_node_count() {
            let Some(next) = path.get_next_node_pos() else {
                break;
            };
            let nx = f64::from(next.x) + 0.5 - current_pos.x;
            let ny = f64::from(next.y) - current_pos.y;
            let nz = f64::from(next.z) + 0.5 - current_pos.z;
            let near_xz = nx.abs() < max_waypoint && nz.abs() < max_waypoint;
            if near_xz && ny.abs() < NODE_REACH_Y {
                path.advance();
                continue;
            }
            // Skip reverse first step: next node is opposite the path end direction.
            if let Some(end) = path.get_node_pos(path.get_node_count() - 1) {
                let ex = f64::from(end.x) + 0.5 - current_pos.x;
                let ez = f64::from(end.z) + 0.5 - current_pos.z;
                let end_dist_sq = ex * ex + ez * ez;
                let dot = nx * ex + nz * ez;
                if end_dist_sq > 2.25 && dot < 0.0 {
                    path.advance();
                    continue;
                }
            }
            break;
        }
    }

    fn set_wanted_position(
        move_control: &Mutex<Box<dyn MoveControlTrait>>,
        destination: Vector3<f64>,
        speed: f64,
    ) {
        move_control.lock().unwrap().set_wanted_position(
            destination.x,
            destination.y,
            destination.z,
            speed,
        );
    }

    fn stop_move_control(move_control: &Mutex<Box<dyn MoveControlTrait>>) {
        move_control.lock().unwrap().stop();
    }

    fn get_ground_y(entity: &LivingEntity, target: Vector3<f64>) -> f64 {
        let target_block = BlockPos::floored_v(target);
        let below = target_block.down();
        let state = entity.entity.world.load().get_block_state(&below);
        if state.is_air() {
            return target.y;
        }

        let collision_height = state
            .get_block_collision_shapes()
            .map(|shape| shape.max.y)
            .fold(0.0, f64::max);
        f64::from(below.0.y) + collision_height
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
            let goal_target = goal.destination.floor_to_i32();
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
    pub async fn tick(
        &mut self,
        entity: &LivingEntity,
        move_control: &Mutex<Box<dyn MoveControlTrait>>,
    ) {
        let Some(goal) = self.current_goal.take() else {
            // Idle: stop the mob — unless a goal armed a strafe this tick
            // (vanilla strafing runs with navigation stopped; clearing here
            // would erase the request before MoveControl ticks).
            self.is_idle.store(true, Ordering::Relaxed);
            if !move_control.lock().unwrap().is_strafing() {
                entity.clear_speed();
                Self::stop_move_control(move_control);
            }
            return;
        };

        if goal.current_progress == goal.destination {
            self.is_idle.store(true, Ordering::Relaxed);
            self.current_path = None;
            entity.clear_speed();
            Self::stop_move_control(move_control);
            return;
        }

        self.total_ticks += 1;
        if self.repath_cooldown > 0 {
            self.repath_cooldown -= 1;
        }

        if self.needs_new_path(&goal) {
            self.current_path = self
                .compute_path(entity, goal.destination, DEFAULT_REACH_RANGE)
                .await;
            self.ticks_on_current_node = 0;
            self.last_node_index = 0;
            self.path_start_pos = Some(entity.entity.pos.load());
            self.repath_cooldown = 10; // repath a bit faster during chase

            // Skip start node(s) already under our feet so we don't first step
            // "backward" onto the path origin block.
            if let Some(path) = self.current_path.as_mut() {
                Self::skip_nodes_already_reached(path, entity);
            }
        }

        // Vanilla `PathNavigation.moveTo` with a null path clears the current
        // path and reports failure (PathNavigation.java:191-195); the mob
        // stands still until a goal issues a new request.
        if self.current_path.is_none() {
            self.is_idle.store(true, Ordering::Relaxed);
            entity.clear_speed();
            Self::stop_move_control(move_control);
            return;
        }

        if let Some(path) = &mut self.current_path {
            if path.is_done() || !path.is_valid() {
                // Arrived or path invalid — idle so goals can re-select.
                self.is_idle.store(true, Ordering::Relaxed);
                self.current_path = None;
                entity.clear_speed();
                Self::stop_move_control(move_control);
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
                // Stuck on a node — give up so wander can pick a new destination.
                self.is_idle.store(true, Ordering::Relaxed);
                self.current_path = None;
                self.ticks_on_current_node = 0;
                entity.clear_speed();
                Self::stop_move_control(move_control);
                return;
            }

            if self.total_ticks.is_multiple_of(100) {
                if let Some(start_pos) = self.path_start_pos {
                    let current_pos = entity.entity.pos.load();
                    let dx = current_pos.x - start_pos.x;
                    let dy = current_pos.y - start_pos.y;
                    let dz = current_pos.z - start_pos.z;
                    let dist_sq = dx * dx + dy * dy + dz * dz;
                    if dist_sq < 0.25 {
                        self.is_idle.store(true, Ordering::Relaxed);
                        self.current_path = None;
                        self.ticks_on_current_node = 0;
                        entity.clear_speed();
                        Self::stop_move_control(move_control);
                        return;
                    }
                }
                self.path_start_pos = Some(entity.entity.pos.load());
            }

            let on_ground = entity.entity.on_ground.load(Ordering::Relaxed);

            if let Some(next_block) = path.get_next_node_pos() {
                // Vanilla: Vec3.atBottomCenterOf(nextNodePos)
                let mut target_pos = Vector3::new(
                    f64::from(next_block.x) + 0.5,
                    f64::from(next_block.y),
                    f64::from(next_block.z) + 0.5,
                );
                target_pos.y = Self::get_ground_y(entity, target_pos);

                let current_pos = entity.entity.pos.load();
                let dx = target_pos.x - current_pos.x;
                let dy = target_pos.y - current_pos.y;
                let dz = target_pos.z - current_pos.z;

                let bbox = entity.entity.bounding_box.load();
                let width = (bbox.max.x - bbox.min.x).max(0.1);
                // Vanilla GroundPathNavigation.maxDistanceToWaypoint
                let max_waypoint = if width > 0.75 {
                    width * 0.5
                } else {
                    0.75 - width * 0.5
                };

                // Vanilla uses axis-aligned |dx|/|dz|, not euclidean — tighter and
                // matches "on the block" rather than a circle that overshoots edges.
                let near_xz = dx.abs() < max_waypoint && dz.abs() < max_waypoint;
                let horizontal_dist_sq = dx * dx + dz * dz;
                let horizontal_dist = horizontal_dist_sq.sqrt();

                // --- Vanilla followThePath reach checks (+ 1-block step-down fix) ---
                // 1) Airborne above next node → consume it while falling.
                if !on_ground && near_xz && dy < 0.0 {
                    path.advance();
                    self.current_goal = Some(goal);
                    return;
                }
                // 2) Vanilla: |dx|<wp && |dz|<wp && |dy|<1.0
                if near_xz && dy.abs() < NODE_REACH_Y {
                    path.advance();
                    self.current_goal = Some(goal);
                    return;
                }
                // 3) Exactly 1-block down: |dy|≈1.0 fails (2). Without this, mobs
                //    orbit the ledge, stuck-detect fires, and A* repaths around.
                if near_xz && dy < 0.0 && dy > -STEP_DOWN_REACH_Y {
                    path.advance();
                    self.current_goal = Some(goal);
                    return;
                }
                // 4) Still above a drop but already over the column — keep node if
                //    we're clearly committed (close XZ, within max fall).
                if dx.abs() < max_waypoint.max(0.6)
                    && dz.abs() < max_waypoint.max(0.6)
                    && dy < 0.0
                    && dy > -MAX_FALL_FOLLOW_Y
                    && (!on_ground || dy > -STEP_DOWN_REACH_Y)
                {
                    path.advance();
                    self.current_goal = Some(goal);
                    return;
                }

                // Vanilla PathNavigation hands each waypoint to MoveControl. Besides
                // setting yaw/speed, that controller jumps when a solid shape blocks a
                // direct fallback, which is what lets sheep clear a one-block stone.
                Self::set_wanted_position(move_control, target_pos, goal.speed);

                // Don't stuck-cancel while we're clearly approaching a step-down
                // (small horizontal progress is expected on the lip).
                if dy < 0.0 && dy > -MAX_FALL_FOLLOW_Y && horizontal_dist < 2.0 {
                    self.ticks_on_current_node = self.ticks_on_current_node.min(40);
                }
            } else {
                self.is_idle.store(true, Ordering::Relaxed);
                self.current_path = None;
                entity.clear_speed();
                Self::stop_move_control(move_control);
            }
        }

        self.current_goal = Some(goal);
    }

    #[must_use]
    pub fn is_idle(&self) -> bool {
        self.is_idle.load(Ordering::Relaxed)
    }
}
