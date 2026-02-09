use std::num::NonZero;
use rustc_hash::FxHashMap;
use crate::command::node::{ArgumentNodeMetadata, Command, CommandNodeMetadata, LiteralNodeMetadata, NodeMetadata, OwnedNodeData, RedirectModifier, Redirection, Requirement};
use crate::command::node::detached::GlobalNodeId;

/// Represents the unique integral number
/// of any node, with respect to a tree.
///
/// A [`NonZero<usize>`] is used internally in this
/// struct. This means [`Option<NodeId>`] carries the
/// same size as of [`NodeId`], but comes at the cost of
/// ID `0` being unassignable.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct NodeId(pub NonZero<usize>);

/// Represents the unique integral number
/// of the root node, with respect to a tree.
///
/// This is unit-sized as it is constant.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct RootNodeId;

/// Represents the unique integral number
/// of a specific literal node, with respect to a tree.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct LiteralNodeId(NonZero<usize>);

/// Represents the unique integral number
/// of a specific command node, with respect to a tree.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct CommandNodeId(NonZero<usize>);

/// Represents the unique integral number
/// of a specific argument node, with respect to a tree.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct ArgumentNodeId(NonZero<usize>);

pub const ROOT_NODE_ID: NodeId = NodeId(NonZero::new(1).unwrap());

impl From<RootNodeId> for NodeId {
    fn from(_id: RootNodeId) -> Self {
        ROOT_NODE_ID
    }
}

impl From<LiteralNodeId> for NodeId {
    fn from(id: LiteralNodeId) -> Self {
        NodeId(id.0)
    }
}

impl From<CommandNodeId> for NodeId {
    fn from(id: CommandNodeId) -> Self {
        NodeId(id.0)
    }
}

impl From<ArgumentNodeId> for NodeId {
    fn from(id: ArgumentNodeId) -> Self {
        NodeId(id.0)
    }
}

/// Represents a node which has been attached as the root of a [`Tree`].
pub struct RootAttachedNode {
    pub owned: OwnedNodeData,
    pub children: FxHashMap<String, NodeId>
}

impl RootAttachedNode {
    pub fn new() -> Self {
        Self {
            owned: OwnedNodeData {
                global_id: GlobalNodeId::new(),
                requirement: Requirement::AlwaysQualified,
                modifier: RedirectModifier::OneSource,
                forks: false,
                command: None,
            },
            children: FxHashMap::default()
        }
    }
}

/// Represents a literal, non-command node that has already been attached
/// to a [`Tree`].
///
/// If you want to start a command with this node, use [`CommandAttachedNode`] instead.
pub struct LiteralAttachedNode {
    pub owned: OwnedNodeData,
    pub children: FxHashMap<String, NodeId>,
    pub redirect: Option<Redirection>,
    pub meta: LiteralNodeMetadata
}

/// Represents a literal, command node that has already been attached
/// to a [`Tree`].
///
/// If you don't want to start a command with this node, use [`LiteralAttachedNode`] instead.
pub struct CommandAttachedNode {
    pub owned: OwnedNodeData,
    pub children: FxHashMap<String, NodeId>,
    pub redirect: Option<Redirection>,
    pub meta: CommandNodeMetadata
}

/// Represents a node that accepts a specific type of argument that has already been attached
/// to a [`Tree`].
pub struct ArgumentAttachedNode {
    pub owned: OwnedNodeData,
    pub children: FxHashMap<String, NodeId>,
    pub redirect: Option<Redirection>,
    pub meta: ArgumentNodeMetadata
}

/// Represents a node not attached to a [`Tree`] yet.
pub enum AttachedNode {
    Root(RootAttachedNode),
    Literal(LiteralAttachedNode),
    Command(CommandAttachedNode),
    Argument(ArgumentAttachedNode)
}

impl AttachedNode {
    /// Creates an [`AttachedNode`] from its properties allowing any [`NodeMetadata`].
    pub fn from_parts(
        owned: OwnedNodeData,
        children: FxHashMap<String, NodeId>,
        redirect: Option<Redirection>,
        meta: NodeMetadata
    ) -> Self {
        match meta {
            NodeMetadata::Root => AttachedNode::Root(
                RootAttachedNode {
                    owned,
                    children
                }
            ),
            NodeMetadata::Literal(meta) => AttachedNode::Literal(
                LiteralAttachedNode {
                    owned,
                    children,
                    redirect,
                    meta
                }
            ),
            NodeMetadata::Command(meta) => AttachedNode::Command(
                CommandAttachedNode {
                    owned,
                    children,
                    redirect,
                    meta
                }
            ),
            NodeMetadata::Argument(meta) => AttachedNode::Argument(
                ArgumentAttachedNode {
                    owned,
                    children,
                    redirect,
                    meta
                }
            ),
        }
    }

    /// Gets the global ID from this node.
    pub fn global_id(&self) -> GlobalNodeId {
        self.owned_node_data_ref().global_id
    }

    /// Gets a reference to the owned data of this node.
    pub fn owned_node_data_ref(&self) -> &OwnedNodeData {
        match self {
            AttachedNode::Root(node) => &node.owned,
            AttachedNode::Literal(node) => &node.owned,
            AttachedNode::Command(node) => &node.owned,
            AttachedNode::Argument(node) => &node.owned
        }
    }

    /// Gets a mutable reference to the owned data of this node.
    pub fn owned_node_data_mut_ref(&mut self) -> &mut OwnedNodeData {
        match self {
            AttachedNode::Root(node) => &mut node.owned,
            AttachedNode::Literal(node) => &mut node.owned,
            AttachedNode::Command(node) => &mut node.owned,
            AttachedNode::Argument(node) => &mut node.owned
        }
    }

    /// Gets a reference to the children IDs of this node.
    pub fn children_ref(&self) -> &FxHashMap<String, NodeId> {
        match self {
            AttachedNode::Root(node) => &node.children,
            AttachedNode::Literal(node) => &node.children,
            AttachedNode::Command(node) => &node.children,
            AttachedNode::Argument(node) => &node.children
        }
    }

    /// Gets a mutable reference to the children IDs of this node.
    pub fn children_mut_ref(&mut self) -> &mut FxHashMap<String, NodeId> {
        match self {
            AttachedNode::Root(node) => &mut node.children,
            AttachedNode::Literal(node) => &mut node.children,
            AttachedNode::Command(node) => &mut node.children,
            AttachedNode::Argument(node) => &mut node.children
        }
    }

    /// Gets the name of this node.
    pub fn name(&self) -> String {
        match self {
            Self::Root(_) => String::new(),
            Self::Literal(node) => node.meta.literal.to_string(),
            Self::Command(node) => node.meta.literal.to_string(),
            Self::Argument(node) => node.meta.name.to_string()
        }
    }

    /// Gets the redirection of this node.
    pub fn redirect(&self) -> Option<Redirection> {
        match self {
            Self::Root(_) => None,
            Self::Literal(node) => node.redirect.clone(),
            Self::Command(node) => node.redirect.clone(),
            Self::Argument(node) => node.redirect.clone()
        }
    }

    /// Gets an [`Option`] of a mutable reference to the redirection of this node.
    pub fn redirect_mut_ref(&mut self) -> Option<&mut Redirection> {
        match self {
            Self::Root(_) => None,
            Self::Literal(node) => node.redirect.as_mut(),
            Self::Command(node) => node.redirect.as_mut(),
            Self::Argument(node) => node.redirect.as_mut()
        }
    }

    // pub global_id: GlobalNodeId,
    //     pub requirement: Requirement,
    //     pub modifier: RedirectModifier,
    //     pub forks: bool,
    //     pub command: Option<Command>

    /// Get the requirement for this node to be run.
    pub fn requirement(&self) -> &Requirement {
        &self.owned_node_data_ref().requirement
    }

    /// Set the requirement for this node to be run to a value.
    pub fn set_requirement(&mut self, requirement: Requirement) {
        self.owned_node_data_mut_ref().requirement = requirement;
    }

    /// Get the modifier for this node to be run.
    pub fn modifier(&self) -> &RedirectModifier {
        &self.owned_node_data_ref().modifier
    }

    /// Set the modifier for this node to a value.
    pub fn set_modifier(&mut self, modifier: RedirectModifier) {
        self.owned_node_data_mut_ref().modifier = modifier;
    }
}