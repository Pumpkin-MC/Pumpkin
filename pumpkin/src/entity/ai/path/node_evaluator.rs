//! Node evaluator types for pathfinding.
//!
//! This module defines the types used for evaluating pathfinding nodes
//! and determining traversal costs.

use std::collections::HashMap;
use std::sync::Arc;

use pumpkin_util::math::position::BlockPos;

use crate::world::World;

use super::path_type::PathType;

/// Configuration for a pathfinding entity.
///
/// Contains entity dimensions and capabilities that affect pathfinding.
#[derive(Clone, Debug)]
pub struct PathfindingMob {
    /// Entity width in blocks
    pub width: f32,
    /// Entity height in blocks
    pub height: f32,
    /// Current block position
    pub block_pos: BlockPos,
    /// Whether the entity can open doors
    pub can_open_doors: bool,
    /// Whether the entity can pass through doors
    pub can_pass_doors: bool,
    /// Whether the entity can float in water
    pub can_float: bool,
    /// Whether the entity can walk over fences
    pub can_walk_over_fences: bool,
    /// Maximum height the entity can step up
    pub max_up_step: f32,
    /// Maximum distance the entity can fall
    pub max_fall_distance: i32,
    /// Custom pathfinding malus values for specific path types
    pub pathfinding_malus: HashMap<PathType, f32>,
}

impl PathfindingMob {
    /// Creates a new pathfinding mob configuration.
    #[must_use]
    pub fn new(width: f32, height: f32, block_pos: BlockPos) -> Self {
        Self {
            width,
            height,
            block_pos,
            can_open_doors: false,
            can_pass_doors: true,
            can_float: false,
            can_walk_over_fences: false,
            max_up_step: 0.6,
            max_fall_distance: 3,
            pathfinding_malus: HashMap::new(),
        }
    }

    /// Gets the pathfinding malus for a specific path type.
    ///
    /// Returns the custom malus if set, otherwise the default for the path type.
    #[must_use]
    pub fn get_pathfinding_malus(&self, path_type: PathType) -> f32 {
        self.pathfinding_malus
            .get(&path_type)
            .copied()
            .unwrap_or_else(|| path_type.malus())
    }

    /// Sets a custom pathfinding malus for a path type.
    pub fn set_pathfinding_malus(&mut self, path_type: PathType, malus: f32) {
        self.pathfinding_malus.insert(path_type, malus);
    }
}

impl Default for PathfindingMob {
    fn default() -> Self {
        Self::new(0.6, 1.8, BlockPos::ZERO)
    }
}

/// Context for pathfinding operations.
///
/// Provides access to the world and caches path type lookups.
pub struct PathfindingContext {
    /// Reference to the world
    pub world: Arc<World>,
    /// Cached path types by position
    path_type_cache: HashMap<i64, PathType>,
    /// The mob being pathfound
    pub mob: PathfindingMob,
}

impl PathfindingContext {
    /// Creates a new pathfinding context.
    #[must_use]
    pub fn new(world: Arc<World>, mob: PathfindingMob) -> Self {
        Self {
            world,
            path_type_cache: HashMap::new(),
            mob,
        }
    }

    /// Clears the path type cache.
    pub fn clear_cache(&mut self) {
        self.path_type_cache.clear();
    }

    /// Gets the cached path type at a position, or computes and caches it.
    pub async fn get_path_type_cached(&mut self, x: i32, y: i32, z: i32) -> PathType {
        let key = BlockPos::new(x, y, z).as_long();
        if let Some(&cached) = self.path_type_cache.get(&key) {
            return cached;
        }

        let path_type = self.compute_path_type(x, y, z).await;
        self.path_type_cache.insert(key, path_type);
        path_type
    }

    /// Computes the path type at a position.
    async fn compute_path_type(&self, x: i32, y: i32, z: i32) -> PathType {
        let pos = BlockPos::new(x, y, z);
        let state = self.world.get_block_state(&pos).await;

        // Air is open
        if state.is_air() {
            return PathType::Open;
        }

        // Check for special blocks
        let block = self.world.get_block(&pos).await;

        // Water
        if block.id == pumpkin_data::Block::WATER.id {
            return PathType::Water;
        }

        // Lava
        if block.id == pumpkin_data::Block::LAVA.id {
            return PathType::Lava;
        }

        // Solid blocks
        if state.is_solid() {
            // Check if walkable on top
            let above_pos = pos.up();
            let above_state = self.world.get_block_state(&above_pos).await;
            if above_state.is_air() {
                return PathType::Walkable;
            }
            return PathType::Blocked;
        }

        PathType::Open
    }

    /// Gets the block state at a position.
    pub async fn get_block_state(&self, pos: &BlockPos) -> &'static pumpkin_data::BlockState {
        self.world.get_block_state(pos).await
    }

    /// Gets the minimum Y level of the world.
    #[must_use]
    pub fn min_y(&self) -> i32 {
        self.world.dimension.min_y
    }

    /// Gets the maximum Y level of the world.
    #[must_use]
    pub fn max_y(&self) -> i32 {
        self.world.dimension.height + self.world.dimension.min_y
    }
}

/// Trait for evaluating nodes during pathfinding.
///
/// Different implementations handle different movement types (walking, swimming, flying).
pub trait NodeEvaluator {
    /// Prepares the evaluator for pathfinding from a starting position.
    fn prepare(&mut self, context: &mut PathfindingContext, start: BlockPos);

    /// Cleans up after pathfinding is complete.
    fn done(&mut self);

    /// Gets the starting node for pathfinding.
    fn get_start(&self) -> Option<&super::node::Node>;

    /// Gets a target node at the specified position.
    fn get_target(&mut self, context: &mut PathfindingContext, x: i32, y: i32, z: i32) -> super::target::Target;

    /// Gets all neighbor nodes of the given node.
    fn get_neighbors(&mut self, context: &mut PathfindingContext, node: &super::node::Node) -> Vec<super::node::Node>;

    /// Gets the path type at a position.
    fn get_path_type(&mut self, context: &mut PathfindingContext, x: i32, y: i32, z: i32) -> PathType;

    /// Gets the path type for the cached position.
    fn get_path_type_cached(&mut self, context: &mut PathfindingContext, x: i32, y: i32, z: i32) -> PathType {
        // Default implementation delegates to get_path_type
        self.get_path_type(context, x, y, z)
    }

    /// Returns whether the entity can open doors.
    fn can_open_doors(&self) -> bool {
        false
    }

    /// Returns whether the entity can pass through doors.
    fn can_pass_doors(&self) -> bool {
        true
    }

    /// Returns whether the entity can float in water.
    fn can_float(&self) -> bool {
        false
    }

    /// Returns whether the entity can walk over fences.
    fn can_walk_over_fences(&self) -> bool {
        false
    }
}
