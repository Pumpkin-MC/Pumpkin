use super::ChunkPos;
use super::chunk_state::StagedChunkEnum;
use slotmap::{Key, SlotMap, new_key_type};

#[derive(Clone, Debug)]
pub struct Node {
    pub pos: ChunkPos,
    pub stage: StagedChunkEnum,
    pub in_degree: u32,
    pub in_queue: bool,
    pub edge: EdgeKey,
}

impl Node {
    pub fn new(pos: ChunkPos, stage: StagedChunkEnum) -> Self {
        Self {
            pos,
            stage,
            in_degree: 0,
            in_queue: false,
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
        let mut edge = self.nodes.remove(node).unwrap().edge;
        // debug!("drop node {node:?}");
        while !edge.is_null() {
            edge = self.edges.remove(edge).unwrap().next;
        }
    }
    pub fn add_edge(&mut self, from: NodeKey, to: NodeKey) {
        self.nodes.get_mut(to).unwrap().in_degree += 1;
        let edge = &mut self.nodes.get_mut(from).unwrap().edge;
        *edge = self.edges.insert(Edge::new(to, *edge));
    }
}
