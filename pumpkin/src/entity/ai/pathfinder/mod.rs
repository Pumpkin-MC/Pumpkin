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

// If I counted correctly this should be equal to the number of iters that vanilla does for
// a zombie (yes, vanilla does a different number of iterations based on the mob and some
// other things)
// TODO: Calculate from mob attributes like in vanilla
const MAX_ITERS: usize = 560;
// Vanilla PathFinder uses ~1.0 heuristic scale (some versions 1.5).
const TARGET_DISTANCE_MULTIPLIER: f32 = 1.0;
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
    pub async fn create_path_to(
        &mut self,
        entity: &LivingEntity,
        destination: Vector3<f64>,
    ) -> Option<Path> {
        self.compute_path(entity, destination).await
    }

    #[allow(clippy::too_many_lines)]
    async fn compute_path(
        &mut self,
        entity: &LivingEntity,
        destination: Vector3<f64>,
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

        let mut target = self.evaluator.get_target(BlockPos::floored_v(destination));

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
                // Prefer nearly-horizontal step-downs: charge mostly XZ distance so a
                // 1-block drop (vanilla getClosedNode edge) beats walking around a hill.
                // Full 3D distance alone makes drops look like √2 while a 2-long flat
                // detour is 2 — still OK, but with malus noise detours won too often.
                let step_cost = {
                    let full = current.distance(&neighbor);
                    let xz = current.distance_xz(&neighbor);
                    let dy = (neighbor.pos.0.y - current.pos.0.y).abs() as f32;
                    if dy > 0.0 && dy <= 1.0 && xz <= 1.5 {
                        // 1-block step up/down: treat as flat adjacency cost.
                        xz.max(1.0)
                    } else {
                        full
                    }
                };
                neighbor.walked_dist = current.walked_dist + step_cost;
                let tentative_g = current.g + step_cost + neighbor.cost_malus;

                let in_heap = self.open_set.contains(&neighbor);
                if neighbor.walked_dist < follow_range
                    && (!in_heap
                        || self
                            .open_set
                            .get_node(&neighbor)
                            .is_some_and(|existing| tentative_g < existing.g))
                {
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

            // Reject incomplete paths that get *farther* from the goal while also
            // being much longer than the straight line. Keep incomplete paths that
            // still move closer — better than freezing (vindicator/golem "stand and
            // stare" when A* can't fully reach).
            if !reached && path_nodes.len() >= 2 {
                let start = path_nodes[0].pos.0;
                let end = path_nodes[path_nodes.len() - 1].pos.0;
                let goal = target.node.pos.0;
                let path_len = {
                    let mut len = 0.0f32;
                    for w in path_nodes.windows(2) {
                        len += w[0].distance(&w[1]);
                    }
                    len
                };
                let start_to_goal = {
                    let dx = (goal.x - start.x) as f32;
                    let dy = (goal.y - start.y) as f32;
                    let dz = (goal.z - start.z) as f32;
                    (dx * dx + dy * dy + dz * dz).sqrt()
                };
                let end_to_goal = {
                    let dx = (goal.x - end.x) as f32;
                    let dy = (goal.y - end.y) as f32;
                    let dz = (goal.z - end.z) as f32;
                    (dx * dx + dy * dy + dz * dz).sqrt()
                };
                // Only drop if we did not get closer and the path is a long detour.
                if end_to_goal >= start_to_goal - 0.5 && path_len > start_to_goal.mul_add(3.0, 12.0)
                {
                    return None;
                }
            }

            let path_target = target.node.pos.0;
            return Some(Path::new(path_nodes, path_target, reached));
        }

        None
    }

    /// True when the path is much longer than the straight line to the goal —
    /// typical of A* circling hills/water while a clear charge exists.
    fn path_is_excessive_detour(
        path: &Path,
        from: Vector3<f64>,
        destination: Vector3<f64>,
    ) -> bool {
        let n = path.get_node_count();
        if n < 2 {
            return false;
        }
        let mut path_len = 0.0f64;
        for i in 0..(n - 1) {
            let Some(a) = path.get_node_pos(i) else {
                break;
            };
            let Some(b) = path.get_node_pos(i + 1) else {
                break;
            };
            let dx = f64::from(b.x - a.x);
            let dy = f64::from(b.y - a.y);
            let dz = f64::from(b.z - a.z);
            path_len += (dx * dx + dy * dy + dz * dz).sqrt();
        }
        let dx = destination.x - from.x;
        let dy = destination.y - from.y;
        let dz = destination.z - from.z;
        let straight = (dx * dx + dy * dy + dz * dz).sqrt().max(0.5);
        // > ~40% longer than straight and at least 4 blocks of waste → detour.
        path_len > straight * 1.4 + 4.0
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

    /// Best-effort straight walk when A* fails — keeps melee chases moving instead
    /// of freezing until the next repath luckily succeeds. This still goes through
    /// `MoveControl`, which owns collision-triggered jumping in vanilla.
    fn direct_walk_toward(
        &self,
        entity: &LivingEntity,
        goal: &NavigatorGoal,
        move_control: &Mutex<Box<dyn MoveControlTrait>>,
    ) {
        let current_pos = entity.entity.pos.load();

        // Iron golem etc.: never direct-walk into water when water is impassable.
        if self.avoids_water() {
            let world = entity.entity.world.load();
            let dest_block = BlockPos::floored_v(goal.destination);
            let is_water = |p: pumpkin_util::math::position::BlockPos| {
                use pumpkin_data::tag::Taggable;
                let id = world.get_block_state_id(&p);
                pumpkin_data::fluid::Fluid::from_state_id(id)
                    .is_some_and(|f| f.has_tag(&pumpkin_data::tag::Fluid::MINECRAFT_WATER))
            };
            if is_water(dest_block) || is_water(dest_block.down()) {
                entity.clear_speed();
                Self::stop_move_control(move_control);
                return;
            }
            // Also refuse a step that would walk off land into water immediately ahead.
            let ahead = current_pos.add_raw(
                (goal.destination.x - current_pos.x)
                    .signum()
                    .clamp(-1.0, 1.0),
                0.0,
                (goal.destination.z - current_pos.z)
                    .signum()
                    .clamp(-1.0, 1.0),
            );
            let ahead_block = BlockPos::floored_v(ahead);
            if is_water(ahead_block) || is_water(ahead_block.down()) {
                entity.clear_speed();
                Self::stop_move_control(move_control);
                return;
            }
        }

        let dx = goal.destination.x - current_pos.x;
        let dz = goal.destination.z - current_pos.z;
        let horizontal = dx.hypot(dz);
        if horizontal < 0.05 {
            entity.clear_speed();
            Self::stop_move_control(move_control);
            return;
        }

        Self::set_wanted_position(move_control, goal.destination, goal.speed);
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
            // Idle: stop the mob
            self.is_idle.store(true, Ordering::Relaxed);
            entity.clear_speed();
            Self::stop_move_control(move_control);
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
            self.current_path = self.compute_path(entity, goal.destination).await;
            self.ticks_on_current_node = 0;
            self.last_node_index = 0;
            self.path_start_pos = Some(entity.entity.pos.load());
            self.repath_cooldown = 10; // repath a bit faster during chase

            // Drop long detours: iron golems / melee should charge the target, not
            // walk behind it then turn around (vanilla MoveControl aims straight
            // when createPath is short; long A* loops look like reverse chase).
            if let Some(path) = self.current_path.as_ref()
                && Self::path_is_excessive_detour(path, entity.entity.pos.load(), goal.destination)
            {
                self.current_path = None;
            }

            // Skip start node(s) already under our feet so we don't first step
            // "backward" onto the path origin block.
            if let Some(path) = self.current_path.as_mut() {
                Self::skip_nodes_already_reached(path, entity);
            }

            // A* failed: keep the goal and walk straight toward the destination
            // so melee mobs (vindicator/golem/zombie) don't freeze in place.
            if self.current_path.is_none() {
                self.is_idle.store(false, Ordering::Relaxed);
                self.direct_walk_toward(entity, &goal, move_control);
                self.current_goal = Some(goal);
                return;
            }
        }

        if self.current_path.is_none() {
            self.is_idle.store(false, Ordering::Relaxed);
            self.direct_walk_toward(entity, &goal, move_control);
            self.current_goal = Some(goal);
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
