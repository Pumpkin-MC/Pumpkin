use std::sync::Arc;

use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
use uuid::Uuid;

use crate::entity::ai::pathfinder::node::Node;

#[derive(Debug, Clone)]
pub struct Path {
    route: Arc<Route>,
    pub next_node_index: usize,
    pub(crate) world_change: Option<(Uuid, u64)>,
}

#[derive(Debug, Clone)]
struct Route {
    nodes: Vec<Node>,
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
            route: Arc::new(Route {
                nodes,
                target,
                dist_to_target,
                reached,
            }),
            next_node_index: 0,
            world_change: None,
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
    pub fn is_done(&self) -> bool {
        self.next_node_index >= self.route.nodes.len()
    }

    #[must_use]
    pub fn get_end_node(&self) -> Option<&Node> {
        self.route.nodes.last()
    }

    #[must_use]
    pub fn get_node(&self, index: usize) -> Option<&Node> {
        self.route.nodes.get(index)
    }

    pub fn truncate_nodes(&mut self, index: usize) {
        if index < self.route.nodes.len() {
            Arc::make_mut(&mut self.route).nodes.truncate(index);
            self.next_node_index = self.next_node_index.min(index);
        }
    }

    pub fn replace_node(&mut self, index: usize, replace_with: Node) {
        if index < self.route.nodes.len() {
            Arc::make_mut(&mut self.route).nodes[index] = replace_with;
        }
    }

    #[must_use]
    pub fn get_node_count(&self) -> usize {
        self.route.nodes.len()
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
        self.route.nodes.get(index).map(|node| {
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
        self.route.nodes.get(index).map(|node| node.pos)
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
        self.route.nodes.get(self.next_node_index)
    }

    #[must_use]
    pub fn get_previous_node(&self) -> Option<&Node> {
        if self.next_node_index > 0 {
            self.route.nodes.get(self.next_node_index - 1)
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
        self.route.nodes.len() == other.route.nodes.len()
            && self
                .route
                .nodes
                .iter()
                .zip(&other.route.nodes)
                .all(|(a, b)| a == b)
    }

    /// Compares route identity and progress without scanning the nodes.
    pub(crate) fn same_snapshot(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.route, &other.route) && self.next_node_index == other.next_node_index
    }

    #[must_use]
    pub fn can_reach(&self) -> bool {
        self.route.reached
    }

    #[must_use]
    pub fn get_target(&self) -> BlockPos {
        self.route.target
    }

    #[must_use]
    pub fn get_dist_to_target(&self) -> f32 {
        self.route.dist_to_target
    }

    #[must_use]
    pub fn copy(&self) -> Self {
        self.clone()
    }

    #[must_use]
    pub fn get_nodes(&self) -> &[Node] {
        &self.route.nodes
    }

    #[must_use]
    pub fn get_nodes_mut(&mut self) -> &mut [Node] {
        &mut Arc::make_mut(&mut self.route).nodes
    }

    #[must_use]
    pub fn get_remaining_nodes(&self) -> &[Node] {
        if self.next_node_index >= self.route.nodes.len() {
            &[]
        } else {
            &self.route.nodes[self.next_node_index..]
        }
    }

    #[must_use]
    pub fn calculate_length(&self) -> f32 {
        if self.route.nodes.len() < 2 {
            return 0.0;
        }

        let mut total_length = 0.0;
        for i in 1..self.route.nodes.len() {
            total_length += self.route.nodes[i - 1].distance_to_node(&self.route.nodes[i]);
        }
        total_length
    }

    #[must_use]
    pub fn get_remaining_distance(&self) -> f32 {
        if self.next_node_index >= self.route.nodes.len() {
            return 0.0;
        }

        let mut remaining = 0.0;
        for i in (self.next_node_index + 1)..self.route.nodes.len() {
            remaining += self.route.nodes[i - 1].distance_to_node(&self.route.nodes[i]);
        }
        remaining
    }

    #[must_use]
    pub fn is_valid(&self) -> bool {
        !self.route.nodes.is_empty()
    }
}

impl PartialEq for Path {
    fn eq(&self, other: &Self) -> bool {
        self.next_node_index == other.next_node_index
            && self.route.reached == other.route.reached
            && self.route.target == other.route.target
            && self.route.nodes == other.route.nodes
    }
}

impl Default for Path {
    fn default() -> Self {
        Self::empty(BlockPos::new(0, 0, 0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editing_a_route_preserves_existing_snapshots() {
        let start = BlockPos::new(0, 64, 0);
        let middle = BlockPos::new(1, 64, 0);
        let end = BlockPos::new(2, 64, 0);
        let mut snapshot = Path::new(
            vec![Node::new(start), Node::new(middle), Node::new(end)],
            end,
            true,
        );
        snapshot.advance();
        let mut edited = snapshot.clone();
        edited.get_nodes_mut()[0] = Node::new(BlockPos::new(0, 65, 0));
        edited.set_next_node_index(0);
        let mut replaced = snapshot.clone();
        replaced.replace_node(1, Node::new(BlockPos::new(1, 65, 0)));
        let mut truncated = snapshot.clone();
        truncated.set_next_node_index(3);
        truncated.truncate_nodes(1);

        assert_eq!(
            snapshot
                .get_nodes()
                .iter()
                .map(|node| node.pos)
                .collect::<Vec<_>>(),
            [start, middle, end]
        );
        assert_eq!(snapshot.get_next_node_index(), 1);
        assert_eq!(edited.get_next_node_index(), 0);
        assert_eq!(edited.get_node_pos(0), Some(BlockPos::new(0, 65, 0)));
        assert_eq!(replaced.get_node_pos(1), Some(BlockPos::new(1, 65, 0)));
        assert_eq!(truncated.get_node_count(), 1);
        assert_eq!(truncated.get_next_node_index(), 1);
    }
}
