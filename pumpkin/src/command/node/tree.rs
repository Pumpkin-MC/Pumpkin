use crate::command::context::command_context::{CommandContextBuilder, ParsedArgument};
use crate::command::context::command_source::CommandSource;
use crate::command::context::string_range::StringRange;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::LITERAL_INCORRECT;
use crate::command::node::Redirection;
use crate::command::node::attached::{
    ArgumentAttachedNode, ArgumentNodeId, AttachedNode, CommandAttachedNode, CommandNodeId,
    LiteralAttachedNode, LiteralNodeId, NodeClassification, NodeId, RootAttachedNode,
};
use crate::command::node::detached::{CommandDetachedNode, DetachedNode, GlobalNodeId};
use crate::command::string_reader::StringReader;
use pumpkin_util::text::TextComponent;
use rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet};
use std::num::NonZero;
use std::ops::{Index, IndexMut};
use std::sync::Arc;

/// The constant local ID occupied by the root node.
pub const ROOT_NODE_ID: NodeId = NodeId(NonZero::new(1).unwrap());

/// A consumer which takes ambiguity of input (when two or more nodes are satisfied)
pub trait AmbiguityConsumer {
    fn ambiguous(
        &mut self,
        tree: &Tree,
        parent: NodeId,
        child: NodeId,
        sibling: NodeId,
        inputs: Vec<String>,
    );
}

/// Allows a way to store the kind of node
/// along with its ID.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NodeIdClassification {
    Root,
    Literal(LiteralNodeId),
    Command(CommandNodeId),
    Argument(ArgumentNodeId),
}

/// Represents an entire tree of nodes.
/// It all starts from the root node, which arise
/// to children nodes, which arise to their children,
/// and so on.
///
/// This allows redirection and forking
/// between two nodes, even if from different commands.
///
/// This tree can be indexed like an array but with [`NodeId`].
///
/// # Hierarchy
/// This tree can have four different types of nodes, which are the following:
///
/// - **Root**:
///   Does not have a parent. Exactly one instance of this type of node
///   exists per [`Tree`]. Always identifiable by [`ROOT_NODE_ID`].
///
/// - **Command**:
///   Its parent must be the root node, and specifies the start of a
///   command definition.
///
/// - **Literal**:
///   Accepts a particular constant word.
///
/// - **Argument**:
///   Parses and accepts a specific type of value. This is very dynamic.
#[derive(Clone)]
pub struct Tree {
    /// All the nodes stored in this tree.
    ///
    /// In this vector, indices starting at 0 indicates the first node (ID = 1),
    /// 1 indicates the second node (ID = 2) and so on.
    nodes: Vec<AttachedNode>,

    /// Keys linking [`GlobalNodeId`] to the [`NodeId`] for this tree.
    /// Useful for redirecting.
    ids_map: FxHashMap<GlobalNodeId, NodeId>,

    /// Cached mappings for each command.
    command_node_mappings: FxHashMap<String, CommandNodeId>,
}

impl Default for Tree {
    fn default() -> Self {
        Self::new()
    }
}

impl Tree {
    /// Constructs a new tree, containing a new root node without children.
    #[must_use]
    pub fn new() -> Self {
        let node = RootAttachedNode::new();
        let mut ids_map = FxHashMap::default();
        ids_map.insert(node.owned.global_id, ROOT_NODE_ID);
        Self {
            nodes: vec![AttachedNode::Root(node)],
            ids_map,
            command_node_mappings: FxHashMap::default(),
        }
    }

    /// Allocates a new [`NodeId`] by creating a new unique one.
    const fn alloc(&self) -> NodeId {
        NodeId(NonZero::new(self.nodes.len() + 1).expect("expected a non-zero id"))
    }

    /// Helper to attach a given [`AttachedNode`], returning
    /// its [`NodeId`].
    fn add(&mut self, node: AttachedNode) -> NodeId {
        let global_id = node.global_id();
        let local_id = self.alloc();

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
        let mut children = FxHashMap::with_capacity_and_hasher(node.children.len(), FxBuildHasher);
        for (child_name, child) in node.children {
            let child_id = self.attach(child);
            children.insert(child_name, child_id);
        }

        // Now create the node to be 'attached'.
        let node = AttachedNode::from_parts(node.owned, children, node.redirect, node.meta);

        self.add(node)
    }

    /// Gets the size of this [`Tree`], which is the number of nodes this tree contains.
    #[must_use]
    pub const fn size(&self) -> usize {
        self.nodes.len()
    }

    /// Gets the size of this [`Tree`], which is the number of nodes this tree contains.
    #[must_use]
    pub fn size_nonzero(&self) -> NonZero<usize> {
        self.nodes
            .len()
            .try_into()
            .expect("Expected non-zero size, but Tree was somehow zero-sized")
    }

    /// Adds a [`CommandDetachedNode`] to the root node of this tree.
    pub fn add_child_to_root(&mut self, node: impl Into<CommandDetachedNode>) -> CommandNodeId {
        // First, attach the node to this tree.
        let node = node.into();
        let name = node.meta.literal.to_string();
        let node = self.attach(node.into());
        self.add_attached_child(ROOT_NODE_ID, node);

        // This is safe as the node ID now points to a `CommandAttachedNode`.
        let node = CommandNodeId(node.0);

        self.command_node_mappings.insert(name, node);
        node
    }

    /// Adds a child to a given node.
    ///
    /// # Panics
    ///
    /// Panics if the node to be added to a non-root node is a [`CommandDetachedNode`].
    ///
    /// Essentially, this means that a [`CommandDetachedNode`] must have the root node
    /// of the tree *as its parent*, to be attached to the tree.
    pub fn add_child(&mut self, parent: NodeId, node: impl Into<DetachedNode>) -> NodeId {
        let node = node.into();
        assert!(
            parent == ROOT_NODE_ID || !matches!(node, DetachedNode::Command(_)),
            "Cannot add a CommandDetachedNode as a child of a non-root node"
        );

        // First, attach the node to this tree.
        let node = self.attach(node);
        self.add_attached_child(parent, node);
        node
    }

    /// Adds an already-attached child to a given node.
    ///
    /// # Panics
    ///
    /// Panics if the node to be added to a non-root node is a [`CommandAttachedNode`],
    /// or if the node to be added to a node is a [`RootAttachedNode`],
    ///
    /// Essentially, this means that a [`CommandAttachedNode`] must have the root node
    /// of the tree *as its parent*, and [`RootAttachedNode`] cannot have a parent.
    fn add_attached_child(&mut self, parent: NodeId, node: NodeId) {
        assert!(
            parent == ROOT_NODE_ID || self[node].classification() != NodeClassification::Command,
            "Cannot add a CommandAttachedNode as a child of a non-root node"
        );

        let node_name = self[node].name();

        let child = self[parent].children_ref().get(&node_name);
        if let Some(child) = child {
            let node_command = self[node].command().clone();
            let node_children: Vec<NodeId> = self[node].children_ref().values().copied().collect();

            let child = *child;
            // Merge onto the child.
            if let Some(command) = node_command {
                self[child].set_command(Some(command));
            }
            for grandchild in node_children {
                self.add_attached_child(child, grandchild);
            }
        } else {
            self[parent].children_mut_ref().insert(node_name, node);
        }
    }

    /// Gets the children of a given node in the tree.
    #[must_use]
    pub fn get_children(&self, node: NodeId) -> Vec<NodeId> {
        self[node].children_ref().values().copied().collect()
    }

    /// Gets the children of the root node in the tree.
    #[must_use]
    pub fn get_root_children(&self) -> Vec<CommandNodeId> {
        self[ROOT_NODE_ID]
            .children_ref()
            .values()
            .copied()
            // This should be fine as all children of the
            // root node are Command Nodes.
            .map(|id| CommandNodeId(id.0))
            .collect()
    }

    /// Returns whether the given node is able to be used by a given source.
    #[must_use]
    pub async fn can_use(&self, node: NodeId, source: &CommandSource) -> bool {
        source
            .has_permission_from_option(self[node].permission())
            .await
            && self[node].requirement().evaluate(source).await
    }

    /// Finds ambiguities of input and gives them to the [`AmbiguityConsumer`].
    pub fn find_ambiguities(&self, node: NodeId, consumer: &mut impl AmbiguityConsumer) {
        let mut matches: FxHashSet<String> = FxHashSet::default();

        for child in self.get_children(node) {
            for sibling in self.get_children(node) {
                if child == sibling {
                    continue;
                }
                for input in self[child].examples() {
                    if self[sibling].is_valid_input(&input) {
                        matches.insert(input.clone());
                    }
                }

                if !matches.is_empty() {
                    consumer.ambiguous(self, node, child, sibling, matches.drain().collect());
                }
            }

            self.find_ambiguities(child, consumer);
        }
    }

    /// Classifies a given node to a typed ID.
    #[must_use]
    pub fn classify_id(&self, node: NodeId) -> NodeIdClassification {
        match self[node].classification() {
            NodeClassification::Root => NodeIdClassification::Root,
            NodeClassification::Literal => NodeIdClassification::Literal(LiteralNodeId(node.0)),
            NodeClassification::Command => NodeIdClassification::Command(CommandNodeId(node.0)),
            NodeClassification::Argument => NodeIdClassification::Argument(ArgumentNodeId(node.0)),
        }
    }

    /// Returns whether the given ID points to a command node.
    #[must_use]
    pub fn is_command_node(&self, node: NodeId) -> bool {
        matches!(self[node].classification(), NodeClassification::Command)
    }

    #[must_use]
    pub fn get_relevant_nodes(&self, reader: &mut StringReader, node: NodeId) -> Vec<NodeId> {
        let children = self.get_children(node);
        let mut literals = Vec::new();
        let mut commands = Vec::new();
        let mut arguments = Vec::new();
        for child in children {
            let id = self.classify_id(child);
            match id {
                NodeIdClassification::Root => {}
                NodeIdClassification::Literal(literal) => literals.push(literal),
                NodeIdClassification::Command(command) => commands.push(command),
                NodeIdClassification::Argument(arg) => arguments.push(arg),
            }
        }

        // Priority order:
        // 1. Commands > Literals
        // 2. Arguments

        if !literals.is_empty() || !commands.is_empty() {
            let cursor = reader.cursor();
            while !matches!(reader.peek(), None | Some(' ')) {
                reader.skip();
            }
            let new_cursor = reader.cursor();
            reader.set_cursor(cursor);
            let text = &reader.string()[cursor..new_cursor];

            for command in commands {
                if self[command].meta.literal == text {
                    return vec![command.into()];
                }
            }
            for literal in literals {
                if self[literal].meta.literal == text {
                    return vec![literal.into()];
                }
            }
        }

        arguments.into_iter().map(ArgumentNodeId::into).collect()
    }

    /// Parses the given node, returning an error on failure.
    pub fn parse(
        &self,
        node_id: NodeId,
        reader: &mut StringReader,
        command_context_builder: &mut CommandContextBuilder,
    ) -> Result<(), CommandSyntaxError> {
        match &self[node_id] {
            AttachedNode::Root(_) => {}
            AttachedNode::Literal(node) => {
                let start = reader.cursor();
                let Ok(end) = AttachedNode::parse_literal(reader, &node.meta.literal) else {
                    return Err(LITERAL_INCORRECT
                        .create(reader, TextComponent::text(node.meta.literal.to_string())));
                };
                command_context_builder.with_node(node_id, StringRange::between(start, end));
            }
            AttachedNode::Command(node) => {
                let start = reader.cursor();
                let Ok(end) = AttachedNode::parse_literal(reader, &node.meta.literal) else {
                    return Err(LITERAL_INCORRECT
                        .create(reader, TextComponent::text(node.meta.literal.to_string())));
                };
                command_context_builder.with_node(node_id, StringRange::between(start, end));
            }
            AttachedNode::Argument(node) => {
                let start = reader.cursor();
                let result = node.meta.argument_type.parse(reader)?;
                let range = StringRange::between(start, reader.cursor());
                let parsed = ParsedArgument::new(range, result);
                command_context_builder.with_argument(node.meta.name.to_string(), Arc::new(parsed));
                command_context_builder.with_node(node_id, range);
            }
        }
        Ok(())
    }

    /// Resolves the given redirection with respect to this tree, which is the node from
    /// which redirection takes place.
    ///
    /// Returns [`Some`] if the node required could be found, and
    /// returns [`None`] otherwise.
    #[must_use]
    pub fn resolve(&self, redirect: Redirection) -> Option<NodeId> {
        match redirect {
            Redirection::Root => Some(ROOT_NODE_ID),
            Redirection::Global(id) => self.ids_map.get(&id).copied(),
            Redirection::Local(id) => (id.0 < self.size_nonzero()).then_some(id),
        }
    }

    /// Gets the command node by ID, given its literal.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<CommandNodeId> {
        self.command_node_mappings.get(name).copied()
    }

    /// Returns an iterator to all the nodes of this tree.
    pub fn iter(&self) -> std::slice::Iter<'_, AttachedNode> {
        self.nodes.iter()
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

/// Macro helper to create [`Index`] and [`IndexMut`] for [`Tree`] with typed IDs.
macro_rules! impl_index_index_mut {
    ($node_id: ident -> AttachedNode::$attached_node_enum: ident($attached_node: ident)) => {
        impl Index<$node_id> for Tree {
            type Output = $attached_node;

            fn index(&self, index: $node_id) -> &Self::Output {
                if let AttachedNode::$attached_node_enum(node) = &self.nodes[index.0.get() - 1] {
                    node
                } else {
                    unreachable!(
                        "Node should have been AttachedNode::{}",
                        stringify!($attached_node_enum)
                    )
                }
            }
        }

        impl IndexMut<$node_id> for Tree {
            fn index_mut(&mut self, index: $node_id) -> &mut Self::Output {
                if let AttachedNode::$attached_node_enum(node) = &mut self.nodes[index.0.get() - 1]
                {
                    node
                } else {
                    unreachable!(
                        "Node should have been AttachedNode::{}",
                        stringify!($attached_node_enum)
                    )
                }
            }
        }
    };
}

impl_index_index_mut!(LiteralNodeId -> AttachedNode::Literal(LiteralAttachedNode));
impl_index_index_mut!(CommandNodeId -> AttachedNode::Command(CommandAttachedNode));
impl_index_index_mut!(ArgumentNodeId -> AttachedNode::Argument(ArgumentAttachedNode));

impl<'a> IntoIterator for &'a Tree {
    type Item = &'a AttachedNode;
    type IntoIter = std::slice::Iter<'a, AttachedNode>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod test {
    use crate::command::argument_builder::{
        ArgumentBuilder, CommandArgumentBuilder, LiteralArgumentBuilder, RequiredArgumentBuilder,
    };
    use crate::command::argument_types::core::string::StringArgumentType;
    use crate::command::node::attached::NodeId;
    use crate::command::node::tree::{AmbiguityConsumer, Tree};

    #[test]
    fn adding_nodes() {
        // New tree (containing only one root node)
        let mut tree = Tree::new();
        assert_eq!(tree.size(), 1);

        // Adding one node.
        tree.add_child_to_root(CommandArgumentBuilder::new("foo", "A test command"));
        assert_eq!(tree.size(), 2);

        // Adding a node with children.
        tree.add_child_to_root(
            // Each subcommand is a child.
            CommandArgumentBuilder::new("bar", "Another test command")
                .then(LiteralArgumentBuilder::new("baz"))
                .then(LiteralArgumentBuilder::new("qux")),
        );
        assert_eq!(tree.size(), 5);
    }

    #[test]
    fn adding_children_to_attached_node() {
        let mut tree = Tree::new();

        let parent: NodeId = tree
            .add_child_to_root(CommandArgumentBuilder::new("foo", "A test command"))
            .into();

        tree.add_child(parent, LiteralArgumentBuilder::new("baz"));
        tree.add_child(parent, LiteralArgumentBuilder::new("qux"));

        assert_eq!(tree.size(), 4);
        assert_eq!(tree.get_children(parent).len(), 2);
    }

    #[test]
    #[should_panic = "Cannot add a CommandDetachedNode as a child of a non-root node"]
    fn adding_command_node_to_non_root_node() {
        let mut tree = Tree::new();

        let parent: NodeId = tree
            .add_child_to_root(CommandArgumentBuilder::new("foo", "A test command"))
            .into();

        tree.add_child(
            parent,
            CommandArgumentBuilder::new("bar", "Another test command"),
        );
    }

    #[test]
    fn finding_ambiguities() {
        struct Consumer {
            inputs_received: usize,
            expected_parent: NodeId,
            expected_sibling: NodeId,
        }

        impl AmbiguityConsumer for Consumer {
            fn ambiguous(
                &mut self,
                _tree: &Tree,
                parent: NodeId,
                _child: NodeId,
                sibling: NodeId,
                inputs: Vec<String>,
            ) {
                self.inputs_received += inputs.len();

                assert_eq!(self.expected_parent, parent);
                assert_eq!(self.expected_sibling, sibling);
            }
        }

        let mut tree = Tree::new();

        let parent: NodeId = tree
            .add_child_to_root(CommandArgumentBuilder::new("foo", "A test command"))
            .into();

        tree.add_child(parent, LiteralArgumentBuilder::new("hello"));
        tree.add_child(parent, LiteralArgumentBuilder::new("bye"));
        let sibling = tree.add_child(
            parent,
            RequiredArgumentBuilder::new("string", StringArgumentType::SingleWord),
        );

        let mut consumer = Consumer {
            inputs_received: 0,
            expected_parent: parent,
            expected_sibling: sibling,
        };
        tree.find_ambiguities(parent, &mut consumer);

        assert_eq!(consumer.inputs_received, 2);
    }
}
