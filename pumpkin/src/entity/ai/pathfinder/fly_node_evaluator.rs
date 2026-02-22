use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
use rand::RngExt;
use std::collections::HashMap;

use crate::entity::ai::pathfinder::{
    node::{Coordinate, Node, PathType, Target},
    node_evaluator::{BaseNodeEvaluator, MobData, NodeEvaluator},
    pathfinding_context::PathfindingContext,
};

const SMALL_MOB_SIZE: f64 = 1.0;
const SMALL_MOB_INFLATED_START_NODE_BOUNDING_BOX: f64 = 1.1;
const MAX_START_NODE_CANDIDATES: usize = 10;

pub struct FlyNodeEvaluator {
    base: BaseNodeEvaluator,
    path_types_cache: HashMap<Vector3<i32>, PathType>,
}

impl FlyNodeEvaluator {
    #[must_use]
    pub fn new() -> Self {
        Self {
            base: BaseNodeEvaluator::new(),
            path_types_cache: HashMap::new(),
        }
    }

    fn get_mob_penalty(&self, path_type: PathType) -> f32 {
        self.base
            .mob_data
            .as_ref()
            .map_or(path_type.get_malus(), |d| {
                d.get_pathfinding_malus(path_type)
            })
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

    async fn can_start_at(&mut self, pos: Vector3<i32>) -> bool {
        let path_type = self.get_cached_path_type(pos).await;
        self.get_mob_penalty(path_type) >= 0.0
    }

    async fn create_start_node(&mut self, pos: Vector3<i32>) -> Node {
        let mut node = self.base.get_node(pos.as_blockpos());
        let path_type = self.get_cached_path_type(pos).await;
        node.path_type = path_type;
        node.cost_malus = self.get_mob_penalty(path_type);
        node
    }

    async fn find_accepted_node(&mut self, pos: Vector3<i32>) -> Option<Node> {
        let path_type = self.get_cached_path_type(pos).await;
        let path_cost = self.get_mob_penalty(path_type);
        if path_cost < 0.0 {
            return None;
        }

        let mut node = self.base.get_node(pos.as_blockpos());
        node.path_type = path_type;
        node.cost_malus = path_cost.max(node.cost_malus);
        if path_type == PathType::Walkable {
            node.cost_malus += 1.0;
        }
        Some(node)
    }

    fn has_malus(node: Option<&Node>) -> bool {
        node.is_some_and(|n| n.cost_malus >= 0.0)
    }

    fn is_open(node: Option<&Node>) -> bool {
        node.is_some_and(|n| !n.closed)
    }

    fn push_if_open(out: &mut Vec<Node>, node: &Option<Node>) {
        if let Some(node) = node.as_ref()
            && !node.closed
        {
            out.push(node.clone());
        }
    }

    fn iterate_start_node_candidate_positions(mob_data: &MobData) -> Vec<Vector3<i32>> {
        let mut candidates = Vec::with_capacity(MAX_START_NODE_CANDIDATES);
        let half_width = f64::from(mob_data.width) * 0.5;
        let min_x = mob_data.position.x - half_width;
        let max_x = mob_data.position.x + half_width;
        let min_y = mob_data.position.y;
        let max_y = mob_data.position.y + f64::from(mob_data.height);
        let min_z = mob_data.position.z - half_width;
        let max_z = mob_data.position.z + half_width;

        let x_size = f64::from(mob_data.width);
        let y_size = f64::from(mob_data.height);
        let z_size = f64::from(mob_data.width);
        let avg_size = (x_size + y_size + z_size) / 3.0;

        if avg_size >= SMALL_MOB_SIZE {
            let block_y = mob_data.position.y.floor() as i32;
            candidates.push(Vector3::new(
                min_x.floor() as i32,
                block_y,
                min_z.floor() as i32,
            ));
            candidates.push(Vector3::new(
                min_x.floor() as i32,
                block_y,
                max_z.floor() as i32,
            ));
            candidates.push(Vector3::new(
                max_x.floor() as i32,
                block_y,
                min_z.floor() as i32,
            ));
            candidates.push(Vector3::new(
                max_x.floor() as i32,
                block_y,
                max_z.floor() as i32,
            ));
            return candidates;
        }

        let z_padding = (SMALL_MOB_INFLATED_START_NODE_BOUNDING_BOX - (max_z - min_z)).max(0.0);
        let x_padding = (SMALL_MOB_INFLATED_START_NODE_BOUNDING_BOX - (max_x - min_x)).max(0.0);
        let y_padding = (SMALL_MOB_INFLATED_START_NODE_BOUNDING_BOX - (max_y - min_y)).max(0.0);

        let min_x_i = (min_x - x_padding).floor() as i32;
        let min_y_i = (min_y - y_padding).floor() as i32;
        let min_z_i = (min_z - z_padding).floor() as i32;
        let max_x_i = (max_x + x_padding).floor() as i32;
        let max_y_i = (max_y + y_padding).floor() as i32;
        let max_z_i = (max_z + z_padding).floor() as i32;

        let low_x = min_x_i.min(max_x_i);
        let high_x = min_x_i.max(max_x_i);
        let low_y = min_y_i.min(max_y_i);
        let high_y = min_y_i.max(max_y_i);
        let low_z = min_z_i.min(max_z_i);
        let high_z = min_z_i.max(max_z_i);

        let mut rng = rand::rng();
        for _ in 0..MAX_START_NODE_CANDIDATES {
            candidates.push(Vector3::new(
                rng.random_range(low_x..=high_x),
                rng.random_range(low_y..=high_y),
                rng.random_range(low_z..=high_z),
            ));
        }

        candidates
    }
}

impl NodeEvaluator for FlyNodeEvaluator {
    fn prepare(&mut self, context: PathfindingContext, mob_data: MobData) {
        self.base.entity_width = mob_data.get_bb_width();
        self.base.entity_height = mob_data.get_bb_height();
        self.base.entity_depth = mob_data.get_bb_width();

        self.base.context = Some(context);
        self.base.mob_data = Some(mob_data);
        self.base.nodes.clear();
        self.path_types_cache.clear();
    }

    fn done(&mut self) {
        self.base.context = None;
        self.base.mob_data = None;
        self.base.nodes.clear();
        self.path_types_cache.clear();
    }

    async fn get_start(&mut self) -> Option<Node> {
        let mob_data = self.base.mob_data.clone()?;
        let mut start_y = (mob_data.position.y + 0.5).floor() as i32;
        let block_x = mob_data.position.x.floor() as i32;
        let block_z = mob_data.position.z.floor() as i32;

        if self.can_float() && mob_data.can_swim {
            let mut steps = 0;
            while steps < 16 {
                let path_type = self
                    .get_cached_path_type(Vector3::new(block_x, start_y, block_z))
                    .await;
                if path_type != PathType::Water {
                    break;
                }
                start_y += 1;
                steps += 1;
            }
        }

        let start_pos = Vector3::new(block_x, start_y, block_z);
        if self.can_start_at(start_pos).await {
            return Some(self.create_start_node(start_pos).await);
        }

        for candidate in Self::iterate_start_node_candidate_positions(&mob_data) {
            if self.can_start_at(candidate).await {
                return Some(self.create_start_node(candidate).await);
            }
        }

        Some(self.create_start_node(start_pos).await)
    }

    fn get_target(&mut self, pos: BlockPos) -> Target {
        let node = self.base.get_node(pos);
        Target::new(node)
    }

    #[allow(clippy::too_many_lines)]
    async fn get_neighbors(&mut self, current: &Node) -> Vec<Node> {
        let mut neighbors = Vec::with_capacity(26);
        let pos = current.pos.0;

        let south = self.find_accepted_node(pos.add_raw(0, 0, 1)).await;
        Self::push_if_open(&mut neighbors, &south);

        let west = self.find_accepted_node(pos.add_raw(-1, 0, 0)).await;
        Self::push_if_open(&mut neighbors, &west);

        let east = self.find_accepted_node(pos.add_raw(1, 0, 0)).await;
        Self::push_if_open(&mut neighbors, &east);

        let north = self.find_accepted_node(pos.add_raw(0, 0, -1)).await;
        Self::push_if_open(&mut neighbors, &north);

        let up = self.find_accepted_node(pos.add_raw(0, 1, 0)).await;
        Self::push_if_open(&mut neighbors, &up);

        let down = self.find_accepted_node(pos.add_raw(0, -1, 0)).await;
        Self::push_if_open(&mut neighbors, &down);

        let south_up = self.find_accepted_node(pos.add_raw(0, 1, 1)).await;
        if Self::is_open(south_up.as_ref())
            && Self::has_malus(south.as_ref())
            && Self::has_malus(up.as_ref())
        {
            Self::push_if_open(&mut neighbors, &south_up);
        }

        let west_up = self.find_accepted_node(pos.add_raw(-1, 1, 0)).await;
        if Self::is_open(west_up.as_ref())
            && Self::has_malus(west.as_ref())
            && Self::has_malus(up.as_ref())
        {
            Self::push_if_open(&mut neighbors, &west_up);
        }

        let east_up = self.find_accepted_node(pos.add_raw(1, 1, 0)).await;
        if Self::is_open(east_up.as_ref())
            && Self::has_malus(east.as_ref())
            && Self::has_malus(up.as_ref())
        {
            Self::push_if_open(&mut neighbors, &east_up);
        }

        let north_up = self.find_accepted_node(pos.add_raw(0, 1, -1)).await;
        if Self::is_open(north_up.as_ref())
            && Self::has_malus(north.as_ref())
            && Self::has_malus(up.as_ref())
        {
            Self::push_if_open(&mut neighbors, &north_up);
        }

        let south_down = self.find_accepted_node(pos.add_raw(0, -1, 1)).await;
        if Self::is_open(south_down.as_ref())
            && Self::has_malus(south.as_ref())
            && Self::has_malus(down.as_ref())
        {
            Self::push_if_open(&mut neighbors, &south_down);
        }

        let west_down = self.find_accepted_node(pos.add_raw(-1, -1, 0)).await;
        if Self::is_open(west_down.as_ref())
            && Self::has_malus(west.as_ref())
            && Self::has_malus(down.as_ref())
        {
            Self::push_if_open(&mut neighbors, &west_down);
        }

        let east_down = self.find_accepted_node(pos.add_raw(1, -1, 0)).await;
        if Self::is_open(east_down.as_ref())
            && Self::has_malus(east.as_ref())
            && Self::has_malus(down.as_ref())
        {
            Self::push_if_open(&mut neighbors, &east_down);
        }

        let north_down = self.find_accepted_node(pos.add_raw(0, -1, -1)).await;
        if Self::is_open(north_down.as_ref())
            && Self::has_malus(north.as_ref())
            && Self::has_malus(down.as_ref())
        {
            Self::push_if_open(&mut neighbors, &north_down);
        }

        let north_east = self.find_accepted_node(pos.add_raw(1, 0, -1)).await;
        if Self::is_open(north_east.as_ref())
            && Self::has_malus(north.as_ref())
            && Self::has_malus(east.as_ref())
        {
            Self::push_if_open(&mut neighbors, &north_east);
        }

        let south_east = self.find_accepted_node(pos.add_raw(1, 0, 1)).await;
        if Self::is_open(south_east.as_ref())
            && Self::has_malus(south.as_ref())
            && Self::has_malus(east.as_ref())
        {
            Self::push_if_open(&mut neighbors, &south_east);
        }

        let north_west = self.find_accepted_node(pos.add_raw(-1, 0, -1)).await;
        if Self::is_open(north_west.as_ref())
            && Self::has_malus(north.as_ref())
            && Self::has_malus(west.as_ref())
        {
            Self::push_if_open(&mut neighbors, &north_west);
        }

        let south_west = self.find_accepted_node(pos.add_raw(-1, 0, 1)).await;
        if Self::is_open(south_west.as_ref())
            && Self::has_malus(south.as_ref())
            && Self::has_malus(west.as_ref())
        {
            Self::push_if_open(&mut neighbors, &south_west);
        }

        let north_east_up = self.find_accepted_node(pos.add_raw(1, 1, -1)).await;
        if Self::is_open(north_east_up.as_ref())
            && Self::has_malus(north_east.as_ref())
            && Self::has_malus(north.as_ref())
            && Self::has_malus(east.as_ref())
            && Self::has_malus(up.as_ref())
            && Self::has_malus(north_up.as_ref())
            && Self::has_malus(east_up.as_ref())
        {
            Self::push_if_open(&mut neighbors, &north_east_up);
        }

        let south_east_up = self.find_accepted_node(pos.add_raw(1, 1, 1)).await;
        if Self::is_open(south_east_up.as_ref())
            && Self::has_malus(south_east.as_ref())
            && Self::has_malus(south.as_ref())
            && Self::has_malus(east.as_ref())
            && Self::has_malus(up.as_ref())
            && Self::has_malus(south_up.as_ref())
            && Self::has_malus(east_up.as_ref())
        {
            Self::push_if_open(&mut neighbors, &south_east_up);
        }

        let north_west_up = self.find_accepted_node(pos.add_raw(-1, 1, -1)).await;
        if Self::is_open(north_west_up.as_ref())
            && Self::has_malus(north_west.as_ref())
            && Self::has_malus(north.as_ref())
            && Self::has_malus(west.as_ref())
            && Self::has_malus(up.as_ref())
            && Self::has_malus(north_up.as_ref())
            && Self::has_malus(west_up.as_ref())
        {
            Self::push_if_open(&mut neighbors, &north_west_up);
        }

        let south_west_up = self.find_accepted_node(pos.add_raw(-1, 1, 1)).await;
        if Self::is_open(south_west_up.as_ref())
            && Self::has_malus(south_west.as_ref())
            && Self::has_malus(south.as_ref())
            && Self::has_malus(west.as_ref())
            && Self::has_malus(up.as_ref())
            && Self::has_malus(south_up.as_ref())
            && Self::has_malus(west_up.as_ref())
        {
            Self::push_if_open(&mut neighbors, &south_west_up);
        }

        let north_east_down = self.find_accepted_node(pos.add_raw(1, -1, -1)).await;
        if Self::is_open(north_east_down.as_ref())
            && Self::has_malus(north_east.as_ref())
            && Self::has_malus(north.as_ref())
            && Self::has_malus(east.as_ref())
            && Self::has_malus(down.as_ref())
            && Self::has_malus(north_down.as_ref())
            && Self::has_malus(east_down.as_ref())
        {
            Self::push_if_open(&mut neighbors, &north_east_down);
        }

        let south_east_down = self.find_accepted_node(pos.add_raw(1, -1, 1)).await;
        if Self::is_open(south_east_down.as_ref())
            && Self::has_malus(south_east.as_ref())
            && Self::has_malus(south.as_ref())
            && Self::has_malus(east.as_ref())
            && Self::has_malus(down.as_ref())
            && Self::has_malus(south_down.as_ref())
            && Self::has_malus(east_down.as_ref())
        {
            Self::push_if_open(&mut neighbors, &south_east_down);
        }

        let north_west_down = self.find_accepted_node(pos.add_raw(-1, -1, -1)).await;
        if Self::is_open(north_west_down.as_ref())
            && Self::has_malus(north_west.as_ref())
            && Self::has_malus(north.as_ref())
            && Self::has_malus(west.as_ref())
            && Self::has_malus(down.as_ref())
            && Self::has_malus(north_down.as_ref())
            && Self::has_malus(west_down.as_ref())
        {
            Self::push_if_open(&mut neighbors, &north_west_down);
        }

        let south_west_down = self.find_accepted_node(pos.add_raw(-1, -1, 1)).await;
        if Self::is_open(south_west_down.as_ref())
            && Self::has_malus(south_west.as_ref())
            && Self::has_malus(south.as_ref())
            && Self::has_malus(west.as_ref())
            && Self::has_malus(down.as_ref())
            && Self::has_malus(south_down.as_ref())
            && Self::has_malus(west_down.as_ref())
        {
            Self::push_if_open(&mut neighbors, &south_west_down);
        }

        neighbors
    }

    async fn get_path_type_of_mob(
        &mut self,
        context: &mut PathfindingContext,
        pos: Vector3<i32>,
        mob_data: &MobData,
    ) -> PathType {
        let mut path_types = Vec::new();
        let mob_block_pos = mob_data.block_position();
        let world_bottom_y = context.world_bottom_y();

        for dy in 0..mob_data.get_bb_height() {
            for dx in 0..mob_data.get_bb_width() {
                for dz in 0..mob_data.get_bb_width() {
                    let check_pos = pos.add_raw(dx, dy, dz);
                    let mut cell_type = context.get_path_type_from_state(check_pos).await;

                    if cell_type == PathType::Open && check_pos.y >= world_bottom_y + 1 {
                        let below_pos = check_pos.add_raw(0, -1, 0);
                        let below_type = context.get_path_type_from_state(below_pos).await;
                        cell_type = match below_type {
                            PathType::DamageFire | PathType::Lava => PathType::DamageFire,
                            PathType::DamageOther => PathType::DamageOther,
                            PathType::Cocoa => PathType::Cocoa,
                            PathType::Fence
                                if (below_pos.x, below_pos.y, below_pos.z) != mob_block_pos =>
                            {
                                PathType::Fence
                            }
                            _ => {
                                if below_type != PathType::Walkable
                                    && below_type != PathType::Open
                                    && below_type != PathType::Water
                                {
                                    PathType::Walkable
                                } else {
                                    PathType::Open
                                }
                            }
                        };
                    }

                    if matches!(cell_type, PathType::Walkable | PathType::Open) {
                        cell_type = context
                            .get_node_type_from_neighbors(check_pos, cell_type)
                            .await;
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
            let raw_center = context.get_path_type_from_state(pos).await;
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

impl Default for FlyNodeEvaluator {
    fn default() -> Self {
        Self::new()
    }
}
