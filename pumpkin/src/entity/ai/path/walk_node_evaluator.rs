//! Walk node evaluator for ground-based pathfinding.
//!
//! This module implements node evaluation for entities that walk on the ground,
//! handling stairs, jumps, falls, and obstacles.

use std::collections::HashMap;
use std::sync::Arc;

use pumpkin_data::Block;
use pumpkin_util::math::position::BlockPos;

use crate::world::World;

use super::node::Node;
use super::node_evaluator::PathfindingMob;
use super::path_type::PathType;
use super::target::Target;

/// Default mob jump height in blocks.
const DEFAULT_MOB_JUMP_HEIGHT: f64 = 1.125;

/// Node evaluator for walking entities.
///
/// Handles ground-based pathfinding including walking, jumping up blocks,
/// falling, and navigating around obstacles.
pub struct WalkNodeEvaluator {
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
    /// Whether the entity can walk over fences
    can_walk_over_fences: bool,
}

impl WalkNodeEvaluator {
    /// Creates a new walk node evaluator.
    #[must_use]
    pub fn new() -> Self {
        Self {
            world: None,
            mob: PathfindingMob::default(),
            nodes: HashMap::new(),
            path_types_cache: HashMap::new(),
            entity_width: 1,
            entity_height: 2,
            entity_depth: 1,
            can_pass_doors: true,
            can_open_doors: false,
            can_float: false,
            can_walk_over_fences: false,
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
        self.can_walk_over_fences = mob.can_walk_over_fences;

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
    pub async fn get_cached_path_type(&mut self, x: i32, y: i32, z: i32) -> PathType {
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
                let below_state = world.get_block_state(&below_pos).await;

                if below_state.is_air() {
                    return PathType::Open;
                }

                let below_block = world.get_block(&below_pos).await;

                if below_block.id == Block::LAVA.id {
                    return PathType::DangerFire;
                }
                if below_block.id == Block::CACTUS.id || below_block.id == Block::SWEET_BERRY_BUSH.id {
                    return PathType::DangerOther;
                }

                if !below_state.is_air() {
                    return PathType::Walkable;
                }
            }
            return PathType::Open;
        }

        let block = world.get_block(&pos).await;

        if block.id == Block::WATER.id {
            return PathType::Water;
        }
        if block.id == Block::LAVA.id {
            return PathType::Lava;
        }
        if block.id == Block::POWDER_SNOW.id {
            return PathType::PowderSnow;
        }
        if block.id == Block::HONEY_BLOCK.id {
            return PathType::StickyHoney;
        }
        if block.id == Block::CACTUS.id || block.id == Block::SWEET_BERRY_BUSH.id {
            return PathType::DamageOther;
        }

        if state.is_solid() {
            PathType::Blocked
        } else {
            PathType::Open
        }
    }

    /// Gets the floor level at a position.
    async fn get_floor_level(&self, pos: BlockPos) -> f64 {
        let Some(world) = &self.world else {
            return pos.0.y as f64;
        };

        let state = world.get_block_state(&pos).await;
        if !state.is_air() {
            let block = world.get_block(&pos).await;
            if block.id == Block::WATER.id && self.can_float {
                return pos.0.y as f64 + 0.5;
            }
        }

        let below = pos.down();
        let below_state = world.get_block_state(&below).await;

        if below_state.is_air() {
            return below.0.y as f64;
        }

        below.0.y as f64 + 1.0
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
                        let state = world.get_block_state(&pos).await;
                        if state.is_air() {
                            break;
                        }
                        let block = world.get_block(&pos).await;
                        if block.id != Block::WATER.id {
                            y -= 1;
                            break;
                        }
                        y += 1;
                    }
                }
            }
        }

        let mut node = self.get_node(mob_pos.0.x, y, mob_pos.0.z);
        node.path_type = self.get_cached_path_type(mob_pos.0.x, y, mob_pos.0.z).await;
        node.cost_malus = self.mob.get_pathfinding_malus(node.path_type);

        Some(node)
    }

    /// Gets a target node for the given coordinates.
    pub fn get_target(&mut self, x: f64, y: f64, z: f64) -> Target {
        Target::from_coords(x.floor() as i32, y.floor() as i32, z.floor() as i32)
    }

    /// Gets the neighboring nodes that can be reached from a given node.
    pub async fn get_neighbors(&mut self, node: &Node) -> Vec<Node> {
        let Some(_world) = &self.world else {
            return Vec::new();
        };

        let mut neighbors = Vec::with_capacity(8);

        let path_type_above = self.get_cached_path_type(node.x, node.y + 1, node.z).await;
        let path_type = self.get_cached_path_type(node.x, node.y, node.z).await;

        let max_up_step = if self.mob.get_pathfinding_malus(path_type_above) >= 0.0
            && path_type != PathType::StickyHoney
        {
            self.mob.max_up_step.floor().max(1.0) as i32
        } else {
            0
        };

        let floor_level = self.get_floor_level(BlockPos::new(node.x, node.y, node.z)).await;

        // Cardinal directions
        let directions = [(0, -1), (0, 1), (-1, 0), (1, 0)];
        let mut cardinal_nodes: [Option<Node>; 4] = [None, None, None, None];

        for (i, (dx, dz)) in directions.iter().enumerate() {
            if let Some(neighbor) = self
                .find_accepted_node(node.x + dx, node.y, node.z + dz, max_up_step, floor_level)
                .await
            {
                if self.is_neighbor_valid(Some(&neighbor), node) {
                    cardinal_nodes[i] = Some(neighbor.clone());
                    neighbors.push(neighbor);
                }
            }
        }

        // Diagonal directions
        let diagonals = [(1, -1, 0, 3), (1, 1, 1, 3), (-1, -1, 0, 2), (-1, 1, 1, 2)];

        for (dx, dz, n1_idx, n2_idx) in diagonals {
            if self.is_diagonal_valid(node, cardinal_nodes[n1_idx].as_ref(), cardinal_nodes[n2_idx].as_ref()) {
                if let Some(neighbor) = self
                    .find_accepted_node(node.x + dx, node.y, node.z + dz, max_up_step, floor_level)
                    .await
                {
                    if neighbor.cost_malus >= 0.0 && !neighbor.closed {
                        neighbors.push(neighbor);
                    }
                }
            }
        }

        neighbors
    }

    fn is_neighbor_valid(&self, node: Option<&Node>, from: &Node) -> bool {
        match node {
            Some(n) => !n.closed && (n.cost_malus >= 0.0 || from.cost_malus < 0.0),
            None => false,
        }
    }

    fn is_diagonal_valid(&self, from: &Node, n1: Option<&Node>, n2: Option<&Node>) -> bool {
        let (Some(n1), Some(n2)) = (n1, n2) else {
            return false;
        };

        if n1.y > from.y || n2.y > from.y {
            return false;
        }

        if n1.path_type == PathType::WalkableDoor || n2.path_type == PathType::WalkableDoor {
            return false;
        }

        let n1_ok = n1.y < from.y || n1.cost_malus >= 0.0;
        let n2_ok = n2.y < from.y || n2.cost_malus >= 0.0;

        n1_ok && n2_ok
    }

    async fn find_accepted_node(
        &mut self,
        x: i32,
        y: i32,
        z: i32,
        max_up_step: i32,
        floor_level: f64,
    ) -> Option<Node> {
        let pos = BlockPos::new(x, y, z);
        let new_floor_level = self.get_floor_level(pos).await;
        let mob_jump_height = DEFAULT_MOB_JUMP_HEIGHT.max(self.mob.max_up_step as f64);

        if new_floor_level - floor_level > mob_jump_height {
            return None;
        }

        let path_type = self.get_cached_path_type(x, y, z).await;
        let malus = self.mob.get_pathfinding_malus(path_type);

        if malus >= 0.0 {
            let mut node = self.get_node(x, y, z);
            node.path_type = path_type;
            node.cost_malus = node.cost_malus.max(malus);
            return Some(node);
        }

        if path_type != PathType::Fence
            && path_type != PathType::UnpassableRail
            && path_type != PathType::Trapdoor
            && path_type != PathType::PowderSnow
            && max_up_step > 0
        {
            return Box::pin(self.find_accepted_node(x, y + 1, z, max_up_step - 1, floor_level)).await;
        }

        if path_type == PathType::Open {
            return self.try_find_ground_node(x, y, z).await;
        }

        None
    }

    async fn try_find_ground_node(&mut self, x: i32, y: i32, z: i32) -> Option<Node> {
        let Some(world) = &self.world else {
            return None;
        };

        let max_fall = self.mob.max_fall_distance;
        let min_y = world.dimension.min_y;

        for dy in 1..=max_fall {
            let new_y = y - dy;
            if new_y < min_y {
                break;
            }

            let path_type = self.get_cached_path_type(x, new_y, z).await;

            if path_type != PathType::Open {
                let malus = self.mob.get_pathfinding_malus(path_type);
                if malus >= 0.0 {
                    let mut node = self.get_node(x, new_y, z);
                    node.path_type = path_type;
                    node.cost_malus = node.cost_malus.max(malus);
                    return Some(node);
                }
                let mut node = self.get_node(x, new_y, z);
                node.path_type = PathType::Blocked;
                node.cost_malus = -1.0;
                return Some(node);
            }
        }

        let mut node = self.get_node(x, y - max_fall, z);
        node.path_type = PathType::Blocked;
        node.cost_malus = -1.0;
        Some(node)
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

    /// Returns whether the entity can open doors.
    #[must_use]
    pub const fn can_open_doors(&self) -> bool {
        self.can_open_doors
    }

    /// Sets whether the entity can open doors.
    pub fn set_can_open_doors(&mut self, can_open_doors: bool) {
        self.can_open_doors = can_open_doors;
    }

    /// Returns whether the entity can pass through doors.
    #[must_use]
    pub const fn can_pass_doors(&self) -> bool {
        self.can_pass_doors
    }

    /// Sets whether the entity can pass through doors.
    pub fn set_can_pass_doors(&mut self, can_pass_doors: bool) {
        self.can_pass_doors = can_pass_doors;
    }

    /// Returns whether the entity can walk over fences.
    #[must_use]
    pub const fn can_walk_over_fences(&self) -> bool {
        self.can_walk_over_fences
    }

    /// Sets whether the entity can walk over fences.
    pub fn set_can_walk_over_fences(&mut self, can_walk_over_fences: bool) {
        self.can_walk_over_fences = can_walk_over_fences;
    }
}

impl Default for WalkNodeEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl super::node_evaluator::NodeEvaluator for WalkNodeEvaluator {
    fn prepare(&mut self, context: &mut super::node_evaluator::PathfindingContext, start: BlockPos) {
        self.nodes.clear();
        self.path_types_cache.clear();

        self.entity_width = (context.mob.width + 1.0).floor() as i32;
        self.entity_height = (context.mob.height + 1.0).floor() as i32;
        self.entity_depth = (context.mob.width + 1.0).floor() as i32;
        self.can_pass_doors = context.mob.can_pass_doors;
        self.can_open_doors = context.mob.can_open_doors;
        self.can_float = context.mob.can_float;
        self.can_walk_over_fences = context.mob.can_walk_over_fences;

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
        // Return a reference to a cached start node if available
        let mob_pos = self.mob.block_pos;
        let hash = Node::create_hash(mob_pos.0.x, mob_pos.0.y, mob_pos.0.z);
        self.nodes.get(&hash)
    }

    fn get_target(&mut self, context: &mut super::node_evaluator::PathfindingContext, x: i32, y: i32, z: i32) -> Target {
        let _ = context;
        Target::from_coords(x, y, z)
    }

    fn get_neighbors(&mut self, context: &mut super::node_evaluator::PathfindingContext, node: &Node) -> Vec<Node> {
        // Synchronous version - returns empty vec, async version should be used
        let _ = context;
        let _ = node;
        Vec::new()
    }

    fn get_path_type(&mut self, context: &mut super::node_evaluator::PathfindingContext, x: i32, y: i32, z: i32) -> PathType {
        let key = BlockPos::new(x, y, z).as_long();
        if let Some(&cached) = self.path_types_cache.get(&key) {
            return cached;
        }
        // For synchronous access, return a default; async version should compute properly
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
        self.can_walk_over_fences
    }
}
