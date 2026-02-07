use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
use std::{collections::HashMap, time::Instant};

use crate::entity::ai::pathfinder::{
    node::{Coordinate, Node, PathType, Target},
    node_evaluator::{BaseNodeEvaluator, MobData, NodeEvaluator},
    pathfinding_context::PathfindingContext,
};

const DIRECTIONS: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
const DIAGONAL_DIRECTIONS: [(i32, i32); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];

const DEFAULT_MOB_JUMP_HEIGHT: f64 = 1.125;

#[derive(Clone, Copy)]
struct StandingResult {
    can_stand: bool,
    path_type: PathType,
}

impl StandingResult {
    const fn walkable(path_type: PathType) -> Self {
        Self {
            can_stand: true,
            path_type,
        }
    }

    const fn open(path_type: PathType) -> Self {
        Self {
            can_stand: true,
            path_type,
        }
    }

    const fn blocked() -> Self {
        Self {
            can_stand: false,
            path_type: PathType::Blocked,
        }
    }
}

pub struct WalkNodeEvaluator {
    base: BaseNodeEvaluator,
    path_types_cache: HashMap<Vector3<i32>, PathType>,
    collision_cache: HashMap<Vector3<i32>, bool>,
    reusable_neighbors: [Option<Node>; 4],
}

impl WalkNodeEvaluator {
    #[must_use]
    pub fn new() -> Self {
        Self {
            base: BaseNodeEvaluator::new(),
            path_types_cache: HashMap::new(),
            collision_cache: HashMap::new(),
            reusable_neighbors: [None, None, None, None],
        }
    }

    const fn is_amphibious(&self) -> bool {
        self.base.can_float
    }

    // TODO: Should theoretically be handled by context
    fn get_floor_level(&self, pos: Vector3<i32>) -> f64 {
        self.base.context.as_ref().map_or_else(
            || {
                self.base
                    .mob_data
                    .as_ref()
                    .map_or_else(|| f64::from(pos.y), |d| f64::from(d.block_position().1))
            },
            |c| f64::from(c.mob_position().y),
        )
    }

    fn get_mob_jump_height(&self) -> f64 {
        self.base
            .mob_data
            .as_ref()
            .map_or(DEFAULT_MOB_JUMP_HEIGHT, |d| f64::from(d.max_step_height))
    }

    fn is_neighbor_valid(&self, neighbor: Option<&Node>, current: &Node) -> bool {
        if let Some(neighbor) = neighbor {
            if neighbor.closed {
                return false;
            }

            let path_type = neighbor.path_type;
            if !path_type.is_passable() {
                return false;
            }

            if let Some(ref mob_data) = self.base.mob_data {
                let malus = mob_data.get_pathfinding_malus(path_type);
                if malus < 0.0 {
                    return false;
                }
            }

            let height_diff = neighbor.pos.0.y - current.pos.0.y;
            let horizontal_diff = f64::from(
                (neighbor.pos.0.x - current.pos.0.x).pow(2)
                    + (neighbor.pos.0.z - current.pos.0.z).pow(2),
            );
            let horizontal_distance = horizontal_diff.sqrt();

            // Check jump height for upward movement
            if height_diff > 0 {
                let jump_height = self.get_mob_jump_height();
                if f64::from(height_diff) > jump_height {
                    return false;
                }

                // TODO: Check ifg calculated correctly
                if horizontal_distance > 1.0 && f64::from(height_diff) > jump_height * 0.8 {
                    return false;
                }
            }

            if height_diff < 0 {
                let fall_distance = -height_diff;
                if fall_distance > 3 {
                    return false;
                }

                if horizontal_distance > 1.0 && fall_distance > 2 {
                    return false;
                }
            }

            true
        } else {
            false
        }
    }

    fn is_diagonal_valid(
        &self,
        current: &Node,
        neighbor1: Option<&Node>,
        neighbor2: Option<&Node>,
    ) -> bool {
        if !self.is_neighbor_valid(neighbor1, current)
            || !self.is_neighbor_valid(neighbor2, current)
        {
            return false;
        }

        if let Some(n1) = neighbor1
            && n1.path_type.has_partial_collision()
        {
            return false;
        }
        if let Some(n2) = neighbor2
            && n2.path_type.has_partial_collision()
        {
            return false;
        }

        true
    }

    fn is_diagonal_node_valid(diagonal: Option<&Node>) -> bool {
        diagonal.is_some_and(|n| !n.path_type.has_partial_collision())
    }

    async fn find_accepted_node(
        &mut self,
        pos: Vector3<i32>,
        max_up_step: i32,
        floor_level: f64,
        facing: (i32, i32),
        current_path_type: PathType,
    ) -> Option<Node> {
        let mut node = None;
        let mut search_pos = pos;

        if let Some(valid_node) = self
            .try_jump_on(
                search_pos.as_blockpos(),
                max_up_step,
                floor_level,
                facing,
                current_path_type,
            )
            .await
        {
            return Some(valid_node);
        }

        let max_jump_height = self.get_mob_jump_height().floor() as i32;
        // TODO: Might need to use floats here
        for dy in 1..=max_jump_height.min(max_up_step) {
            search_pos.y = pos.y + dy;

            if let Some(valid_node) = self
                .try_jump_on(
                    search_pos.as_blockpos(),
                    max_up_step,
                    floor_level,
                    facing,
                    current_path_type,
                )
                .await
            {
                node = Some(valid_node);
                break;
            }
        }

        if node.is_none() {
            for dy in 1..=3 {
                search_pos.y = pos.y - dy;

                if let Some(valid_node) = self
                    .try_jump_on(
                        search_pos.as_blockpos(),
                        max_up_step,
                        floor_level,
                        facing,
                        current_path_type,
                    )
                    .await
                {
                    node = Some(valid_node);
                    break;
                }

                if let Some(valid_node) = self.try_find_first_ground_node_below(search_pos).await {
                    node = Some(valid_node);
                    break;
                }
            }
        }

        if node.is_none()
            && self.is_amphibious()
            && let Some(valid_node) = self.try_find_first_non_water_below(pos, None).await
        {
            node = Some(valid_node);
        }

        node
    }

    async fn try_jump_on(
        &mut self,
        pos: BlockPos,
        _max_up_step: i32,
        floor_level: f64,
        _facing: (i32, i32),
        _current_path_type: PathType,
    ) -> Option<Node> {
        let standing_result = self.can_stand_at(pos.as_vector3()).await;
        if !standing_result.can_stand {
            return None;
        }

        if f64::from(pos.0.y) > floor_level {
            let jump_height = f64::from(pos.0.y) - floor_level;
            if jump_height > self.get_mob_jump_height() {
                return None;
            }
        }

        if let Some(ref mob_data) = self.base.mob_data {
            for dy in 1..mob_data.get_bb_height() {
                let above_pos = pos.add(0, dy, 0);
                let above_type = self.get_cached_path_type(above_pos.as_vector3()).await;
                if !above_type.is_passable() {
                    return None;
                }
            }
        }

        let mut node = self.base.get_node(pos);
        node.path_type = standing_result.path_type;
        node.cost_malus = standing_result.path_type.get_malus();

        Some(node)
    }

    async fn can_stand_at(&mut self, pos: Vector3<i32>) -> StandingResult {
        let current_type = self.get_cached_path_type(pos).await;
        if !current_type.is_passable() {
            return StandingResult::blocked();
        }

        let ground_result = self.find_ground_below(pos, 4).await;
        if let Some((ground_y, _ground_type)) = ground_result {
            let fall_distance = pos.y - ground_y;
            if fall_distance > 3 {
                return StandingResult::blocked();
            }

            if fall_distance == 1 {
                return StandingResult::walkable(current_type);
            }
            return StandingResult::open(current_type);
        }

        StandingResult::blocked()
    }

    async fn find_ground_below(
        &mut self,
        pos: Vector3<i32>,
        max_distance: i32,
    ) -> Option<(i32, PathType)> {
        for dy in 1..=max_distance {
            let path_type = self
                .get_cached_path_type(pos.sub(&Vector3::new(0, dy, 0)))
                .await;

            if path_type == PathType::Blocked || path_type == PathType::Walkable {
                return Some((pos.y - dy, path_type));
            }
        }
        None
    }

    async fn try_find_first_non_water_below(
        &mut self,
        pos: Vector3<i32>,
        _start_node: Option<&Node>,
    ) -> Option<Node> {
        let mut pos = pos;
        for _ in 0..16 {
            let path_type = self.get_cached_path_type(pos).await;
            if !path_type.is_water() {
                if path_type.is_passable() {
                    let mut node = self.base.get_node(pos.as_blockpos());
                    node.path_type = path_type;
                    node.cost_malus = path_type.get_malus();
                    return Some(node);
                }
                break;
            }
            pos.y -= 1;
        }

        None
    }

    async fn try_find_first_ground_node_below(&mut self, pos: Vector3<i32>) -> Option<Node> {
        for dy in 0..=3 {
            let pos = pos.sub(&Vector3::new(0, dy, 0));
            let path_type = self.get_cached_path_type(pos).await;

            if path_type == PathType::Blocked || path_type == PathType::Walkable {
                let above_type = self.get_cached_path_type(pos.add_raw(0, 1, 0)).await;
                let two_above_type = self.get_cached_path_type(pos.add_raw(0, 2, 0)).await;

                if above_type.is_passable() && two_above_type.is_passable() {
                    let mut node = self.base.get_node(pos.add_raw(0, 1, 0).as_blockpos());
                    node.path_type = above_type;
                    node.cost_malus = above_type.get_malus();

                    if dy > 0 {
                        // Small penalty for falling
                        // TODO: Check if correct
                        node.cost_malus += dy as f32 * 0.5;
                    }

                    return Some(node);
                }
            }

            if path_type == PathType::Walkable {
                let above_type = self.get_cached_path_type(pos.add_raw(0, 1, 0)).await;
                if above_type.is_passable() {
                    let mut node = self.base.get_node(pos.as_blockpos());
                    node.path_type = path_type;
                    node.cost_malus = path_type.get_malus();

                    if dy > 0 {
                        node.cost_malus += dy as f32 * 0.5;
                    }

                    return Some(node);
                }
            }
        }

        None
    }

    async fn get_cached_path_type(&mut self, pos: Vector3<i32>) -> PathType {
        if let Some(&cached) = self.path_types_cache.get(&pos) {
            return cached;
        }

        // Temporarily take the context out to avoid overlapping borrows when calling
        // the async helper which requires `&mut self`
        // Clone mob_data so we can call helper while `self.base.context` is None.
        let path_type = if let Some(ctx) = self.base.context.take()
            && let Some(mob_clone) = self.base.mob_data.clone()
        {
            let start = Instant::now();
            let res = self.get_path_type_of_mob(&ctx, pos, &mob_clone).await;
            let _dur = start.elapsed();
            self.base.context = Some(ctx);
            res
        } else {
            PathType::Blocked
        };

        self.path_types_cache.insert(pos, path_type);
        path_type
    }

    fn has_collisions(&mut self, center: Vector3<i32>) -> bool {
        if let Some(&cached) = self.collision_cache.get(&center) {
            return cached;
        }

        let has_collision = self
            .base
            .context
            .as_mut()
            .is_some_and(|c| c.has_collisions(center));

        self.collision_cache.insert(center, has_collision);
        has_collision
    }

    async fn can_start_at(&mut self, pos: Vector3<i32>) -> bool {
        let path_type = self.get_cached_path_type(pos).await;
        path_type.is_passable() && !self.has_collisions(pos)
    }

    async fn get_start_node(&mut self, pos: Vector3<i32>) -> Option<Node> {
        if !self.can_start_at(pos).await {
            return None;
        }

        let mut node = self.base.get_node(pos.as_blockpos());
        let path_type = self.get_cached_path_type(pos).await;
        node.path_type = path_type;
        node.cost_malus = path_type.get_malus();

        Some(node)
    }
}

impl NodeEvaluator for WalkNodeEvaluator {
    fn prepare(&mut self, context: PathfindingContext, mob_data: MobData) {
        self.base.entity_width = mob_data.get_bb_width();
        self.base.entity_height = mob_data.get_bb_height();
        self.base.entity_depth = mob_data.get_bb_width();

        self.base.context = Some(context);
        self.base.mob_data = Some(mob_data);
        self.path_types_cache.clear();
        self.collision_cache.clear();
    }

    fn done(&mut self) {
        self.base.context = None;
        self.base.mob_data = None;
        self.path_types_cache.clear();
        self.collision_cache.clear();
    }

    async fn get_start(&mut self) -> Option<Node> {
        if let Some(ref mob_data) = self.base.mob_data {
            let start_pos = mob_data.block_position();

            if let Some(node) = self.get_start_node(start_pos.into()).await {
                return Some(node);
            }

            for &(dx, dz) in &DIRECTIONS {
                let try_pos = (start_pos.0 + dx, start_pos.1, start_pos.2 + dz);
                if let Some(node) = self.get_start_node(try_pos.into()).await {
                    return Some(node);
                }
            }

            let above_pos = Vector3::new(start_pos.0, start_pos.1 + 1, start_pos.2);
            return self.get_start_node(above_pos).await;
        }
        None
    }

    fn get_target(&mut self, pos: BlockPos) -> Target {
        let node = self.base.get_node(pos);
        Target::new(node)
    }

    async fn get_neighbors(&mut self, current: &Node) -> Vec<Node> {
        let mut out_neighbors: Vec<Node> = Vec::new();
        let max_up_step = self.get_mob_jump_height().floor() as i32;
        let floor_level = self.get_floor_level(current.pos.0);

        for i in 0..4 {
            self.reusable_neighbors[i] = None;
        }

        for dy in [-1, 1] {
            let vertical_pos = current.pos.0.add_raw(0, dy, 0);

            if dy > 0 && f64::from(dy) > self.get_mob_jump_height() {
                continue;
            }

            if dy < 0 && -dy > 3 {
                continue;
            }

            if let Some(vertical_neighbor) = self
                .find_accepted_node(
                    vertical_pos,
                    max_up_step,
                    floor_level,
                    (0, 0),
                    current.path_type,
                )
                .await
                && self.is_neighbor_valid(Some(&vertical_neighbor), current)
            {
                out_neighbors.push(vertical_neighbor);
            }
        }

        for (i, &(dx, dz)) in DIRECTIONS.iter().enumerate() {
            let neighbor_pos = current.pos.0.add_raw(dx, 0, dz);

            let start = Instant::now();
            let neighbor_opt = self
                .find_accepted_node(
                    neighbor_pos,
                    max_up_step,
                    floor_level,
                    (dx, dz),
                    current.path_type,
                )
                .await;
            let _dur = start.elapsed();

            if let Some(neighbor) = neighbor_opt {
                self.reusable_neighbors[i] = Some(neighbor.clone());
                if self.is_neighbor_valid(Some(&neighbor), current) {
                    out_neighbors.push(neighbor);
                }
            }
        }

        for &(dx, dz) in &DIAGONAL_DIRECTIONS {
            let dir1_idx = DIRECTIONS
                .iter()
                .position(|&(x, z)| x == dx && z == 0)
                .unwrap_or(0);
            let dir2_idx = DIRECTIONS
                .iter()
                .position(|&(x, z)| x == 0 && z == dz)
                .unwrap_or(1);

            if self.is_diagonal_valid(
                current,
                self.reusable_neighbors[dir1_idx].as_ref(),
                self.reusable_neighbors[dir2_idx].as_ref(),
            ) {
                let diagonal_pos = current.pos.0.add_raw(dx, 0, dz);

                let start_diag = Instant::now();
                let diagonal_opt = self
                    .find_accepted_node(
                        diagonal_pos,
                        max_up_step,
                        floor_level,
                        (dx, dz),
                        current.path_type,
                    )
                    .await;
                let _dur_diag = start_diag.elapsed();

                if let Some(mut diagonal) = diagonal_opt {
                    // Prioritize straight movement
                    // TODO: Check
                    diagonal.cost_malus += 0.4;

                    if Self::is_diagonal_node_valid(Some(&diagonal)) {
                        out_neighbors.push(diagonal);
                    }
                }
            }
        }

        out_neighbors
    }

    async fn get_path_type_of_mob(
        &mut self,
        context: &PathfindingContext,
        pos: Vector3<i32>,
        mob_data: &MobData,
    ) -> PathType {
        let mut path_types = Vec::new();

        for dy in 0..mob_data.get_bb_height() {
            for dx in 0..mob_data.get_bb_width() {
                for dz in 0..mob_data.get_bb_width() {
                    let check_pos = pos.add_raw(dx, dy, dz);
                    let path_type = context.compute_path_type_from_state(check_pos).await;
                    path_types.push(path_type);
                }
            }
        }

        let mut result = PathType::Open;
        for path_type in path_types {
            if path_type.is_blocked() {
                return path_type;
            }
            if path_type.get_malus() > result.get_malus() {
                result = path_type;
            }
        }

        result
    }

    async fn get_path_type(
        &mut self,
        context: &mut PathfindingContext,
        pos: Vector3<i32>,
    ) -> PathType {
        context.get_path_type_from_state(pos).await
    }

    fn set_can_pass_doors(&mut self, can_pass: bool) {
        self.base.can_pass_doors = can_pass;
    }

    fn set_can_open_doors(&mut self, can_open: bool) {
        self.base.can_open_doors = can_open;
    }

    fn set_can_float(&mut self, can_float: bool) {
        self.base.can_float = can_float;
    }

    fn set_can_walk_over_fences(&mut self, can_walk: bool) {
        self.base.can_walk_over_fences = can_walk;
    }

    fn can_pass_doors(&self) -> bool {
        self.base.can_pass_doors
    }

    fn can_open_doors(&self) -> bool {
        self.base.can_open_doors
    }

    fn can_float(&self) -> bool {
        self.base.can_float
    }

    fn can_walk_over_fences(&self) -> bool {
        self.base.can_walk_over_fences
    }
}

impl Default for WalkNodeEvaluator {
    fn default() -> Self {
        Self::new()
    }
}
