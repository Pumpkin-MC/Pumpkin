//! Pathfinding node representation.
//!
//! This module defines the `Node` struct used in A* pathfinding, representing
//! a single position in the pathfinding graph with associated costs and metadata.

use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

use super::path_type::PathType;

/// A node in the pathfinding graph.
///
/// Each node represents a block position with associated pathfinding metadata
/// including costs, parent reference, and path type information.
#[derive(Clone, Debug)]
pub struct Node {
    /// X coordinate in block space
    pub x: i32,
    /// Y coordinate in block space
    pub y: i32,
    /// Z coordinate in block space
    pub z: i32,
    /// Precomputed hash for quick lookup
    hash: i32,
    /// Index in the binary heap (-1 if not in open set)
    pub heap_idx: i32,
    /// Cost from start to this node (g-score)
    pub g: f32,
    /// Heuristic cost from this node to goal (h-score)
    pub h: f32,
    /// Total estimated cost (f-score = g + h)
    pub f: f32,
    /// Parent node index for path reconstruction
    pub came_from: Option<usize>,
    /// Whether this node has been fully evaluated
    pub closed: bool,
    /// Total distance walked to reach this node
    pub walked_distance: f32,
    /// Additional cost penalty for this node
    pub cost_malus: f32,
    /// The type of path at this position
    pub path_type: PathType,
}

impl Node {
    /// Creates a new node at the specified coordinates.
    #[must_use]
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self {
            x,
            y,
            z,
            hash: Self::create_hash(x, y, z),
            heap_idx: -1,
            g: 0.0,
            h: 0.0,
            f: 0.0,
            came_from: None,
            closed: false,
            walked_distance: 0.0,
            cost_malus: 0.0,
            path_type: PathType::Blocked,
        }
    }

    /// Creates a new node from a block position.
    #[must_use]
    pub fn from_block_pos(pos: BlockPos) -> Self {
        Self::new(pos.0.x, pos.0.y, pos.0.z)
    }

    /// Creates a hash value for the given coordinates.
    ///
    /// This hash is used for quick node lookup in hash maps.
    #[must_use]
    pub const fn create_hash(x: i32, y: i32, z: i32) -> i32 {
        // Pack coordinates into a single i32:
        // y: bits 0-7 (256 values)
        // x: bits 8-22 (15 bits, signed with bit 31)
        // z: bits 24-38 (15 bits, signed with bit 15)
        let mut hash = (y & 0xFF) | ((x & 0x7FFF) << 8) | ((z & 0x7FFF) << 24);
        if x < 0 {
            hash |= i32::MIN;
        }
        if z < 0 {
            hash |= 0x8000;
        }
        hash
    }

    /// Returns the hash value for this node.
    #[must_use]
    pub const fn hash_code(&self) -> i32 {
        self.hash
    }

    /// Calculates the Euclidean distance to another node.
    #[must_use]
    pub fn distance_to(&self, other: &Node) -> f32 {
        let dx = (other.x - self.x) as f32;
        let dy = (other.y - self.y) as f32;
        let dz = (other.z - self.z) as f32;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// Calculates the horizontal (XZ plane) distance to another node.
    #[must_use]
    pub fn distance_to_xz(&self, other: &Node) -> f32 {
        let dx = (other.x - self.x) as f32;
        let dz = (other.z - self.z) as f32;
        (dx * dx + dz * dz).sqrt()
    }

    /// Calculates the Euclidean distance to a block position.
    #[must_use]
    pub fn distance_to_pos(&self, pos: BlockPos) -> f32 {
        let dx = (pos.0.x - self.x) as f32;
        let dy = (pos.0.y - self.y) as f32;
        let dz = (pos.0.z - self.z) as f32;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// Calculates the squared Euclidean distance to another node.
    #[must_use]
    pub fn distance_to_sqr(&self, other: &Node) -> f32 {
        let dx = (other.x - self.x) as f32;
        let dy = (other.y - self.y) as f32;
        let dz = (other.z - self.z) as f32;
        dx * dx + dy * dy + dz * dz
    }

    /// Calculates the squared Euclidean distance to a block position.
    #[must_use]
    pub fn distance_to_pos_sqr(&self, pos: BlockPos) -> f32 {
        let dx = (pos.0.x - self.x) as f32;
        let dy = (pos.0.y - self.y) as f32;
        let dz = (pos.0.z - self.z) as f32;
        dx * dx + dy * dy + dz * dz
    }

    /// Calculates the Manhattan distance to another node.
    #[must_use]
    pub fn distance_manhattan(&self, other: &Node) -> f32 {
        let dx = (other.x - self.x).abs() as f32;
        let dy = (other.y - self.y).abs() as f32;
        let dz = (other.z - self.z).abs() as f32;
        dx + dy + dz
    }

    /// Calculates the Manhattan distance to a block position.
    #[must_use]
    pub fn distance_manhattan_pos(&self, pos: BlockPos) -> f32 {
        let dx = (pos.0.x - self.x).abs() as f32;
        let dy = (pos.0.y - self.y).abs() as f32;
        let dz = (pos.0.z - self.z).abs() as f32;
        dx + dy + dz
    }

    /// Converts this node to a block position.
    #[must_use]
    pub const fn as_block_pos(&self) -> BlockPos {
        BlockPos::new(self.x, self.y, self.z)
    }

    /// Converts this node to a Vector3.
    #[must_use]
    pub const fn as_vec3(&self) -> Vector3<i32> {
        Vector3::new(self.x, self.y, self.z)
    }

    /// Converts this node to a Vector3<f64> (block center at bottom).
    #[must_use]
    pub fn as_vec3_f64(&self) -> Vector3<f64> {
        Vector3::new(
            self.x as f64 + 0.5,
            self.y as f64,
            self.z as f64 + 0.5,
        )
    }

    /// Returns whether this node is in the open set (binary heap).
    #[must_use]
    pub const fn in_open_set(&self) -> bool {
        self.heap_idx >= 0
    }

    /// Creates a clone of this node moved to new coordinates.
    #[must_use]
    pub fn clone_and_move(&self, x: i32, y: i32, z: i32) -> Self {
        let mut new_node = Self::new(x, y, z);
        new_node.heap_idx = self.heap_idx;
        new_node.g = self.g;
        new_node.h = self.h;
        new_node.f = self.f;
        new_node.came_from = self.came_from;
        new_node.closed = self.closed;
        new_node.walked_distance = self.walked_distance;
        new_node.cost_malus = self.cost_malus;
        new_node.path_type = self.path_type;
        new_node
    }

    /// Resets this node for reuse in pathfinding.
    pub fn reset(&mut self) {
        self.heap_idx = -1;
        self.g = 0.0;
        self.h = 0.0;
        self.f = 0.0;
        self.came_from = None;
        self.closed = false;
        self.walked_distance = 0.0;
        self.cost_malus = 0.0;
        self.path_type = PathType::Blocked;
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash && self.x == other.x && self.y == other.y && self.z == other.z
    }
}

impl Eq for Node {}

impl Hash for Node {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        // Compare by f-score (total estimated cost)
        // Lower f-score = higher priority
        self.f
            .partial_cmp(&other.f)
            .unwrap_or(Ordering::Equal)
            .reverse()
    }
}
