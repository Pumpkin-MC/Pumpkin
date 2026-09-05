use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
use rustc_hash::FxHashMap;

use crate::entity::ai::pathfinder::{
    node::{Node, PathComputationType, PathType, Target},
    node_evaluator::{BaseNodeEvaluator, MobData, NodeEvaluator},
    pathfinding_context::PathfindingContext,
};

pub struct SwimNodeEvaluator {
    pub base: BaseNodeEvaluator,
    allow_breaching: bool,
    path_types_cache: FxHashMap<Vector3<i32>, PathType>,
}

impl SwimNodeEvaluator {
    #[must_use]
    pub fn new(allow_breaching: bool) -> Self {
        Self {
            base: BaseNodeEvaluator::new(),
            allow_breaching,
            path_types_cache: FxHashMap::default(),
        }
    }

    pub fn get_cached_block_type(&mut self, pos: Vector3<i32>) -> PathType {
        if let Some(&cached) = self.path_types_cache.get(&pos) {
            return cached;
        }

        let path_type = if let Some(mut ctx) = self.base.context.take() {
            let res = self.get_path_type(&mut ctx, pos);
            self.base.context = Some(ctx);
            res
        } else {
            PathType::Blocked
        };

        self.path_types_cache.insert(pos, path_type);
        path_type
    }

    fn find_accepted_node(&mut self, x: i32, y: i32, z: i32) -> Option<Node> {
        let path_type = self.get_cached_block_type(Vector3::new(x, y, z));
        if (self.allow_breaching && path_type == PathType::Breach) || path_type == PathType::Water {
            let path_cost = self.get_mob_penalty(path_type);
            (path_cost >= 0.0).then(|| {
                let mut best = self.base.get_node(BlockPos::new(x, y, z));
                best.path_type = path_type;
                best.cost_malus = best.cost_malus.max(path_cost);
                if let Some(ref ctx) = self.base.context
                    && !ctx.is_water(&BlockPos::new(x, y, z))
                {
                    best.cost_malus += 8.0;
                }
                best
            })
        } else {
            None
        }
    }

    const fn is_node_valid(node: Option<&Node>) -> bool {
        if let Some(n) = node { !n.closed } else { false }
    }

    const fn has_malus(node: Option<&Node>) -> bool {
        if let Some(n) = node {
            n.cost_malus >= 0.0
        } else {
            false
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
}

impl NodeEvaluator for SwimNodeEvaluator {
    fn prepare(&mut self, context: PathfindingContext, mob_data: MobData) {
        self.base.entity_width = mob_data.get_bb_width();
        self.base.entity_height = mob_data.get_bb_height();
        self.base.entity_depth = mob_data.get_bb_width();

        self.base.context = Some(context);
        self.base.mob_data = Some(mob_data);
        self.path_types_cache.clear();
    }

    fn done(&mut self) {
        self.base.context = None;
        self.base.mob_data = None;
        self.path_types_cache.clear();
    }

    fn get_start(&mut self) -> Option<Node> {
        let mob_data = self.base.mob_data?;
        let min_x = (mob_data.position.x - f64::from(mob_data.width) / 2.0).floor() as i32;
        let min_y = (mob_data.position.y + 0.5).floor() as i32;
        let min_z = (mob_data.position.z - f64::from(mob_data.width) / 2.0).floor() as i32;

        let mut node = self.base.get_node(BlockPos::new(min_x, min_y, min_z));
        node.path_type = self.get_cached_block_type(Vector3::new(min_x, min_y, min_z));
        node.cost_malus = self.get_mob_penalty(node.path_type);
        Some(node)
    }

    fn get_target(&mut self, pos: BlockPos) -> Target {
        Target::new(self.base.get_node(pos))
    }

    fn get_neighbors(&mut self, current: &Node, out: &mut Vec<Node>) {
        let x = current.pos.0.x;
        let y = current.pos.0.y;
        let z = current.pos.0.z;

        let mut nodes = [None; 6];
        let directions = [
            (0usize, 0, -1, 0),
            (1usize, 0, 1, 0),
            (2usize, 0, 0, -1),
            (3usize, 0, 0, 1),
            (4usize, -1, 0, 0),
            (5usize, 1, 0, 0),
        ];

        for &(idx, dx, dy, dz) in &directions {
            let node = self.find_accepted_node(x + dx, y + dy, z + dz);
            nodes[idx] = node;
            if Self::is_node_valid(node.as_ref())
                && let Some(n) = node
            {
                out.push(n);
            }
        }

        let horizontal = [
            (2usize, 5usize, 1, -1),
            (3usize, 4usize, -1, 1),
            (4usize, 2usize, -1, -1),
            (5usize, 3usize, 1, 1),
        ];

        for &(dir_idx, cw_idx, dx, dz) in &horizontal {
            if Self::has_malus(nodes[dir_idx].as_ref()) && Self::has_malus(nodes[cw_idx].as_ref()) {
                let diagonal_node = self.find_accepted_node(x + dx, y, z + dz);
                if Self::is_node_valid(diagonal_node.as_ref())
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
        _mob_data: &MobData,
    ) -> PathType {
        for xx in pos.x..(pos.x + self.base.entity_width) {
            for yy in pos.y..(pos.y + self.base.entity_height) {
                for zz in pos.z..(pos.z + self.base.entity_depth) {
                    let check_pos = BlockPos::new(xx, yy, zz);
                    let is_water = context.is_water(&check_pos);
                    let below_pos = check_pos.down();
                    let below_pathfindable =
                        context.is_pathfindable(&below_pos, PathComputationType::Water);
                    let is_air = context.is_air(&check_pos);
                    if !is_water && below_pathfindable && is_air {
                        return PathType::Breach;
                    }
                    if !is_water {
                        return PathType::Blocked;
                    }
                }
            }
        }

        let p = pos.to_block_pos();
        if context.is_pathfindable(&p, PathComputationType::Water) {
            PathType::Water
        } else {
            PathType::Blocked
        }
    }

    fn get_path_type(&mut self, context: &mut PathfindingContext, pos: Vector3<i32>) -> PathType {
        let mob_data = self.base.mob_data.unwrap_or(MobData {
            position: Vector3::new(0.0, 0.0, 0.0),
            width: 0.6,
            height: 1.95,
            max_step_height: 1.0,
            max_fall_distance: 3.0,
            can_swim: true,
            can_walk_on_water: false,
            avoids_fire: true,
            avoids_water: false,
            on_ground: false,
            is_in_water: true,
            sea_level: 63,
            min_y: -64,
            path_type_malus: [None; crate::entity::ai::pathfinder::node::PATH_TYPE_COUNT],
        });
        self.get_path_type_of_mob(context, pos, &mob_data)
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
