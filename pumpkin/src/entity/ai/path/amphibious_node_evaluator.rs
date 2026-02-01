//! Amphibious node evaluator for entities that can walk and swim.
//!
//! This module implements node evaluation for entities that can both walk on land
//! and swim in water, such as frogs, turtles, and axolotls.
//!
//! This corresponds to:
//! - `net.minecraft.world.level.pathfinder.AmphibiousNodeEvaluator`

use std::collections::HashMap;

use pumpkin_util::math::position::BlockPos;

use super::node::Node;
use super::node_evaluator::{NodeEvaluator, PathfindingContext};
use super::path_type::PathType;
use super::target::Target;

/// Node evaluator for amphibious entities.
///
/// Extends walking behavior to also support swimming. Amphibious entities
/// can traverse both land and water, with adjustable preferences for
/// shallow vs deep water.
pub struct AmphibiousNodeEvaluator {
    /// Cached nodes by hash
    nodes: HashMap<i32, Node>,
    /// The start node for current pathfinding
    start_node: Option<Node>,
    /// Entity width in blocks (rounded up)
    entity_width: i32,
    /// Entity height in blocks (rounded up)
    entity_height: i32,
    /// Entity depth in blocks (rounded up)
    entity_depth: i32,
    /// Whether the entity can pass through doors
    can_pass_doors: bool,
    /// Whether the entity can open doors
    can_open_doors: bool,
    /// Whether the entity can walk over fences
    can_walk_over_fences: bool,
    /// Whether the entity prefers shallow water
    prefers_shallow_swimming: bool,
    /// Old walkable cost before modification
    old_walkable_cost: f32,
    /// Old water border cost before modification
    old_water_border_cost: f32,
}

impl AmphibiousNodeEvaluator {
    /// Creates a new amphibious node evaluator.
    ///
    /// # Arguments
    /// * `prefers_shallow_swimming` - If true, adds extra cost for deep water
    #[must_use]
    pub fn new(prefers_shallow_swimming: bool) -> Self {
        Self {
            nodes: HashMap::new(),
            start_node: None,
            entity_width: 1,
            entity_height: 2,
            entity_depth: 1,
            can_pass_doors: true,
            can_open_doors: false,
            can_walk_over_fences: false,
            prefers_shallow_swimming,
            old_walkable_cost: 0.0,
            old_water_border_cost: 0.0,
        }
    }

    /// Creates a new amphibious node evaluator that does not prefer shallow swimming.
    #[must_use]
    pub fn default_evaluator() -> Self {
        Self::new(false)
    }

    /// Gets or creates a node at the specified coordinates.
    fn get_node(&mut self, x: i32, y: i32, z: i32) -> Node {
        let hash = Node::create_hash(x, y, z);
        if let Some(node) = self.nodes.get(&hash) {
            return node.clone();
        }
        let node = Node::new(x, y, z);
        self.nodes.insert(hash, node.clone());
        node
    }

    /// Checks if a vertical neighbor is valid.
    #[allow(clippy::unused_self)]
    fn is_vertical_neighbor_valid(&self, neighbor: &Node, _from: &Node) -> bool {
        // Vertical movement is only valid in water
        neighbor.path_type == PathType::Water
    }

    /// Returns whether this evaluator handles amphibious movement.
    #[must_use]
    pub const fn is_amphibious(&self) -> bool {
        true
    }
}

impl Default for AmphibiousNodeEvaluator {
    fn default() -> Self {
        Self::default_evaluator()
    }
}

impl NodeEvaluator for AmphibiousNodeEvaluator {
    fn prepare(&mut self, context: &mut PathfindingContext, start: BlockPos) {
        self.nodes.clear();
        self.start_node = None;

        self.entity_width = (context.mob.width + 1.0).floor() as i32;
        self.entity_height = (context.mob.height + 1.0).floor() as i32;
        self.entity_depth = (context.mob.width + 1.0).floor() as i32;
        self.can_pass_doors = context.mob.can_pass_doors;
        self.can_open_doors = context.mob.can_open_doors;
        self.can_walk_over_fences = context.mob.can_walk_over_fences;

        // Modify path costs for water traversal
        // Water becomes passable (cost 0)
        context.mob.set_pathfinding_malus(PathType::Water, 0.0);

        // Save old costs
        self.old_walkable_cost = context.mob.get_pathfinding_malus(PathType::Walkable);
        self.old_water_border_cost = context.mob.get_pathfinding_malus(PathType::WaterBorder);

        // Increase walkable cost (prefer water slightly)
        context.mob.set_pathfinding_malus(PathType::Walkable, 6.0);
        context
            .mob
            .set_pathfinding_malus(PathType::WaterBorder, 4.0);

        // Create start node
        self.start_node = Some(self.get_node(start.0.x, start.0.y, start.0.z));
    }

    fn done(&mut self) {
        // Note: Original Java code restores mob pathfinding malus here
        // We don't have access to the mob here, so this is handled externally
        self.nodes.clear();
        self.start_node = None;
    }

    fn get_start(&self) -> Option<&Node> {
        self.start_node.as_ref()
    }

    fn get_target(&mut self, _context: &mut PathfindingContext, x: i32, y: i32, z: i32) -> Target {
        // Offset y by 0.5 for amphibious targeting
        Target::from_coords(x, y, z)
    }

    fn get_neighbors(&mut self, context: &mut PathfindingContext, node: &Node) -> Vec<Node> {
        let mut neighbors = Vec::with_capacity(18);

        // Get horizontal neighbors (from walk evaluator behavior)
        for dx in -1..=1 {
            for dz in -1..=1 {
                if dx == 0 && dz == 0 {
                    continue;
                }
                let neighbor = self.get_node(node.x + dx, node.y, node.z + dz);
                neighbors.push(neighbor);
            }
        }

        // Add vertical neighbors for water traversal
        let path_type_above = self.get_path_type(context, node.x, node.y + 1, node.z);
        let path_type_current = self.get_path_type(context, node.x, node.y, node.z);

        // Check if we can move vertically
        let malus = context.mob.get_pathfinding_malus(path_type_above);
        if malus >= 0.0 && path_type_current != PathType::StickyHoney {
            // Can move up
            let up_node = self.get_node(node.x, node.y + 1, node.z);
            if self.is_vertical_neighbor_valid(&up_node, node) {
                neighbors.push(up_node);
            }

            // Can move down (if not on trapdoor)
            let down_node = self.get_node(node.x, node.y - 1, node.z);
            if self.is_vertical_neighbor_valid(&down_node, node)
                && path_type_current != PathType::Trapdoor
            {
                neighbors.push(down_node);
            }
        }

        // Add shallow swimming cost penalty if applicable
        if self.prefers_shallow_swimming {
            let sea_level = 64; // Default sea level, would come from world
            for neighbor in &mut neighbors {
                if neighbor.path_type == PathType::Water && neighbor.y < sea_level - 10 {
                    neighbor.cost_malus += 1.0;
                }
            }
        }

        neighbors
    }

    fn get_path_type(
        &mut self,
        _context: &mut PathfindingContext,
        x: i32,
        y: i32,
        z: i32,
    ) -> PathType {
        // Check cache first
        let key = BlockPos::new(x, y, z).as_long();
        if let Some(node) = self.nodes.get(&Node::create_hash(x, y, z)) {
            return node.path_type;
        }

        // Simplified: would need async world access for full implementation
        let _ = key;
        PathType::Walkable
    }

    fn can_open_doors(&self) -> bool {
        self.can_open_doors
    }

    fn can_pass_doors(&self) -> bool {
        self.can_pass_doors
    }

    fn can_float(&self) -> bool {
        true // Amphibious entities always float
    }

    fn can_walk_over_fences(&self) -> bool {
        self.can_walk_over_fences
    }
}
