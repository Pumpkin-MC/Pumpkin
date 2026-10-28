use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

use crate::entity::living::LivingEntity;
use crate::world::World;

use crate::entity::ai::pathfinder::amphibious_node_evaluator::AmphibiousNodeEvaluator;
use crate::entity::ai::pathfinder::binary_heap::BinaryHeap;
use crate::entity::ai::pathfinder::fly_node_evaluator::FlyNodeEvaluator;
use crate::entity::ai::pathfinder::node::Node;
use crate::entity::ai::pathfinder::node::PathType;
use crate::entity::ai::pathfinder::node_evaluator::{BaseNodeEvaluator, MobData, NodeEvaluator};
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

const TARGET_DISTANCE_MULTIPLIER: f32 = 1.5;
const NODE_REACH_Y: f64 = 1.0;
const MAX_TIME_RECOMPUTE: u64 = 20;
const MAX_VISITED_NODES_MULTIPLIER: f32 = 16.0;

pub struct PathFinder {
    max_visited_nodes: usize,
    open_set: BinaryHeap,
    neighbors_buf: Vec<Node>,
}

impl PathFinder {
    #[must_use]
    pub fn new(max_visited_nodes: usize) -> Self {
        Self {
            max_visited_nodes,
            open_set: BinaryHeap::new(),
            neighbors_buf: Vec::with_capacity(32),
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

    #[allow(clippy::too_many_lines)]
    pub fn find_path(
        &mut self,
        evaluator: &mut EvaluatorKind,
        targets: &[BlockPos],
        max_path_length: f32,
        reach_range: i32,
        max_visited_nodes_multiplier: f32,
    ) -> Option<Path> {
        self.open_set.clear();
        self.neighbors_buf.clear();
        if targets.is_empty() {
            return None;
        }

        let mut from = evaluator.get_start()?;

        let mut target_entries: Vec<(crate::entity::ai::pathfinder::node::Target, BlockPos)> =
            targets
                .iter()
                .map(|&pos| (evaluator.get_target(pos), pos))
                .collect();

        from.g = 0.0;
        from.h = Self::get_best_h(&from, &mut target_entries);
        from.f = from.h;
        from.walked_dist = 0.0;
        from.came_from = None;
        from.closed = false;

        evaluator.base_mut().update_node(from);
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

            current = evaluator.base_mut().get_node(current.pos);
            current.closed = true;
            evaluator.base_mut().update_node(current);

            for (idx, (target, _)) in target_entries.iter_mut().enumerate() {
                if current.distance_manhattan_node(&target.node) <= reach_range as f32 {
                    target.set_reached();
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

                for neighbor in self.neighbors_buf.drain(..) {
                    let mut neighbor = evaluator.base_mut().get_node(neighbor.pos);
                    if neighbor.closed {
                        continue;
                    }
                    let distance = current.distance_to_node(&neighbor);
                    neighbor.walked_dist = current.walked_dist + distance;
                    let tentative_g = current.g + distance + neighbor.cost_malus;

                    let in_open = self.open_set.contains(&neighbor);
                    let is_better = !in_open || tentative_g < neighbor.g;

                    if neighbor.walked_dist < max_path_length && is_better {
                        neighbor.came_from = Some(current.pos.0);
                        neighbor.g = tentative_g;
                        neighbor.h = Self::get_best_h(&neighbor, &mut target_entries)
                            * TARGET_DISTANCE_MULTIPLIER;
                        neighbor.f = neighbor.g + neighbor.h;

                        if in_open {
                            self.open_set.update_node(&neighbor, neighbor);
                        } else {
                            self.open_set.insert(neighbor);
                        }
                    }
                    evaluator.base_mut().update_node(neighbor);
                }
            }
        }

        let all_nodes = &evaluator.base_mut().nodes;

        if reached_targets.is_empty() {
            target_entries
                .into_iter()
                .filter_map(|(target, target_pos)| {
                    target
                        .get_best_node()
                        .and_then(|pos| all_nodes.get(&pos.0))
                        .map(|best| Self::reconstruct_path(best, target_pos, false, all_nodes))
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
                    target
                        .get_best_node()
                        .and_then(|pos| all_nodes.get(&pos.0))
                        .map(|best| Self::reconstruct_path(best, *target_pos, true, all_nodes))
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

pub struct NavigationTarget {
    pub position: Vector3<f64>,
    pub speed_modifier: f64,
}

pub enum EvaluatorKind {
    Walk(WalkNodeEvaluator),
    Fly(FlyNodeEvaluator),
    Swim(SwimNodeEvaluator),
    Amphibious(AmphibiousNodeEvaluator),
}

impl EvaluatorKind {
    const fn base_mut(&mut self) -> &mut BaseNodeEvaluator {
        match self {
            Self::Walk(e) => &mut e.base,
            Self::Fly(e) => &mut e.walk.base,
            Self::Swim(e) => &mut e.base,
            Self::Amphibious(e) => &mut e.walk.base,
        }
    }

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

    fn get_path_type(&mut self, context: &mut PathfindingContext, pos: BlockPos) -> PathType {
        match self {
            Self::Walk(e) => e.get_path_type(context, pos.0),
            Self::Fly(e) => e.get_path_type(context, pos.0),
            Self::Swim(e) => e.get_path_type(context, pos.0),
            Self::Amphibious(e) => e.get_path_type(context, pos.0),
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

/// Navigation methods run under their owning mob's navigator mutex.
/// Implementations must not relock it or invoke callbacks that can acquire it.
pub trait PathNavigationTrait: Send + Sync {
    fn set_progress(&mut self, goal: NavigatorGoal);
    fn set_speed(&mut self, speed: f64);
    fn get_speed_modifier(&self) -> f64;
    fn stop(&mut self);
    fn is_idle(&self) -> bool;
    fn is_done(&self) -> bool;
    fn is_in_progress(&self) -> bool;
    fn is_stuck(&self) -> bool;
    fn get_path(&self) -> Option<&Path>;
    fn get_path_mut(&mut self) -> Option<&mut Path>;
    fn get_path_type(&mut self, entity: &LivingEntity, position: BlockPos) -> PathType;
    fn set_pathfinding_malus(&mut self, path_type: PathType, malus: f32);
    fn get_pathfinding_malus(&self, path_type: PathType) -> f32;
    fn can_reach_within(
        &mut self,
        entity: &LivingEntity,
        destination: Vector3<f64>,
        distance: f32,
    ) -> bool;
    fn tick(&mut self, entity: &LivingEntity) -> Option<NavigationTarget>;
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
    fn should_recompute_path(&self, entity: &LivingEntity, position: BlockPos) -> bool;
    fn set_avoid_sun(&mut self, avoid_sun: bool);
    fn set_can_walk_over_fences(&mut self, can_walk: bool);
    fn set_can_open_doors(&mut self, can_open: bool);
    fn set_can_pass_doors(&mut self, can_pass: bool);
    fn set_can_float(&mut self, can_float: bool);
    fn can_float(&self) -> bool;
    fn can_navigate_ground(&self) -> bool;
    fn set_required_path_length(&mut self, length: f32);
    /// Furthest a search reaches from the mob.
    fn max_path_length(&self, entity: &LivingEntity) -> f32;
    /// Vanilla `updatePathfinderMaxVisitedNodes`, applied before the next search.
    fn update_pathfinder_max_visited_nodes(&mut self);
    fn set_max_visited_nodes_multiplier(&mut self, multiplier: f32);
    fn reset_max_visited_nodes_multiplier(&mut self);
    fn get_target_pos(&self) -> Option<BlockPos>;
    fn can_path_to_targets_below_surface(&self) -> bool;
    fn set_can_path_to_targets_below_surface(&mut self, can_path: bool);

    fn is_stable_destination(&self, world: &World, pos: &BlockPos) -> bool {
        world.get_block_state(&pos.down()).is_solid_render()
    }
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
    pub path_type_overrides: FxHashMap<PathType, f32>,
    has_pending_path: bool,
    pub can_float: bool,
    pub can_walk_over_fences: bool,
    pub can_open_doors: bool,
    pub can_pass_doors: bool,
    pub avoid_sun: bool,
    pub can_path_to_targets_below_surface: bool,
    path_finder: Option<PathFinder>,
    update_pathfinder_max_visited_nodes: bool,
    pub is_idle: AtomicBool,
}

impl Default for PathNavigation {
    fn default() -> Self {
        Self::new(EvaluatorKind::Walk(WalkNodeEvaluator::default()))
    }
}

impl PathNavigation {
    const fn waypoint_margin(width: f32) -> f32 {
        if width > 0.75 {
            width * 0.5
        } else {
            0.75 - width * 0.5
        }
    }

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
            reach_range: 0,
            max_visited_nodes_multiplier: 1.0,
            is_stuck: false,
            required_path_length: 16.0,
            path_type_overrides: FxHashMap::default(),
            has_pending_path: false,
            can_float: false,
            can_walk_over_fences: false,
            can_open_doors: false,
            can_pass_doors: true,
            avoid_sun: false,
            can_path_to_targets_below_surface: false,
            path_finder: None,
            update_pathfinder_max_visited_nodes: false,
            is_idle: AtomicBool::new(true),
        }
    }

    pub fn set_progress(&mut self, goal: NavigatorGoal) {
        self.is_idle.store(false, Ordering::Relaxed);
        self.speed_modifier = goal.speed;
        self.last_stuck_check = self.tick_count;
        self.last_stuck_check_pos = goal.current_progress;
        self.has_pending_path = true;
        self.current_goal = Some(goal);
        self.reset_stuck_timeout();
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
        self.has_pending_path = false;
        self.path = None;
    }

    fn install_path(&mut self, mut path: Path, entity: &LivingEntity) {
        let world = entity.entity.world.load();
        path.world_change = Some((
            world.uuid,
            world.navigation_change_sequence.load(Ordering::Acquire),
        ));
        self.target_pos = Some(path.get_target());
        self.path = Some(path);
    }

    fn ground_target(&self, entity: &LivingEntity) -> Option<NavigationTarget> {
        let mut target = self.target(entity)?;
        let below = BlockPos::floored_v(target.position).down();
        let state = entity.entity.world.load().get_block_state(&below);
        if !state.is_air() {
            target.position.y = f64::from(below.0.y)
                + state
                    .get_block_collision_shapes_at(&below)
                    .map(|shape| shape.max.y)
                    .fold(0.0, f64::max);
        }
        Some(target)
    }

    fn target(&self, entity: &LivingEntity) -> Option<NavigationTarget> {
        let path = self.path.as_ref().filter(|path| !path.is_done())?;
        Some(NavigationTarget {
            position: path.get_next_entity_pos(entity.entity.width())?,
            speed_modifier: self.speed_modifier,
        })
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

    fn prepare_evaluator(&mut self, entity: &LivingEntity) -> (uuid::Uuid, u64) {
        let world = entity.entity.world.load_full();
        let world_change = (
            world.uuid,
            world.navigation_change_sequence.load(Ordering::Acquire),
        );
        let mut mob_data = MobData::from_entity(entity, &world);
        let context = PathfindingContext::new(BlockPos::floored_v(mob_data.position).0, world);
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

        world_change
    }

    fn get_path_type(&mut self, entity: &LivingEntity, position: BlockPos) -> PathType {
        self.prepare_evaluator(entity);
        let mut context = PathfindingContext::new(
            entity.entity.block_pos.load().0,
            entity.entity.world.load_full(),
        );
        let path_type = self.evaluator.get_path_type(&mut context, position);
        self.evaluator.done();
        path_type
    }

    pub fn compute_path(
        &mut self,
        entity: &LivingEntity,
        destination: Vector3<f64>,
        reach_range: i32,
    ) -> Option<Path> {
        let world_change = self.prepare_evaluator(entity);

        let max_path_length = self.mob_max_follow_range(entity);
        // Vanilla sizes the search from the base follow range until the first update request.
        let finder = self.path_finder.get_or_insert_with(|| {
            PathFinder::new(
                (entity.get_attribute_base(&Attributes::FOLLOW_RANGE)
                    * f64::from(MAX_VISITED_NODES_MULTIPLIER))
                .floor() as usize,
            )
        });
        if self.update_pathfinder_max_visited_nodes {
            finder.set_max_visited_nodes(
                (max_path_length * MAX_VISITED_NODES_MULTIPLIER).floor() as usize
            );
            self.update_pathfinder_max_visited_nodes = false;
        }
        let mut path = finder.find_path_single(
            &mut self.evaluator,
            BlockPos::floored_v(destination),
            max_path_length,
            reach_range,
            self.max_visited_nodes_multiplier,
        );
        self.evaluator.done();
        if let Some(path) = &mut path {
            path.world_change = Some(world_change);
        }
        path
    }

    fn can_update_path(&self, entity: &LivingEntity) -> bool {
        let in_liquid = entity.is_in_water() || entity.entity.touching_lava.load(Ordering::Relaxed);
        match &self.evaluator {
            EvaluatorKind::Walk(_) => {
                entity.entity.on_ground.load(Ordering::Relaxed)
                    || in_liquid
                    || entity.entity.has_vehicle()
            }
            EvaluatorKind::Fly(_) => (self.can_float && in_liquid) || !entity.entity.has_vehicle(),
            EvaluatorKind::Swim(e) => e.allow_breaching() || in_liquid,
            EvaluatorKind::Amphibious(_) => true,
        }
    }

    fn create_path(
        &mut self,
        entity: &LivingEntity,
        destination: Vector3<f64>,
        reach_range: i32,
    ) -> Option<Path> {
        let world = entity.entity.world.load_full();
        if entity.entity.pos.load().y < f64::from(world.get_bottom_y())
            || !self.can_update_path(entity)
        {
            return None;
        }
        let mut target = BlockPos::floored_v(destination);
        if matches!(self.evaluator, EvaluatorKind::Walk(_))
            && !self.can_path_to_targets_below_surface
        {
            target = Self::find_surface_position(&world, target);
        }
        if self.target_pos == Some(target)
            && let Some(path) = &self.path
            && !path.is_done()
        {
            return Some(path.clone());
        }
        let path = self.compute_path(entity, target.0.to_f64(), reach_range);
        if let Some(path) = &path {
            self.target_pos = Some(path.get_target());
            self.reach_range = reach_range;
            self.reset_stuck_timeout();
        }
        path
    }

    fn should_recompute_path(&self, entity: &LivingEntity, position: BlockPos) -> bool {
        if self.has_delayed_recomputation {
            return false;
        }
        let Some(path) = self.path.as_ref().filter(|path| !path.is_done()) else {
            return false;
        };
        let Some(end) = path.get_end_node() else {
            return false;
        };
        let middle = (end.pos.0.to_f64() + entity.entity.pos.load()) * 0.5;
        let radius = (path.get_node_count() - path.get_next_node_index()) as f64;
        position.dist_to_center_sqr(middle.x, middle.y, middle.z) < radius * radius
    }

    fn recompute_path(&mut self, entity: &LivingEntity) {
        let world_age = entity.entity.world.load().get_world_age() as u64;
        if world_age.saturating_sub(self.time_last_recompute) <= MAX_TIME_RECOMPUTE
            || !self.can_update_path(entity)
        {
            self.has_delayed_recomputation = true;
        } else if let Some(target) = self.target_pos {
            self.path = None;
            self.path = self.create_path(entity, target.0.to_f64(), self.reach_range);
            self.time_last_recompute = world_age;
            self.has_delayed_recomputation = false;
        }
    }

    fn update_path(&mut self, entity: &LivingEntity) {
        if self.has_delayed_recomputation {
            self.recompute_path(entity);
        }
        if self.has_pending_path {
            self.has_pending_path = false;
            if let Some(goal) = self.current_goal {
                // Vanilla moveTo(Path): an equivalent route keeps the installed path and its progress.
                match self.create_path(entity, goal.destination, 1) {
                    Some(path) if path.same_as(self.path.as_ref()) => {}
                    path => self.path = path,
                }
                if self.path.as_ref().is_some_and(|path| !path.is_done()) {
                    self.trim_path(entity);
                }
            }
        }
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
        world
            .clip(&pumpkin_world::clip::ClipContext::new(
                start_pos,
                to,
                pumpkin_world::clip::ClipBlock::Collider,
                if blocked_by_fluids {
                    pumpkin_world::clip::ClipFluid::Any
                } else {
                    pumpkin_world::clip::ClipFluid::None
                },
            ))
            .is_miss
    }

    pub fn find_surface_position(world: &World, mut pos: BlockPos) -> BlockPos {
        let block_state = world.get_block_state(&pos);
        if block_state.is_air() {
            let mut column_pos = pos.down();
            while column_pos.0.y >= world.get_bottom_y()
                && world.get_block_state(&column_pos).is_air()
            {
                column_pos.0.y -= 1;
            }
            if column_pos.0.y >= world.get_bottom_y() {
                return BlockPos::new(column_pos.0.x, column_pos.0.y + 1, column_pos.0.z);
            }
            column_pos.0.y = pos.0.y + 1;
            while column_pos.0.y <= world.get_top_y() && world.get_block_state(&column_pos).is_air()
            {
                column_pos.0.y += 1;
            }
            pos = column_pos;
        }
        if !world.get_block_state(&pos).is_solid() {
            return pos;
        }
        let mut column_pos = pos.up();
        while column_pos.0.y <= world.get_top_y() && world.get_block_state(&column_pos).is_solid() {
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
            let speed = entity.get_speed();
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
                let speed = entity.get_speed();
                self.timeout_limit = if speed > 0.0 {
                    dist_to_node / f64::from(speed) * 20.0
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

    const fn can_cut_corner(path_type: PathType) -> bool {
        !matches!(
            path_type,
            PathType::DangerFire | PathType::DangerOther | PathType::WalkableDoor
        )
    }

    #[must_use]
    pub fn should_target_next_node_in_direction(
        mob_pos: Vector3<f64>,
        path: &Path,
        can_move_directly: impl FnOnce() -> bool,
    ) -> bool {
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
        if can_move_directly() {
            return true;
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
    pub fn tick_ground(&mut self, entity: &LivingEntity) -> Option<NavigationTarget> {
        self.tick_count += 1;
        self.update_path(entity);

        if self.path.is_none() {
            self.stop();
            return None;
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
                self.stop();
                return None;
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

                let width = entity.entity.width();
                self.max_distance_to_waypoint = Self::waypoint_margin(width);

                if !on_ground
                    && horizontal_dist < f64::from(self.max_distance_to_waypoint)
                    && dy < -0.5
                {
                    path.advance();
                    return self.ground_target(entity);
                }

                let close_enough = horizontal_dist < f64::from(self.max_distance_to_waypoint)
                    && dy.abs() < NODE_REACH_Y;

                let corner_cut = path
                    .get_next_node()
                    .is_some_and(|n| Self::can_cut_corner(n.path_type))
                    && Self::should_target_next_node_in_direction(mob_pos, path, || false);

                if close_enough || corner_cut {
                    path.advance();
                    return self.ground_target(entity);
                }
            } else {
                self.stop();
                return None;
            }
        }

        self.ground_target(entity)
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

    fn get_speed_modifier(&self) -> f64 {
        self.inner.speed_modifier
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

    fn get_path_type(&mut self, entity: &LivingEntity, position: BlockPos) -> PathType {
        self.inner.get_path_type(entity, position)
    }

    fn set_pathfinding_malus(&mut self, path_type: PathType, malus: f32) {
        self.inner.set_pathfinding_malus(path_type, malus);
    }

    fn get_pathfinding_malus(&self, path_type: PathType) -> f32 {
        self.inner.get_pathfinding_malus(path_type)
    }

    fn can_reach_within(
        &mut self,
        entity: &LivingEntity,
        destination: Vector3<f64>,
        distance: f32,
    ) -> bool {
        self.inner.can_reach_within(entity, destination, distance)
    }

    fn tick(&mut self, entity: &LivingEntity) -> Option<NavigationTarget> {
        self.inner.tick_ground(entity)
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
        self.inner.has_pending_path = false;
        self.inner.current_goal = None;
        if let Some(new_path) = path {
            self.inner.install_path(new_path, entity);
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
            self.inner.reset_stuck_timeout();
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
        self.inner.create_path(entity, destination, reach_range)
    }

    fn recompute_path(&mut self, entity: &LivingEntity) {
        self.inner.recompute_path(entity);
    }

    fn should_recompute_path(&self, entity: &LivingEntity, position: BlockPos) -> bool {
        self.inner.should_recompute_path(entity, position)
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
        self.inner.update_pathfinder_max_visited_nodes = true;
    }

    fn max_path_length(&self, entity: &LivingEntity) -> f32 {
        self.inner.mob_max_follow_range(entity)
    }

    fn update_pathfinder_max_visited_nodes(&mut self) {
        self.inner.update_pathfinder_max_visited_nodes = true;
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

    fn get_speed_modifier(&self) -> f64 {
        self.inner.speed_modifier
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

    fn get_path_type(&mut self, entity: &LivingEntity, position: BlockPos) -> PathType {
        self.inner.get_path_type(entity, position)
    }

    fn set_pathfinding_malus(&mut self, path_type: PathType, malus: f32) {
        self.inner.set_pathfinding_malus(path_type, malus);
    }

    fn get_pathfinding_malus(&self, path_type: PathType) -> f32 {
        self.inner.get_pathfinding_malus(path_type)
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
    fn tick(&mut self, entity: &LivingEntity) -> Option<NavigationTarget> {
        self.inner.tick_count += 1;
        self.inner.update_path(entity);

        let mob_pos = entity.entity.pos.load();
        let can_update = self.inner.can_update_path(entity);
        let width = entity.entity.width();
        self.inner.max_distance_to_waypoint = PathNavigation::waypoint_margin(width);
        if !self.is_done()
            && let Some(path) = &mut self.inner.path
            && let Some(pos) = path.get_next_node_pos()
        {
            let reached = if can_update {
                let margin = f64::from(self.inner.max_distance_to_waypoint);
                ((mob_pos.x - (f64::from(pos.0.x) + 0.5)).abs() < margin
                    && (mob_pos.z - (f64::from(pos.0.z) + 0.5)).abs() < margin
                    && (mob_pos.y - f64::from(pos.0.y)).abs() < NODE_REACH_Y)
                    || (path
                        .get_next_node()
                        .is_some_and(|node| PathNavigation::can_cut_corner(node.path_type))
                        && PathNavigation::should_target_next_node_in_direction(
                            mob_pos,
                            path,
                            || {
                                path.get_next_entity_pos(width).is_some_and(|target| {
                                    PathNavigation::can_move_directly(
                                        &entity.entity.world.load(),
                                        mob_pos,
                                        target,
                                        entity.entity.height(),
                                        true,
                                    )
                                })
                            },
                        ))
            } else {
                path.get_next_entity_pos(width).is_some_and(|target| {
                    entity.entity.block_pos.load() == BlockPos::floored_v(target)
                })
            };
            if reached {
                path.advance();
            }
            if can_update {
                self.inner.do_stuck_detection(mob_pos, entity);
            }
        }
        self.inner.is_idle.store(self.is_done(), Ordering::Relaxed);
        self.inner.target(entity)
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
        self.inner.has_pending_path = false;
        self.inner.current_goal = None;
        if let Some(new_path) = path {
            self.inner.install_path(new_path, entity);
            if self.is_done() {
                return false;
            }
            self.inner.speed_modifier = speed;
            let mob_pos = entity.entity.pos.load();
            self.inner.last_stuck_check = self.inner.tick_count;
            self.inner.last_stuck_check_pos = mob_pos;
            self.inner.reset_stuck_timeout();
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
        self.inner.create_path(entity, destination, reach_range)
    }

    fn recompute_path(&mut self, entity: &LivingEntity) {
        self.inner.recompute_path(entity);
    }

    fn should_recompute_path(&self, entity: &LivingEntity, position: BlockPos) -> bool {
        self.inner.should_recompute_path(entity, position)
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
        self.inner.update_pathfinder_max_visited_nodes = true;
    }

    fn max_path_length(&self, entity: &LivingEntity) -> f32 {
        self.inner.mob_max_follow_range(entity)
    }

    fn update_pathfinder_max_visited_nodes(&mut self) {
        self.inner.update_pathfinder_max_visited_nodes = true;
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
    const MAX_VERTICAL_DISTANCE_TO_WAYPOINT: f64 = 0.5;

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

    fn is_stable_destination(&self, world: &World, pos: &BlockPos) -> bool {
        !world.get_block_state(pos).is_solid_render()
    }

    fn set_speed(&mut self, speed: f64) {
        self.inner.set_speed(speed);
    }

    fn get_speed_modifier(&self) -> f64 {
        self.inner.speed_modifier
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

    fn get_path_type(&mut self, entity: &LivingEntity, position: BlockPos) -> PathType {
        self.inner.get_path_type(entity, position)
    }

    fn set_pathfinding_malus(&mut self, path_type: PathType, malus: f32) {
        self.inner.set_pathfinding_malus(path_type, malus);
    }

    fn get_pathfinding_malus(&self, path_type: PathType) -> f32 {
        self.inner.get_pathfinding_malus(path_type)
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
    fn tick(&mut self, entity: &LivingEntity) -> Option<NavigationTarget> {
        self.inner.tick_count += 1;
        self.inner.update_path(entity);
        if self.is_done() {
            self.inner.stop();
            return None;
        }
        let current_pos = entity.entity.pos.load();
        let mob_pos = current_pos.add_raw(0.0, f64::from(entity.entity.height()) * 0.5, 0.0);
        let can_update = self.inner.can_update_path(entity);
        let width = entity.entity.width();
        self.inner.max_distance_to_waypoint = PathNavigation::waypoint_margin(width);
        if let Some(path) = &mut self.inner.path
            && let Some(pos) = path.get_next_node_pos()
        {
            let reached = if can_update {
                let margin = f64::from(self.inner.max_distance_to_waypoint);
                ((current_pos.x - (f64::from(pos.0.x) + 0.5)).abs() < margin
                    && (current_pos.z - (f64::from(pos.0.z) + 0.5)).abs() < margin
                    && (current_pos.y - f64::from(pos.0.y)).abs()
                        < Self::MAX_VERTICAL_DISTANCE_TO_WAYPOINT)
                    || (path
                        .get_next_node()
                        .is_some_and(|node| PathNavigation::can_cut_corner(node.path_type))
                        && PathNavigation::should_target_next_node_in_direction(
                            mob_pos,
                            path,
                            || {
                                path.get_next_entity_pos(width).is_some_and(|target| {
                                    PathNavigation::can_move_directly(
                                        &entity.entity.world.load(),
                                        mob_pos,
                                        target,
                                        entity.entity.height(),
                                        false,
                                    )
                                })
                            },
                        ))
            } else {
                path.get_next_entity_pos(width).is_some_and(|target| {
                    mob_pos.y > target.y
                        && !entity.entity.on_ground.load(Ordering::Relaxed)
                        && mob_pos.x.floor() == target.x.floor()
                        && mob_pos.z.floor() == target.z.floor()
                })
            };
            if reached {
                path.advance();
            }
        }
        if can_update {
            self.inner.do_stuck_detection(mob_pos, entity);
        }
        self.inner.target(entity)
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
        self.inner.has_pending_path = false;
        self.inner.current_goal = None;
        if let Some(new_path) = path {
            self.inner.install_path(new_path, entity);
            if self.is_done() {
                return false;
            }
            self.inner.speed_modifier = speed;
            let mob_pos = Vector3::new(
                entity.entity.pos.load().x,
                entity.entity.pos.load().y + f64::from(entity.entity.height()) * 0.5,
                entity.entity.pos.load().z,
            );
            self.inner.last_stuck_check = self.inner.tick_count;
            self.inner.last_stuck_check_pos = mob_pos;
            self.inner.reset_stuck_timeout();
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
        self.inner.create_path(entity, destination, reach_range)
    }

    fn recompute_path(&mut self, entity: &LivingEntity) {
        self.inner.recompute_path(entity);
    }

    fn should_recompute_path(&self, entity: &LivingEntity, position: BlockPos) -> bool {
        self.inner.should_recompute_path(entity, position)
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
        self.inner.update_pathfinder_max_visited_nodes = true;
    }

    fn max_path_length(&self, entity: &LivingEntity) -> f32 {
        self.inner.mob_max_follow_range(entity)
    }

    fn update_pathfinder_max_visited_nodes(&mut self) {
        self.inner.update_pathfinder_max_visited_nodes = true;
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
        self.path_to_position = Some(BlockPos::floored_v(goal.destination));
        self.inner.set_progress(goal);
    }

    fn set_speed(&mut self, speed: f64) {
        self.inner.set_speed(speed);
    }

    fn get_speed_modifier(&self) -> f64 {
        self.inner.get_speed_modifier()
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

    fn get_path_type(&mut self, entity: &LivingEntity, position: BlockPos) -> PathType {
        self.inner.get_path_type(entity, position)
    }

    fn set_pathfinding_malus(&mut self, path_type: PathType, malus: f32) {
        self.inner.set_pathfinding_malus(path_type, malus);
    }

    fn get_pathfinding_malus(&self, path_type: PathType) -> f32 {
        self.inner.get_pathfinding_malus(path_type)
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
    fn tick(&mut self, entity: &LivingEntity) -> Option<NavigationTarget> {
        if self.inner.inner.has_pending_path {
            self.inner.inner.update_path(entity);
        }
        if !self.inner.is_done() {
            return self.inner.tick(entity);
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
            let dz = target_center.z - current_pos.z;
            let dist_sq = dx * dx + dz * dz;

            let close_enough = dist_sq < width * width
                && (current_pos.y > f64::from(target_pos.0.y)
                    || (current_pos.y - f64::from(target_pos.0.y)).abs() < 1.0);

            if close_enough {
                self.path_to_position = None;
                self.inner.inner.stop();
            } else {
                return Some(NavigationTarget {
                    position: Vector3::new(
                        f64::from(target_pos.0.x),
                        f64::from(target_pos.0.y),
                        f64::from(target_pos.0.z),
                    ),
                    speed_modifier: self.inner.inner.speed_modifier,
                });
            }
        }
        None
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
        self.path_to_position = Some(BlockPos::floored_v(destination));
        self.inner.create_path(entity, destination, reach_range)
    }

    fn recompute_path(&mut self, entity: &LivingEntity) {
        self.inner.recompute_path(entity);
    }

    fn should_recompute_path(&self, entity: &LivingEntity, position: BlockPos) -> bool {
        self.inner.should_recompute_path(entity, position)
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

    fn max_path_length(&self, entity: &LivingEntity) -> f32 {
        self.inner.max_path_length(entity)
    }

    fn update_pathfinder_max_visited_nodes(&mut self) {
        self.inner.update_pathfinder_max_visited_nodes();
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

    fn get_speed_modifier(&self) -> f64 {
        self.inner.speed_modifier
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

    fn get_path_type(&mut self, entity: &LivingEntity, position: BlockPos) -> PathType {
        self.inner.get_path_type(entity, position)
    }

    fn set_pathfinding_malus(&mut self, path_type: PathType, malus: f32) {
        self.inner.set_pathfinding_malus(path_type, malus);
    }

    fn get_pathfinding_malus(&self, path_type: PathType) -> f32 {
        self.inner.get_pathfinding_malus(path_type)
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
    fn tick(&mut self, entity: &LivingEntity) -> Option<NavigationTarget> {
        if entity.entity.touching_water.load(Ordering::Relaxed) {
            self.inner.tick_count += 1;
            self.inner.update_path(entity);

            let mob_pos = Vector3::new(
                entity.entity.pos.load().x,
                entity.entity.pos.load().y + f64::from(entity.entity.height()) * 0.5,
                entity.entity.pos.load().z,
            );
            self.inner.do_stuck_detection(mob_pos, entity);

            if self.is_done() {
                self.inner.stop();
            } else if let Some(path) = &mut self.inner.path
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
        } else {
            self.inner.tick_ground(entity)?;
        }
        self.inner.target(entity)
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
        self.inner.has_pending_path = false;
        self.inner.current_goal = None;
        if let Some(new_path) = path {
            self.inner.install_path(new_path, entity);
            if self.is_done() {
                return false;
            }
            self.inner.speed_modifier = speed;
            let mob_pos = Vector3::new(
                entity.entity.pos.load().x,
                entity.entity.pos.load().y + f64::from(entity.entity.height()) * 0.5,
                entity.entity.pos.load().z,
            );
            self.inner.last_stuck_check = self.inner.tick_count;
            self.inner.last_stuck_check_pos = mob_pos;
            self.inner.reset_stuck_timeout();
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
        self.inner.create_path(entity, destination, reach_range)
    }

    fn recompute_path(&mut self, entity: &LivingEntity) {
        self.inner.recompute_path(entity);
    }

    fn should_recompute_path(&self, entity: &LivingEntity, position: BlockPos) -> bool {
        self.inner.should_recompute_path(entity, position)
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
        self.inner.update_pathfinder_max_visited_nodes = true;
    }

    fn max_path_length(&self, entity: &LivingEntity) -> f32 {
        self.inner.mob_max_follow_range(entity)
    }

    fn update_pathfinder_max_visited_nodes(&mut self) {
        self.inner.update_pathfinder_max_visited_nodes = true;
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use pumpkin_data::{dimension::Dimension, entity::EntityType};
    use pumpkin_util::math::boundingbox::EntityDimensions;

    use super::*;
    use crate::entity::{
        Entity,
        ai::test_support::{world, world_in_dimension},
    };

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    fn living(world: Arc<World>, entity_type: &'static EntityType) -> LivingEntity {
        LivingEntity::new(Entity::new(
            world,
            Vector3::new(0.5, 64.0, 0.5),
            entity_type,
        ))
    }

    #[tokio::test]
    async fn water_navigation_finishes_at_the_position_it_steers_toward() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let living = living(world.clone(), &EntityType::COD);
        living.entity.touching_water.store(true, Ordering::Relaxed);
        let target = BlockPos::new(0, 64, 0);
        let path = Path::new(vec![Node::new(target)], target, true);
        let mut navigation = WaterBoundPathNavigation::new(false);
        navigation.move_to_path(Some(path), 1.0, &living);
        let steering_target = navigation.tick(&living);
        world.level.shutdown().await;
        assert!(steering_target.is_none());
        assert!(navigation.is_done());
        Ok(())
    }

    #[tokio::test]
    async fn fish_paths_cross_water_plants() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let chunk = pumpkin_world::chunk::ChunkData::empty_sync(0, 0);
        for z in 0..=4 {
            chunk.set_block_absolute_y(0, 64, z, pumpkin_data::Block::WATER.default_state.id);
        }
        world.level.loaded_chunks.insert(
            pumpkin_util::math::vector2::Vector2::new(0, 0),
            chunk.clone(),
        );
        let living = living(world.clone(), &EntityType::COD);
        living.entity.touching_water.store(true, Ordering::Relaxed);
        for plant in [
            &pumpkin_data::Block::SEAGRASS,
            &pumpkin_data::Block::TALL_SEAGRASS,
            &pumpkin_data::Block::KELP,
            &pumpkin_data::Block::KELP_PLANT,
        ] {
            chunk.set_block_absolute_y(0, 64, 2, plant.default_state.id);
            let mut navigation = WaterBoundPathNavigation::new(false);
            let path = navigation.create_path(&living, Vector3::new(0.5, 64.0, 4.5), 0);
            assert!(
                path.is_some_and(|path| path.can_reach()),
                "{} blocked the water corridor",
                plant.name
            );
        }
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn swim_search_starts_at_the_current_mob_footprint() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let living = living(world.clone(), &EntityType::IRON_GOLEM);
        let mut navigation =
            PathNavigation::new(EvaluatorKind::Swim(SwimNodeEvaluator::new(false)));

        let path = navigation.compute_path(&living, living.entity.pos.load(), 0);
        world.level.shutdown().await;

        let start = path
            .as_ref()
            .and_then(|path| path.get_node(0))
            .map(|node| node.pos);
        assert_eq!(start, Some(BlockPos::new(-1, 64, -1)));
        Ok(())
    }

    #[tokio::test]
    async fn floating_search_uses_water_contact_instead_of_falling_to_the_world_floor() -> TestResult
    {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let living = living(world.clone(), &EntityType::COW);
        living.entity.touching_water.store(true, Ordering::Relaxed);
        living.entity.on_ground.store(false, Ordering::Relaxed);
        let mut navigation = GroundPathNavigation::new();
        navigation.set_can_float(true);

        let floating = navigation
            .inner
            .compute_path(&living, living.entity.pos.load(), 0);
        living.entity.touching_water.store(false, Ordering::Relaxed);
        let airborne = navigation
            .inner
            .compute_path(&living, living.entity.pos.load(), 0);
        world.level.shutdown().await;

        assert_eq!(
            floating.and_then(|path| path.get_node(0).map(|node| node.pos.0.y)),
            Some(64)
        );
        assert_eq!(
            airborne.and_then(|path| path.get_node(0).map(|node| node.pos.0.y)),
            Some(-63)
        );
        Ok(())
    }

    #[tokio::test]
    async fn search_and_surface_scan_use_the_dimension_height_limits() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world_in_dimension(&folder, Dimension::THE_NETHER.clone());
        let living = living(world.clone(), &EntityType::COW);
        living.entity.on_ground.store(false, Ordering::Relaxed);
        let mut navigation = GroundPathNavigation::new();

        let path = navigation
            .inner
            .compute_path(&living, living.entity.pos.load(), 0);
        let surface = PathNavigation::find_surface_position(&world, BlockPos::new(0, 64, 0));
        world.level.shutdown().await;

        assert_eq!(
            path.and_then(|path| path.get_node(0).map(|node| node.pos.0.y)),
            Some(1)
        );
        assert_eq!(surface, BlockPos::new(0, 256, 0));
        Ok(())
    }

    #[tokio::test]
    async fn waypoint_reach_uses_current_dimensions_after_a_size_change() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let living = living(world.clone(), &EntityType::IRON_GOLEM);
        living.entity.pos.store(Vector3::new(-0.1, 64.0, 0.5));
        living.entity.on_ground.store(true, Ordering::Relaxed);
        let mut navigation = GroundPathNavigation::new();
        navigation.set_progress(NavigatorGoal::new(
            living.entity.pos.load(),
            Vector3::new(6.5, 64.0, 0.5),
            1.0,
        ));
        navigation.move_to_path(
            Some(Path::new(
                vec![
                    Node::new(BlockPos::new(0, 64, 0)),
                    Node::new(BlockPos::new(3, 64, 0)),
                    Node::new(BlockPos::new(6, 64, 0)),
                ],
                BlockPos::new(6, 64, 0),
                true,
            )),
            1.0,
            &living,
        );

        navigation.tick(&living);
        let first = navigation.get_path().map(Path::get_next_node_index);
        living
            .entity
            .entity_dimension
            .store(EntityDimensions::new(2.4, 2.7, 2.3));
        living.entity.pos.store(Vector3::new(2.6, 64.0, 0.5));
        navigation.tick(&living);
        let second = navigation.get_path().map(Path::get_next_node_index);
        world.level.shutdown().await;

        assert_eq!((first, second), (Some(1), Some(2)));
        Ok(())
    }

    #[tokio::test]
    async fn search_closes_nodes_before_evaluating_neighbors_and_resets_between_searches()
    -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let living = living(world.clone(), &EntityType::BAT);
        let data = MobData::from_entity(&living, &world);
        let mut evaluator = EvaluatorKind::Fly(FlyNodeEvaluator::new());
        let mut finder = PathFinder::new(40);
        let target = BlockPos::new(100, 64, 100);
        let mut paths = Vec::new();
        for _ in 0..2 {
            evaluator.prepare(
                PathfindingContext::new(BlockPos::floored_v(data.position).0, world.clone()),
                data,
            );
            paths.push(finder.find_path_single(&mut evaluator, target, 5.0, 0, 1.0));
            let start = evaluator.base_mut().get_node(BlockPos::new(0, 64, 0));
            assert!(start.closed);
            assert_eq!(start.came_from, None);
            let mut neighbors = Vec::new();
            evaluator.get_neighbors(&Node::new(BlockPos::new(1, 64, 0)), &mut neighbors);
            assert!(!neighbors.iter().any(|node| node.pos == start.pos));
            evaluator.done();
        }
        world.level.shutdown().await;
        assert!(
            paths[0]
                .as_ref()
                .is_some_and(|first| first.same_as(paths[1].as_ref()))
        );
        Ok(())
    }

    #[tokio::test]
    async fn flying_navigation_advances_from_the_start_node_and_gains_height() -> TestResult {
        use crate::entity::{ai::control::MoveControlTrait, passive::allay::AllayEntity};

        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let allay = AllayEntity::new(crate::entity::Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &EntityType::ALLAY,
        ));
        let living = &allay.mob_entity.living_entity;
        living.entity.on_ground.store(true, Ordering::Relaxed);
        let mut navigation = FlyingPathNavigation::new();
        let target = BlockPos::new(2, 66, 0);
        navigation.move_to_path(
            Some(Path::new(
                vec![
                    Node::new(BlockPos::new(0, 64, 0)),
                    Node::new(BlockPos::new(1, 65, 0)),
                    Node::new(target),
                ],
                target,
                true,
            )),
            1.0,
            living,
        );
        let next = navigation.tick(living).ok_or("missing flight waypoint")?;
        assert_eq!(
            navigation.get_path().map(Path::get_next_node_index),
            Some(1)
        );
        assert_eq!(next.position, Vector3::new(1.5, 65.0, 0.5));
        let mut control =
            crate::entity::ai::control::flying_move_control::FlyingMoveControl::new(20, true);
        for _ in 0..80 {
            if let Some(next) = navigation.tick(living) {
                control.set_wanted_position(
                    next.position.x,
                    next.position.y,
                    next.position.z,
                    next.speed_modifier,
                );
            }
            control.tick(allay.as_ref());
            living.travel_flying(allay.as_ref());
        }
        let pos = living.entity.pos.load();
        world.level.shutdown().await;
        assert!(pos.y > 65.0, "Allay failed to gain height: {pos:?}");
        assert!(
            pos.x > 1.0,
            "Allay stayed around its starting node: {pos:?}"
        );
        Ok(())
    }

    #[tokio::test]
    async fn flying_navigation_cuts_a_corner_when_the_flight_line_is_clear() -> TestResult {
        use crate::entity::passive::allay::AllayEntity;

        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let allay = AllayEntity::new(crate::entity::Entity::new(
            world.clone(),
            Vector3::new(1.0, 64.0, 0.5),
            &EntityType::ALLAY,
        ));
        let living = &allay.mob_entity.living_entity;
        let mut navigation = FlyingPathNavigation::new();
        let target = BlockPos::new(4, 64, 0);
        navigation.move_to_path(
            Some(Path::new(
                vec![
                    Node::new(BlockPos::new(2, 64, 0)),
                    Node::new(BlockPos::new(3, 64, 0)),
                    Node::new(target),
                ],
                target,
                true,
            )),
            1.0,
            living,
        );
        navigation.tick(living);
        let next_index = navigation.get_path().map(Path::get_next_node_index);
        world.level.shutdown().await;
        assert_eq!(next_index, Some(1));
        Ok(())
    }

    #[tokio::test]
    async fn navigation_respects_a_zero_search_budget() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let living = living(world.clone(), &EntityType::BAT);
        let mut navigation = FlyingPathNavigation::new();
        navigation.set_max_visited_nodes_multiplier(0.0);
        let path = navigation.create_path(&living, Vector3::new(8.0, 64.0, 0.0), 0);
        world.level.shutdown().await;
        assert_eq!(path.as_ref().map(Path::get_node_count), Some(1));
        assert!(path.is_some_and(|path| !path.can_reach()));
        Ok(())
    }

    #[tokio::test]
    async fn repeated_target_preserves_progress_but_new_target_replaces_the_route() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let living = living(world.clone(), &EntityType::BAT);
        let mut navigation = FlyingPathNavigation::new();
        navigation.move_to_coords(8.0, 64.0, 0.0, 1.0, &living);
        navigation.tick(&living);
        let before = navigation.get_path_mut().ok_or("missing initial route")?;
        before.set_next_node_index(3);
        navigation.move_to_coords(8.0, 64.0, 0.0, 2.0, &living);
        navigation.tick(&living);
        let reused_index = navigation.get_path().map(Path::get_next_node_index);
        navigation.move_to_coords(0.0, 64.0, 8.0, 1.0, &living);
        navigation.tick(&living);
        let target = navigation.get_path().map(Path::get_target);
        world.level.shutdown().await;
        assert_eq!(reused_index, Some(3));
        assert_eq!(target, Some(BlockPos::new(0, 64, 8)));
        Ok(())
    }

    #[tokio::test]
    async fn reachability_query_does_not_retarget_the_installed_route() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let living = living(world.clone(), &EntityType::BAT);
        let mut navigation = FlyingPathNavigation::new();
        let path = navigation.create_path(&living, Vector3::new(8.0, 64.0, 0.0), 0);
        navigation.move_to_path(path, 1.0, &living);
        navigation.can_reach_within(&living, Vector3::new(0.0, 64.0, 8.0), 1.0);
        world
            .level_time
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .world_age = 21;
        navigation.recompute_path(&living);
        let target = navigation.get_path().map(Path::get_target);
        world.level.shutdown().await;
        assert_eq!(target, Some(BlockPos::new(8, 64, 0)));
        Ok(())
    }

    #[tokio::test]
    async fn explicit_path_and_cancellation_supersede_pending_requests() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let living = living(world.clone(), &EntityType::COW);
        let mut navigation = GroundPathNavigation::new();
        let destination = BlockPos::new(8, 64, 0);
        navigation.move_to_coords(0.0, 64.0, 8.0, 1.0, &living);
        navigation.move_to_path(
            Some(Path::new(vec![Node::new(destination)], destination, true)),
            1.0,
            &living,
        );
        navigation.tick(&living);
        let installed = navigation.get_path().map(Path::get_target);
        navigation.move_to_coords(0.0, 64.0, 8.0, 1.0, &living);
        navigation.move_to_path(None, 1.0, &living);
        navigation.tick(&living);
        let cancelled = navigation.get_path().is_none();
        world.level.shutdown().await;
        assert_eq!(installed, Some(destination));
        assert!(cancelled);
        Ok(())
    }

    #[tokio::test]
    async fn recomputation_waits_past_twenty_ticks_and_until_ground_navigation_is_eligible()
    -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let living = living(world.clone(), &EntityType::COW);
        living.entity.on_ground.store(true, Ordering::Relaxed);
        let mut navigation = GroundPathNavigation::new();
        navigation.set_can_path_to_targets_below_surface(true);
        navigation.move_to_path(
            Some(Path::new(
                vec![
                    Node::new(BlockPos::new(0, 64, 0)),
                    Node::new(BlockPos::new(4, 64, 0)),
                    Node::new(BlockPos::new(8, 64, 0)),
                ],
                BlockPos::new(8, 64, 0),
                true,
            )),
            1.0,
            &living,
        );
        navigation
            .get_path_mut()
            .ok_or("missing initial route")?
            .set_next_node_index(2);
        world
            .level_time
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .world_age = 20;
        navigation.recompute_path(&living);
        assert_eq!(
            navigation.get_path().map(Path::get_next_node_index),
            Some(2)
        );
        living.entity.on_ground.store(false, Ordering::Relaxed);
        world
            .level_time
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .world_age = 21;
        navigation.recompute_path(&living);
        assert_eq!(
            navigation.get_path().map(Path::get_next_node_index),
            Some(2)
        );
        living.entity.on_ground.store(true, Ordering::Relaxed);
        navigation.tick(&living);
        let recomputed = navigation.get_path().map(Path::get_next_node_index);
        world.level.shutdown().await;
        assert_eq!(recomputed, Some(1));
        assert!(!navigation.inner.has_delayed_recomputation);
        Ok(())
    }
    #[tokio::test]
    async fn wall_climber_resolves_pending_search_before_using_its_fallback() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let living = living(world.clone(), &EntityType::SPIDER);
        living.entity.on_ground.store(true, Ordering::Relaxed);
        let mut navigation = WallClimberNavigation::new();
        navigation.set_can_path_to_targets_below_surface(true);
        navigation.move_to_coords(8.0, 64.0, 0.0, 1.0, &living);
        navigation.tick(&living);
        let target = navigation.get_path().map(Path::get_target);
        world.level.shutdown().await;
        assert_eq!(target, Some(BlockPos::new(8, 64, 0)));
        Ok(())
    }
    #[tokio::test]
    async fn invalidation_uses_remaining_nodes_and_the_actual_path_end() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let living = living(world.clone(), &EntityType::BAT);
        living.entity.pos.store(Vector3::new(0.0, 65.0, 1.0));
        let mut navigation = FlyingPathNavigation::new();
        navigation.move_to_path(
            Some(Path::new(
                vec![
                    Node::new(BlockPos::new(0, 64, 0)),
                    Node::new(BlockPos::new(3, 64, 0)),
                    Node::new(BlockPos::new(7, 64, 0)),
                ],
                BlockPos::new(100, 64, 0),
                false,
            )),
            1.0,
            &living,
        );
        navigation
            .get_path_mut()
            .ok_or("missing route")?
            .set_next_node_index(2);
        assert!(navigation.should_recompute_path(&living, BlockPos::new(3, 64, 0)));
        assert!(!navigation.should_recompute_path(&living, BlockPos::new(4, 64, 0)));
        navigation
            .get_path_mut()
            .ok_or("missing route")?
            .set_next_node_index(1);
        assert!(navigation.should_recompute_path(&living, BlockPos::new(4, 64, 0)));
        navigation.recompute_path(&living);
        assert!(!navigation.should_recompute_path(&living, BlockPos::new(3, 64, 0)));
        world.level.shutdown().await;
        Ok(())
    }
}
