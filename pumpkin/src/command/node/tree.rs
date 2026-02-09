use std::num::NonZero;
use std::ops::{Index, IndexMut};
use rustc_hash::{FxBuildHasher, FxHashMap};
use crate::command::node::attached::{AttachedNode, NodeId, RootAttachedNode, ROOT_NODE_ID};
use crate::command::node::detached::{DetachedNode, GlobalNodeId};

/// Represents an entire tree of nodes.
/// It all starts from the root node, which arise
/// to children nodes, which arise to their children,
/// and so on.
///
/// This allows redirection and forking
/// between two nodes, even if from different commands.
///
/// This tree can be indexed like an array but with [`NodeId`].
pub struct Tree {
    /// All the nodes stored in this tree.
    ///
    /// In this vector, indices starting at 0 indicates the first node (ID = 1),
    /// 1 indicates the second node (ID = 2) and so on.
    nodes: Vec<AttachedNode>,

    /// Keys linking [`GlobalNodeId`] to the [`NodeId`] for this tree.
    /// Useful for redirecting.
    ids_map: FxHashMap<GlobalNodeId, NodeId>
}

impl Tree {
    /// Constructs a new tree, containing a new root node without children.
    pub fn new() -> Tree {
        let node = RootAttachedNode::new();
        let mut ids_map = FxHashMap::default();
        ids_map.insert(node.owned.global_id, ROOT_NODE_ID);
        Tree {
            nodes: vec![AttachedNode::Root(node)],
            ids_map
        }
    }

    /// Helper to attach a given [`AttachedNode`], returning
    /// its [`NodeId`].
    fn add(&mut self, node: AttachedNode) -> NodeId {
        let global_id = node.global_id();
        let local_id = NodeId(NonZero::new(self.nodes.len() + 1).expect("expected a non-zero id"));

        // Update state variables.
        self.nodes.push(node);
        self.ids_map.insert(global_id, local_id);

        local_id
    }

    /// Helper to attach a [`DetachedNode`] irreversibly
    /// into this [`Tree`], returning the ID of the now attached
    /// node.
    fn attach(&mut self, node: DetachedNode) -> NodeId {
        // First, we decompose this node.
        let node = node.decompose();

        // Add its children to this tree.
        let mut children = FxHashMap::with_capacity_and_hasher(
            node.children.len(),
            FxBuildHasher::default()
        );
        for (child_name, child) in node.children {
            let child_id = self.attach(child);
            children.insert(child_name, child_id);
        }

        // Now create the node to be 'attached'.
        let node = AttachedNode::from_parts(
            node.owned,
            children,
            node.redirect,
            node.meta
        );

        self.add(node)
    }

    /// Adds a child to a given node.
    ///
    /// # Panics
    ///
    /// Panics if the node provided is a root node.
    pub fn add_child(&mut self, parent: NodeId, node: DetachedNode) {
        // First, attach the node to this tree.
        let node = self.attach(node);
        let node_name = self[node].name();
        let node_command = self[node].owned_node_data_ref().command.clone();
        let node_children = self[node].children_ref().values().cloned();

        let parent = &self[parent];
        let child = parent.children_ref().get(&node_name);

        if let Some(child) = child {
            let child = *child;
            // Merge onto the child.
            if let Some(command) = node_command {
                self[child].owned_node_data_mut_ref().command = Some(command);
            }
            for grandchild in node_children {
                self.add_child(child.id, grandchild);
            }
        }
    }
}

impl Index<NodeId> for Tree {
    type Output = AttachedNode;

    fn index(&self, index: NodeId) -> &Self::Output {
        &self.nodes[index.0.get() - 1]
    }
}

impl IndexMut<NodeId> for Tree {
    fn index_mut(&mut self, index: NodeId) -> &mut Self::Output {
        &mut self.nodes[index.0.get() - 1]
    }
}