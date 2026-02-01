//! Swim node evaluator for water-based pathfinding.
//!
//! This module implements node evaluation for entities that swim in water,
//! handling 3D movement through water volumes.

use std::collections::HashMap;
use std::sync::Arc;

use pumpkin_data::Block;
use pumpkin_util::math::position::BlockPos;

use crate::world::World;

use super::node::Node;
use super::node_evaluator::PathfindingMob;
use super::path_type::PathType;
use super::target::Target;

/// Node evaluator for swimming entities.
///
/// Handles water-based pathfinding with full 3D movement,
/// optionally allowing breaching (jumping out of water).
pub struct SwimNodeEvaluator {
    /// Reference to the world
    world: Option<Arc<World>>,
    /// Mob configuration
    mob: PathfindingMob,
    /// Cached nodes by hash
    nodes: HashMap<i32, Node>,
    /// Cached path types by position
    path_types_cache: HashMap<i64, PathType>,
    /// Entity width in blocks (rounded up)
    entity_width: i32,
    /// Entity height in blocks (rounded up)
    entity_height: i32,
    /// Entity depth in blocks (rounded up)
    entity_depth: i32,
    /// Whether the entity can pass through doors
    can_pass_doors: bool,
    /// Whether the entity can breach (jump out of water)
    allow_breaching: bool,
}

impl SwimNodeEvaluator {
    /// Creates a new swim node evaluator.
    #[must_use]
    pub fn new(allow_breaching: bool) -> Self {
        Self {
            world: None,
            mob: PathfindingMob::default(),
            nodes: HashMap::new(),
            path_types_cache: HashMap::new(),
            entity_width: 1,
            entity_height: 1,
            entity_depth: 1,
            can_pass_doors: false,
            allow_breaching,
        }
    }

    /// Prepares the evaluator for pathfinding.
    pub fn prepare(&mut self, world: Arc<World>, mob: PathfindingMob) {
        self.nodes.clear();
        self.path_types_cache.clear();

        self.entity_width = (mob.width + 1.0).floor() as i32;
        self.entity_height = (mob.height + 1.0).floor() as i32;
        self.entity_depth = (mob.width + 1.0).floor() as i32;
        self.can_pass_doors = mob.can_pass_doors;

        self.world = Some(world);
        self.mob = mob;
    }

    /// Cleans up after pathfinding.
    pub fn done(&mut self) {
        self.world = None;
        self.nodes.clear();
        self.path_types_cache.clear();
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

    /// Gets the cached path type at a position.
    async fn get_cached_path_type(&mut self, x: i32, y: i32, z: i32) -> PathType {
        let key = BlockPos::new(x, y, z).as_long();
        if let Some(&cached) = self.path_types_cache.get(&key) {
            return cached;
        }

        let path_type = self.compute_path_type(x, y, z).await;
        self.path_types_cache.insert(key, path_type);
        path_type
    }

    /// Computes the path type at a position.
    async fn compute_path_type(&self, x: i32, y: i32, z: i32) -> PathType {
        let Some(world) = &self.world else {
            return PathType::Blocked;
        };

        for dx in 0..self.entity_width {
            for dy in 0..self.entity_height {
                for dz in 0..self.entity_depth {
                    let pos = BlockPos::new(x + dx, y + dy, z + dz);
                    let state = world.get_block_state(&pos).await;

                    if state.is_air() {
                        return PathType::Breach;
                    }

                    let block = world.get_block(&pos).await;
                    if block.id != Block::WATER.id {
                        return PathType::Blocked;
                    }
                }
            }
        }

        PathType::Water
    }

    /// Checks if a node is valid for swimming.
    fn is_node_valid(&self, node: Option<&Node>) -> bool {
        match node {
            Some(n) => !n.closed,
            None => false,
        }
    }

    /// Checks if a node has a non-negative malus.
    fn has_malus(node: Option<&Node>) -> bool {
        match node {
            Some(n) => n.cost_malus >= 0.0,
            None => false,
        }
    }

    /// Gets the starting node for pathfinding.
    pub async fn get_start(&mut self) -> Option<Node> {
        let _world = self.world.as_ref()?;
        let mob_pos = self.mob.block_pos;

        Some(self.get_node(mob_pos.0.x, mob_pos.0.y, mob_pos.0.z))
    }

    /// Gets a target node for the given coordinates.
    pub fn get_target(&mut self, x: f64, y: f64, z: f64) -> Target {
        Target::from_coords(x.floor() as i32, y.floor() as i32, z.floor() as i32)
    }

    /// Finds an accepted node at the given position for swimming.
    async fn find_accepted_node(&mut self, x: i32, y: i32, z: i32) -> Option<Node> {
        let path_type = self.get_cached_path_type(x, y, z).await;

        if !(self.allow_breaching && path_type == PathType::Breach) && path_type != PathType::Water {
            return None;
        }

        let malus = self.mob.get_pathfinding_malus(path_type);
        if malus < 0.0 {
            return None;
        }

        let mut node = self.get_node(x, y, z);
        node.path_type = path_type;
        node.cost_malus = node.cost_malus.max(malus);

        let Some(world) = &self.world else {
            return Some(node);
        };

        let pos = BlockPos::new(x, y, z);
        let state = world.get_block_state(&pos).await;
        if state.is_air() {
            node.cost_malus += 8.0;
        }

        Some(node)
    }

    /// Gets the neighboring nodes that can be reached from a given node.
    pub async fn get_neighbors(&mut self, node: &Node) -> Vec<Node> {
        let mut neighbors = Vec::with_capacity(26);

        // All 6 cardinal directions
        let cardinal_directions = [
            (1, 0, 0),
            (-1, 0, 0),
            (0, 1, 0),
            (0, -1, 0),
            (0, 0, 1),
            (0, 0, -1),
        ];

        let mut cardinal_nodes: HashMap<(i32, i32, i32), Node> = HashMap::new();

        for (dx, dy, dz) in cardinal_directions {
            if let Some(neighbor) = self
                .find_accepted_node(node.x + dx, node.y + dy, node.z + dz)
                .await
            {
                if self.is_node_valid(Some(&neighbor)) {
                    cardinal_nodes.insert((dx, dy, dz), neighbor.clone());
                    neighbors.push(neighbor);
                }
            }
        }

        // Horizontal diagonals (XZ plane)
        let horizontal_diagonals = [
            ((1, 0), (0, 1)),
            ((1, 0), (0, -1)),
            ((-1, 0), (0, 1)),
            ((-1, 0), (0, -1)),
        ];

        for ((dx1, dz1), (dx2, dz2)) in horizontal_diagonals {
            let n1 = cardinal_nodes.get(&(dx1, 0, dz1));
            let n2 = cardinal_nodes.get(&(dx2, 0, dz2));

            if Self::has_malus(n1) && Self::has_malus(n2) {
                if let Some(neighbor) = self
                    .find_accepted_node(node.x + dx1 + dx2, node.y, node.z + dz1 + dz2)
                    .await
                {
                    if self.is_node_valid(Some(&neighbor)) {
                        neighbors.push(neighbor);
                    }
                }
            }
        }

        neighbors
    }

    /// Returns whether breaching is allowed.
    #[must_use]
    pub const fn allows_breaching(&self) -> bool {
        self.allow_breaching
    }
}

impl Default for SwimNodeEvaluator {
    fn default() -> Self {
        Self::new(false)
    }
}

impl super::node_evaluator::NodeEvaluator for SwimNodeEvaluator {
    fn prepare(&mut self, context: &mut super::node_evaluator::PathfindingContext, start: BlockPos) {
        self.nodes.clear();
        self.path_types_cache.clear();

        self.entity_width = (context.mob.width + 1.0).floor() as i32;
        self.entity_height = (context.mob.height + 1.0).floor() as i32;
        self.entity_depth = (context.mob.width + 1.0).floor() as i32;
        self.can_pass_doors = context.mob.can_pass_doors;

        self.world = Some(context.world.clone());
        self.mob = context.mob.clone();
        self.mob.block_pos = start;
    }

    fn done(&mut self) {
        self.world = None;
        self.nodes.clear();
        self.path_types_cache.clear();
    }

    fn get_start(&self) -> Option<&Node> {
        let mob_pos = self.mob.block_pos;
        let hash = Node::create_hash(mob_pos.0.x, mob_pos.0.y, mob_pos.0.z);
        self.nodes.get(&hash)
    }

    fn get_target(&mut self, context: &mut super::node_evaluator::PathfindingContext, x: i32, y: i32, z: i32) -> Target {
        let _ = context;
        Target::from_coords(x, y, z)
    }

    fn get_neighbors(&mut self, context: &mut super::node_evaluator::PathfindingContext, node: &Node) -> Vec<Node> {
        let _ = context;
        let _ = node;
        Vec::new()
    }

    fn get_path_type(&mut self, context: &mut super::node_evaluator::PathfindingContext, x: i32, y: i32, z: i32) -> PathType {
        let key = BlockPos::new(x, y, z).as_long();
        if let Some(&cached) = self.path_types_cache.get(&key) {
            return cached;
        }
        let _ = context;
        PathType::Blocked
    }

    fn can_open_doors(&self) -> bool {
        false
    }

    fn can_pass_doors(&self) -> bool {
        self.can_pass_doors
    }

    fn can_float(&self) -> bool {
        true // Swimming entities always float
    }

    fn can_walk_over_fences(&self) -> bool {
        false
    }
}
