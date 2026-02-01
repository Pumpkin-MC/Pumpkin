//! Path target representation.
//!
//! This module defines the `Target` struct used to represent goal positions
//! in pathfinding, tracking the best path found to reach each target.

use pumpkin_util::math::position::BlockPos;

use super::node::Node;

/// A target position for pathfinding.
///
/// Targets track the best node found so far during pathfinding,
/// allowing for partial paths when the destination cannot be fully reached.
#[derive(Clone, Debug)]
pub struct Target {
    /// The target node position
    pub node: Node,
    /// Best heuristic value found (distance from best node to target)
    pub best_heuristic: f32,
    /// Index of the best node found so far
    pub best_node_index: Option<usize>,
    /// Whether this target has been reached
    pub reached: bool,
}

impl Target {
    /// Creates a new target from a node.
    #[must_use]
    pub const fn new(node: Node) -> Self {
        Self {
            node,
            best_heuristic: f32::MAX,
            best_node_index: None,
            reached: false,
        }
    }

    /// Creates a new target at the specified coordinates.
    #[must_use]
    pub const fn from_coords(x: i32, y: i32, z: i32) -> Self {
        Self::new(Node::new(x, y, z))
    }

    /// Creates a new target from a block position.
    #[must_use]
    pub const fn from_block_pos(pos: BlockPos) -> Self {
        Self::new(Node::from_block_pos(pos))
    }

    /// Returns the block position of this target.
    #[must_use]
    pub const fn as_block_pos(&self) -> BlockPos {
        self.node.as_block_pos()
    }

    /// Returns the X coordinate of this target.
    #[must_use]
    pub const fn x(&self) -> i32 {
        self.node.x
    }

    /// Returns the Y coordinate of this target.
    #[must_use]
    pub const fn y(&self) -> i32 {
        self.node.y
    }

    /// Returns the Z coordinate of this target.
    #[must_use]
    pub const fn z(&self) -> i32 {
        self.node.z
    }

    /// Updates the best node if the given heuristic is better.
    ///
    /// Returns true if the best node was updated.
    pub fn update_best(&mut self, heuristic: f32, node_index: usize) -> bool {
        if heuristic < self.best_heuristic {
            self.best_heuristic = heuristic;
            self.best_node_index = Some(node_index);
            true
        } else {
            false
        }
    }

    /// Marks this target as reached.
    pub const fn set_reached(&mut self) {
        self.reached = true;
    }

    /// Returns whether this target has been reached.
    #[must_use]
    pub const fn is_reached(&self) -> bool {
        self.reached
    }

    /// Calculates the distance from a node to this target.
    #[must_use]
    pub fn distance_from(&self, node: &Node) -> f32 {
        node.distance_to(&self.node)
    }

    /// Calculates the Manhattan distance from a node to this target.
    #[must_use]
    pub fn distance_manhattan_from(&self, node: &Node) -> f32 {
        node.distance_manhattan(&self.node)
    }
}

impl PartialEq for Target {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node
    }
}

impl Eq for Target {}

impl std::hash::Hash for Target {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.node.hash(state);
    }
}
