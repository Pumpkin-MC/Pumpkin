use std::future::Future;
use std::pin::Pin;

use pumpkin_util::math::{boundingbox::BoundingBox, position::BlockPos, vector3::Vector3};
use rustc_hash::FxHashMap;

use crate::entity::ai::pathfinder::{
    node::{Coordinate, Node, PathType, Target},
    node_evaluator::{BaseNodeEvaluator, MobData, NodeEvaluator},
    pathfinding_context::PathfindingContext,
};

const DIRECTIONS: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
const DIAGONAL_DIRECTIONS: [(i32, i32); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];

const DEFAULT_MOB_JUMP_HEIGHT: f64 = 1.125;

pub struct WalkNodeEvaluator {
    pub(crate) base: BaseNodeEvaluator,
    path_types_cache: FxHashMap<Vector3<i32>, PathType>,
    reusable_neighbors: [Option<Node>; 4],
    /// Vanilla `WalkNodeEvaluator.isAmphibious()` returns `false`; only
    /// `AmphibiousNodeEvaluator` overrides it to `true`
    /// (`AmphibiousNodeEvaluator.java:88-91`). The composition-based
    /// amphibious evaluator sets this flag on its inner walk evaluator to
    /// mirror that virtual dispatch.
    amphibious: bool,
}

impl WalkNodeEvaluator {
    #[must_use]
    pub fn new() -> Self {
        Self {
            base: BaseNodeEvaluator::new(),
            path_types_cache: FxHashMap::default(),
            reusable_neighbors: [None, None, None, None],
            amphibious: false,
        }
    }

    pub(crate) const fn set_amphibious(&mut self, amphibious: bool) {
        self.amphibious = amphibious;
    }

    const fn is_amphibious(&self) -> bool {
        self.amphibious
    }

    /// Vanilla `WalkNodeEvaluator.getFloorLevel` (`WalkNodeEvaluator.java:193-199`):
    /// floaters and amphibious mobs treat a water cell's floor as `y + 0.5`,
    /// everyone else uses the collision top of the block below.
    pub(crate) fn get_floor_level(&self, pos: Vector3<i32>) -> f64 {
        self.base.context.as_ref().map_or_else(
            || f64::from(pos.y),
            |context| {
                if (self.base.can_float || self.is_amphibious()) && context.is_water_at(pos) {
                    return f64::from(pos.y) + 0.5;
                }
                context.get_floor_level(pos)
            },
        )
    }

    fn get_mob_jump_height(&self) -> f64 {
        self.base
            .mob_data
            .as_ref()
            .map_or(DEFAULT_MOB_JUMP_HEIGHT, |d| {
                f64::from(d.max_step_height).max(DEFAULT_MOB_JUMP_HEIGHT)
            })
    }

    pub(crate) fn is_neighbor_valid(neighbor: Option<&Node>, current: &Node) -> bool {
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
        // Vanilla WalkNodeEvaluator.java:160-162: wide mobs never cut a diagonal
        // past a cell that carries any cost penalty.
        if mob_width > 1.0 && (adj_x.cost_malus > 0.0 || adj_z.cost_malus > 0.0) {
            return false;
        }
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

    /// Vanilla `WalkNodeEvaluator.getNodeAndUpdateCostToMax`
    /// (`WalkNodeEvaluator.java:245-250`). The updated malus is written back to
    /// the node map so repeated lookups accumulate like vanilla's shared nodes.
    fn get_node_and_update_cost_to_max(
        &mut self,
        pos: Vector3<i32>,
        path_type: PathType,
        cost: f32,
    ) -> Node {
        let mut n = self.base.get_node(pos.as_blockpos());
        n.path_type = path_type;
        n.cost_malus = n.cost_malus.max(cost);
        self.base.nodes.insert(pos, n);
        n
    }

    /// Vanilla `WalkNodeEvaluator.getBlockedNode` (`WalkNodeEvaluator.java:252-257`).
    fn get_blocked_node(&mut self, pos: Vector3<i32>) -> Node {
        let mut n = self.base.get_node(pos.as_blockpos());
        n.path_type = PathType::Blocked;
        n.cost_malus = -1.0;
        self.base.nodes.insert(pos, n);
        n
    }

    /// Vanilla `WalkNodeEvaluator.getClosedNode` (`WalkNodeEvaluator.java:259-265`).
    fn get_closed_node(&mut self, pos: Vector3<i32>, path_type: PathType) -> Node {
        let mut n = self.base.get_node(pos.as_blockpos());
        n.closed = true;
        n.path_type = path_type;
        n.cost_malus = path_type.get_malus();
        self.base.nodes.insert(pos, n);
        n
    }

    /// Vanilla `WalkNodeEvaluator.findAcceptedNode` (`WalkNodeEvaluator.java:211-239`):
    /// returns the best path node for the given position, handling step-ups,
    /// falls, and blocked nodes.
    pub(crate) async fn find_accepted_node(
        &mut self,
        pos: Vector3<i32>,
        jump_size: i32,
        node_height: f64,
        facing: (i32, i32),
        current_path_type: PathType,
    ) -> Option<Node> {
        // WalkNodeEvaluator.java:214-217
        let max_y_target = self.get_floor_level(pos);
        if max_y_target - node_height > self.get_mob_jump_height() {
            return None;
        }

        let path_type = self.get_cached_path_type(pos).await;
        let path_cost = self.get_mob_penalty(path_type);

        // WalkNodeEvaluator.java:220-222
        let mut best = (path_cost >= 0.0)
            .then(|| self.get_node_and_update_cost_to_max(pos, path_type, path_cost));

        // WalkNodeEvaluator.java:223-225: leaving a partial-collision cell
        // (fence, closed door) requires a swept-AABB collision check.
        if Self::does_block_have_partial_collision(current_path_type)
            && best.as_ref().is_some_and(|n| n.cost_malus >= 0.0)
            && !self.can_reach_without_collision(pos)
        {
            best = None;
        }

        // WalkNodeEvaluator.java:226-228
        if path_type == PathType::Walkable || (self.is_amphibious() && path_type == PathType::Water)
        {
            return best;
        }

        // WalkNodeEvaluator.java:229-237
        if (best.is_none() || best.as_ref().is_some_and(|n| n.cost_malus < 0.0))
            && jump_size > 0
            && (path_type != PathType::Fence || self.base.can_walk_over_fences)
            && path_type != PathType::UnpassableRail
            && path_type != PathType::Trapdoor
            && path_type != PathType::PowderSnow
        {
            best = self
                .try_jump_on(pos, jump_size, node_height, facing, current_path_type)
                .await;
        } else if !self.is_amphibious() && path_type == PathType::Water && !self.base.can_float {
            best = self.try_find_first_non_water_below(pos, best).await;
        } else if path_type == PathType::Open {
            best = Some(self.try_find_first_ground_node_below(pos).await);
        } else if Self::does_block_have_partial_collision(path_type) && best.is_none() {
            best = Some(self.get_closed_node(pos, path_type));
        }

        best
    }

    /// Type-erased recursion bridge: `find_accepted_node` -> `try_jump_on` ->
    /// `find_accepted_node` needs one boxed link to keep the future sized.
    fn find_accepted_node_boxed<'a>(
        &'a mut self,
        pos: Vector3<i32>,
        jump_size: i32,
        node_height: f64,
        facing: (i32, i32),
        current_path_type: PathType,
    ) -> Pin<Box<dyn Future<Output = Option<Node>> + Send + 'a>> {
        Box::pin(self.find_accepted_node(pos, jump_size, node_height, facing, current_path_type))
    }

    /// Vanilla `WalkNodeEvaluator.tryJumpOn` (`WalkNodeEvaluator.java:267-283`):
    /// recursively evaluates the cell one block up with `jumpSize - 1`, and for
    /// mobs narrower than one block verifies the head-room with a grown AABB
    /// spanning from the source column to the landing node.
    async fn try_jump_on(
        &mut self,
        pos: Vector3<i32>,
        jump_size: i32,
        node_height: f64,
        facing: (i32, i32),
        current_path_type: PathType,
    ) -> Option<Node> {
        // WalkNodeEvaluator.java:268-271
        let node_above = self
            .find_accepted_node_boxed(
                Vector3::new(pos.x, pos.y + 1, pos.z),
                jump_size - 1,
                node_height,
                facing,
                current_path_type,
            )
            .await?;

        // WalkNodeEvaluator.java:272-274
        let width = self.base.mob_data.as_ref().map_or(0.6, |d| d.width);
        if width >= 1.0 {
            return Some(node_above);
        }

        // WalkNodeEvaluator.java:275-277
        if node_above.path_type != PathType::Open && node_above.path_type != PathType::Walkable {
            return Some(node_above);
        }

        // WalkNodeEvaluator.java:278-282: ceiling check while jumping from the
        // source column (pos - facing) onto the raised node.
        let height = self.base.mob_data.as_ref().map_or(1.8, |d| d.height);
        let center_x = f64::from(pos.x - facing.0) + 0.5;
        let center_z = f64::from(pos.z - facing.1) + 0.5;
        let half_width = f64::from(width) / 2.0;
        let source_floor = self.get_floor_level(Vector3::new(
            center_x.floor() as i32,
            pos.y + 1,
            center_z.floor() as i32,
        ));
        let target_floor = self.get_floor_level(node_above.pos.0);
        let grow = BoundingBox::new(
            Vector3::new(
                center_x - half_width,
                source_floor + 0.001,
                center_z - half_width,
            ),
            Vector3::new(
                center_x + half_width,
                f64::from(height) + target_floor - 0.002,
                center_z + half_width,
            ),
        );
        if self.has_collisions(&grow) {
            None
        } else {
            Some(node_above)
        }
    }

    /// Vanilla `WalkNodeEvaluator.tryFindFirstGroundNodeBelow`
    /// (`WalkNodeEvaluator.java:298-312`): walk down through consecutive `OPEN`
    /// cells until a standable type within `getMaxFallDistance`.
    async fn try_find_first_ground_node_below(&mut self, pos: Vector3<i32>) -> Node {
        let max_fall_distance = self
            .base
            .mob_data
            .as_ref()
            .map_or(3, |d| d.max_fall_distance as i32);
        let min_y = self
            .base
            .context
            .as_ref()
            .map_or(i32::MIN, PathfindingContext::min_y);

        let mut current_y = pos.y - 1;
        while current_y >= min_y {
            // WalkNodeEvaluator.java:300-302
            if pos.y - current_y > max_fall_distance {
                return self.get_blocked_node(Vector3::new(pos.x, current_y, pos.z));
            }
            let check = Vector3::new(pos.x, current_y, pos.z);
            let path_type = self.get_cached_path_type(check).await;
            let path_cost = self.get_mob_penalty(path_type);
            if path_type != PathType::Open {
                // WalkNodeEvaluator.java:306-309
                if path_cost >= 0.0 {
                    return self.get_node_and_update_cost_to_max(check, path_type, path_cost);
                }
                return self.get_blocked_node(check);
            }
            current_y -= 1;
        }

        // WalkNodeEvaluator.java:311
        self.get_blocked_node(pos)
    }

    /// Vanilla `WalkNodeEvaluator.tryFindFirstNonWaterBelow`
    /// (`WalkNodeEvaluator.java:285-296`).
    async fn try_find_first_non_water_below(
        &mut self,
        pos: Vector3<i32>,
        mut best: Option<Node>,
    ) -> Option<Node> {
        let min_y = self
            .base
            .context
            .as_ref()
            .map_or(i32::MIN, PathfindingContext::min_y);
        let mut y = pos.y - 1;
        while y > min_y {
            let check = Vector3::new(pos.x, y, pos.z);
            let path_type = self.get_cached_path_type(check).await;
            if path_type != PathType::Water {
                return best;
            }
            let path_cost = self.get_mob_penalty(path_type);
            best = Some(self.get_node_and_update_cost_to_max(check, path_type, path_cost));
            y -= 1;
        }
        best
    }

    /// Vanilla `WalkNodeEvaluator.canReachWithoutCollision`
    /// (`WalkNodeEvaluator.java:181-191`): sweep the mob's bounding box in
    /// steps toward the target cell and reject the move on any collision.
    fn can_reach_without_collision(&mut self, pos: Vector3<i32>) -> bool {
        let Some(mob_data) = self.base.mob_data else {
            return true;
        };
        let mut bb = mob_data.bounding_box();
        let delta = Vector3::new(
            f64::from(pos.x) - mob_data.position.x + (bb.max.x - bb.min.x) / 2.0,
            f64::from(pos.y) - mob_data.position.y + (bb.max.y - bb.min.y) / 2.0,
            f64::from(pos.z) - mob_data.position.z + (bb.max.z - bb.min.z) / 2.0,
        );
        // WalkNodeEvaluator.java:184: steps = ceil(|delta| / average bb size).
        let steps = (delta.length() / bb.get_average_side_length()).ceil() as i32;
        if steps <= 0 {
            return true;
        }
        let step_delta = delta * (1.0 / f64::from(steps));
        for _ in 1..=steps {
            bb = bb.shift(step_delta);
            if self.has_collisions(&bb) {
                return false;
            }
        }
        true
    }

    pub(crate) fn get_mob_penalty(&self, path_type: PathType) -> f32 {
        self.base
            .mob_data
            .as_ref()
            .map_or(path_type.get_malus(), |d| {
                d.get_pathfinding_malus(path_type)
            })
    }

    /// Vanilla `WalkNodeEvaluator.doesBlockHavePartialCollision`
    /// (`WalkNodeEvaluator.java:177-179`).
    const fn does_block_have_partial_collision(path_type: PathType) -> bool {
        matches!(
            path_type,
            PathType::Fence | PathType::DoorWoodClosed | PathType::DoorIronClosed
        )
    }

    pub(crate) async fn get_cached_path_type(&mut self, pos: Vector3<i32>) -> PathType {
        if let Some(&cached) = self.path_types_cache.get(&pos) {
            return cached;
        }

        // Temporarily take the context out to avoid overlapping borrows when calling
        // the async helper which requires `&mut self`
        let path_type = if let Some(mut ctx) = self.base.context.take()
            && let Some(mob_data) = self.base.mob_data
        {
            let res = self.get_path_type_of_mob(&mut ctx, pos, &mob_data).await;
            self.base.context = Some(ctx);
            res
        } else {
            PathType::Blocked
        };

        self.path_types_cache.insert(pos, path_type);
        path_type
    }

    /// Vanilla `WalkNodeEvaluator.hasCollisions` (`WalkNodeEvaluator.java:314-316`).
    fn has_collisions(&mut self, aabb: &BoundingBox) -> bool {
        self.base
            .context
            .as_mut()
            .is_some_and(|ctx| ctx.has_collisions(aabb))
    }

    /// Per-cell classification, mirroring the virtual `getPathType` dispatch
    /// inside vanilla `getPathTypeWithinMobBB` (`WalkNodeEvaluator.java:371`):
    /// `AmphibiousNodeEvaluator.getPathType`
    /// (`AmphibiousNodeEvaluator.java:93-107`) overrides it, while the walk
    /// default is the land node type (`WalkNodeEvaluator.getPathType` ->
    /// `getPathTypeStatic`).
    fn cell_path_type(&self, context: &mut PathfindingContext, pos: Vector3<i32>) -> PathType {
        if self.amphibious {
            crate::entity::ai::pathfinder::amphibious_node_evaluator::amphibious_cell_path_type(
                context, pos,
            )
        } else {
            context.get_land_node_type(pos)
        }
    }

    /// Vanilla `WalkNodeEvaluator.canStartAt` (`WalkNodeEvaluator.java:114-117`):
    /// `type != OPEN && getPathfindingMalus(type) >= 0`.
    async fn can_start_at(&mut self, pos: Vector3<i32>) -> bool {
        let path_type = self.get_cached_path_type(pos).await;
        path_type != PathType::Open && self.get_mob_penalty(path_type) >= 0.0
    }

    /// Vanilla `WalkNodeEvaluator.getStartNode` (`WalkNodeEvaluator.java:107-112`).
    pub(crate) async fn get_start_node(&mut self, pos: Vector3<i32>) -> Node {
        let mut node = self.base.get_node(pos.as_blockpos());
        let path_type = self.get_cached_path_type(pos).await;
        node.path_type = path_type;
        node.cost_malus = self.get_mob_penalty(path_type);
        node
    }
}

impl NodeEvaluator for WalkNodeEvaluator {
    fn prepare(&mut self, context: PathfindingContext, mob_data: MobData) {
        self.base.entity_width = mob_data.get_bb_width();
        self.base.entity_height = mob_data.get_bb_height();
        self.base.entity_depth = mob_data.get_bb_width();

        self.base.context = Some(context);
        self.base.mob_data = Some(mob_data);
        // Vanilla allocates a fresh node map per path compute — stale closed /
        // cost flags from a prior search must not leak into the next one.
        self.base.nodes.clear();
        self.path_types_cache.clear();
    }

    fn done(&mut self) {
        self.base.context = None;
        self.base.mob_data = None;
        self.base.nodes.clear();
        self.path_types_cache.clear();
    }

    /// Vanilla `WalkNodeEvaluator.getStart` (`WalkNodeEvaluator.java:70-105`).
    ///
    /// Gap: vanilla first handles mobs that `canStandOnFluid` (striders on
    /// lava, `WalkNodeEvaluator.java:75-79`); Pumpkin has no such hook yet.
    async fn get_start(&mut self) -> Option<Node> {
        let mob_data = *self.base.mob_data.as_ref()?;
        let block_x = mob_data.position.x.floor() as i32;
        let block_z = mob_data.position.z.floor() as i32;
        let min_y = self
            .base
            .context
            .as_ref()
            .map_or(i32::MIN, PathfindingContext::min_y);

        // Vanilla WalkNodeEvaluator.java:73: `startY = mob.getBlockY()`.
        let mut start_y = mob_data.position.y.floor() as i32;

        if self.base.can_float && mob_data.is_in_water {
            // WalkNodeEvaluator.java:80-84: floaters start at the water surface.
            let mut check = self
                .base
                .context
                .as_ref()
                .is_some_and(|ctx| ctx.is_water_at(Vector3::new(block_x, start_y, block_z)));
            while check {
                start_y += 1;
                check =
                    self.base.context.as_ref().is_some_and(|ctx| {
                        ctx.is_water_at(Vector3::new(block_x, start_y, block_z))
                    });
            }
            start_y -= 1;
        } else if mob_data.on_ground {
            // WalkNodeEvaluator.java:85-86
            start_y = (mob_data.position.y + 0.5).floor() as i32;
        } else {
            // WalkNodeEvaluator.java:87-96: airborne — descend while the block
            // below is air or land-pathfindable.
            let mut check_y = (mob_data.position.y + 1.0).floor() as i32;
            while check_y > min_y {
                start_y = check_y;
                check_y -= 1;
                let passable = self.base.context.as_ref().is_some_and(|ctx| {
                    ctx.is_air_or_land_pathfindable(Vector3::new(block_x, check_y, block_z))
                });
                if !passable {
                    break;
                }
            }
        }

        // WalkNodeEvaluator.java:97-103: if the mob's own column can't start,
        // try the four bounding-box corners.
        let center = Vector3::new(block_x, start_y, block_z);
        if !self.can_start_at(center).await {
            let bb = mob_data.bounding_box();
            let corners = [
                (bb.min.x.floor() as i32, bb.min.z.floor() as i32),
                (bb.min.x.floor() as i32, bb.max.z.floor() as i32),
                (bb.max.x.floor() as i32, bb.min.z.floor() as i32),
                (bb.max.x.floor() as i32, bb.max.z.floor() as i32),
            ];
            for (corner_x, corner_z) in corners {
                let corner = Vector3::new(corner_x, start_y, corner_z);
                if self.can_start_at(corner).await {
                    return Some(self.get_start_node(corner).await);
                }
            }
        }

        // WalkNodeEvaluator.java:104: always fall back to the center node.
        Some(self.get_start_node(center).await)
    }

    fn get_target(&mut self, pos: BlockPos) -> Target {
        let node = self.base.get_node(pos);
        Target::new(node)
    }

    async fn get_neighbors(&mut self, current: &Node, out_neighbors: &mut Vec<Node>) {
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

        let floor_level = self.get_floor_level(current.pos.0);

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
                self.reusable_neighbors[i] = Some(neighbor);
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
                    let mut cell_type = self.cell_path_type(context, check_pos);

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
                        // Vanilla WalkNodeEvaluator.java:380: virtual `getPathType`.
                        let mob_type = self.cell_path_type(context, mob_pos);
                        let mob_below_type = self.cell_path_type(context, mob_below);
                        if mob_type != PathType::Rail && mob_below_type != PathType::Rail {
                            cell_type = PathType::UnpassableRail;
                        }
                    }

                    path_types.push(cell_type);
                }
            }
        }

        // Sort+dedup to match vanilla's EnumSet ordinal iteration order
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
            // Vanilla WalkNodeEvaluator.java:346: virtual `getPathType` again.
            let raw_center = self.cell_path_type(context, pos);
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
        context.get_path_type_from_state(pos)
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
