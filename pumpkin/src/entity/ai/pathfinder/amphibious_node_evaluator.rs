//! Vanilla `AmphibiousNodeEvaluator` port (`AmphibiousNodeEvaluator.java`),
//! the node evaluator behind `AmphibiousPathNavigation` (axolotls).
//!
//! Vanilla inherits from `WalkNodeEvaluator`; this port composes an inner
//! [`WalkNodeEvaluator`] with its `amphibious` flag set, which mirrors the
//! `isAmphibious()` override (`AmphibiousNodeEvaluator.java:88-91`) at every
//! virtual-dispatch site of the walk evaluator.

use pumpkin_util::math::{position::BlockPos, vector3::Vector3};

use crate::entity::ai::pathfinder::{
    node::{Node, PathType, Target},
    node_evaluator::{MobData, NodeEvaluator},
    pathfinding_context::PathfindingContext,
    walk_node_evaluator::WalkNodeEvaluator,
};

/// Vanilla `Direction.values()` order (`Direction.java:51-56`), used by the
/// WATER_BORDER scan of `AmphibiousNodeEvaluator.getPathType`
/// (`AmphibiousNodeEvaluator.java:98-103`).
const DIRECTIONS_6: [(i32, i32, i32); 6] = [
    (0, -1, 0), // DOWN
    (0, 1, 0),  // UP
    (0, 0, -1), // NORTH
    (0, 0, 1),  // SOUTH
    (-1, 0, 0), // WEST
    (1, 0, 0),  // EAST
];

/// Prepare-time malus overrides, vanilla `AmphibiousNodeEvaluator.prepare`
/// (`AmphibiousNodeEvaluator.java:34,36,38`): WATER 0.0, WALKABLE 6.0,
/// WATER_BORDER 4.0.
///
/// Vanilla saves and restores the previous WALKABLE/WATER_BORDER maluses in
/// `done()` (`AmphibiousNodeEvaluator.java:35,37,42-45`) because they live on
/// the shared `Mob`; Pumpkin rebuilds `MobData` per path compute, so the
/// overrides apply to a per-search copy and need no restore.
pub(crate) const fn apply_amphibious_maluses(mob_data: &mut MobData) {
    mob_data.set_pathfinding_malus(PathType::Water, 0.0);
    mob_data.set_pathfinding_malus(PathType::Walkable, 6.0);
    mob_data.set_pathfinding_malus(PathType::WaterBorder, 4.0);
}

/// Per-cell classification override, vanilla
/// `AmphibiousNodeEvaluator.getPathType` (`AmphibiousNodeEvaluator.java:93-107`):
/// a WATER cell touching any BLOCKED neighbor is WATER_BORDER; everything
/// else falls through to the walk classification (`super.getPathType`,
/// `AmphibiousNodeEvaluator.java:106`).
pub(crate) fn amphibious_cell_path_type(
    context: &mut PathfindingContext,
    pos: Vector3<i32>,
) -> PathType {
    if context.get_path_type_from_state(pos) == PathType::Water {
        // AmphibiousNodeEvaluator.java:97-103
        for &(dx, dy, dz) in &DIRECTIONS_6 {
            if context.get_path_type_from_state(pos.add_raw(dx, dy, dz)) == PathType::Blocked {
                return PathType::WaterBorder;
            }
        }
        // AmphibiousNodeEvaluator.java:104
        return PathType::Water;
    }
    context.get_land_node_type(pos)
}

pub struct AmphibiousNodeEvaluator {
    walk: WalkNodeEvaluator,
    /// `AmphibiousNodeEvaluator.java:23`; `false` for
    /// `AmphibiousPathNavigation` mobs (`AmphibiousPathNavigation.java:22`).
    prefers_shallow_swimming: bool,
}

impl AmphibiousNodeEvaluator {
    /// Vanilla `AmphibiousNodeEvaluator(boolean prefersShallowSwimming)`
    /// (`AmphibiousNodeEvaluator.java:27-29`).
    #[must_use]
    pub fn new(prefers_shallow_swimming: bool) -> Self {
        let mut walk = WalkNodeEvaluator::new();
        // Mirrors the `isAmphibious()` override (AmphibiousNodeEvaluator.java:88-91).
        walk.set_amphibious(true);
        Self {
            walk,
            prefers_shallow_swimming,
        }
    }

    /// Vanilla `AmphibiousNodeEvaluator.isVerticalNeighborValid`
    /// (`AmphibiousNodeEvaluator.java:84-86`).
    fn is_vertical_neighbor_valid(vertical: Option<&Node>, current: &Node) -> bool {
        WalkNodeEvaluator::is_neighbor_valid(vertical, current)
            && vertical.is_some_and(|n| n.path_type == PathType::Water)
    }
}

impl NodeEvaluator for AmphibiousNodeEvaluator {
    /// Vanilla `AmphibiousNodeEvaluator.prepare`
    /// (`AmphibiousNodeEvaluator.java:31-39`).
    fn prepare(&mut self, context: PathfindingContext, mut mob_data: MobData) {
        apply_amphibious_maluses(&mut mob_data);
        self.walk.prepare(context, mob_data);
    }

    /// Vanilla `AmphibiousNodeEvaluator.done`
    /// (`AmphibiousNodeEvaluator.java:41-46`); the malus restore is
    /// unnecessary here — see [`apply_amphibious_maluses`].
    fn done(&mut self) {
        self.walk.done();
    }

    /// Vanilla `AmphibiousNodeEvaluator.getStart`
    /// (`AmphibiousNodeEvaluator.java:48-54`): out of water defer to the walk
    /// start; in water start at `(floor(bb.minX), floor(bb.minY + 0.5),
    /// floor(bb.minZ))` via `getStartNode`.
    async fn get_start(&mut self) -> Option<Node> {
        let mob_data = self.walk.base.mob_data?;
        if !mob_data.is_in_water {
            return self.walk.get_start().await;
        }
        let bb = mob_data.bounding_box();
        let pos = Vector3::new(
            bb.min.x.floor() as i32,
            (bb.min.y + 0.5).floor() as i32,
            bb.min.z.floor() as i32,
        );
        Some(self.walk.get_start_node(pos).await)
    }

    /// Vanilla `AmphibiousNodeEvaluator.getTarget`
    /// (`AmphibiousNodeEvaluator.java:56-59`): `getTargetNodeAt(x, y + 0.5, z)`.
    /// The pathfinder always passes whole-block coordinates
    /// (`PathFinder.findPath` targets), so `floor(y + 0.5) == y` and the
    /// vanilla offset is a no-op for a [`BlockPos`] target.
    fn get_target(&mut self, pos: BlockPos) -> Target {
        self.walk.get_target(pos)
    }

    /// Vanilla `AmphibiousNodeEvaluator.getNeighbors`
    /// (`AmphibiousNodeEvaluator.java:61-82`): the walk neighbors plus
    /// straight-up and straight-down water nodes, then the optional
    /// shallow-swimming penalty.
    async fn get_neighbors(&mut self, current: &Node, out_neighbors: &mut Vec<Node>) {
        // AmphibiousNodeEvaluator.java:63
        self.walk.get_neighbors(current, out_neighbors).await;

        let above_pos = current.pos.0.add_raw(0, 1, 0);
        // AmphibiousNodeEvaluator.java:64-65
        let above_type = self.walk.get_cached_path_type(above_pos).await;
        let current_type = self.walk.get_cached_path_type(current.pos.0).await;

        // AmphibiousNodeEvaluator.java:66:
        // `floor(max(1.0f, mob.maxUpStep()))`, gated on a passable headroom
        // cell and a non-sticky current cell.
        let max_up_step = self
            .walk
            .base
            .mob_data
            .as_ref()
            .map_or(0.6, |d| d.max_step_height);
        let jump_size = if self.walk.get_mob_penalty(above_type) >= 0.0
            && current_type != PathType::StickyHoney
        {
            max_up_step.max(1.0).floor() as i32
        } else {
            0
        };

        // AmphibiousNodeEvaluator.java:67
        let pos_height = self.walk.get_floor_level(current.pos.0);

        // AmphibiousNodeEvaluator.java:68: UP has no horizontal step.
        let up_node = self
            .walk
            .find_accepted_node(
                above_pos,
                (jump_size - 1).max(0),
                pos_height,
                (0, 0),
                current_type,
            )
            .await;
        // AmphibiousNodeEvaluator.java:69
        let down_node = self
            .walk
            .find_accepted_node(
                current.pos.0.add_raw(0, -1, 0),
                jump_size,
                pos_height,
                (0, 0),
                current_type,
            )
            .await;

        // AmphibiousNodeEvaluator.java:70-72
        if let Some(up) = up_node
            && Self::is_vertical_neighbor_valid(Some(&up), current)
        {
            out_neighbors.push(up);
        }
        // AmphibiousNodeEvaluator.java:73-75
        if let Some(down) = down_node
            && Self::is_vertical_neighbor_valid(Some(&down), current)
            && current_type != PathType::Trapdoor
        {
            out_neighbors.push(down);
        }

        // AmphibiousNodeEvaluator.java:76-80: deep-water penalty when the mob
        // prefers shallow swimming (`y < seaLevel - 10` → +1.0).
        if self.prefers_shallow_swimming {
            let sea_level = self
                .walk
                .base
                .context
                .as_ref()
                .map_or(i32::MIN, PathfindingContext::sea_level);
            for neighbor in &mut *out_neighbors {
                if neighbor.path_type == PathType::Water && neighbor.pos.0.y < sea_level - 10 {
                    neighbor.cost_malus += 1.0;
                }
            }
        }
    }

    /// Vanilla inherits `WalkNodeEvaluator.getPathTypeOfMob`; the amphibious
    /// per-cell override reaches it through the inner evaluator's
    /// `amphibious` flag (see `WalkNodeEvaluator::cell_path_type`).
    async fn get_path_type_of_mob(
        &mut self,
        context: &mut PathfindingContext,
        pos: Vector3<i32>,
        mob_data: &MobData,
    ) -> PathType {
        self.walk.get_path_type_of_mob(context, pos, mob_data).await
    }

    /// Vanilla `AmphibiousNodeEvaluator.getPathType`
    /// (`AmphibiousNodeEvaluator.java:93-107`).
    async fn get_path_type(
        &mut self,
        context: &mut PathfindingContext,
        pos: Vector3<i32>,
    ) -> PathType {
        amphibious_cell_path_type(context, pos)
    }

    fn set_can_pass_doors(&mut self, can_pass: bool) {
        self.walk.set_can_pass_doors(can_pass);
    }

    fn set_can_open_doors(&mut self, can_open: bool) {
        self.walk.set_can_open_doors(can_open);
    }

    fn set_can_float(&mut self, can_float: bool) {
        self.walk.set_can_float(can_float);
    }

    fn set_can_walk_over_fences(&mut self, can_walk: bool) {
        self.walk.set_can_walk_over_fences(can_walk);
    }

    fn can_pass_doors(&self) -> bool {
        self.walk.can_pass_doors()
    }

    fn can_open_doors(&self) -> bool {
        self.walk.can_open_doors()
    }

    fn can_float(&self) -> bool {
        self.walk.can_float()
    }

    fn can_walk_over_fences(&self) -> bool {
        self.walk.can_walk_over_fences()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_util::math::vector3::Vector3;

    #[test]
    fn prepare_maluses_match_vanilla() {
        // AmphibiousNodeEvaluator.java:34,36,38: WATER 0.0, WALKABLE 6.0,
        // WATER_BORDER 4.0.
        let mut mob_data = MobData::new(Vector3::new(0.0, 0.0, 0.0), 0.75, 0.42, 1.0);
        apply_amphibious_maluses(&mut mob_data);
        assert!(mob_data.get_pathfinding_malus(PathType::Water).abs() < f32::EPSILON);
        assert!((mob_data.get_pathfinding_malus(PathType::Walkable) - 6.0).abs() < f32::EPSILON);
        assert!((mob_data.get_pathfinding_malus(PathType::WaterBorder) - 4.0).abs() < f32::EPSILON);
    }

    #[test]
    fn vertical_neighbor_requires_water() {
        // AmphibiousNodeEvaluator.java:84-86: valid neighbor AND type WATER.
        let current = Node::default();
        let water = Node {
            path_type: PathType::Water,
            ..Node::default()
        };
        assert!(AmphibiousNodeEvaluator::is_vertical_neighbor_valid(
            Some(&water),
            &current
        ));
        let walkable = Node {
            path_type: PathType::Walkable,
            ..water
        };
        assert!(!AmphibiousNodeEvaluator::is_vertical_neighbor_valid(
            Some(&walkable),
            &current
        ));
        assert!(!AmphibiousNodeEvaluator::is_vertical_neighbor_valid(
            None, &current
        ));
    }
}
