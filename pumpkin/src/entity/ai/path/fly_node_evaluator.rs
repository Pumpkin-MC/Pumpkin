//! Fly node evaluator for flying entity pathfinding.
//!
//! This module implements node evaluation for entities that can fly,
//! handling full 3D movement through air.

use std::collections::HashMap;
use std::sync::Arc;

use pumpkin_data::Block;
use pumpkin_util::math::position::BlockPos;

use crate::world::World;

use super::node::Node;
use super::node_evaluator::PathfindingMob;
use super::path_type::PathType;
use super::target::Target;

/// Node evaluator for flying entities.
///
/// Handles air-based pathfinding with full 3D movement,
/// allowing traversal through open air spaces.
pub struct FlyNodeEvaluator {
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
    /// Whether the entity can open doors
    can_open_doors: bool,
    /// Whether the entity can float in water
    can_float: bool,
}

impl FlyNodeEvaluator {
    /// Creates a new fly node evaluator.
    #[must_use]
    pub fn new() -> Self {
        Self {
            world: None,
            mob: PathfindingMob::default(),
            nodes: HashMap::new(),
            path_types_cache: HashMap::new(),
            entity_width: 1,
            entity_height: 1,
            entity_depth: 1,
            can_pass_doors: true,
            can_open_doors: false,
            can_float: false,
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
        self.can_open_doors = mob.can_open_doors;
        self.can_float = mob.can_float;

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

        let pos = BlockPos::new(x, y, z);
        let state = world.get_block_state(&pos).await;

        if state.is_air() {
            if y > world.dimension.min_y {
                let below_pos = pos.down();
                let below_block = world.get_block(&below_pos).await;

                if below_block.id == Block::LAVA.id || below_block.id == Block::FIRE.id {
                    return PathType::DangerFire;
                }
                if below_block.id == Block::CACTUS.id {
                    return PathType::DangerOther;
                }
            }
            return PathType::Open;
        }

        let block = world.get_block(&pos).await;

        if block.id == Block::LAVA.id || block.id == Block::FIRE.id {
            return PathType::DamageFire;
        }
        if block.id == Block::WATER.id {
            return PathType::Water;
        }
        if block.id == Block::CACTUS.id {
            return PathType::DamageOther;
        }

        if state.is_solid() {
            return PathType::Blocked;
        }

        PathType::Open
    }

    /// Checks if a node is open (not closed and not blocked).
    fn is_open(&self, node: Option<&Node>) -> bool {
        match node {
            Some(n) => !n.closed,
            None => false,
        }
    }

    /// Checks if a node has a non-negative malus.
    fn has_malus(&self, node: Option<&Node>) -> bool {
        match node {
            Some(n) => n.cost_malus >= 0.0,
            None => false,
        }
    }

    /// Gets the starting node for pathfinding.
    pub async fn get_start(&mut self) -> Option<Node> {
        let world = self.world.as_ref()?;
        let mob_pos = self.mob.block_pos;

        let mut y = mob_pos.0.y;

        if self.can_float {
            let state = world.get_block_state(&mob_pos).await;
            if !state.is_air() {
                let block = world.get_block(&mob_pos).await;
                if block.id == Block::WATER.id {
                    while y < world.dimension.height + world.dimension.min_y {
                        let pos = BlockPos::new(mob_pos.0.x, y, mob_pos.0.z);
                        let block = world.get_block(&pos).await;
                        if block.id != Block::WATER.id {
                            break;
                        }
                        y += 1;
                    }
                }
            }
        }

        Some(self.get_node(mob_pos.0.x, y, mob_pos.0.z))
    }

    /// Gets a target node for the given coordinates.
    pub fn get_target(&mut self, x: f64, y: f64, z: f64) -> Target {
        Target::from_coords(x.floor() as i32, y.floor() as i32, z.floor() as i32)
    }

    /// Finds an accepted node at the given position for flying.
    async fn find_accepted_node(&mut self, x: i32, y: i32, z: i32) -> Option<Node> {
        let path_type = self.get_cached_path_type(x, y, z).await;
        let malus = self.mob.get_pathfinding_malus(path_type);

        if malus < 0.0 {
            return None;
        }

        let mut node = self.get_node(x, y, z);
        node.path_type = path_type;
        node.cost_malus = node.cost_malus.max(malus);

        // Flying entities prefer air over walkable ground
        if path_type == PathType::Walkable {
            node.cost_malus += 1.0;
        }

        Some(node)
    }

    /// Gets the neighboring nodes that can be reached from a given node.
    pub async fn get_neighbors(&mut self, node: &Node) -> Vec<Node> {
        let mut neighbors = Vec::with_capacity(26);

        // All 6 cardinal directions
        let cardinal_nodes: [Option<Node>; 6] = [
            self.find_accepted_node(node.x, node.y, node.z + 1).await, // South
            self.find_accepted_node(node.x - 1, node.y, node.z).await, // West
            self.find_accepted_node(node.x + 1, node.y, node.z).await, // East
            self.find_accepted_node(node.x, node.y, node.z - 1).await, // North
            self.find_accepted_node(node.x, node.y + 1, node.z).await, // Up
            self.find_accepted_node(node.x, node.y - 1, node.z).await, // Down
        ];

        for cn in &cardinal_nodes {
            if self.is_open(cn.as_ref()) {
                if let Some(n) = cn {
                    neighbors.push(n.clone());
                }
            }
        }

        // Horizontal diagonals
        let horiz_diagonals = [
            (2, 3, 1, -1),  // East + North
            (2, 0, 1, 1),   // East + South
            (1, 3, -1, -1), // West + North
            (1, 0, -1, 1),  // West + South
        ];

        for (idx1, idx2, dx, dz) in horiz_diagonals {
            if self.has_malus(cardinal_nodes[idx1].as_ref())
                && self.has_malus(cardinal_nodes[idx2].as_ref())
            {
                if let Some(neighbor) = self
                    .find_accepted_node(node.x + dx, node.y, node.z + dz)
                    .await
                {
                    if self.is_open(Some(&neighbor)) {
                        neighbors.push(neighbor);
                    }
                }
            }
        }

        neighbors
    }

    /// Returns whether the entity can float in water.
    #[must_use]
    pub const fn can_float(&self) -> bool {
        self.can_float
    }

    /// Sets whether the entity can float in water.
    pub fn set_can_float(&mut self, can_float: bool) {
        self.can_float = can_float;
    }
}

impl Default for FlyNodeEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl super::node_evaluator::NodeEvaluator for FlyNodeEvaluator {
    fn prepare(
        &mut self,
        context: &mut super::node_evaluator::PathfindingContext,
        start: BlockPos,
    ) {
        self.nodes.clear();
        self.path_types_cache.clear();

        self.entity_width = (context.mob.width + 1.0).floor() as i32;
        self.entity_height = (context.mob.height + 1.0).floor() as i32;
        self.entity_depth = (context.mob.width + 1.0).floor() as i32;
        self.can_pass_doors = context.mob.can_pass_doors;
        self.can_open_doors = context.mob.can_open_doors;
        self.can_float = context.mob.can_float;

        self.world = Some(context.world.clone());
        self.mob = context.mob.clone();
        self.mob.block_pos = start;
    }

    fn done(&mut self) {
        self.world = None;
        self.nodes.clear();
        self.path_types_cache.clear();
    }

    fn get_start(&self) -> Option<&super::node::Node> {
        let mob_pos = self.mob.block_pos;
        let hash = super::node::Node::create_hash(mob_pos.0.x, mob_pos.0.y, mob_pos.0.z);
        self.nodes.get(&hash)
    }

    fn get_target(
        &mut self,
        context: &mut super::node_evaluator::PathfindingContext,
        x: i32,
        y: i32,
        z: i32,
    ) -> Target {
        let _ = context;
        Target::from_coords(x, y, z)
    }

    fn get_neighbors(
        &mut self,
        context: &mut super::node_evaluator::PathfindingContext,
        node: &super::node::Node,
    ) -> Vec<super::node::Node> {
        let _ = context;
        let _ = node;
        Vec::new()
    }

    fn get_path_type(
        &mut self,
        context: &mut super::node_evaluator::PathfindingContext,
        x: i32,
        y: i32,
        z: i32,
    ) -> PathType {
        let key = BlockPos::new(x, y, z).as_long();
        if let Some(&cached) = self.path_types_cache.get(&key) {
            return cached;
        }
        let _ = context;
        PathType::Blocked
    }

    fn can_open_doors(&self) -> bool {
        self.can_open_doors
    }

    fn can_pass_doors(&self) -> bool {
        self.can_pass_doors
    }

    fn can_float(&self) -> bool {
        self.can_float
    }

    fn can_walk_over_fences(&self) -> bool {
        false
    }
}
