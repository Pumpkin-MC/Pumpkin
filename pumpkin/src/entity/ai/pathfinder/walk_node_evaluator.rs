use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
use std::collections::HashMap;

use crate::entity::ai::pathfinder::{
    node::{Coordinate, Node, PathType, Target},
    node_evaluator::{BaseNodeEvaluator, MobData, NodeEvaluator},
    pathfinding_context::PathfindingContext,
};

const DIRECTIONS: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
const DIAGONAL_DIRECTIONS: [(i32, i32); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];

const DEFAULT_MOB_JUMP_HEIGHT: f64 = 1.125;

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

    async fn get_floor_level(&mut self, pos: Vector3<i32>) -> f64 {
        if let Some(ref mut ctx) = self.base.context {
            let below = Vector3::new(pos.x, pos.y - 1, pos.z);
            let block_pos = below.as_blockpos();
            let state = ctx.world.get_block_state(&block_pos).await;
            let max_y = state
                .get_block_collision_shapes()
                .map(|bb| bb.max.y)
                .fold(0.0_f64, f64::max);
            if max_y > 0.0 {
                f64::from(pos.y - 1) + max_y
            } else {
                f64::from(pos.y - 1)
            }
        } else {
            f64::from(pos.y)
        }
    }

    fn get_mob_jump_height(&self) -> f64 {
        self.base
            .mob_data
            .as_ref()
            .map_or(DEFAULT_MOB_JUMP_HEIGHT, |d| {
                f64::from(d.max_step_height).max(DEFAULT_MOB_JUMP_HEIGHT)
            })
    }

    fn is_neighbor_valid(neighbor: Option<&Node>, current: &Node) -> bool {
        if let Some(neighbor) = neighbor {
            if neighbor.closed {
                return false;
            }
            neighbor.cost_malus >= 0.0 || current.cost_malus < 0.0
        } else {
            false
        }
    }

    fn is_diagonal_valid(
        &self,
        current: &Node,
        adj_x: Option<&Node>,
        adj_z: Option<&Node>,
    ) -> bool {
        let (Some(adj_x), Some(adj_z)) = (adj_x, adj_z) else {
            return false;
        };
        if adj_x.pos.0.y > current.pos.0.y || adj_z.pos.0.y > current.pos.0.y {
            return false;
        }
        if adj_z.path_type == PathType::WalkableDoor || adj_x.path_type == PathType::WalkableDoor {
            return false;
        }
        let mob_width = self.base.mob_data.as_ref().map_or(0.6, |d| d.width);
        let both_fence = adj_x.path_type == PathType::Fence && adj_z.path_type == PathType::Fence;
        let fence_exception = both_fence && mob_width < 0.5;

        (adj_x.pos.0.y < current.pos.0.y || adj_x.cost_malus >= 0.0 || fence_exception)
            && (adj_z.pos.0.y < current.pos.0.y || adj_z.cost_malus >= 0.0 || fence_exception)
    }

    fn is_diagonal_node_valid(diagonal: Option<&Node>) -> bool {
        diagonal.is_some_and(|n| {
            !n.closed && n.path_type != PathType::WalkableDoor && n.cost_malus >= 0.0
        })
    }

    async fn find_accepted_node(
        &mut self,
        pos: Vector3<i32>,
        max_y_step: i32,
        last_feet_y: f64,
        facing: (i32, i32),
        current_path_type: PathType,
    ) -> Option<Node> {
        let feet_y = self.get_floor_level(pos).await;
        if feet_y - last_feet_y > self.get_mob_jump_height() {
            return None;
        }

        let path_type = self.get_cached_path_type(pos).await;
        let penalty = self.get_mob_penalty(path_type);

        let mut node = (penalty >= 0.0).then(|| {
            let mut n = self.base.get_node(pos.as_blockpos());
            n.path_type = path_type;
            n.cost_malus = penalty.max(n.cost_malus);
            n
        });

        if path_type != PathType::Walkable
            && !(self.is_amphibious() && path_type == PathType::Water)
        {
            if (node.is_none() || node.as_ref().is_some_and(|n| n.cost_malus < 0.0))
                && max_y_step > 0
                && (path_type != PathType::Fence || self.base.can_walk_over_fences)
                && path_type != PathType::UnpassableRail
                && path_type != PathType::Trapdoor
                && path_type != PathType::PowderSnow
            {
                let jump_node = self
                    .get_jump_on_top_node(pos, max_y_step, last_feet_y, facing, current_path_type)
                    .await;
                if jump_node.is_some() {
                    node = jump_node;
                }
            } else if !self.is_amphibious() && path_type == PathType::Water && !self.base.can_float
            {
                node = self.get_non_water_node_below(pos, node).await;
            } else if path_type == PathType::Open {
                node = Some(self.get_open_node(pos).await);
            } else if Self::is_blocked_type(path_type) && node.is_none() {
                let mut n = self.base.get_node(pos.as_blockpos());
                n.closed = true;
                n.path_type = path_type;
                n.cost_malus = path_type.get_malus();
                node = Some(n);
            }
        }

        node
    }

    async fn get_jump_on_top_node(
        &mut self,
        pos: Vector3<i32>,
        max_y_step: i32,
        last_feet_y: f64,
        _facing: (i32, i32),
        _current_path_type: PathType,
    ) -> Option<Node> {
        for dy in 1..=max_y_step {
            let step_pos = Vector3::new(pos.x, pos.y + dy, pos.z);
            let remaining_steps = max_y_step - dy;

            let feet_y = self.get_floor_level(step_pos).await;
            if feet_y - last_feet_y > self.get_mob_jump_height() {
                return None;
            }

            let path_type = self.get_cached_path_type(step_pos).await;
            let penalty = self.get_mob_penalty(path_type);

            if penalty >= 0.0
                && (path_type == PathType::Walkable
                    || (self.is_amphibious() && path_type == PathType::Water))
            {
                let mut n = self.base.get_node(step_pos.as_blockpos());
                n.path_type = path_type;
                n.cost_malus = penalty.max(n.cost_malus);
                return Some(n);
            }

            if remaining_steps > 0
                && (path_type != PathType::Fence || self.base.can_walk_over_fences)
                && path_type != PathType::UnpassableRail
                && path_type != PathType::Trapdoor
                && path_type != PathType::PowderSnow
            {
                continue;
            }

            if path_type == PathType::Open {
                return Some(self.get_open_node(step_pos).await);
            }

            return None;
        }

        None
    }

    async fn get_open_node(&mut self, pos: Vector3<i32>) -> Node {
        let safe_fall_distance = self
            .base
            .mob_data
            .as_ref()
            .map_or(3, |d| d.max_fall_distance as i32);

        let mut check_y = pos.y - 1;
        let bottom_y = pos.y - safe_fall_distance - 2;

        while check_y >= bottom_y {
            let fall_dist = pos.y - check_y;
            if fall_dist > safe_fall_distance {
                let mut n = self.base.get_node(BlockPos::new(pos.x, check_y, pos.z));
                n.path_type = PathType::Blocked;
                n.cost_malus = -1.0;
                return n;
            }

            let path_type = self
                .get_cached_path_type(Vector3::new(pos.x, check_y, pos.z))
                .await;
            let penalty = self.get_mob_penalty(path_type);

            if path_type != PathType::Open {
                if penalty >= 0.0 {
                    let mut n = self.base.get_node(BlockPos::new(pos.x, check_y, pos.z));
                    n.path_type = path_type;
                    n.cost_malus = penalty.max(n.cost_malus);
                    return n;
                }
                let mut n = self.base.get_node(BlockPos::new(pos.x, check_y, pos.z));
                n.path_type = PathType::Blocked;
                n.cost_malus = -1.0;
                return n;
            }

            check_y -= 1;
        }

        let mut n = self.base.get_node(pos.as_blockpos());
        n.path_type = PathType::Blocked;
        n.cost_malus = -1.0;
        n
    }

    async fn get_non_water_node_below(
        &mut self,
        pos: Vector3<i32>,
        mut node: Option<Node>,
    ) -> Option<Node> {
        let mut y = pos.y - 1;
        while y > pos.y - 16 {
            let path_type = self
                .get_cached_path_type(Vector3::new(pos.x, y, pos.z))
                .await;
            if path_type != PathType::Water {
                return node;
            }
            let penalty = self.get_mob_penalty(path_type);
            let mut n = self.base.get_node(BlockPos::new(pos.x, y, pos.z));
            n.path_type = path_type;
            n.cost_malus = penalty.max(n.cost_malus);
            node = Some(n);
            y -= 1;
        }
        node
    }

    fn get_mob_penalty(&self, path_type: PathType) -> f32 {
        self.base
            .mob_data
            .as_ref()
            .map_or(path_type.get_malus(), |d| {
                d.get_pathfinding_malus(path_type)
            })
    }

    const fn is_blocked_type(path_type: PathType) -> bool {
        matches!(
            path_type,
            PathType::Fence | PathType::DoorWoodClosed | PathType::DoorIronClosed
        )
    }

    async fn get_cached_path_type(&mut self, pos: Vector3<i32>) -> PathType {
        if let Some(&cached) = self.path_types_cache.get(&pos) {
            return cached;
        }

        let path_type = if let Some(mut ctx) = self.base.context.take()
            && let Some(mob_clone) = self.base.mob_data.clone()
        {
            let res = self.get_path_type_of_mob(&mut ctx, pos, &mob_clone).await;
            self.base.context = Some(ctx);
            res
        } else {
            PathType::Blocked
        };

        self.path_types_cache.insert(pos, path_type);
        path_type
    }

    async fn has_collisions(&mut self, center: Vector3<i32>) -> bool {
        if let Some(&cached) = self.collision_cache.get(&center) {
            return cached;
        }

        let has_collision = if let Some(ref mut ctx) = self.base.context {
            ctx.has_collisions(center).await
        } else {
            false
        };

        self.collision_cache.insert(center, has_collision);
        has_collision
    }

    async fn can_start_at(&mut self, pos: Vector3<i32>) -> bool {
        let path_type = self.get_cached_path_type(pos).await;
        path_type.is_passable() && !self.has_collisions(pos).await
    }

    async fn get_start_node(&mut self, pos: Vector3<i32>) -> Option<Node> {
        if !self.can_start_at(pos).await {
            return None;
        }

        let mut node = self.base.get_node(pos.as_blockpos());
        let path_type = self.get_cached_path_type(pos).await;
        node.path_type = path_type;
        node.cost_malus = self.get_mob_penalty(path_type);

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
        let mob_data = self.base.mob_data.as_ref()?;
        let mob_x = mob_data.position.x;
        let mob_y_f64 = mob_data.position.y;
        let mob_z = mob_data.position.z;
        let on_ground = mob_data.on_ground;

        let y = if on_ground {
            (mob_y_f64 + 0.5).floor() as i32
        } else {
            let start_y = (mob_y_f64 + 1.0).floor() as i32;
            let bottom_y = start_y - 64;
            let mut found_y = start_y;
            for check_y in (bottom_y..start_y).rev() {
                let path_type = self
                    .get_cached_path_type(Vector3::new(
                        mob_x.floor() as i32,
                        check_y,
                        mob_z.floor() as i32,
                    ))
                    .await;
                if path_type != PathType::Open && path_type != PathType::Water {
                    found_y = check_y + 1;
                    break;
                }
            }
            found_y
        };

        let block_x = mob_x.floor() as i32;
        let block_z = mob_z.floor() as i32;
        let start_pos = Vector3::new(block_x, y, block_z);

        if let Some(node) = self.get_start_node(start_pos).await {
            return Some(node);
        }

        for &(dx, dz) in &DIRECTIONS {
            let try_pos = Vector3::new(block_x + dx, y, block_z + dz);
            if let Some(node) = self.get_start_node(try_pos).await {
                return Some(node);
            }
        }

        let above_pos = Vector3::new(block_x, y + 1, block_z);
        self.get_start_node(above_pos).await
    }

    fn get_target(&mut self, pos: BlockPos) -> Target {
        let node = self.base.get_node(pos);
        Target::new(node)
    }

    async fn get_neighbors(&mut self, current: &Node) -> Vec<Node> {
        let mut out_neighbors: Vec<Node> = Vec::new();

        let headroom_type = self
            .get_cached_path_type(current.pos.0.add_raw(0, 1, 0))
            .await;
        let current_type = self.get_cached_path_type(current.pos.0).await;

        let headroom_penalty = self.get_mob_penalty(headroom_type);
        let max_y_step = if headroom_penalty >= 0.0 && current_type != PathType::StickyHoney {
            self.get_mob_jump_height().floor() as i32
        } else {
            0
        };

        let floor_level = self.get_floor_level(current.pos.0).await;

        for i in 0..4 {
            self.reusable_neighbors[i] = None;
        }

        for (i, &(dx, dz)) in DIRECTIONS.iter().enumerate() {
            let neighbor_pos = current.pos.0.add_raw(dx, 0, dz);

            let neighbor_opt = self
                .find_accepted_node(
                    neighbor_pos,
                    max_y_step,
                    floor_level,
                    (dx, dz),
                    current.path_type,
                )
                .await;

            if let Some(neighbor) = neighbor_opt {
                self.reusable_neighbors[i] = Some(neighbor.clone());
                if Self::is_neighbor_valid(Some(&neighbor), current) {
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

                let diagonal_opt = self
                    .find_accepted_node(
                        diagonal_pos,
                        max_y_step,
                        floor_level,
                        (dx, dz),
                        current.path_type,
                    )
                    .await;

                if let Some(diagonal) = diagonal_opt
                    && Self::is_diagonal_node_valid(Some(&diagonal))
                {
                    out_neighbors.push(diagonal);
                }
            }
        }

        out_neighbors
    }

    async fn get_path_type_of_mob(
        &mut self,
        context: &mut PathfindingContext,
        pos: Vector3<i32>,
        mob_data: &MobData,
    ) -> PathType {
        let mut path_types = Vec::new();
        let mob_block_pos = mob_data.block_position();

        for dy in 0..mob_data.get_bb_height() {
            for dx in 0..mob_data.get_bb_width() {
                for dz in 0..mob_data.get_bb_width() {
                    let check_pos = pos.add_raw(dx, dy, dz);
                    let mut cell_type = context.get_land_node_type(check_pos).await;

                    if cell_type == PathType::DoorWoodClosed
                        && self.base.can_open_doors
                        && self.base.can_pass_doors
                    {
                        cell_type = PathType::WalkableDoor;
                    }

                    if cell_type == PathType::DoorOpen && !self.base.can_pass_doors {
                        cell_type = PathType::Blocked;
                    }

                    if cell_type == PathType::Rail {
                        let mob_pos =
                            Vector3::new(mob_block_pos.0, mob_block_pos.1, mob_block_pos.2);
                        let mob_below =
                            Vector3::new(mob_block_pos.0, mob_block_pos.1 - 1, mob_block_pos.2);
                        let mob_type = context.get_land_node_type(mob_pos).await;
                        let mob_below_type = context.get_land_node_type(mob_below).await;
                        if mob_type != PathType::Rail && mob_below_type != PathType::Rail {
                            cell_type = PathType::UnpassableRail;
                        }
                    }

                    path_types.push(cell_type);
                }
            }
        }

        path_types.sort();
        path_types.dedup();

        if path_types.contains(&PathType::Fence) {
            return PathType::Fence;
        }
        if path_types.contains(&PathType::UnpassableRail) {
            return PathType::UnpassableRail;
        }

        let mut result = PathType::Blocked;
        for &path_type in &path_types {
            let penalty = mob_data.get_pathfinding_malus(path_type);
            if penalty < 0.0 {
                return path_type;
            }

            let result_penalty = mob_data.get_pathfinding_malus(result);
            if penalty >= result_penalty {
                result = path_type;
            }
        }

        if self.base.entity_width <= 1
            && result != PathType::Open
            && mob_data.get_pathfinding_malus(result) == 0.0
        {
            let raw_center = context.get_land_node_type(pos).await;
            if raw_center == PathType::Open {
                return PathType::Open;
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
