use pumpkin_util::math::{position::BlockPos, vector3::Vector3};

use crate::entity::ai::pathfinder::node::Node;

#[derive(Debug, Clone)]
pub struct Path {
    nodes: Vec<Node>,
    pub next_node_index: usize,
    target: BlockPos,
    dist_to_target: f32,
    reached: bool,
}

impl Path {
    #[must_use]
    pub fn new(nodes: Vec<Node>, target: BlockPos, reached: bool) -> Self {
        let dist_to_target = if nodes.is_empty() {
            f32::MAX
        } else {
            let last_node = &nodes[nodes.len() - 1];
            last_node.distance_manhattan_block_pos(target)
        };

        Self {
            nodes,
            next_node_index: 0,
            target,
            dist_to_target,
            reached,
        }
    }

    #[must_use]
    pub fn empty(target: BlockPos) -> Self {
        Self::new(Vec::new(), target, false)
    }

    pub const fn advance(&mut self) {
        self.next_node_index += 1;
    }

    #[must_use]
    pub const fn not_started(&self) -> bool {
        self.next_node_index == 0
    }

    #[must_use]
    pub const fn is_done(&self) -> bool {
        self.next_node_index >= self.nodes.len()
    }

    #[must_use]
    pub fn get_end_node(&self) -> Option<&Node> {
        self.nodes.last()
    }

    #[must_use]
    pub fn get_node(&self, index: usize) -> Option<&Node> {
        self.nodes.get(index)
    }

    pub fn truncate_nodes(&mut self, index: usize) {
        if index < self.nodes.len() {
            self.nodes.truncate(index);
            if self.next_node_index > self.nodes.len() {
                self.next_node_index = self.nodes.len();
            }
        }
    }

    pub fn replace_node(&mut self, index: usize, replace_with: Node) {
        if index < self.nodes.len() {
            self.nodes[index] = replace_with;
        }
    }

    #[must_use]
    pub const fn get_node_count(&self) -> usize {
        self.nodes.len()
    }

    #[must_use]
    pub const fn get_next_node_index(&self) -> usize {
        self.next_node_index
    }

    pub const fn set_next_node_index(&mut self, index: usize) {
        self.next_node_index = index;
    }

    #[must_use]
    pub fn get_entity_pos_at_node(&self, entity_width: f32, index: usize) -> Option<Vector3<f64>> {
        self.nodes.get(index).map(|node| {
            let offset = f64::from((entity_width + 1.0) as i32) * 0.5;
            Vector3::new(
                f64::from(node.pos.0.x) + offset,
                f64::from(node.pos.0.y),
                f64::from(node.pos.0.z) + offset,
            )
        })
    }

    #[must_use]
    pub fn get_node_pos(&self, index: usize) -> Option<BlockPos> {
        self.nodes.get(index).map(|node| node.pos)
    }

    #[must_use]
    pub fn get_next_entity_pos(&self, entity_width: f32) -> Option<Vector3<f64>> {
        self.get_entity_pos_at_node(entity_width, self.next_node_index)
    }

    #[must_use]
    pub fn get_next_node_pos(&self) -> Option<BlockPos> {
        self.get_node_pos(self.next_node_index)
    }

    #[must_use]
    pub fn get_next_node(&self) -> Option<&Node> {
        self.nodes.get(self.next_node_index)
    }

    #[must_use]
    pub fn get_previous_node(&self) -> Option<&Node> {
        if self.next_node_index > 0 {
            self.nodes.get(self.next_node_index - 1)
        } else {
            None
        }
    }

    #[must_use]
    pub fn same_as(&self, other: Option<&Self>) -> bool {
        other.is_some_and(|o| self.same_as_path(o))
    }

    #[must_use]
    pub fn same_as_path(&self, other: &Self) -> bool {
        self.nodes.len() == other.nodes.len()
            && self.nodes.iter().zip(&other.nodes).all(|(a, b)| a == b)
    }

    #[must_use]
    pub const fn can_reach(&self) -> bool {
        self.reached
    }

    #[must_use]
    pub const fn get_target(&self) -> BlockPos {
        self.target
    }

    #[must_use]
    pub const fn get_dist_to_target(&self) -> f32 {
        self.dist_to_target
    }

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

    #[must_use]
    pub fn get_nodes(&self) -> &[Node] {
        &self.nodes
    }

    #[must_use]
    pub fn get_nodes_mut(&mut self) -> &mut [Node] {
        &mut self.nodes
    }

    #[must_use]
    pub fn get_remaining_nodes(&self) -> &[Node] {
        if self.next_node_index >= self.nodes.len() {
            &[]
        } else {
            &self.nodes[self.next_node_index..]
        }
    }

    #[must_use]
    pub fn calculate_length(&self) -> f32 {
        if self.nodes.len() < 2 {
            return 0.0;
        }

        let mut total_length = 0.0;
        for i in 1..self.nodes.len() {
            total_length += self.nodes[i - 1].distance_to_node(&self.nodes[i]);
        }
        total_length
    }

    #[must_use]
    pub fn get_remaining_distance(&self) -> f32 {
        if self.next_node_index >= self.nodes.len() {
            return 0.0;
        }

        let mut remaining = 0.0;
        for i in (self.next_node_index + 1)..self.nodes.len() {
            remaining += self.nodes[i - 1].distance_to_node(&self.nodes[i]);
        }
        remaining
    }

    #[must_use]
    pub const fn is_valid(&self) -> bool {
        !self.nodes.is_empty()
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

impl Default for Path {
    fn default() -> Self {
        Self::empty(BlockPos::new(0, 0, 0))
    }
}
