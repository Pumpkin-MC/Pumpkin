//! Vanilla `SwimNodeEvaluator` port (`SwimNodeEvaluator.java`), the node
//! evaluator behind `WaterBoundPathNavigation` (fish, dolphins, guardians).

use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
use rustc_hash::FxHashMap;

use crate::entity::ai::pathfinder::{
    node::{Coordinate, Node, PathType, Target},
    node_evaluator::{BaseNodeEvaluator, MobData, NodeEvaluator},
    pathfinding_context::PathfindingContext,
};

/// Vanilla `Direction.values()` order (`Direction.java:51-56`):
/// DOWN, UP, NORTH, SOUTH, WEST, EAST — the six neighbors expanded at
/// `SwimNodeEvaluator.java:67-72`.
const DIRECTIONS_6: [(i32, i32, i32); 6] = [
    (0, -1, 0), // DOWN
    (0, 1, 0),  // UP
    (0, 0, -1), // NORTH
    (0, 0, 1),  // SOUTH
    (-1, 0, 0), // WEST
    (1, 0, 0),  // EAST
];

/// Diagonal expansion pairs of `SwimNodeEvaluator.java:73-78`:
/// `Direction.Plane.HORIZONTAL` iterates NORTH, EAST, SOUTH, WEST
/// (`Direction.java:661`), each paired with its `getClockWise()`
/// (`Direction.java:242-249`: N→E, E→S, S→W, W→N). Values are indices into
/// [`DIRECTIONS_6`].
const DIAGONAL_PAIRS: [(usize, usize); 4] = [(2, 5), (5, 3), (3, 4), (4, 2)];

/// Cost added to a node whose cell holds no fluid
/// (`SwimNodeEvaluator.java:98-99`).
const OUT_OF_FLUID_MALUS: f32 = 8.0;

pub struct SwimNodeEvaluator {
    base: BaseNodeEvaluator,
    /// `SwimNodeEvaluator.java:34`; true only for dolphins
    /// (`WaterBoundPathNavigation.java:25`).
    allow_breaching: bool,
    /// `pathTypesByPosCache` (`SwimNodeEvaluator.java:35`).
    path_types_by_pos_cache: FxHashMap<Vector3<i32>, PathType>,
}

impl SwimNodeEvaluator {
    /// Vanilla `SwimNodeEvaluator(boolean allowBreaching)`
    /// (`SwimNodeEvaluator.java:37-39`).
    #[must_use]
    pub fn new(allow_breaching: bool) -> Self {
        Self {
            base: BaseNodeEvaluator::new(),
            allow_breaching,
            path_types_by_pos_cache: FxHashMap::default(),
        }
    }

    /// Vanilla `SwimNodeEvaluator.isNodeValid` (`SwimNodeEvaluator.java:82-84`):
    /// non-null and not closed.
    fn is_node_valid(node: Option<&Node>) -> bool {
        node.is_some_and(|n| !n.closed)
    }

    /// Vanilla `SwimNodeEvaluator.hasMalus` (`SwimNodeEvaluator.java:86-88`).
    fn has_malus(node: Option<&Node>) -> bool {
        node.is_some_and(|n| n.cost_malus >= 0.0)
    }

    fn get_mob_penalty(&self, path_type: PathType) -> f32 {
        self.base.mob_data.as_ref().map_or_else(
            || path_type.get_malus(),
            |d| d.get_pathfinding_malus(path_type),
        )
    }

    /// Vanilla `SwimNodeEvaluator.findAcceptedNode`
    /// (`SwimNodeEvaluator.java:90-103`).
    async fn find_accepted_node(&mut self, pos: Vector3<i32>) -> Option<Node> {
        let path_type = self.get_cached_block_type(pos).await;
        // SwimNodeEvaluator.java:94: only BREACH (breachers) or WATER pass.
        let accepted =
            path_type == PathType::Water || (self.allow_breaching && path_type == PathType::Breach);
        if !accepted {
            return None;
        }
        let path_cost = self.get_mob_penalty(path_type);
        if path_cost < 0.0 {
            return None;
        }
        // SwimNodeEvaluator.java:95-97
        let mut node = self.base.get_node(pos.as_blockpos());
        node.path_type = path_type;
        node.cost_malus = node.cost_malus.max(path_cost);
        // SwimNodeEvaluator.java:98-100: penalize cells with no fluid.
        let fluid_empty = self
            .base
            .context
            .as_ref()
            .is_some_and(|ctx| ctx.is_fluid_empty_at(pos));
        if fluid_empty {
            node.cost_malus += OUT_OF_FLUID_MALUS;
        }
        // Vanilla nodes are shared objects; persist the accumulated malus.
        self.base.nodes.insert(pos, node);
        Some(node)
    }

    /// Vanilla `SwimNodeEvaluator.getCachedBlockType`
    /// (`SwimNodeEvaluator.java:105-107`).
    async fn get_cached_block_type(&mut self, pos: Vector3<i32>) -> PathType {
        if let Some(&cached) = self.path_types_by_pos_cache.get(&pos) {
            return cached;
        }

        // Take the context out to avoid overlapping borrows with `&mut self`.
        let path_type = if let Some(mut ctx) = self.base.context.take()
            && let Some(mob_data) = self.base.mob_data
        {
            let res = self.get_path_type_of_mob(&mut ctx, pos, &mob_data).await;
            self.base.context = Some(ctx);
            res
        } else {
            PathType::Blocked
        };

        self.path_types_by_pos_cache.insert(pos, path_type);
        path_type
    }
}

impl NodeEvaluator for SwimNodeEvaluator {
    /// Vanilla `SwimNodeEvaluator.prepare` (`SwimNodeEvaluator.java:41-45`) on
    /// top of the base `NodeEvaluator.prepare` (`NodeEvaluator.java:37-44`:
    /// entity box dims = `floor(bb + 1)`, fresh node map).
    fn prepare(&mut self, context: PathfindingContext, mob_data: MobData) {
        self.base.entity_width = mob_data.get_bb_width();
        self.base.entity_height = mob_data.get_bb_height();
        self.base.entity_depth = mob_data.get_bb_width();

        self.base.context = Some(context);
        self.base.mob_data = Some(mob_data);
        self.base.nodes.clear();
        self.path_types_by_pos_cache.clear();
    }

    /// Vanilla `SwimNodeEvaluator.done` (`SwimNodeEvaluator.java:47-51`).
    fn done(&mut self) {
        self.base.context = None;
        self.base.mob_data = None;
        self.base.nodes.clear();
        self.path_types_by_pos_cache.clear();
    }

    /// Vanilla `SwimNodeEvaluator.getStart` (`SwimNodeEvaluator.java:53-56`):
    /// `getNode(floor(bb.minX), floor(bb.minY + 0.5), floor(bb.minZ))`.
    async fn get_start(&mut self) -> Option<Node> {
        let mob_data = *self.base.mob_data.as_ref()?;
        let bb = mob_data.bounding_box();
        let pos = Vector3::new(
            bb.min.x.floor() as i32,
            (bb.min.y + 0.5).floor() as i32,
            bb.min.z.floor() as i32,
        );
        Some(self.base.get_node(pos.as_blockpos()))
    }

    /// Vanilla `SwimNodeEvaluator.getTarget` (`SwimNodeEvaluator.java:58-61`),
    /// `getTargetNodeAt` (`NodeEvaluator.java:63-65`).
    fn get_target(&mut self, pos: BlockPos) -> Target {
        let node = self.base.get_node(pos);
        Target::new(node)
    }

    /// Vanilla `SwimNodeEvaluator.getNeighbors` (`SwimNodeEvaluator.java:63-80`):
    /// the six axis neighbors, then the four horizontal diagonals whose two
    /// adjacent axis nodes both carry a non-negative malus.
    async fn get_neighbors(&mut self, current: &Node, out_neighbors: &mut Vec<Node>) {
        let mut axis_nodes: [Option<Node>; 6] = [None; 6];

        // SwimNodeEvaluator.java:67-72
        for (i, &(dx, dy, dz)) in DIRECTIONS_6.iter().enumerate() {
            let node = self
                .find_accepted_node(current.pos.0.add_raw(dx, dy, dz))
                .await;
            axis_nodes[i] = node;
            if let Some(n) = node
                && Self::is_node_valid(Some(&n))
            {
                out_neighbors.push(n);
            }
        }

        // SwimNodeEvaluator.java:73-78
        for &(first, second) in &DIAGONAL_PAIRS {
            if !Self::has_malus(axis_nodes[first].as_ref())
                || !Self::has_malus(axis_nodes[second].as_ref())
            {
                continue;
            }
            let (dx1, _, dz1) = DIRECTIONS_6[first];
            let (dx2, _, dz2) = DIRECTIONS_6[second];
            // SwimNodeEvaluator.java:76: diagonals stay on the current Y.
            let diagonal = self
                .find_accepted_node(current.pos.0.add_raw(dx1 + dx2, 0, dz1 + dz2))
                .await;
            if let Some(n) = diagonal
                && Self::is_node_valid(Some(&n))
            {
                out_neighbors.push(n);
            }
        }
    }

    /// Vanilla `SwimNodeEvaluator.getPathTypeOfMob`
    /// (`SwimNodeEvaluator.java:115-136`): scan every cell of the mob-sized
    /// box; an airy cell over pathable water is BREACH, any non-water cell is
    /// BLOCKED, otherwise classify by the last visited cell.
    async fn get_path_type_of_mob(
        &mut self,
        context: &mut PathfindingContext,
        pos: Vector3<i32>,
        _mob_data: &MobData,
    ) -> PathType {
        let mut last_cell = pos;
        // SwimNodeEvaluator.java:117-130
        for dx in 0..self.base.entity_width {
            for dy in 0..self.base.entity_height {
                for dz in 0..self.base.entity_depth {
                    let cell = pos.add_raw(dx, dy, dz);
                    last_cell = cell;
                    // SwimNodeEvaluator.java:120-125: air cell above
                    // water-pathfindable support → BREACH.
                    if context.is_fluid_empty_at(cell)
                        && context.is_water_pathfindable(cell.add_raw(0, -1, 0))
                        && context.is_air_at(cell)
                    {
                        return PathType::Breach;
                    }
                    // SwimNodeEvaluator.java:126-127: any non-water cell blocks.
                    if !context.is_water_fluid_at(cell) {
                        return PathType::Blocked;
                    }
                }
            }
        }
        // SwimNodeEvaluator.java:131-135: vanilla re-reads the mutable pos,
        // which holds the last cell of the box after the loops.
        if context.is_water_pathfindable(last_cell) {
            PathType::Water
        } else {
            PathType::Blocked
        }
    }

    /// Vanilla `SwimNodeEvaluator.getPathType` (`SwimNodeEvaluator.java:109-112`)
    /// delegates to `getPathTypeOfMob` with the prepared mob.
    async fn get_path_type(
        &mut self,
        context: &mut PathfindingContext,
        pos: Vector3<i32>,
    ) -> PathType {
        let Some(mob_data) = self.base.mob_data else {
            return PathType::Blocked;
        };
        self.get_path_type_of_mob(context, pos, &mob_data).await
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagonal_pairs_mirror_vanilla_horizontal_clockwise() {
        // Direction.Plane.HORIZONTAL (Direction.java:661) × getClockWise()
        // (Direction.java:242-249): (N,E), (E,S), (S,W), (W,N).
        for &(first, second) in &DIAGONAL_PAIRS {
            let (dx1, dy1, dz1) = DIRECTIONS_6[first];
            let (dx2, dy2, dz2) = DIRECTIONS_6[second];
            // Both must be horizontal and orthogonal to each other.
            assert_eq!(dy1, 0);
            assert_eq!(dy2, 0);
            assert_eq!(dx1 * dx2 + dz1 * dz2, 0);
            // The pair sums to a true diagonal.
            assert_eq!((dx1 + dx2).abs(), 1);
            assert_eq!((dz1 + dz2).abs(), 1);
        }
    }

    #[test]
    fn swim_maluses_match_vanilla_path_type_table() {
        // PathType.java: BREACH carries a 4.0 malus, WATER 8.0 — the two
        // types SwimNodeEvaluator.findAcceptedNode accepts
        // (SwimNodeEvaluator.java:94).
        assert!((PathType::Breach.get_malus() - 4.0).abs() < f32::EPSILON);
        assert!((PathType::Water.get_malus() - 8.0).abs() < f32::EPSILON);
    }
}
