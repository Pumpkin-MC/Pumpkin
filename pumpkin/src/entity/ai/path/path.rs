//! Path representation for entity navigation.
//!
//! This module defines the `Path` struct which represents a sequence of nodes
//! from a starting position to a target destination.

use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

use super::node::Node;

/// A path consisting of a sequence of nodes from start to destination.
///
/// The path tracks the current progress through the node sequence and
/// provides methods for path traversal and querying.
#[derive(Clone, Debug)]
pub struct Path {
    /// The sequence of nodes forming this path
    nodes: Vec<Node>,
    /// Current position in the path (index of next node to visit)
    next_node_index: usize,
    /// The target block position
    target: BlockPos,
    /// Distance from the last node to the target
    dist_to_target: f32,
    /// Whether the path reaches the target
    reached: bool,
}

impl Path {
    /// Creates a new path from a list of nodes.
    ///
    /// # Arguments
    /// * `nodes` - The sequence of nodes forming the path
    /// * `target` - The target block position
    /// * `reached` - Whether the path reaches the target
    #[must_use]
    pub fn new(nodes: Vec<Node>, target: BlockPos, reached: bool) -> Self {
        let dist_to_target = if nodes.is_empty() {
            f32::MAX
        } else {
            nodes.last().unwrap().distance_manhattan_pos(target)
        };

        Self {
            nodes,
            next_node_index: 0,
            target,
            dist_to_target,
            reached,
        }
    }

    /// Creates an empty path that cannot be reached.
    #[must_use]
    pub const fn empty(target: BlockPos) -> Self {
        Self {
            nodes: Vec::new(),
            next_node_index: 0,
            target,
            dist_to_target: f32::MAX,
            reached: false,
        }
    }

    /// Advances to the next node in the path.
    pub const fn advance(&mut self) {
        self.next_node_index += 1;
    }

    /// Returns whether the path has not yet started (at first node).
    #[must_use]
    pub const fn not_started(&self) -> bool {
        self.next_node_index == 0
    }

    /// Returns whether the path traversal is complete.
    #[must_use]
    pub const fn is_done(&self) -> bool {
        self.next_node_index >= self.nodes.len()
    }

    /// Returns the last node in the path, if any.
    #[must_use]
    pub fn get_end_node(&self) -> Option<&Node> {
        self.nodes.last()
    }

    /// Returns a reference to a node at the specified index.
    #[must_use]
    pub fn get_node(&self, index: usize) -> Option<&Node> {
        self.nodes.get(index)
    }

    /// Returns a mutable reference to a node at the specified index.
    pub fn get_node_mut(&mut self, index: usize) -> Option<&mut Node> {
        self.nodes.get_mut(index)
    }

    /// Truncates the path at the specified index.
    ///
    /// All nodes at and after `index` are removed.
    pub fn truncate_nodes(&mut self, index: usize) {
        if index < self.nodes.len() {
            self.nodes.truncate(index);
        }
    }

    /// Replaces a node at the specified index.
    pub fn replace_node(&mut self, index: usize, node: Node) {
        if index < self.nodes.len() {
            self.nodes[index] = node;
        }
    }

    /// Returns the total number of nodes in the path.
    #[must_use]
    pub const fn get_node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Returns the index of the next node to visit.
    #[must_use]
    pub const fn get_next_node_index(&self) -> usize {
        self.next_node_index
    }

    /// Sets the index of the next node to visit.
    pub const fn set_next_node_index(&mut self, index: usize) {
        self.next_node_index = index;
    }

    /// Returns the position for an entity at a specific node index.
    ///
    /// The position is centered on the block with the entity's width considered.
    #[must_use]
    pub fn get_entity_pos_at_node(&self, index: usize, entity_width: f32) -> Option<Vector3<f64>> {
        self.nodes.get(index).map(|node| {
            let offset = ((entity_width + 1.0) as i32) as f64 * 0.5;
            Vector3::new(
                node.x as f64 + offset,
                node.y as f64,
                node.z as f64 + offset,
            )
        })
    }

    /// Returns the block position at a specific node index.
    #[must_use]
    pub fn get_node_pos(&self, index: usize) -> Option<BlockPos> {
        self.nodes.get(index).map(super::node::Node::as_block_pos)
    }

    /// Returns the position for an entity at the next node.
    #[must_use]
    pub fn get_next_entity_pos(&self, entity_width: f32) -> Option<Vector3<f64>> {
        self.get_entity_pos_at_node(self.next_node_index, entity_width)
    }

    /// Returns the block position of the next node.
    #[must_use]
    pub fn get_next_node_pos(&self) -> Option<BlockPos> {
        self.get_node_pos(self.next_node_index)
    }

    /// Returns a reference to the next node.
    #[must_use]
    pub fn get_next_node(&self) -> Option<&Node> {
        self.nodes.get(self.next_node_index)
    }

    /// Returns a reference to the previous node.
    #[must_use]
    pub fn get_previous_node(&self) -> Option<&Node> {
        if self.next_node_index > 0 {
            self.nodes.get(self.next_node_index - 1)
        } else {
            None
        }
    }

    /// Checks if this path is the same as another path.
    #[must_use]
    pub fn same_as(&self, other: Option<&Self>) -> bool {
        other.is_some_and(|other| self.nodes == other.nodes)
    }

    /// Returns whether this path can reach its target.
    #[must_use]
    pub const fn can_reach(&self) -> bool {
        self.reached
    }

    /// Returns the target block position.
    #[must_use]
    pub const fn get_target(&self) -> BlockPos {
        self.target
    }

    /// Returns the distance from the last node to the target.
    #[must_use]
    pub const fn get_dist_to_target(&self) -> f32 {
        self.dist_to_target
    }

    /// Creates a copy of this path.
    #[must_use]
    pub fn copy(&self) -> Self {
        Self {
            nodes: self.nodes.clone(),
            next_node_index: self.next_node_index,
            target: self.target,
            dist_to_target: self.dist_to_target,
            reached: self.reached,
        }
    }

    /// Returns an iterator over the nodes in this path.
    pub fn iter(&self) -> impl Iterator<Item = &Node> + '_ {
        self.nodes.iter()
    }

    /// Returns the remaining nodes from the current position.
    #[must_use]
    pub fn remaining_nodes(&self) -> &[Node] {
        if self.next_node_index < self.nodes.len() {
            &self.nodes[self.next_node_index..]
        } else {
            &[]
        }
    }
}

impl PartialEq for Path {
    fn eq(&self, other: &Self) -> bool {
        self.next_node_index == other.next_node_index
            && self.reached == other.reached
            && self.target == other.target
            && self.nodes == other.nodes
    }
}

impl Eq for Path {}

impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Path(length={})", self.nodes.len())
    }
}
