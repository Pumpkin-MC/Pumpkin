use pumpkin_data::BlockDirection;
use pumpkin_util::math::{boundingbox::BoundingBox, position::BlockPos, vector3::Vector3};
use rustc_hash::FxHashMap;

use crate::entity::ai::pathfinder::{
    node::{Coordinate, Node, PathType, Target},
    node_evaluator::{BaseNodeEvaluator, MobData, NodeEvaluator},
    pathfinding_context::PathfindingContext,
};

pub struct WalkNodeEvaluator {
    pub base: BaseNodeEvaluator,
    path_types_cache: FxHashMap<Vector3<i32>, PathType>,
    reusable_neighbors: [Option<Node>; 4],
    pub is_amphibious: bool,
}

impl WalkNodeEvaluator {
    #[must_use]
    pub fn new() -> Self {
        Self {
            base: BaseNodeEvaluator::new(),
            path_types_cache: FxHashMap::default(),
            reusable_neighbors: [None, None, None, None],
            is_amphibious: false,
        }
    }

    #[must_use]
    pub fn get_floor_level(&self, pos: &BlockPos) -> f64 {
        self.base.context.as_ref().map_or_else(
            || f64::from(pos.0.y),
            |ctx| {
                if (self.base.can_float || self.is_amphibious) && ctx.is_water(pos) {
                    f64::from(pos.0.y) + 0.5
                } else {
                    ctx.get_floor_level(pos)
                }
            },
        )
    }

    #[must_use]
    pub fn get_mob_jump_height(&self) -> f64 {
        self.base
            .mob_data
            .as_ref()
            .map_or(1.125, |d| f64::from(d.max_step_height).max(1.125))
    }

    #[must_use]
    pub fn is_neighbor_valid(neighbor: Option<&Node>, current: &Node) -> bool {
        neighbor.is_some_and(|n| !n.closed && (n.cost_malus >= 0.0 || current.cost_malus < 0.0))
    }

    #[must_use]
    pub fn is_diagonal_valid(&self, current: &Node, ew: Option<&Node>, ns: Option<&Node>) -> bool {
        let (Some(ew), Some(ns)) = (ew, ns) else {
            return false;
        };
        if ns.pos.0.y > current.pos.0.y || ew.pos.0.y > current.pos.0.y {
            return false;
        }
        if ew.path_type != PathType::WalkableDoor && ns.path_type != PathType::WalkableDoor {
            let mob_width = self.base.mob_data.as_ref().map_or(0.6, |d| d.width);
            if mob_width <= 1.0 || (ew.cost_malus <= 0.0 && ns.cost_malus <= 0.0) {
                let can_pass_between_posts = ns.path_type == PathType::Fence
                    && ew.path_type == PathType::Fence
                    && mob_width < 0.5;
                (ns.pos.0.y < current.pos.0.y || ns.cost_malus >= 0.0 || can_pass_between_posts)
                    && (ew.pos.0.y < current.pos.0.y
                        || ew.cost_malus >= 0.0
                        || can_pass_between_posts)
            } else {
                false
            }
        } else {
            false
        }
    }

    #[must_use]
    pub fn is_diagonal_node_valid(diagonal: Option<&Node>) -> bool {
        diagonal.is_some_and(|n| {
            !n.closed && n.path_type != PathType::WalkableDoor && n.cost_malus >= 0.0
        })
    }

    #[must_use]
    pub const fn does_block_have_partial_collision(path_type: PathType) -> bool {
        matches!(
            path_type,
            PathType::Fence | PathType::DoorWoodClosed | PathType::DoorIronClosed
        )
    }

    pub fn can_reach_without_collision(&mut self, pos_to: Vector3<i32>) -> bool {
        let Some(mob_data) = self.base.mob_data else {
            return true;
        };
        let mut bb = BoundingBox::new_from_pos(
            mob_data.position.x,
            mob_data.position.y,
            mob_data.position.z,
            &pumpkin_util::math::boundingbox::EntityDimensions::new(
                mob_data.width,
                mob_data.height,
                0.0,
            ),
        );
        let x_size = bb.max.x - bb.min.x;
        let y_size = bb.max.y - bb.min.y;
        let z_size = bb.max.z - bb.min.z;
        let avg_size = bb.get_average_side_length();
        let mut delta = Vector3::new(
            f64::from(pos_to.x) - mob_data.position.x + x_size / 2.0,
            f64::from(pos_to.y) - mob_data.position.y + y_size / 2.0,
            f64::from(pos_to.z) - mob_data.position.z + z_size / 2.0,
        );
        let steps = (delta.length() / avg_size).ceil() as i32;
        if steps > 0 {
            delta = delta * (1.0 / f64::from(steps));
            for _ in 1..=steps {
                bb = bb.shift(delta);
                if self.has_collision(&bb) {
                    return false;
                }
            }
        }
        true
    }

    #[expect(clippy::too_many_arguments)]
    pub fn find_accepted_node(
        &mut self,
        x: i32,
        y: i32,
        z: i32,
        jump_size: i32,
        node_height: f64,
        travel_direction: BlockDirection,
        block_path_type_current: PathType,
    ) -> Option<Node> {
        let max_y_target = self.get_floor_level(&BlockPos::new(x, y, z));
        if max_y_target - node_height > self.get_mob_jump_height() {
            return None;
        }

        let path_type = self.get_cached_path_type(Vector3::new(x, y, z));
        let path_cost = self.get_mob_penalty(path_type);
        let mut best = (path_cost >= 0.0)
            .then(|| self.get_node_and_update_cost_to_max(x, y, z, path_type, path_cost));

        if Self::does_block_have_partial_collision(block_path_type_current)
            && best.as_ref().is_some_and(|b| b.cost_malus >= 0.0)
            && !self.can_reach_without_collision(Vector3::new(x, y, z))
        {
            best = None;
        }

        if path_type != PathType::Walkable && (!self.is_amphibious || path_type != PathType::Water)
        {
            if (best.is_none() || best.as_ref().is_some_and(|b| b.cost_malus < 0.0))
                && jump_size > 0
                && (path_type != PathType::Fence || self.base.can_walk_over_fences)
                && path_type != PathType::UnpassableRail
                && path_type != PathType::Trapdoor
                && path_type != PathType::PowderSnow
            {
                best = self.try_jump_on(
                    x,
                    y,
                    z,
                    jump_size,
                    node_height,
                    travel_direction,
                    block_path_type_current,
                );
            } else if !self.is_amphibious && path_type == PathType::Water && !self.base.can_float {
                best = self.try_find_first_non_water_below(x, y, z, best);
            } else if path_type == PathType::Open {
                best = Some(self.try_find_first_ground_node_below(x, y, z));
            } else if Self::does_block_have_partial_collision(path_type) && best.is_none() {
                best = Some(self.get_closed_node(x, y, z, path_type));
            }
        }

        best
    }

    #[expect(clippy::too_many_arguments)]
    fn try_jump_on(
        &mut self,
        x: i32,
        y: i32,
        z: i32,
        jump_size: i32,
        node_height: f64,
        travel_direction: BlockDirection,
        block_path_type_current: PathType,
    ) -> Option<Node> {
        let node_above = self.find_accepted_node(
            x,
            y + 1,
            z,
            jump_size - 1,
            node_height,
            travel_direction,
            block_path_type_current,
        )?;
        let mob_data = self.base.mob_data?;
        if mob_data.width >= 1.0 {
            return Some(node_above);
        }
        if node_above.path_type != PathType::Open && node_above.path_type != PathType::Walkable {
            return Some(node_above);
        }

        let center_x = f64::from(x) - f64::from(travel_direction.to_offset().x) + 0.5;
        let center_z = f64::from(z) - f64::from(travel_direction.to_offset().z) + 0.5;
        let half_width = f64::from(mob_data.width) / 2.0;
        let floor_below = self.get_floor_level(&BlockPos::new(
            center_x.floor() as i32,
            y + 1,
            center_z.floor() as i32,
        ));
        let floor_above = self.get_floor_level(&node_above.pos);
        let grow = BoundingBox::new(
            Vector3::new(
                center_x - half_width,
                floor_below + 0.001,
                center_z - half_width,
            ),
            Vector3::new(
                center_x + half_width,
                f64::from(mob_data.height) + floor_above - 0.002,
                center_z + half_width,
            ),
        );
        if self.has_collision(&grow) {
            None
        } else {
            Some(node_above)
        }
    }

    fn try_find_first_non_water_below(
        &mut self,
        x: i32,
        mut y: i32,
        z: i32,
        mut best: Option<Node>,
    ) -> Option<Node> {
        let min_y = self.base.mob_data.map_or(-64, |d| d.min_y);
        y -= 1;
        while y > min_y {
            let path_type_local = self.get_cached_path_type(Vector3::new(x, y, z));
            if path_type_local != PathType::Water {
                return best;
            }
            let penalty = self.get_mob_penalty(path_type_local);
            best = Some(self.get_node_and_update_cost_to_max(x, y, z, path_type_local, penalty));
            y -= 1;
        }
        best
    }

    fn try_find_first_ground_node_below(&mut self, x: i32, y: i32, z: i32) -> Node {
        let max_fall_distance = self
            .base
            .mob_data
            .map_or(3, |d| d.max_fall_distance.floor() as i32);
        let min_y = self.base.mob_data.map_or(-64, |d| d.min_y);

        for current_y in (min_y..y).rev() {
            if y - current_y > max_fall_distance {
                return self.get_blocked_node(x, current_y, z);
            }
            let path_type = self.get_cached_path_type(Vector3::new(x, current_y, z));
            let path_cost = self.get_mob_penalty(path_type);
            if path_type != PathType::Open {
                if path_cost >= 0.0 {
                    return self
                        .get_node_and_update_cost_to_max(x, current_y, z, path_type, path_cost);
                }
                return self.get_blocked_node(x, current_y, z);
            }
        }
        self.get_blocked_node(x, y, z)
    }

    fn get_node_and_update_cost_to_max(
        &mut self,
        x: i32,
        y: i32,
        z: i32,
        path_type: PathType,
        cost: f32,
    ) -> Node {
        let mut node = self.base.get_node(BlockPos::new(x, y, z));
        node.path_type = path_type;
        node.cost_malus = node.cost_malus.max(cost);
        node
    }

    fn get_blocked_node(&mut self, x: i32, y: i32, z: i32) -> Node {
        let mut node = self.base.get_node(BlockPos::new(x, y, z));
        node.path_type = PathType::Blocked;
        node.cost_malus = -1.0;
        node
    }

    fn get_closed_node(&mut self, x: i32, y: i32, z: i32, path_type: PathType) -> Node {
        let mut node = self.base.get_node(BlockPos::new(x, y, z));
        node.closed = true;
        node.path_type = path_type;
        node.cost_malus = path_type.get_malus();
        node
    }

    #[must_use]
    pub fn get_mob_penalty(&self, path_type: PathType) -> f32 {
        self.base
            .mob_data
            .as_ref()
            .map_or(path_type.get_malus(), |d| {
                d.get_pathfinding_malus(path_type)
            })
    }

    pub fn get_cached_path_type(&mut self, pos: Vector3<i32>) -> PathType {
        if let Some(&cached) = self.path_types_cache.get(&pos) {
            return cached;
        }

        let path_type = if let Some(mut ctx) = self.base.context.take()
            && let Some(mob_data) = self.base.mob_data
        {
            let res = self.get_path_type_of_mob(&mut ctx, pos, &mob_data);
            self.base.context = Some(ctx);
            res
        } else {
            PathType::Blocked
        };

        self.path_types_cache.insert(pos, path_type);
        path_type
    }

    pub fn has_collision(&mut self, bb: &BoundingBox) -> bool {
        self.base
            .context
            .as_mut()
            .is_some_and(|ctx| ctx.has_collision(bb))
    }

    pub fn can_start_at(&mut self, pos: Vector3<i32>) -> bool {
        let path_type = self.get_cached_path_type(pos);
        path_type != PathType::Open && self.get_mob_penalty(path_type) >= 0.0
    }

    pub fn get_start_node(&mut self, pos: Vector3<i32>) -> Node {
        let path_type = self.get_cached_path_type(pos);
        let mut node = self.base.get_node(pos.as_blockpos());
        node.path_type = path_type;
        node.cost_malus = self.get_mob_penalty(path_type);
        node
    }

    pub fn get_path_type_within_mob_bb(
        &mut self,
        context: &mut PathfindingContext,
        x: i32,
        y: i32,
        z: i32,
        mob_data: &MobData,
    ) -> Vec<PathType> {
        let mut block_types = Vec::new();
        let mob_block_pos = mob_data.block_position();

        for dx in 0..self.base.entity_width {
            for dy in 0..self.base.entity_height {
                for dz in 0..self.base.entity_depth {
                    let check_pos = Vector3::new(x + dx, y + dy, z + dz);
                    let mut block_type = self.get_path_type(context, check_pos);

                    if block_type == PathType::DoorWoodClosed
                        && self.base.can_open_doors
                        && self.base.can_pass_doors
                    {
                        block_type = PathType::WalkableDoor;
                    }

                    if block_type == PathType::DoorOpen && !self.base.can_pass_doors {
                        block_type = PathType::Blocked;
                    }

                    if block_type == PathType::Rail {
                        let mob_pos =
                            Vector3::new(mob_block_pos.0, mob_block_pos.1, mob_block_pos.2);
                        let mob_below =
                            Vector3::new(mob_block_pos.0, mob_block_pos.1 - 1, mob_block_pos.2);
                        let mob_type = self.get_path_type(context, mob_pos);
                        let mob_below_type = self.get_path_type(context, mob_below);
                        if mob_type != PathType::Rail && mob_below_type != PathType::Rail {
                            block_type = PathType::UnpassableRail;
                        }
                    }

                    block_types.push(block_type);
                }
            }
        }

        block_types.sort();
        block_types.dedup();
        block_types
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
        self.reusable_neighbors = [None, None, None, None];
    }

    fn done(&mut self) {
        self.base.context = None;
        self.base.mob_data = None;
        self.path_types_cache.clear();
        self.reusable_neighbors = [None, None, None, None];
    }

    fn get_start(&mut self) -> Option<Node> {
        let mob_data = self.base.mob_data?;
        let mob_x = mob_data.position.x;
        let mob_y = mob_data.position.y;
        let mob_z = mob_data.position.z;
        let block_x = mob_x.floor() as i32;
        let block_y = mob_y.floor() as i32;
        let block_z = mob_z.floor() as i32;

        let mut start_y = block_y;

        if !mob_data.can_walk_on_water {
            if self.base.can_float && mob_data.is_in_water {
                let mut check_y = block_y;
                if let Some(ref ctx) = self.base.context {
                    while ctx.is_water(&BlockPos::new(block_x, check_y + 1, block_z)) {
                        check_y += 1;
                    }
                }
                start_y = check_y;
            } else if mob_data.on_ground {
                start_y = (mob_y + 0.5).floor() as i32;
            } else {
                let start_check_y = (mob_y + 1.0).floor() as i32;
                start_y = start_check_y;
                let min_y = mob_data.min_y;
                let mut check_y = start_check_y;
                while check_y > min_y {
                    start_y = check_y;
                    check_y -= 1;
                    let below = BlockPos::new(block_x, check_y, block_z);
                    if let Some(ref ctx) = self.base.context
                        && !ctx.is_air(&below)
                        && !ctx.is_pathfindable(
                            &below,
                            crate::entity::ai::pathfinder::node::PathComputationType::Land,
                        )
                    {
                        break;
                    }
                }
            }
        }

        let start_pos = Vector3::new(block_x, start_y, block_z);
        if !self.can_start_at(start_pos) {
            let half_width = f64::from(mob_data.width) / 2.0;
            let min_x = (mob_x - half_width).floor() as i32;
            let max_x = (mob_x + half_width).floor() as i32;
            let min_z = (mob_z - half_width).floor() as i32;
            let max_z = (mob_z + half_width).floor() as i32;

            for candidate in [
                Vector3::new(min_x, start_y, min_z),
                Vector3::new(min_x, start_y, max_z),
                Vector3::new(max_x, start_y, min_z),
                Vector3::new(max_x, start_y, max_z),
            ] {
                if self.can_start_at(candidate) {
                    return Some(self.get_start_node(candidate));
                }
            }
        }

        Some(self.get_start_node(start_pos))
    }

    fn get_target(&mut self, pos: BlockPos) -> Target {
        Target::new(self.base.get_node(pos))
    }

    fn get_neighbors(&mut self, current: &Node, out: &mut Vec<Node>) {
        let block_path_type_above = self.get_cached_path_type(current.pos.0.add_raw(0, 1, 0));
        let block_path_type_current = self.get_cached_path_type(current.pos.0);

        let jump_size = if self.get_mob_penalty(block_path_type_above) >= 0.0
            && block_path_type_current != PathType::StickyHoney
        {
            self.get_mob_jump_height().floor() as i32
        } else {
            0
        };

        let pos_height = self.get_floor_level(&current.pos);

        self.reusable_neighbors.fill(None);

        let horizontal_directions = [
            (2usize, 0, -1, BlockDirection::North),
            (0usize, 0, 1, BlockDirection::South),
            (1usize, -1, 0, BlockDirection::West),
            (3usize, 1, 0, BlockDirection::East),
        ];

        for &(dir_idx, dx, dz, block_dir) in &horizontal_directions {
            let node = self.find_accepted_node(
                current.pos.0.x + dx,
                current.pos.0.y,
                current.pos.0.z + dz,
                jump_size,
                pos_height,
                block_dir,
                block_path_type_current,
            );
            self.reusable_neighbors[dir_idx] = node;
            if Self::is_neighbor_valid(node.as_ref(), current)
                && let Some(n) = node
            {
                out.push(n);
            }
        }

        let diagonal_directions = [
            (2usize, 3usize, 1, -1, BlockDirection::North),
            (0usize, 1usize, -1, 1, BlockDirection::South),
            (1usize, 2usize, -1, -1, BlockDirection::West),
            (3usize, 0usize, 1, 1, BlockDirection::East),
        ];

        for &(dir_idx, cw_idx, dx, dz, block_dir) in &diagonal_directions {
            if self.is_diagonal_valid(
                current,
                self.reusable_neighbors[dir_idx].as_ref(),
                self.reusable_neighbors[cw_idx].as_ref(),
            ) {
                let diagonal_node = self.find_accepted_node(
                    current.pos.0.x + dx,
                    current.pos.0.y,
                    current.pos.0.z + dz,
                    jump_size,
                    pos_height,
                    block_dir,
                    block_path_type_current,
                );
                if Self::is_diagonal_node_valid(diagonal_node.as_ref())
                    && let Some(d) = diagonal_node
                {
                    out.push(d);
                }
            }
        }
    }

    fn get_path_type_of_mob(
        &mut self,
        context: &mut PathfindingContext,
        pos: Vector3<i32>,
        mob_data: &MobData,
    ) -> PathType {
        let block_types = self.get_path_type_within_mob_bb(context, pos.x, pos.y, pos.z, mob_data);
        if block_types.len() == 1 {
            return block_types[0];
        }

        if block_types.contains(&PathType::Fence) {
            return PathType::Fence;
        }

        if block_types.contains(&PathType::UnpassableRail) {
            return PathType::UnpassableRail;
        }

        let mut highest_malus_path_type_within_bb = PathType::Blocked;
        let mut highest_malus_within_bb =
            mob_data.get_pathfinding_malus(highest_malus_path_type_within_bb);

        for &path_type in &block_types {
            let malus = mob_data.get_pathfinding_malus(path_type);
            if malus < 0.0 {
                return path_type;
            }

            if malus >= highest_malus_within_bb {
                highest_malus_within_bb = malus;
                highest_malus_path_type_within_bb = path_type;
            }
        }

        let current_node_path_type = self.get_path_type(context, pos);
        let is_large_mob = self.base.entity_width > 1;
        if is_large_mob {
            let is_current_node_cheaper =
                mob_data.get_pathfinding_malus(current_node_path_type) < highest_malus_within_bb;
            let cap_malus_due_to_cheap_node = is_current_node_cheaper
                && mob_data.get_pathfinding_malus(PathType::BigMobsCloseToDanger)
                    < highest_malus_within_bb;
            if cap_malus_due_to_cheap_node {
                PathType::BigMobsCloseToDanger
            } else {
                highest_malus_path_type_within_bb
            }
        } else if current_node_path_type == PathType::Open
            && highest_malus_path_type_within_bb != PathType::Open
            && highest_malus_within_bb == 0.0
        {
            PathType::Open
        } else {
            highest_malus_path_type_within_bb
        }
    }

    fn get_path_type(&mut self, context: &mut PathfindingContext, pos: Vector3<i32>) -> PathType {
        context.get_land_node_type(pos)
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
