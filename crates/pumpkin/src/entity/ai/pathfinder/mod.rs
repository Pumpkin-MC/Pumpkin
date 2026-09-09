use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::math::wrap_degrees;

use crate::entity::living::LivingEntity;
use crate::world::World;

use crate::entity::ai::pathfinder::amphibious_node_evaluator::AmphibiousNodeEvaluator;
use crate::entity::ai::pathfinder::binary_heap::BinaryHeap;
use crate::entity::ai::pathfinder::fly_node_evaluator::FlyNodeEvaluator;
use crate::entity::ai::pathfinder::node::Node;
use crate::entity::ai::pathfinder::node::PathType;
use crate::entity::ai::pathfinder::node_evaluator::{MobData, NodeEvaluator};
use crate::entity::ai::pathfinder::path::Path;
use crate::entity::ai::pathfinder::pathfinding_context::PathfindingContext;
use crate::entity::ai::pathfinder::swim_node_evaluator::SwimNodeEvaluator;
use crate::entity::ai::pathfinder::walk_node_evaluator::WalkNodeEvaluator;
use pumpkin_data::attributes::Attributes;
use rustc_hash::{FxHashMap, FxHashSet};
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicBool, Ordering};

pub mod amphibious_node_evaluator;
pub mod binary_heap;
pub mod fly_node_evaluator;
pub mod node;
pub mod node_evaluator;
pub mod path;
pub mod path_type_cache;
pub mod pathfinding_context;
pub mod swim_node_evaluator;
pub mod walk_node_evaluator;

const MAX_ITERS: usize = 560;
const TARGET_DISTANCE_MULTIPLIER: f32 = 1.5;
const NODE_REACH_Y: f64 = 1.0;
const MAX_YAW_TURN_PER_TICK: f32 = 90.0;

pub struct PathFinder {
    max_visited_nodes: usize,
    open_set: BinaryHeap,
    neighbors_buf: Vec<Node>,
    all_nodes: FxHashMap<Vector3<i32>, Node>,
}

impl PathFinder {
    #[must_use]
    pub fn new(max_visited_nodes: usize) -> Self {
        Self {
            max_visited_nodes,
            open_set: BinaryHeap::new(),
            neighbors_buf: Vec::with_capacity(32),
            all_nodes: FxHashMap::default(),
        }
    }

    pub const fn set_max_visited_nodes(&mut self, max_visited_nodes: usize) {
        self.max_visited_nodes = max_visited_nodes;
    }

    #[must_use]
    pub const fn get_max_visited_nodes(&self) -> usize {
        self.max_visited_nodes
    }

    pub fn find_path_single(
        &mut self,
        evaluator: &mut EvaluatorKind,
        target: BlockPos,
        max_path_length: f32,
        reach_range: i32,
        max_visited_nodes_multiplier: f32,
    ) -> Option<Path> {
        self.find_path(
            evaluator,
            &[target],
            max_path_length,
            reach_range,
            max_visited_nodes_multiplier,
        )
    }

    #[expect(clippy::too_many_lines)]
    pub fn find_path(
        &mut self,
        evaluator: &mut EvaluatorKind,
        targets: &[BlockPos],
        max_path_length: f32,
        reach_range: i32,
        max_visited_nodes_multiplier: f32,
    ) -> Option<Path> {
        if targets.is_empty() {
            return None;
        }

        let mut from = evaluator.get_start()?;

        let mut target_entries: Vec<(crate::entity::ai::pathfinder::node::Target, BlockPos)> =
            targets
                .iter()
                .map(|&pos| (evaluator.get_target(pos), pos))
                .collect();

        self.all_nodes.clear();
        self.open_set.clear();

        from.g = 0.0;
        from.h = Self::get_best_h(&from, &mut target_entries);
        from.f = from.h;
        from.walked_dist = 0.0;
        from.came_from = None;
        from.closed = false;

        self.all_nodes.insert(from.pos.0, from);
        self.open_set.insert(from);

        let mut count = 0usize;
        let mut reached_targets: Vec<usize> = Vec::new();
        let max_visited_nodes_adjusted =
            (self.max_visited_nodes as f32 * max_visited_nodes_multiplier) as usize;

        while !self.open_set.is_empty() {
            count += 1;
            if count >= max_visited_nodes_adjusted {
                break;
            }

            let Some(mut current) = self.open_set.pop() else {
                break;
            };

            current.closed = true;
            self.all_nodes.insert(current.pos.0, current);

            for (idx, (target, _)) in target_entries.iter_mut().enumerate() {
                if current.distance_manhattan_node(&target.node) <= reach_range as f32 {
                    target.set_reached();
                    target.update_best(0.0, &current);
                    if !reached_targets.contains(&idx) {
                        reached_targets.push(idx);
                    }
                }
            }

            if !reached_targets.is_empty() {
                break;
            }

            if current.distance_to_node(&from) < max_path_length {
                self.neighbors_buf.clear();
                evaluator.get_neighbors(&current, &mut self.neighbors_buf);

                for mut neighbor in self.neighbors_buf.drain(..) {
                    let distance = current.distance_to_node(&neighbor);
                    neighbor.walked_dist = current.walked_dist + distance;
                    let tentative_g = current.g + distance + neighbor.cost_malus;

                    let in_open = self.open_set.contains(&neighbor);
                    let is_better = if in_open {
                        self.open_set
                            .get_node(&neighbor)
                            .is_some_and(|existing| tentative_g < existing.g)
                    } else {
                        true
                    };

                    if neighbor.walked_dist < max_path_length && is_better {
                        neighbor.came_from = Some(current.pos.0);
                        neighbor.g = tentative_g;
                        neighbor.h = Self::get_best_h(&neighbor, &mut target_entries) * 1.5;
                        neighbor.f = neighbor.g + neighbor.h;

                        if in_open {
                            self.open_set.update_node(&neighbor, neighbor);
                        } else {
                            self.open_set.insert(neighbor);
                        }
                        self.all_nodes.insert(neighbor.pos.0, neighbor);
                    }
                }
            }
        }

        for node in self.open_set.drain() {
            self.all_nodes.entry(node.pos.0).or_insert(node);
        }

        if reached_targets.is_empty() {
            target_entries
                .into_iter()
                .filter_map(|(target, target_pos)| {
                    target.get_best_node().map(|best| {
                        Self::reconstruct_path(&best, target_pos, false, &self.all_nodes)
                    })
                })
                .min_by(|a, b| {
                    a.get_dist_to_target()
                        .total_cmp(&b.get_dist_to_target())
                        .then_with(|| a.get_node_count().cmp(&b.get_node_count()))
                })
        } else {
            reached_targets
                .into_iter()
                .filter_map(|idx| {
                    let (target, target_pos) = &target_entries[idx];
                    target.get_best_node().map(|best| {
                        Self::reconstruct_path(&best, *target_pos, true, &self.all_nodes)
                    })
                })
                .min_by_key(Path::get_node_count)
        }
    }

    fn reconstruct_path(
        closest: &Node,
        target: BlockPos,
        reached: bool,
        all_nodes: &FxHashMap<Vector3<i32>, Node>,
    ) -> Path {
        let mut nodes = Vec::new();
        let mut current = *closest;
        nodes.push(current);
        let mut visited = FxHashSet::default();
        visited.insert(current.pos.0);

        while let Some(prev_pos) = current.came_from {
            if prev_pos == current.pos.0 || !visited.insert(prev_pos) {
                break;
            }
            if let Some(prev_node) = all_nodes.get(&prev_pos) {
                nodes.push(*prev_node);
                current = *prev_node;
            } else {
                break;
            }
        }

        nodes.reverse();
        Path::new(nodes, target, reached)
    }

    fn get_best_h(
        from: &Node,
        targets: &mut [(crate::entity::ai::pathfinder::node::Target, BlockPos)],
    ) -> f32 {
        let mut best_h = f32::MAX;
        for (target, _) in targets.iter_mut() {
            let h = from.distance_to_node(&target.node);
            target.update_best(h, from);
            best_h = best_h.min(h);
        }
        best_h
    }
}

#[derive(Clone, Copy, Debug)]
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

pub enum EvaluatorKind {
    Walk(WalkNodeEvaluator),
    Fly(FlyNodeEvaluator),
    Swim(SwimNodeEvaluator),
    Amphibious(AmphibiousNodeEvaluator),
}

impl EvaluatorKind {
    pub fn prepare(&mut self, context: PathfindingContext, mob_data: MobData) {
        match self {
            Self::Walk(e) => e.prepare(context, mob_data),
            Self::Fly(e) => e.prepare(context, mob_data),
            Self::Swim(e) => e.prepare(context, mob_data),
            Self::Amphibious(e) => e.prepare(context, mob_data),
        }
    }

    pub fn done(&mut self) {
        match self {
            Self::Walk(e) => e.done(),
            Self::Fly(e) => e.done(),
            Self::Swim(e) => e.done(),
            Self::Amphibious(e) => e.done(),
        }
    }

    pub fn get_start(&mut self) -> Option<Node> {
        match self {
            Self::Walk(e) => e.get_start(),
            Self::Fly(e) => e.get_start(),
            Self::Swim(e) => e.get_start(),
            Self::Amphibious(e) => e.get_start(),
        }
    }

    pub fn get_target(&mut self, pos: BlockPos) -> crate::entity::ai::pathfinder::node::Target {
        match self {
            Self::Walk(e) => e.get_target(pos),
            Self::Fly(e) => e.get_target(pos),
            Self::Swim(e) => e.get_target(pos),
            Self::Amphibious(e) => e.get_target(pos),
        }
    }

    pub fn get_neighbors(&mut self, current: &Node, out: &mut Vec<Node>) {
        match self {
            Self::Walk(e) => e.get_neighbors(current, out),
            Self::Fly(e) => e.get_neighbors(current, out),
            Self::Swim(e) => e.get_neighbors(current, out),
            Self::Amphibious(e) => e.get_neighbors(current, out),
        }
    }

    pub fn set_can_float(&mut self, can_float: bool) {
        match self {
            Self::Walk(e) => e.set_can_float(can_float),
            Self::Fly(e) => e.set_can_float(can_float),
            Self::Swim(e) => e.set_can_float(can_float),
            Self::Amphibious(e) => e.set_can_float(can_float),
        }
    }

    pub fn set_can_open_doors(&mut self, can_open: bool) {
        match self {
            Self::Walk(e) => e.set_can_open_doors(can_open),
            Self::Fly(e) => e.set_can_open_doors(can_open),
            Self::Swim(e) => e.set_can_open_doors(can_open),
            Self::Amphibious(e) => e.set_can_open_doors(can_open),
        }
    }

    pub fn set_can_pass_doors(&mut self, can_pass: bool) {
        match self {
            Self::Walk(e) => e.set_can_pass_doors(can_pass),
            Self::Fly(e) => e.set_can_pass_doors(can_pass),
            Self::Swim(e) => e.set_can_pass_doors(can_pass),
            Self::Amphibious(e) => e.set_can_pass_doors(can_pass),
        }
    }

    pub fn set_can_walk_over_fences(&mut self, can_walk: bool) {
        match self {
            Self::Walk(e) => e.set_can_walk_over_fences(can_walk),
            Self::Fly(e) => e.set_can_walk_over_fences(can_walk),
            Self::Swim(e) => e.set_can_walk_over_fences(can_walk),
            Self::Amphibious(e) => e.set_can_walk_over_fences(can_walk),
        }
    }
}

pub trait PathNavigationTrait: Send + Sync {
    fn set_progress(&mut self, goal: NavigatorGoal);
    fn set_speed(&mut self, speed: f64);
    fn stop(&mut self);
    fn is_idle(&self) -> bool;
    fn is_done(&self) -> bool;
    fn is_in_progress(&self) -> bool;
    fn is_stuck(&self) -> bool;
    fn get_path(&self) -> Option<&Path>;
    fn get_path_mut(&mut self) -> Option<&mut Path>;
    fn set_pathfinding_malus(&mut self, path_type: PathType, malus: f32);
    fn get_pathfinding_malus(&self, path_type: PathType) -> f32;
    fn set_mob_dimensions(&mut self, width: f32, height: f32);
    fn can_reach_within(
        &mut self,
        entity: &LivingEntity,
        destination: Vector3<f64>,
        distance: f32,
    ) -> bool;
    fn tick(&mut self, entity: &LivingEntity);
    fn move_to_coords(&mut self, x: f64, y: f64, z: f64, speed: f64, entity: &LivingEntity)
    -> bool;
    fn move_to_pos(&mut self, pos: BlockPos, speed: f64, entity: &LivingEntity) -> bool;
    fn move_to_entity(&mut self, target: &LivingEntity, speed: f64, entity: &LivingEntity) -> bool;
    fn move_to_path(&mut self, path: Option<Path>, speed: f64, entity: &LivingEntity) -> bool;
    fn create_path(
        &mut self,
        entity: &LivingEntity,
        destination: Vector3<f64>,
        reach_range: i32,
    ) -> Option<Path>;
    fn recompute_path(&mut self, entity: &LivingEntity);
    fn set_avoid_sun(&mut self, avoid_sun: bool);
    fn set_can_walk_over_fences(&mut self, can_walk: bool);
    fn set_can_open_doors(&mut self, can_open: bool);
    fn set_can_pass_doors(&mut self, can_pass: bool);
    fn set_can_float(&mut self, can_float: bool);
    fn can_float(&self) -> bool;
    fn can_navigate_ground(&self) -> bool;
    fn set_required_path_length(&mut self, length: f32);
    fn set_max_visited_nodes_multiplier(&mut self, multiplier: f32);
    fn reset_max_visited_nodes_multiplier(&mut self);
    fn get_target_pos(&self) -> Option<BlockPos>;
    fn can_path_to_targets_below_surface(&self) -> bool;
    fn set_can_path_to_targets_below_surface(&mut self, can_path: bool);
}

pub struct PathNavigation {
    pub current_goal: Option<NavigatorGoal>,
    pub evaluator: EvaluatorKind,
    pub path: Option<Path>,
    pub speed_modifier: f64,
    pub tick_count: u32,
    pub last_stuck_check: u32,
    pub last_stuck_check_pos: Vector3<f64>,
    pub timeout_cached_node: Vector3<i32>,
    pub timeout_timer: u64,
    pub last_timeout_check: u64,
    pub timeout_limit: f64,
    pub max_distance_to_waypoint: f32,
    pub has_delayed_recomputation: bool,
    pub time_last_recompute: u64,
    pub target_pos: Option<BlockPos>,
    pub reach_range: i32,
    pub max_visited_nodes_multiplier: f32,
    pub is_stuck: bool,
    pub required_path_length: f32,
    pub ticks_on_current_node: u32,
    pub last_node_index: usize,
    pub total_ticks: u32,
    pub path_start_pos: Option<Vector3<f64>>,
    pub path_type_overrides: FxHashMap<PathType, f32>,
    pub mob_width: f32,
    pub mob_height: f32,
    pub repath_cooldown: u32,
    pub can_float: bool,
    pub can_walk_over_fences: bool,
    pub can_open_doors: bool,
    pub can_pass_doors: bool,
    pub avoid_sun: bool,
    pub can_path_to_targets_below_surface: bool,
    pub open_set: BinaryHeap,
    pub neighbors_buf: Vec<Node>,
    pub is_idle: AtomicBool,
}

impl Default for PathNavigation {
    fn default() -> Self {
        Self::new(EvaluatorKind::Walk(WalkNodeEvaluator::default()))
    }
}

impl PathNavigation {
    #[must_use]
    pub fn new(evaluator: EvaluatorKind) -> Self {
        Self {
            current_goal: None,
            evaluator,
            path: None,
            speed_modifier: 1.0,
            tick_count: 0,
            last_stuck_check: 0,
            last_stuck_check_pos: Vector3::new(0.0, 0.0, 0.0),
            timeout_cached_node: Vector3::new(0, 0, 0),
            timeout_timer: 0,
            last_timeout_check: 0,
            timeout_limit: 0.0,
            max_distance_to_waypoint: 0.5,
            has_delayed_recomputation: false,
            time_last_recompute: 0,
            target_pos: None,
            reach_range: 1,
            max_visited_nodes_multiplier: 1.0,
            is_stuck: false,
            required_path_length: 16.0,
            ticks_on_current_node: 0,
            last_node_index: 0,
            total_ticks: 0,
            path_start_pos: None,
            path_type_overrides: FxHashMap::default(),
            mob_width: 0.6,
            mob_height: 1.95,
            repath_cooldown: 0,
            can_float: false,
            can_walk_over_fences: false,
            can_open_doors: false,
            can_pass_doors: true,
            avoid_sun: false,
            can_path_to_targets_below_surface: false,
            open_set: BinaryHeap::new(),
            neighbors_buf: Vec::new(),
            is_idle: AtomicBool::new(true),
        }
    }

    pub fn set_progress(&mut self, goal: NavigatorGoal) {
        self.is_idle.store(false, Ordering::Relaxed);
        self.speed_modifier = goal.speed;
        self.current_goal = Some(goal);
        self.path = None;
    }

    pub const fn set_speed(&mut self, speed: f64) {
        self.speed_modifier = speed;
        if let Some(goal) = &mut self.current_goal {
            goal.speed = speed;
        }
    }

    pub fn stop(&mut self) {
        self.is_idle.store(true, Ordering::Relaxed);
        self.current_goal = None;
        self.path = None;
        self.ticks_on_current_node = 0;
        self.total_ticks = 0;
        self.path_start_pos = None;
        self.reset_stuck_timeout();
    }

    pub fn finish_navigation(&mut self, entity: &LivingEntity) {
        self.stop();
        entity.movement_input.store(Vector3::new(0.0, 0.0, 0.0));
        entity.jumping.store(false, Ordering::Relaxed);
    }

    pub fn set_pathfinding_malus(&mut self, path_type: PathType, malus: f32) {
        self.path_type_overrides.insert(path_type, malus);
    }

    #[must_use]
    pub fn get_pathfinding_malus(&self, path_type: PathType) -> f32 {
        self.path_type_overrides
            .get(&path_type)
            .copied()
            .unwrap_or_else(|| path_type.get_malus())
    }

    pub const fn set_mob_dimensions(&mut self, width: f32, height: f32) {
        self.mob_width = width;
        self.mob_height = height;
    }

    pub fn can_reach_within(
        &mut self,
        entity: &LivingEntity,
        destination: Vector3<f64>,
        distance: f32,
    ) -> bool {
        self.compute_path(entity, destination, 1)
            .is_some_and(|path| path.can_reach() || path.get_dist_to_target() <= distance)
    }

    fn mob_max_follow_range(&self, entity: &LivingEntity) -> f32 {
        let follow_range = entity.get_attribute_value(&Attributes::FOLLOW_RANGE) as f32;
        follow_range.max(self.required_path_length)
    }

    #[allow(clippy::too_many_lines)]
    pub fn compute_path(
        &mut self,
        entity: &LivingEntity,
        destination: Vector3<f64>,
        reach_range: i32,
    ) -> Option<Path> {
        let start_pos_f = entity.entity.pos.load();
        let start_block_vec = start_pos_f.to_i32();
        let mob_position = Vector3::new(start_block_vec.x, start_block_vec.y, start_block_vec.z);

        let context = PathfindingContext::new(mob_position, entity.entity.world.load_full());
        let mut mob_data = MobData::new(start_pos_f, self.mob_width, self.mob_height, 1.0);
        mob_data.on_ground = entity.entity.on_ground.load(Ordering::Relaxed);
        mob_data.can_swim = self.can_float;

        mob_data.set_pathfinding_malus(PathType::DangerFire, 16.0);
        mob_data.set_pathfinding_malus(PathType::DamageFire, -1.0);
        mob_data.set_pathfinding_malus(PathType::Water, if self.can_float { 0.0 } else { 8.0 });
        mob_data.set_pathfinding_malus(PathType::Lava, -1.0);
        mob_data.set_pathfinding_malus(PathType::DangerOther, 8.0);

        for (&path_type, &malus) in &self.path_type_overrides {
            mob_data.set_pathfinding_malus(path_type, malus);
        }

        self.evaluator.set_can_float(self.can_float);
        self.evaluator.set_can_open_doors(self.can_open_doors);
        self.evaluator.set_can_pass_doors(self.can_pass_doors);
        self.evaluator
            .set_can_walk_over_fences(self.can_walk_over_fences);
        self.evaluator.prepare(context, mob_data);

        let mut start_node = self.evaluator.get_start()?;
        let mut target = self.evaluator.get_target(destination.to_block_pos());

        start_node.g = 0.0;
        let start_dist = start_node.distance(&target);
        target.update_best(start_dist, &start_node);
        start_node.h = start_dist;
        start_node.f = start_node.h;
        start_node.walked_dist = 0.0;
        start_node.came_from = None;

        let start_pos = start_node.pos.0;
        let mut closed_set: FxHashMap<Vector3<i32>, Node> = FxHashMap::default();

        self.open_set.clear();
        self.open_set.insert(start_node);

        let mut iterations = 0usize;
        let mut reached = false;
        let max_iters = ((self.mob_max_follow_range(entity)
            * 16.0
            * self.max_visited_nodes_multiplier) as usize)
            .clamp(100, 2048)
            .max(MAX_ITERS);

        while !self.open_set.is_empty() {
            iterations += 1;
            if iterations >= max_iters {
                break;
            }

            let Some(current) = self.open_set.pop() else {
                break;
            };

            if current.distance_manhattan(&target) <= reach_range as f32 {
                target.reached = true;
                reached = true;
                target.update_best(0.0, &current);
                closed_set.insert(current.pos.0, current);
                break;
            }

            let dx = (current.pos.0.x - start_pos.x) as f32;
            let dy = (current.pos.0.y - start_pos.y) as f32;
            let dz = (current.pos.0.z - start_pos.z) as f32;
            let euclidean = (dx * dx + dy * dy + dz * dz).sqrt();
            let follow_range = self.mob_max_follow_range(entity);
            if euclidean >= follow_range {
                closed_set.insert(current.pos.0, current);
                continue;
            }

            self.neighbors_buf.clear();
            self.evaluator
                .get_neighbors(&current, &mut self.neighbors_buf);

            for mut neighbor in self.neighbors_buf.drain(..) {
                let step_cost = current.distance(&neighbor);
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

        self.evaluator.done();

        for node in self.open_set.drain() {
            closed_set.entry(node.pos.0).or_insert(node);
        }

        if let Some(best_node) = target.best_node {
            let mut path_nodes: Vec<Node> = Vec::new();
            let mut current_pos = best_node.pos.0;
            path_nodes.push(best_node);
            let mut visited: FxHashSet<Vector3<i32>> = FxHashSet::default();
            visited.insert(current_pos);
            while let Some(node) = closed_set.get(&current_pos) {
                if let Some(prev_pos) = node.came_from {
                    if prev_pos == current_pos || !visited.insert(prev_pos) {
                        break;
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

            let path_target = target.node.pos;
            return Some(Path::new(path_nodes, path_target, reached));
        }

        None
    }

    pub fn needs_new_path(&self, goal: &NavigatorGoal) -> bool {
        if self.path.is_none() {
            return true;
        }
        if self.repath_cooldown > 0 {
            return false;
        }
        self.path.as_ref().is_some_and(|p| {
            let path_target = p.get_target();
            let goal_target = goal.destination.to_i32();
            let dx = f64::from(path_target.0.x - goal_target.x);
            let dy = f64::from(path_target.0.y - goal_target.y);
            let dz = f64::from(path_target.0.z - goal_target.z);
            let distance_sq = dx * dx + dy * dy + dz * dz;
            let remaining = p.get_remaining_distance().clamp(4.0, 16.0);
            let threshold = remaining * 0.5;
            distance_sq > f64::from(threshold * threshold)
        })
    }

    pub fn can_move_directly(
        world: &World,
        start_pos: Vector3<f64>,
        stop_pos: Vector3<f64>,
        mob_height: f32,
        blocked_by_fluids: bool,
    ) -> bool {
        let to = Vector3::new(
            stop_pos.x,
            stop_pos.y + f64::from(mob_height) * 0.5,
            stop_pos.z,
        );
        let diff = to - start_pos;
        let dist = diff.length();
        if dist < 1e-5 {
            return true;
        }
        let steps = (dist * 2.0).ceil() as usize;
        let step_vec = diff / (steps as f64);
        let mut current = start_pos;
        for _ in 0..steps {
            current += step_vec;
            let bp = BlockPos::new(
                current.x.floor() as i32,
                current.y.floor() as i32,
                current.z.floor() as i32,
            );
            let block_state = world.get_block_state(&bp);
            if !block_state.is_air() {
                if blocked_by_fluids && block_state.is_liquid() {
                    return false;
                }
                if block_state.is_solid() {
                    return false;
                }
            }
        }
        true
    }

    pub fn find_surface_position(world: &World, mut pos: BlockPos) -> BlockPos {
        let block_state = world.get_block_state(&pos);
        if block_state.is_air() {
            let mut column_pos = pos;
            while column_pos.0.y >= -64 && world.get_block_state(&column_pos).is_air() {
                column_pos.0.y -= 1;
            }
            if column_pos.0.y >= -64 {
                return BlockPos::new(column_pos.0.x, column_pos.0.y + 1, column_pos.0.z);
            }
            column_pos.0.y = pos.0.y + 1;
            while column_pos.0.y <= 320 && world.get_block_state(&column_pos).is_air() {
                column_pos.0.y += 1;
            }
            pos = column_pos;
        }
        if !world.get_block_state(&pos).is_solid() {
            return pos;
        }
        let mut column_pos = pos;
        while column_pos.0.y <= 320 && world.get_block_state(&column_pos).is_solid() {
            column_pos.0.y += 1;
        }
        column_pos
    }

    pub fn get_surface_y(&self, entity: &LivingEntity) -> f64 {
        let pos = entity.entity.pos.load();
        if entity.entity.touching_water.load(Ordering::Relaxed) && self.can_float {
            let mut surface = pos.y.floor() as i32;
            let world = entity.entity.world.load();
            let mut steps = 0;
            loop {
                let bp = BlockPos::new(pos.x.floor() as i32, surface, pos.z.floor() as i32);
                let state = world.get_block_state(&bp);
                if state.is_liquid() {
                    surface += 1;
                    steps += 1;
                    if steps > 16 {
                        return pos.y;
                    }
                } else {
                    break;
                }
            }
            f64::from(surface)
        } else {
            (pos.y + 0.5).floor()
        }
    }

    pub const fn reset_stuck_timeout(&mut self) {
        self.timeout_cached_node = Vector3::new(0, 0, 0);
        self.timeout_timer = 0;
        self.timeout_limit = 0.0;
        self.is_stuck = false;
    }

    pub fn do_stuck_detection(&mut self, mob_pos: Vector3<f64>, entity: &LivingEntity) {
        let world_age = entity.entity.world.load().get_world_age() as u64;
        if self.tick_count.saturating_sub(self.last_stuck_check) > 100 {
            let speed = entity.get_attribute_value(&Attributes::MOVEMENT_SPEED) as f32;
            let effective_speed = if speed >= 1.0 { speed } else { speed * speed };
            let threshold_distance = effective_speed * 100.0 * 0.25;
            let dx = mob_pos.x - self.last_stuck_check_pos.x;
            let dy = mob_pos.y - self.last_stuck_check_pos.y;
            let dz = mob_pos.z - self.last_stuck_check_pos.z;
            let dist_sq = dx * dx + dy * dy + dz * dz;

            if dist_sq < f64::from(threshold_distance * threshold_distance) {
                self.is_stuck = true;
                self.stop();
            } else {
                self.is_stuck = false;
            }
            self.last_stuck_check = self.tick_count;
            self.last_stuck_check_pos = mob_pos;
        }

        if let Some(path) = &self.path
            && !path.is_done()
            && let Some(pos) = path.get_next_node_pos()
        {
            if pos.0 == self.timeout_cached_node {
                self.timeout_timer = self
                    .timeout_timer
                    .saturating_add(world_age.saturating_sub(self.last_timeout_check));
            } else {
                self.timeout_cached_node = pos.0;
                let node_center = Vector3::new(
                    f64::from(pos.0.x) + 0.5,
                    f64::from(pos.0.y),
                    f64::from(pos.0.z) + 0.5,
                );
                let dist_to_node = (mob_pos - node_center).length();
                let speed = entity.get_attribute_value(&Attributes::MOVEMENT_SPEED);
                self.timeout_limit = if speed > 0.0 {
                    dist_to_node / speed * 20.0
                } else {
                    0.0
                };
            }

            if self.timeout_limit > 0.0 && self.timeout_timer as f64 > self.timeout_limit * 3.0 {
                self.reset_stuck_timeout();
                self.stop();
            }
            self.last_timeout_check = world_age;
        }
    }

    #[must_use]
    pub fn should_target_next_node_in_direction(mob_pos: Vector3<f64>, path: &Path) -> bool {
        if path.get_next_node_index() + 1 >= path.get_node_count() {
            return false;
        }
        let Some(curr_pos) = path.get_next_node_pos() else {
            return false;
        };
        let current_node = Vector3::new(
            f64::from(curr_pos.0.x) + 0.5,
            f64::from(curr_pos.0.y),
            f64::from(curr_pos.0.z) + 0.5,
        );
        let dx = mob_pos.x - current_node.x;
        let dy = mob_pos.y - current_node.y;
        let dz = mob_pos.z - current_node.z;
        if dx * dx + dy * dy + dz * dz > 4.0 {
            return false;
        }

        let next_idx = path.get_next_node_index() + 1;
        let Some(next_pos) = path.get_node_pos(next_idx) else {
            return false;
        };
        let next_node = Vector3::new(
            f64::from(next_pos.0.x) + 0.5,
            f64::from(next_pos.0.y),
            f64::from(next_pos.0.z) + 0.5,
        );

        let mob_to_current = current_node - mob_pos;
        let mob_to_next = next_node - mob_pos;
        let mob_to_curr_sqr = mob_to_current.length_squared();
        let mob_to_next_sqr = mob_to_next.length_squared();

        let closer_to_next = mob_to_next_sqr < mob_to_curr_sqr;
        let within_curr = mob_to_curr_sqr < 0.5;
        if !closer_to_next && !within_curr {
            return false;
        }

        let curr_len = mob_to_current.length();
        let next_len = mob_to_next.length();
        if curr_len < 1e-5 || next_len < 1e-5 {
            return false;
        }
        let mob_dir = mob_to_current / curr_len;
        let path_dir = mob_to_next / next_len;
        path_dir.dot(&mob_dir) < 0.0
    }

    pub fn trim_path(&mut self, entity: &LivingEntity) {
        if self.avoid_sun {
            let pos = entity.entity.pos.load();
            let bp = BlockPos::new(
                pos.x.floor() as i32,
                (pos.y + 0.5).floor() as i32,
                pos.z.floor() as i32,
            );
            let world = entity.entity.world.load();
            if world.get_sky_light_level(&bp) < 15
                && let Some(path) = &mut self.path
            {
                let mut cut_index = None;
                for i in 0..path.get_node_count() {
                    if let Some(node) = path.get_node(i) {
                        let node_bp = BlockPos::new(node.pos.0.x, node.pos.0.y, node.pos.0.z);
                        if world.get_sky_light_level(&node_bp) >= 15 {
                            cut_index = Some(i);
                            break;
                        }
                    }
                }
                if let Some(idx) = cut_index {
                    path.truncate_nodes(idx);
                }
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn tick_ground(&mut self, entity: &LivingEntity) {
        self.tick_count += 1;
        let world_age = entity.entity.world.load().get_world_age() as u64;

        if self.has_delayed_recomputation
            && world_age.saturating_sub(self.time_last_recompute) > 20
            && let Some(target_pos) = self.target_pos
        {
            let target_v = Vector3::new(
                f64::from(target_pos.0.x) + 0.5,
                f64::from(target_pos.0.y),
                f64::from(target_pos.0.z) + 0.5,
            );
            self.path = self.compute_path(entity, target_v, self.reach_range);
            self.time_last_recompute = world_age;
            self.has_delayed_recomputation = false;
        }

        let Some(goal) = self.current_goal.take() else {
            self.is_idle.store(true, Ordering::Relaxed);
            entity.movement_input.store(Vector3::new(0.0, 0.0, 0.0));
            return;
        };

        if goal.current_progress == goal.destination {
            self.finish_navigation(entity);
            return;
        }

        self.total_ticks += 1;
        if self.repath_cooldown > 0 {
            self.repath_cooldown -= 1;
        }

        if self.needs_new_path(&goal) {
            let mut dest_pos = goal.destination.to_block_pos();
            if !self.can_path_to_targets_below_surface {
                let world = entity.entity.world.load();
                dest_pos = Self::find_surface_position(&world, dest_pos);
            }
            let dest_v = Vector3::new(
                f64::from(dest_pos.0.x) + 0.5,
                f64::from(dest_pos.0.y),
                f64::from(dest_pos.0.z) + 0.5,
            );
            self.path = self.compute_path(entity, dest_v, self.reach_range);
            self.ticks_on_current_node = 0;
            self.last_node_index = 0;
            self.path_start_pos = Some(entity.entity.pos.load());
            self.repath_cooldown = 15;
            self.time_last_recompute = world_age;
        }

        if self.path.is_none() {
            self.finish_navigation(entity);
            return;
        }

        let surface_y = self.get_surface_y(entity);
        let mob_pos = Vector3::new(
            entity.entity.pos.load().x,
            surface_y,
            entity.entity.pos.load().z,
        );

        self.do_stuck_detection(mob_pos, entity);

        if let Some(path) = &mut self.path {
            if path.is_done() {
                self.finish_navigation(entity);
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
                self.finish_navigation(entity);
                return;
            }

            if self.total_ticks.is_multiple_of(100) {
                if let Some(start_pos) = self.path_start_pos {
                    let current_pos = entity.entity.pos.load();
                    let dx = current_pos.x - start_pos.x;
                    let dy = current_pos.y - start_pos.y;
                    let dz = current_pos.z - start_pos.z;
                    let dist_sq = dx * dx + dy * dy + dz * dz;
                    if dist_sq < 4.0 {
                        self.finish_navigation(entity);
                        return;
                    }
                }
                self.path_start_pos = Some(entity.entity.pos.load());
            }

            let on_ground = entity.entity.on_ground.load(Ordering::Relaxed);

            if let Some(next_block) = path.get_next_node_pos() {
                let target_pos = Vector3::new(
                    f64::from(next_block.0.x) + 0.5,
                    f64::from(next_block.0.y),
                    f64::from(next_block.0.z) + 0.5,
                );

                let current_pos = entity.entity.pos.load();
                let dx = target_pos.x - current_pos.x;
                let dy = target_pos.y - current_pos.y;
                let dz = target_pos.z - current_pos.z;

                let horizontal_dist_sq = dx * dx + dz * dz;
                let horizontal_dist = horizontal_dist_sq.sqrt();

                self.max_distance_to_waypoint = if self.mob_width > 0.75 {
                    self.mob_width * 0.5
                } else {
                    0.75 - self.mob_width * 0.5
                };

                if !on_ground
                    && horizontal_dist < f64::from(self.max_distance_to_waypoint)
                    && dy < -0.5
                {
                    path.advance();
                    self.current_goal = Some(goal);
                    return;
                }

                let close_enough = horizontal_dist < f64::from(self.max_distance_to_waypoint)
                    && dy.abs() < NODE_REACH_Y;

                let corner_cut = path.get_next_node().is_some_and(|n| {
                    n.path_type != PathType::DangerFire
                        && n.path_type != PathType::DamageFire
                        && n.path_type != PathType::WalkableDoor
                }) && Self::should_target_next_node_in_direction(mob_pos, path);

                if close_enough || corner_cut {
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

                let mob_speed =
                    goal.speed * entity.get_attribute_value(&Attributes::MOVEMENT_SPEED);

                entity
                    .movement_input
                    .store(Vector3::new(0.0, 0.0, mob_speed));

                let step_height = entity.get_attribute_value(&Attributes::STEP_HEIGHT);
                let jump_distance = 1.0f64.max(f64::from(self.mob_width));

                if (dy > step_height || f64::from(next_block.0.y) > current_pos.y)
                    && horizontal_dist_sq < jump_distance * jump_distance
                {
                    entity.jumping.store(true, Ordering::SeqCst);
                } else {
                    entity.jumping.store(false, Ordering::SeqCst);
                }
            } else {
                self.finish_navigation(entity);
                return;
            }
        }

        self.current_goal = Some(goal);
    }
}

pub struct GroundPathNavigation {
    pub inner: PathNavigation,
}

impl Default for GroundPathNavigation {
    fn default() -> Self {
        Self::new()
    }
}

impl GroundPathNavigation {
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: PathNavigation::new(EvaluatorKind::Walk(WalkNodeEvaluator::default())),
        }
    }
}

impl PathNavigationTrait for GroundPathNavigation {
    fn set_progress(&mut self, goal: NavigatorGoal) {
        self.inner.set_progress(goal);
    }

    fn set_speed(&mut self, speed: f64) {
        self.inner.set_speed(speed);
    }

    fn stop(&mut self) {
        self.inner.stop();
    }

    fn is_idle(&self) -> bool {
        self.inner.is_idle.load(Ordering::Relaxed)
    }

    fn is_done(&self) -> bool {
        self.inner.path.as_ref().is_none_or(Path::is_done)
    }

    fn is_in_progress(&self) -> bool {
        !self.is_done()
    }

    fn is_stuck(&self) -> bool {
        self.inner.is_stuck
    }

    fn get_path(&self) -> Option<&Path> {
        self.inner.path.as_ref()
    }

    fn get_path_mut(&mut self) -> Option<&mut Path> {
        self.inner.path.as_mut()
    }

    fn set_pathfinding_malus(&mut self, path_type: PathType, malus: f32) {
        self.inner.set_pathfinding_malus(path_type, malus);
    }

    fn get_pathfinding_malus(&self, path_type: PathType) -> f32 {
        self.inner.get_pathfinding_malus(path_type)
    }

    fn set_mob_dimensions(&mut self, width: f32, height: f32) {
        self.inner.set_mob_dimensions(width, height);
    }

    fn can_reach_within(
        &mut self,
        entity: &LivingEntity,
        destination: Vector3<f64>,
        distance: f32,
    ) -> bool {
        self.inner.can_reach_within(entity, destination, distance)
    }

    fn tick(&mut self, entity: &LivingEntity) {
        self.inner.tick_ground(entity);
    }

    fn move_to_coords(
        &mut self,
        x: f64,
        y: f64,
        z: f64,
        speed: f64,
        entity: &LivingEntity,
    ) -> bool {
        let pos = entity.entity.pos.load();
        self.set_progress(NavigatorGoal::new(pos, Vector3::new(x, y, z), speed));
        true
    }

    fn move_to_pos(&mut self, pos: BlockPos, speed: f64, entity: &LivingEntity) -> bool {
        let p = entity.entity.pos.load();
        let target = Vector3::new(
            f64::from(pos.0.x) + 0.5,
            f64::from(pos.0.y) + 0.5,
            f64::from(pos.0.z) + 0.5,
        );
        self.set_progress(NavigatorGoal::new(p, target, speed));
        true
    }

    fn move_to_entity(&mut self, target: &LivingEntity, speed: f64, entity: &LivingEntity) -> bool {
        let p = entity.entity.pos.load();
        let target_pos = target.entity.pos.load();
        self.set_progress(NavigatorGoal::new(p, target_pos, speed));
        true
    }

    fn move_to_path(&mut self, path: Option<Path>, speed: f64, entity: &LivingEntity) -> bool {
        if let Some(new_path) = path {
            self.inner.path = Some(new_path);
            if self.is_done() {
                return false;
            }
            self.inner.trim_path(entity);
            if self
                .inner
                .path
                .as_ref()
                .map_or(0, path::Path::get_node_count)
                == 0
            {
                return false;
            }
            self.inner.speed_modifier = speed;
            let mob_pos = Vector3::new(
                entity.entity.pos.load().x,
                self.inner.get_surface_y(entity),
                entity.entity.pos.load().z,
            );
            self.inner.last_stuck_check = self.inner.tick_count;
            self.inner.last_stuck_check_pos = mob_pos;
            self.inner.is_idle.store(false, Ordering::Relaxed);
            true
        } else {
            self.inner.path = None;
            self.inner.is_idle.store(true, Ordering::Relaxed);
            false
        }
    }

    fn create_path(
        &mut self,
        entity: &LivingEntity,
        destination: Vector3<f64>,
        reach_range: i32,
    ) -> Option<Path> {
        let mut dest_pos = destination.to_block_pos();
        if !self.inner.can_path_to_targets_below_surface {
            let world = entity.entity.world.load();
            dest_pos = PathNavigation::find_surface_position(&world, dest_pos);
        }
        let dest_v = Vector3::new(
            f64::from(dest_pos.0.x) + 0.5,
            f64::from(dest_pos.0.y),
            f64::from(dest_pos.0.z) + 0.5,
        );
        self.inner.compute_path(entity, dest_v, reach_range)
    }

    fn recompute_path(&mut self, entity: &LivingEntity) {
        let world_age = entity.entity.world.load().get_world_age() as u64;
        if world_age.saturating_sub(self.inner.time_last_recompute) <= 20 {
            self.inner.has_delayed_recomputation = true;
        } else if let Some(target_pos) = self.inner.target_pos {
            let target_v = Vector3::new(
                f64::from(target_pos.0.x) + 0.5,
                f64::from(target_pos.0.y),
                f64::from(target_pos.0.z) + 0.5,
            );
            self.inner.path = self
                .inner
                .compute_path(entity, target_v, self.inner.reach_range);
            self.inner.time_last_recompute = world_age;
            self.inner.has_delayed_recomputation = false;
        }
    }

    fn set_avoid_sun(&mut self, avoid_sun: bool) {
        self.inner.avoid_sun = avoid_sun;
    }

    fn set_can_walk_over_fences(&mut self, can_walk: bool) {
        self.inner.can_walk_over_fences = can_walk;
    }

    fn set_can_open_doors(&mut self, can_open: bool) {
        self.inner.can_open_doors = can_open;
    }

    fn set_can_pass_doors(&mut self, can_pass: bool) {
        self.inner.can_pass_doors = can_pass;
    }

    fn set_can_float(&mut self, can_float: bool) {
        self.inner.can_float = can_float;
    }

    fn can_float(&self) -> bool {
        self.inner.can_float
    }

    fn can_navigate_ground(&self) -> bool {
        true
    }

    fn set_required_path_length(&mut self, length: f32) {
        self.inner.required_path_length = length;
    }

    fn set_max_visited_nodes_multiplier(&mut self, multiplier: f32) {
        self.inner.max_visited_nodes_multiplier = multiplier;
    }

    fn reset_max_visited_nodes_multiplier(&mut self) {
        self.inner.max_visited_nodes_multiplier = 1.0;
    }

    fn get_target_pos(&self) -> Option<BlockPos> {
        self.inner.target_pos
    }

    fn can_path_to_targets_below_surface(&self) -> bool {
        self.inner.can_path_to_targets_below_surface
    }

    fn set_can_path_to_targets_below_surface(&mut self, can_path: bool) {
        self.inner.can_path_to_targets_below_surface = can_path;
    }
}

pub struct FlyingPathNavigation {
    pub inner: PathNavigation,
}

impl Default for FlyingPathNavigation {
    fn default() -> Self {
        Self::new()
    }
}

impl FlyingPathNavigation {
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: PathNavigation::new(EvaluatorKind::Fly(FlyNodeEvaluator::default())),
        }
    }
}

impl PathNavigationTrait for FlyingPathNavigation {
    fn set_progress(&mut self, goal: NavigatorGoal) {
        self.inner.set_progress(goal);
    }

    fn set_speed(&mut self, speed: f64) {
        self.inner.set_speed(speed);
    }

    fn stop(&mut self) {
        self.inner.stop();
    }

    fn is_idle(&self) -> bool {
        self.inner.is_idle.load(Ordering::Relaxed)
    }

    fn is_done(&self) -> bool {
        self.inner.path.as_ref().is_none_or(Path::is_done)
    }

    fn is_in_progress(&self) -> bool {
        !self.is_done()
    }

    fn is_stuck(&self) -> bool {
        self.inner.is_stuck
    }

    fn get_path(&self) -> Option<&Path> {
        self.inner.path.as_ref()
    }

    fn get_path_mut(&mut self) -> Option<&mut Path> {
        self.inner.path.as_mut()
    }

    fn set_pathfinding_malus(&mut self, path_type: PathType, malus: f32) {
        self.inner.set_pathfinding_malus(path_type, malus);
    }

    fn get_pathfinding_malus(&self, path_type: PathType) -> f32 {
        self.inner.get_pathfinding_malus(path_type)
    }

    fn set_mob_dimensions(&mut self, width: f32, height: f32) {
        self.inner.set_mob_dimensions(width, height);
    }

    fn can_reach_within(
        &mut self,
        entity: &LivingEntity,
        destination: Vector3<f64>,
        distance: f32,
    ) -> bool {
        self.inner.can_reach_within(entity, destination, distance)
    }

    #[allow(clippy::too_many_lines)]
    fn tick(&mut self, entity: &LivingEntity) {
        self.inner.tick_count += 1;
        let world_age = entity.entity.world.load().get_world_age() as u64;

        if self.inner.has_delayed_recomputation {
            self.recompute_path(entity);
        }

        if let Some(goal) = self.inner.current_goal.take() {
            if self.inner.needs_new_path(&goal) {
                self.inner.path = self.inner.compute_path(entity, goal.destination, 1);
                self.inner.ticks_on_current_node = 0;
                self.inner.last_node_index = 0;
                self.inner.path_start_pos = Some(entity.entity.pos.load());
                self.inner.repath_cooldown = 15;
                self.inner.time_last_recompute = world_age;
            }
            self.inner.current_goal = Some(goal);
        }

        let mob_pos = entity.entity.pos.load();
        self.inner.do_stuck_detection(mob_pos, entity);

        if self.is_done() {
            self.inner.finish_navigation(entity);
        } else {
            if let Some(path) = &mut self.inner.path
                && let Some(pos) = path.get_next_node_pos()
            {
                let target_pos = Vector3::new(
                    f64::from(pos.0.x) + 0.5,
                    f64::from(pos.0.y) + 0.5,
                    f64::from(pos.0.z) + 0.5,
                );
                let current_pos = entity.entity.pos.load();
                let dx = target_pos.x - current_pos.x;
                let dy = target_pos.y - current_pos.y;
                let dz = target_pos.z - current_pos.z;
                let dist_sq = dx * dx + dy * dy + dz * dz;

                if dist_sq < 0.5 * 0.5 {
                    path.advance();
                }
            }

            if !self.is_done()
                && let Some(path) = &self.inner.path
                && let Some(next_block) = path.get_next_node_pos()
            {
                let target_pos = Vector3::new(
                    f64::from(next_block.0.x) + 0.5,
                    f64::from(next_block.0.y) + 0.5,
                    f64::from(next_block.0.z) + 0.5,
                );
                let current_pos = entity.entity.pos.load();
                let dx = target_pos.x - current_pos.x;
                let dy = target_pos.y - current_pos.y;
                let dz = target_pos.z - current_pos.z;
                let sd = dx.hypot(dz);

                let desired_yaw = wrap_degrees((dz.atan2(dx) as f32).to_degrees() - 90.0);
                let desired_pitch = wrap_degrees(-(dy.atan2(sd) as f32).to_degrees());

                entity.entity.yaw.store(desired_yaw);
                entity.entity.head_yaw.store(desired_yaw);
                entity.entity.body_yaw.store(desired_yaw);
                entity.entity.pitch.store(desired_pitch);

                let flying_speed = entity.get_attribute_value(&Attributes::FLYING_SPEED);
                let base_speed = if flying_speed > 0.0 {
                    flying_speed
                } else {
                    entity.get_attribute_value(&Attributes::MOVEMENT_SPEED)
                };
                let speed = self.inner.speed_modifier * base_speed;
                let y_input = if dy.abs() > 0.1 {
                    if dy > 0.0 { speed } else { -speed }
                } else {
                    0.0
                };
                entity
                    .movement_input
                    .store(Vector3::new(0.0, y_input, speed));
                entity.entity.set_has_no_gravity(true);
            }
        }
    }

    fn move_to_coords(
        &mut self,
        x: f64,
        y: f64,
        z: f64,
        speed: f64,
        entity: &LivingEntity,
    ) -> bool {
        let pos = entity.entity.pos.load();
        self.set_progress(NavigatorGoal::new(pos, Vector3::new(x, y, z), speed));
        true
    }

    fn move_to_pos(&mut self, pos: BlockPos, speed: f64, entity: &LivingEntity) -> bool {
        let p = entity.entity.pos.load();
        let target = Vector3::new(
            f64::from(pos.0.x) + 0.5,
            f64::from(pos.0.y) + 0.5,
            f64::from(pos.0.z) + 0.5,
        );
        self.set_progress(NavigatorGoal::new(p, target, speed));
        true
    }

    fn move_to_entity(&mut self, target: &LivingEntity, speed: f64, entity: &LivingEntity) -> bool {
        let p = entity.entity.pos.load();
        let target_pos = target.entity.pos.load();
        self.set_progress(NavigatorGoal::new(p, target_pos, speed));
        true
    }

    fn move_to_path(&mut self, path: Option<Path>, speed: f64, entity: &LivingEntity) -> bool {
        if let Some(new_path) = path {
            self.inner.path = Some(new_path);
            if self.is_done() {
                return false;
            }
            self.inner.speed_modifier = speed;
            let mob_pos = entity.entity.pos.load();
            self.inner.last_stuck_check = self.inner.tick_count;
            self.inner.last_stuck_check_pos = mob_pos;
            self.inner.is_idle.store(false, Ordering::Relaxed);
            true
        } else {
            self.inner.path = None;
            self.inner.is_idle.store(true, Ordering::Relaxed);
            false
        }
    }

    fn create_path(
        &mut self,
        entity: &LivingEntity,
        destination: Vector3<f64>,
        reach_range: i32,
    ) -> Option<Path> {
        self.inner.compute_path(entity, destination, reach_range)
    }

    fn recompute_path(&mut self, entity: &LivingEntity) {
        let world_age = entity.entity.world.load().get_world_age() as u64;
        if world_age.saturating_sub(self.inner.time_last_recompute) <= 20 {
            self.inner.has_delayed_recomputation = true;
        } else if let Some(target_pos) = self.inner.target_pos {
            let target_v = Vector3::new(
                f64::from(target_pos.0.x) + 0.5,
                f64::from(target_pos.0.y) + 0.5,
                f64::from(target_pos.0.z) + 0.5,
            );
            self.inner.path = self
                .inner
                .compute_path(entity, target_v, self.inner.reach_range);
            self.inner.time_last_recompute = world_age;
            self.inner.has_delayed_recomputation = false;
        }
    }

    fn set_avoid_sun(&mut self, avoid_sun: bool) {
        self.inner.avoid_sun = avoid_sun;
    }

    fn set_can_walk_over_fences(&mut self, can_walk: bool) {
        self.inner.can_walk_over_fences = can_walk;
    }

    fn set_can_open_doors(&mut self, can_open: bool) {
        self.inner.can_open_doors = can_open;
    }

    fn set_can_pass_doors(&mut self, can_pass: bool) {
        self.inner.can_pass_doors = can_pass;
    }

    fn set_can_float(&mut self, can_float: bool) {
        self.inner.can_float = can_float;
    }

    fn can_float(&self) -> bool {
        self.inner.can_float
    }

    fn can_navigate_ground(&self) -> bool {
        false
    }

    fn set_required_path_length(&mut self, length: f32) {
        self.inner.required_path_length = length;
    }

    fn set_max_visited_nodes_multiplier(&mut self, multiplier: f32) {
        self.inner.max_visited_nodes_multiplier = multiplier;
    }

    fn reset_max_visited_nodes_multiplier(&mut self) {
        self.inner.max_visited_nodes_multiplier = 1.0;
    }

    fn get_target_pos(&self) -> Option<BlockPos> {
        self.inner.target_pos
    }

    fn can_path_to_targets_below_surface(&self) -> bool {
        self.inner.can_path_to_targets_below_surface
    }

    fn set_can_path_to_targets_below_surface(&mut self, can_path: bool) {
        self.inner.can_path_to_targets_below_surface = can_path;
    }
}

pub struct WaterBoundPathNavigation {
    pub inner: PathNavigation,
    pub allow_breaching: bool,
}

impl Default for WaterBoundPathNavigation {
    fn default() -> Self {
        Self::new(false)
    }
}

impl WaterBoundPathNavigation {
    #[must_use]
    pub fn new(allow_breaching: bool) -> Self {
        Self {
            inner: PathNavigation::new(EvaluatorKind::Swim(SwimNodeEvaluator::new(
                allow_breaching,
            ))),
            allow_breaching,
        }
    }
}

impl PathNavigationTrait for WaterBoundPathNavigation {
    fn set_progress(&mut self, goal: NavigatorGoal) {
        self.inner.set_progress(goal);
    }

    fn set_speed(&mut self, speed: f64) {
        self.inner.set_speed(speed);
    }

    fn stop(&mut self) {
        self.inner.stop();
    }

    fn is_idle(&self) -> bool {
        self.inner.is_idle.load(Ordering::Relaxed)
    }

    fn is_done(&self) -> bool {
        self.inner.path.as_ref().is_none_or(Path::is_done)
    }

    fn is_in_progress(&self) -> bool {
        !self.is_done()
    }

    fn is_stuck(&self) -> bool {
        self.inner.is_stuck
    }

    fn get_path(&self) -> Option<&Path> {
        self.inner.path.as_ref()
    }

    fn get_path_mut(&mut self) -> Option<&mut Path> {
        self.inner.path.as_mut()
    }

    fn set_pathfinding_malus(&mut self, path_type: PathType, malus: f32) {
        self.inner.set_pathfinding_malus(path_type, malus);
    }

    fn get_pathfinding_malus(&self, path_type: PathType) -> f32 {
        self.inner.get_pathfinding_malus(path_type)
    }

    fn set_mob_dimensions(&mut self, width: f32, height: f32) {
        self.inner.set_mob_dimensions(width, height);
    }

    fn can_reach_within(
        &mut self,
        entity: &LivingEntity,
        destination: Vector3<f64>,
        distance: f32,
    ) -> bool {
        self.inner.can_reach_within(entity, destination, distance)
    }

    #[allow(clippy::too_many_lines)]
    fn tick(&mut self, entity: &LivingEntity) {
        self.inner.tick_count += 1;
        let world_age = entity.entity.world.load().get_world_age() as u64;

        if self.inner.has_delayed_recomputation {
            self.recompute_path(entity);
        }

        if let Some(goal) = self.inner.current_goal.take() {
            if self.inner.needs_new_path(&goal) {
                self.inner.path = self.inner.compute_path(entity, goal.destination, 1);
                self.inner.ticks_on_current_node = 0;
                self.inner.last_node_index = 0;
                self.inner.path_start_pos = Some(entity.entity.pos.load());
                self.inner.repath_cooldown = 15;
                self.inner.time_last_recompute = world_age;
            }
            self.inner.current_goal = Some(goal);
        }

        let mob_pos = Vector3::new(
            entity.entity.pos.load().x,
            entity.entity.pos.load().y + f64::from(self.inner.mob_height) * 0.5,
            entity.entity.pos.load().z,
        );
        self.inner.do_stuck_detection(mob_pos, entity);

        if self.is_done() {
            self.inner.finish_navigation(entity);
        } else {
            if let Some(path) = &mut self.inner.path
                && let Some(pos) = path.get_next_node_pos()
            {
                let target_pos = Vector3::new(
                    f64::from(pos.0.x) + 0.5,
                    f64::from(pos.0.y) + 0.5,
                    f64::from(pos.0.z) + 0.5,
                );
                let current_pos = entity.entity.pos.load();
                let dx = target_pos.x - current_pos.x;
                let dy = target_pos.y - current_pos.y;
                let dz = target_pos.z - current_pos.z;
                let dist_sq = dx * dx + dy * dy + dz * dz;

                if dist_sq < 0.5 * 0.5 {
                    path.advance();
                }
            }

            if !self.is_done()
                && let Some(path) = &self.inner.path
                && let Some(next_block) = path.get_next_node_pos()
            {
                let target_pos = Vector3::new(
                    f64::from(next_block.0.x) + 0.5,
                    f64::from(next_block.0.y) + 0.5,
                    f64::from(next_block.0.z) + 0.5,
                );
                let current_pos = entity.entity.pos.load();
                let dx = target_pos.x - current_pos.x;
                let dy = target_pos.y - current_pos.y;
                let dz = target_pos.z - current_pos.z;
                let sd = dx.hypot(dz);

                let desired_yaw = wrap_degrees((dz.atan2(dx) as f32).to_degrees() - 90.0);
                let desired_pitch = wrap_degrees(-(dy.atan2(sd) as f32).to_degrees());

                entity.entity.yaw.store(desired_yaw);
                entity.entity.head_yaw.store(desired_yaw);
                entity.entity.body_yaw.store(desired_yaw);
                entity.entity.pitch.store(desired_pitch);

                let speed = self.inner.speed_modifier
                    * entity.get_attribute_value(&Attributes::MOVEMENT_SPEED);
                let y_input = if dy.abs() > 0.1 {
                    if dy > 0.0 { speed } else { -speed }
                } else {
                    0.0
                };
                entity
                    .movement_input
                    .store(Vector3::new(0.0, y_input, speed));
            }
        }
    }

    fn move_to_coords(
        &mut self,
        x: f64,
        y: f64,
        z: f64,
        speed: f64,
        entity: &LivingEntity,
    ) -> bool {
        let pos = entity.entity.pos.load();
        self.set_progress(NavigatorGoal::new(pos, Vector3::new(x, y, z), speed));
        true
    }

    fn move_to_pos(&mut self, pos: BlockPos, speed: f64, entity: &LivingEntity) -> bool {
        let p = entity.entity.pos.load();
        let target = Vector3::new(
            f64::from(pos.0.x) + 0.5,
            f64::from(pos.0.y) + 0.5,
            f64::from(pos.0.z) + 0.5,
        );
        self.set_progress(NavigatorGoal::new(p, target, speed));
        true
    }

    fn move_to_entity(&mut self, target: &LivingEntity, speed: f64, entity: &LivingEntity) -> bool {
        let p = entity.entity.pos.load();
        let target_pos = target.entity.pos.load();
        self.set_progress(NavigatorGoal::new(p, target_pos, speed));
        true
    }

    fn move_to_path(&mut self, path: Option<Path>, speed: f64, entity: &LivingEntity) -> bool {
        if let Some(new_path) = path {
            self.inner.path = Some(new_path);
            if self.is_done() {
                return false;
            }
            self.inner.speed_modifier = speed;
            let mob_pos = Vector3::new(
                entity.entity.pos.load().x,
                entity.entity.pos.load().y + f64::from(self.inner.mob_height) * 0.5,
                entity.entity.pos.load().z,
            );
            self.inner.last_stuck_check = self.inner.tick_count;
            self.inner.last_stuck_check_pos = mob_pos;
            self.inner.is_idle.store(false, Ordering::Relaxed);
            true
        } else {
            self.inner.path = None;
            self.inner.is_idle.store(true, Ordering::Relaxed);
            false
        }
    }

    fn create_path(
        &mut self,
        entity: &LivingEntity,
        destination: Vector3<f64>,
        reach_range: i32,
    ) -> Option<Path> {
        self.inner.compute_path(entity, destination, reach_range)
    }

    fn recompute_path(&mut self, entity: &LivingEntity) {
        let world_age = entity.entity.world.load().get_world_age() as u64;
        if world_age.saturating_sub(self.inner.time_last_recompute) <= 20 {
            self.inner.has_delayed_recomputation = true;
        } else if let Some(target_pos) = self.inner.target_pos {
            let target_v = Vector3::new(
                f64::from(target_pos.0.x) + 0.5,
                f64::from(target_pos.0.y) + 0.5,
                f64::from(target_pos.0.z) + 0.5,
            );
            self.inner.path = self
                .inner
                .compute_path(entity, target_v, self.inner.reach_range);
            self.inner.time_last_recompute = world_age;
            self.inner.has_delayed_recomputation = false;
        }
    }

    fn set_avoid_sun(&mut self, avoid_sun: bool) {
        self.inner.avoid_sun = avoid_sun;
    }

    fn set_can_walk_over_fences(&mut self, can_walk: bool) {
        self.inner.can_walk_over_fences = can_walk;
    }

    fn set_can_open_doors(&mut self, can_open: bool) {
        self.inner.can_open_doors = can_open;
    }

    fn set_can_pass_doors(&mut self, can_pass: bool) {
        self.inner.can_pass_doors = can_pass;
    }

    fn set_can_float(&mut self, _can_float: bool) {}

    fn can_float(&self) -> bool {
        true
    }

    fn can_navigate_ground(&self) -> bool {
        false
    }

    fn set_required_path_length(&mut self, length: f32) {
        self.inner.required_path_length = length;
    }

    fn set_max_visited_nodes_multiplier(&mut self, multiplier: f32) {
        self.inner.max_visited_nodes_multiplier = multiplier;
    }

    fn reset_max_visited_nodes_multiplier(&mut self) {
        self.inner.max_visited_nodes_multiplier = 1.0;
    }

    fn get_target_pos(&self) -> Option<BlockPos> {
        self.inner.target_pos
    }

    fn can_path_to_targets_below_surface(&self) -> bool {
        self.inner.can_path_to_targets_below_surface
    }

    fn set_can_path_to_targets_below_surface(&mut self, can_path: bool) {
        self.inner.can_path_to_targets_below_surface = can_path;
    }
}

pub struct WallClimberNavigation {
    pub inner: GroundPathNavigation,
    pub path_to_position: Option<BlockPos>,
}

impl Default for WallClimberNavigation {
    fn default() -> Self {
        Self::new()
    }
}

impl WallClimberNavigation {
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: GroundPathNavigation::new(),
            path_to_position: None,
        }
    }
}

impl PathNavigationTrait for WallClimberNavigation {
    fn set_progress(&mut self, goal: NavigatorGoal) {
        self.path_to_position = Some(goal.destination.to_block_pos());
        self.inner.set_progress(goal);
    }

    fn set_speed(&mut self, speed: f64) {
        self.inner.set_speed(speed);
    }

    fn stop(&mut self) {
        self.path_to_position = None;
        self.inner.stop();
    }

    fn is_idle(&self) -> bool {
        self.path_to_position.is_none() && self.inner.is_idle()
    }

    fn is_done(&self) -> bool {
        self.path_to_position.is_none() && self.inner.is_done()
    }

    fn is_in_progress(&self) -> bool {
        !self.is_done()
    }

    fn is_stuck(&self) -> bool {
        self.inner.is_stuck()
    }

    fn get_path(&self) -> Option<&Path> {
        self.inner.get_path()
    }

    fn get_path_mut(&mut self) -> Option<&mut Path> {
        self.inner.get_path_mut()
    }

    fn set_pathfinding_malus(&mut self, path_type: PathType, malus: f32) {
        self.inner.set_pathfinding_malus(path_type, malus);
    }

    fn get_pathfinding_malus(&self, path_type: PathType) -> f32 {
        self.inner.get_pathfinding_malus(path_type)
    }

    fn set_mob_dimensions(&mut self, width: f32, height: f32) {
        self.inner.set_mob_dimensions(width, height);
    }

    fn can_reach_within(
        &mut self,
        entity: &LivingEntity,
        destination: Vector3<f64>,
        distance: f32,
    ) -> bool {
        self.inner.can_reach_within(entity, destination, distance)
    }

    #[allow(clippy::too_many_lines)]
    fn tick(&mut self, entity: &LivingEntity) {
        if !self.inner.is_done() {
            self.inner.tick(entity);
        } else if let Some(target_pos) = self.path_to_position {
            let current_pos = entity.entity.pos.load();
            let bb = entity.entity.bounding_box.load();
            let width = bb.max.x - bb.min.x;
            let target_center = Vector3::new(
                f64::from(target_pos.0.x) + 0.5,
                f64::from(target_pos.0.y),
                f64::from(target_pos.0.z) + 0.5,
            );
            let dx = target_center.x - current_pos.x;
            let dy = target_center.y - current_pos.y;
            let dz = target_center.z - current_pos.z;
            let dist_sq = dx * dx + dz * dz;

            let close_enough = dist_sq < width * width
                && (current_pos.y > f64::from(target_pos.0.y)
                    || (current_pos.y - f64::from(target_pos.0.y)).abs() < 1.0);

            if close_enough {
                self.path_to_position = None;
                self.inner.inner.finish_navigation(entity);
            } else {
                let desired_yaw = wrap_degrees((dz.atan2(dx) as f32).to_degrees() - 90.0);
                entity.entity.yaw.store(desired_yaw);
                entity.entity.head_yaw.store(desired_yaw);
                entity.entity.body_yaw.store(desired_yaw);

                let speed = self.inner.inner.speed_modifier
                    * entity.get_attribute_value(&Attributes::MOVEMENT_SPEED);
                entity.movement_input.store(Vector3::new(0.0, 0.0, speed));
                if dy > 0.0 {
                    entity.jumping.store(true, Ordering::SeqCst);
                } else {
                    entity.jumping.store(false, Ordering::SeqCst);
                }
            }
        }
    }

    fn move_to_coords(
        &mut self,
        x: f64,
        y: f64,
        z: f64,
        speed: f64,
        entity: &LivingEntity,
    ) -> bool {
        self.path_to_position = Some(BlockPos::new(
            x.floor() as i32,
            y.floor() as i32,
            z.floor() as i32,
        ));
        self.inner.move_to_coords(x, y, z, speed, entity)
    }

    fn move_to_pos(&mut self, pos: BlockPos, speed: f64, entity: &LivingEntity) -> bool {
        self.path_to_position = Some(pos);
        self.inner.move_to_pos(pos, speed, entity)
    }

    fn move_to_entity(&mut self, target: &LivingEntity, speed: f64, entity: &LivingEntity) -> bool {
        self.path_to_position = Some(target.entity.block_pos.load());
        let path = self.create_path(entity, target.entity.pos.load(), 0);
        if path.is_some() {
            self.move_to_path(path, speed, entity)
        } else {
            self.inner.inner.speed_modifier = speed;
            true
        }
    }

    fn move_to_path(&mut self, path: Option<Path>, speed: f64, entity: &LivingEntity) -> bool {
        self.inner.move_to_path(path, speed, entity)
    }

    fn create_path(
        &mut self,
        entity: &LivingEntity,
        destination: Vector3<f64>,
        reach_range: i32,
    ) -> Option<Path> {
        self.path_to_position = Some(destination.to_block_pos());
        self.inner.create_path(entity, destination, reach_range)
    }

    fn recompute_path(&mut self, entity: &LivingEntity) {
        self.inner.recompute_path(entity);
    }

    fn set_avoid_sun(&mut self, avoid_sun: bool) {
        self.inner.set_avoid_sun(avoid_sun);
    }

    fn set_can_walk_over_fences(&mut self, can_walk: bool) {
        self.inner.set_can_walk_over_fences(can_walk);
    }

    fn set_can_open_doors(&mut self, can_open: bool) {
        self.inner.set_can_open_doors(can_open);
    }

    fn set_can_pass_doors(&mut self, can_pass: bool) {
        self.inner.set_can_pass_doors(can_pass);
    }

    fn set_can_float(&mut self, can_float: bool) {
        self.inner.set_can_float(can_float);
    }

    fn can_float(&self) -> bool {
        self.inner.can_float()
    }

    fn can_navigate_ground(&self) -> bool {
        true
    }

    fn set_required_path_length(&mut self, length: f32) {
        self.inner.set_required_path_length(length);
    }

    fn set_max_visited_nodes_multiplier(&mut self, multiplier: f32) {
        self.inner.set_max_visited_nodes_multiplier(multiplier);
    }

    fn reset_max_visited_nodes_multiplier(&mut self) {
        self.inner.reset_max_visited_nodes_multiplier();
    }

    fn get_target_pos(&self) -> Option<BlockPos> {
        self.inner.get_target_pos()
    }

    fn can_path_to_targets_below_surface(&self) -> bool {
        self.inner.can_path_to_targets_below_surface()
    }

    fn set_can_path_to_targets_below_surface(&mut self, can_path: bool) {
        self.inner.set_can_path_to_targets_below_surface(can_path);
    }
}

pub struct AmphibiousPathNavigation {
    pub inner: PathNavigation,
}

impl Default for AmphibiousPathNavigation {
    fn default() -> Self {
        Self::new(false)
    }
}

impl AmphibiousPathNavigation {
    #[must_use]
    pub fn new(prefers_shallow_swimming: bool) -> Self {
        Self {
            inner: PathNavigation::new(EvaluatorKind::Amphibious(AmphibiousNodeEvaluator::new(
                prefers_shallow_swimming,
            ))),
        }
    }
}

impl PathNavigationTrait for AmphibiousPathNavigation {
    fn set_progress(&mut self, goal: NavigatorGoal) {
        self.inner.set_progress(goal);
    }

    fn set_speed(&mut self, speed: f64) {
        self.inner.set_speed(speed);
    }

    fn stop(&mut self) {
        self.inner.stop();
    }

    fn is_idle(&self) -> bool {
        self.inner.is_idle.load(Ordering::Relaxed)
    }

    fn is_done(&self) -> bool {
        self.inner.path.as_ref().is_none_or(Path::is_done)
    }

    fn is_in_progress(&self) -> bool {
        !self.is_done()
    }

    fn is_stuck(&self) -> bool {
        self.inner.is_stuck
    }

    fn get_path(&self) -> Option<&Path> {
        self.inner.path.as_ref()
    }

    fn get_path_mut(&mut self) -> Option<&mut Path> {
        self.inner.path.as_mut()
    }

    fn set_pathfinding_malus(&mut self, path_type: PathType, malus: f32) {
        self.inner.set_pathfinding_malus(path_type, malus);
    }

    fn get_pathfinding_malus(&self, path_type: PathType) -> f32 {
        self.inner.get_pathfinding_malus(path_type)
    }

    fn set_mob_dimensions(&mut self, width: f32, height: f32) {
        self.inner.set_mob_dimensions(width, height);
    }

    fn can_reach_within(
        &mut self,
        entity: &LivingEntity,
        destination: Vector3<f64>,
        distance: f32,
    ) -> bool {
        self.inner.can_reach_within(entity, destination, distance)
    }

    #[allow(clippy::too_many_lines)]
    fn tick(&mut self, entity: &LivingEntity) {
        if entity.entity.touching_water.load(Ordering::Relaxed) {
            self.inner.tick_count += 1;
            let world_age = entity.entity.world.load().get_world_age() as u64;

            if self.inner.has_delayed_recomputation {
                self.recompute_path(entity);
            }

            if let Some(goal) = self.inner.current_goal.take() {
                if self.inner.needs_new_path(&goal) {
                    self.inner.path = self.inner.compute_path(entity, goal.destination, 1);
                    self.inner.ticks_on_current_node = 0;
                    self.inner.last_node_index = 0;
                    self.inner.path_start_pos = Some(entity.entity.pos.load());
                    self.inner.repath_cooldown = 15;
                    self.inner.time_last_recompute = world_age;
                }
                self.inner.current_goal = Some(goal);
            }

            let mob_pos = Vector3::new(
                entity.entity.pos.load().x,
                entity.entity.pos.load().y + f64::from(self.inner.mob_height) * 0.5,
                entity.entity.pos.load().z,
            );
            self.inner.do_stuck_detection(mob_pos, entity);

            if self.is_done() {
                self.inner.finish_navigation(entity);
            } else {
                if let Some(path) = &mut self.inner.path
                    && let Some(pos) = path.get_next_node_pos()
                {
                    let target_pos = Vector3::new(
                        f64::from(pos.0.x) + 0.5,
                        f64::from(pos.0.y) + 0.5,
                        f64::from(pos.0.z) + 0.5,
                    );
                    let current_pos = entity.entity.pos.load();
                    let dx = target_pos.x - current_pos.x;
                    let dy = target_pos.y - current_pos.y;
                    let dz = target_pos.z - current_pos.z;
                    let dist_sq = dx * dx + dy * dy + dz * dz;

                    if dist_sq < 0.5 * 0.5 {
                        path.advance();
                    }
                }

                if !self.is_done()
                    && let Some(path) = &self.inner.path
                    && let Some(next_block) = path.get_next_node_pos()
                {
                    let target_pos = Vector3::new(
                        f64::from(next_block.0.x) + 0.5,
                        f64::from(next_block.0.y) + 0.5,
                        f64::from(next_block.0.z) + 0.5,
                    );
                    let current_pos = entity.entity.pos.load();
                    let dx = target_pos.x - current_pos.x;
                    let dy = target_pos.y - current_pos.y;
                    let dz = target_pos.z - current_pos.z;
                    let sd = dx.hypot(dz);

                    let desired_yaw = wrap_degrees((dz.atan2(dx) as f32).to_degrees() - 90.0);
                    let desired_pitch = wrap_degrees(-(dy.atan2(sd) as f32).to_degrees());

                    entity.entity.yaw.store(desired_yaw);
                    entity.entity.head_yaw.store(desired_yaw);
                    entity.entity.body_yaw.store(desired_yaw);
                    entity.entity.pitch.store(desired_pitch);

                    let speed = self.inner.speed_modifier
                        * entity.get_attribute_value(&Attributes::MOVEMENT_SPEED);
                    let y_input = if dy.abs() > 0.1 {
                        if dy > 0.0 { speed } else { -speed }
                    } else {
                        0.0
                    };
                    entity
                        .movement_input
                        .store(Vector3::new(0.0, y_input, speed));
                }
            }
        } else {
            self.inner.tick_ground(entity);
        }
    }

    fn move_to_coords(
        &mut self,
        x: f64,
        y: f64,
        z: f64,
        speed: f64,
        entity: &LivingEntity,
    ) -> bool {
        let pos = entity.entity.pos.load();
        self.set_progress(NavigatorGoal::new(pos, Vector3::new(x, y, z), speed));
        true
    }

    fn move_to_pos(&mut self, pos: BlockPos, speed: f64, entity: &LivingEntity) -> bool {
        let p = entity.entity.pos.load();
        let target = Vector3::new(
            f64::from(pos.0.x) + 0.5,
            f64::from(pos.0.y) + 0.5,
            f64::from(pos.0.z) + 0.5,
        );
        self.set_progress(NavigatorGoal::new(p, target, speed));
        true
    }

    fn move_to_entity(&mut self, target: &LivingEntity, speed: f64, entity: &LivingEntity) -> bool {
        let p = entity.entity.pos.load();
        let target_pos = target.entity.pos.load();
        self.set_progress(NavigatorGoal::new(p, target_pos, speed));
        true
    }

    fn move_to_path(&mut self, path: Option<Path>, speed: f64, entity: &LivingEntity) -> bool {
        if let Some(new_path) = path {
            self.inner.path = Some(new_path);
            if self.is_done() {
                return false;
            }
            self.inner.speed_modifier = speed;
            let mob_pos = Vector3::new(
                entity.entity.pos.load().x,
                entity.entity.pos.load().y + f64::from(self.inner.mob_height) * 0.5,
                entity.entity.pos.load().z,
            );
            self.inner.last_stuck_check = self.inner.tick_count;
            self.inner.last_stuck_check_pos = mob_pos;
            self.inner.is_idle.store(false, Ordering::Relaxed);
            true
        } else {
            self.inner.path = None;
            self.inner.is_idle.store(true, Ordering::Relaxed);
            false
        }
    }

    fn create_path(
        &mut self,
        entity: &LivingEntity,
        destination: Vector3<f64>,
        reach_range: i32,
    ) -> Option<Path> {
        self.inner.compute_path(entity, destination, reach_range)
    }

    fn recompute_path(&mut self, entity: &LivingEntity) {
        let world_age = entity.entity.world.load().get_world_age() as u64;
        if world_age.saturating_sub(self.inner.time_last_recompute) <= 20 {
            self.inner.has_delayed_recomputation = true;
        } else if let Some(target_pos) = self.inner.target_pos {
            let target_v = Vector3::new(
                f64::from(target_pos.0.x) + 0.5,
                f64::from(target_pos.0.y) + 0.5,
                f64::from(target_pos.0.z) + 0.5,
            );
            self.inner.path = self
                .inner
                .compute_path(entity, target_v, self.inner.reach_range);
            self.inner.time_last_recompute = world_age;
            self.inner.has_delayed_recomputation = false;
        }
    }

    fn set_avoid_sun(&mut self, avoid_sun: bool) {
        self.inner.avoid_sun = avoid_sun;
    }

    fn set_can_walk_over_fences(&mut self, can_walk: bool) {
        self.inner.can_walk_over_fences = can_walk;
    }

    fn set_can_open_doors(&mut self, can_open: bool) {
        self.inner.can_open_doors = can_open;
    }

    fn set_can_pass_doors(&mut self, can_pass: bool) {
        self.inner.can_pass_doors = can_pass;
    }

    fn set_can_float(&mut self, _can_float: bool) {}

    fn can_float(&self) -> bool {
        true
    }

    fn can_navigate_ground(&self) -> bool {
        true
    }

    fn set_required_path_length(&mut self, length: f32) {
        self.inner.required_path_length = length;
    }

    fn set_max_visited_nodes_multiplier(&mut self, multiplier: f32) {
        self.inner.max_visited_nodes_multiplier = multiplier;
    }

    fn reset_max_visited_nodes_multiplier(&mut self) {
        self.inner.max_visited_nodes_multiplier = 1.0;
    }

    fn get_target_pos(&self) -> Option<BlockPos> {
        self.inner.target_pos
    }

    fn can_path_to_targets_below_surface(&self) -> bool {
        self.inner.can_path_to_targets_below_surface
    }

    fn set_can_path_to_targets_below_surface(&mut self, can_path: bool) {
        self.inner.can_path_to_targets_below_surface = can_path;
    }
}

pub struct Navigator {
    inner: Box<dyn PathNavigationTrait>,
}

impl Default for Navigator {
    fn default() -> Self {
        Self::ground()
    }
}

impl Navigator {
    #[must_use]
    pub fn new<N: PathNavigationTrait + 'static>(nav: N) -> Self {
        Self {
            inner: Box::new(nav),
        }
    }

    #[must_use]
    pub fn ground() -> Self {
        Self::new(GroundPathNavigation::new())
    }

    #[must_use]
    pub fn flying() -> Self {
        Self::new(FlyingPathNavigation::new())
    }

    #[must_use]
    pub fn water_bound(allow_breaching: bool) -> Self {
        Self::new(WaterBoundPathNavigation::new(allow_breaching))
    }

    #[must_use]
    pub fn wall_climber() -> Self {
        Self::new(WallClimberNavigation::new())
    }

    #[must_use]
    pub fn amphibious(prefers_shallow_swimming: bool) -> Self {
        Self::new(AmphibiousPathNavigation::new(prefers_shallow_swimming))
    }
}

impl Deref for Navigator {
    type Target = dyn PathNavigationTrait;

    fn deref(&self) -> &Self::Target {
        &*self.inner
    }
}

impl DerefMut for Navigator {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.inner
    }
}
