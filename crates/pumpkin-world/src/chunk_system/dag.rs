use super::ChunkPos;
use super::chunk_state::StagedChunkEnum;
use slotmap::{Key, SlotMap, new_key_type};

#[derive(Clone, Debug)]
pub struct Node {
    pub pos: ChunkPos,
    pub stage: StagedChunkEnum,
    pub in_degree: u32,
    pub in_queue: bool,
    pub in_flight: bool,
    pub edge: EdgeKey,
}

impl Node {
    #[must_use]
    pub fn new(pos: ChunkPos, stage: StagedChunkEnum) -> Self {
        Self {
            pos,
            stage,
            in_degree: 0,
            in_queue: false,
            in_flight: false,
            edge: EdgeKey::null(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Edge {
    pub to: NodeKey,
    pub next: EdgeKey,
}

impl Edge {
    #[must_use]
    pub const fn new(to: NodeKey, next: EdgeKey) -> Self {
        Self { to, next }
    }
}

new_key_type! { pub struct NodeKey; }
new_key_type! { pub struct EdgeKey; }

#[derive(Default)]
pub struct DAG {
    pub nodes: SlotMap<NodeKey, Node>,
    pub edges: SlotMap<EdgeKey, Edge>,
}

impl DAG {
    pub fn fast_drop_node(&mut self, node: NodeKey) {
        if let Some(removed_node) = self.nodes.remove(node) {
            let mut edge = removed_node.edge;
            while !edge.is_null() {
                if let Some(removed_edge) = self.edges.remove(edge) {
                    edge = removed_edge.next;
                } else {
                    break;
                }
            }
        }
    }
    pub fn add_edge(&mut self, from: NodeKey, to: NodeKey) {
        // Ensure both nodes exist before adding edge
        if !self.nodes.contains_key(to) || !self.nodes.contains_key(from) {
            return;
        }
        if let Some(node) = self.nodes.get_mut(to) {
            node.in_degree += 1;
        }
        if let Some(node) = self.nodes.get_mut(from) {
            let edge = &mut node.edge;
            *edge = self.edges.insert(Edge::new(to, *edge));
        }
    }

    pub fn drop_edge_chain(&mut self, mut head: EdgeKey) {
        while !head.is_null() {
            if let Some(edge) = self.edges.remove(head) {
                head = edge.next;
            } else {
                break;
            }
        }
    }

    pub fn prune_edge_chain(&mut self, head: &mut EdgeKey) -> bool {
        let mut cur_edge = *head;
        let mut prev_edge = EdgeKey::null();
        let mut change_head = None;
        let mut has_valid_target = false;

        while !cur_edge.is_null() {
            let Some(edge) = self.edges.get(cur_edge) else {
                break;
            };
            if self.nodes.contains_key(edge.to) {
                prev_edge = cur_edge;
                cur_edge = edge.next;
                has_valid_target = true;
            } else {
                let next = edge.next;
                self.edges.remove(cur_edge);
                cur_edge = next;
                if prev_edge.is_null() {
                    change_head = Some(next);
                } else if let Some(prev) = self.edges.get_mut(prev_edge) {
                    prev.next = next;
                }
            }
        }
        if let Some(next) = change_head {
            *head = next;
        }

        has_valid_target
    }
}
