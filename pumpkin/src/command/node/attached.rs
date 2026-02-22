use crate::command::context::string_range::StringRange;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::LITERAL_INCORRECT;
use crate::command::node::detached::GlobalNodeId;
use crate::command::node::tree::ROOT_NODE_ID;
use crate::command::node::{
    ArgumentNodeMetadata, Command, CommandNodeMetadata, LiteralNodeMetadata, NodeMetadata,
    OwnedNodeData, RedirectModifier, Redirection, Requirement,
};
use crate::command::string_reader::StringReader;
use pumpkin_util::text::TextComponent;
use rustc_hash::FxHashMap;
use std::borrow::Cow;
use std::num::NonZero;

/// Represents the unique integral number
/// of any node, with respect to a tree.
///
/// A [`NonZero<usize>`] is used internally in this
/// struct. This means [`Option<NodeId>`] carries the
/// same size as of [`NodeId`], but comes at the cost of
/// ID `0` being unassignable.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
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
pub struct LiteralNodeId(pub NonZero<usize>);

/// Represents the unique integral number
/// of a specific command node, with respect to a tree.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct CommandNodeId(pub NonZero<usize>);

/// Represents the unique integral number
/// of a specific argument node, with respect to a tree.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct ArgumentNodeId(pub NonZero<usize>);

impl From<RootNodeId> for NodeId {
    fn from(_id: RootNodeId) -> Self {
        ROOT_NODE_ID
    }
}

impl From<LiteralNodeId> for NodeId {
    fn from(id: LiteralNodeId) -> Self {
        Self(id.0)
    }
}

impl From<CommandNodeId> for NodeId {
    fn from(id: CommandNodeId) -> Self {
        Self(id.0)
    }
}

impl From<ArgumentNodeId> for NodeId {
    fn from(id: ArgumentNodeId) -> Self {
        Self(id.0)
    }
}

/// Represents a node which has been attached as the root of a [`Tree`].
#[derive(Clone)]
pub struct RootAttachedNode {
    pub owned: OwnedNodeData,
    pub children: FxHashMap<String, NodeId>,
}

impl Default for RootAttachedNode {
    fn default() -> Self {
        Self::new()
    }
}

impl RootAttachedNode {
    #[must_use]
    pub fn new() -> Self {
        Self {
            owned: OwnedNodeData {
                global_id: GlobalNodeId::new(),
                requirement: Requirement::AlwaysQualified,
                modifier: RedirectModifier::OneSource,
                permission: None,
                forks: false,
                command: None,
            },
            children: FxHashMap::default(),
        }
    }
}

/// Represents a literal, non-command node that has already been attached
/// to a [`Tree`].
#[derive(Clone)]
pub struct LiteralAttachedNode {
    pub owned: OwnedNodeData,
    pub children: FxHashMap<String, NodeId>,
    pub redirect: Option<Redirection>,
    pub meta: LiteralNodeMetadata,
}

/// Represents a literal, command node that has already been attached
/// to a [`Tree`].
#[derive(Clone)]
pub struct CommandAttachedNode {
    pub owned: OwnedNodeData,
    pub children: FxHashMap<String, NodeId>,
    pub redirect: Option<Redirection>,
    pub meta: CommandNodeMetadata,
}

/// Represents a node that accepts a specific type of argument that has already been attached
/// to a [`Tree`].
#[derive(Clone)]
pub struct ArgumentAttachedNode {
    pub owned: OwnedNodeData,
    pub children: FxHashMap<String, NodeId>,
    pub redirect: Option<Redirection>,
    pub meta: ArgumentNodeMetadata,
}

/// Allows a way to store the kind of node
/// without any actual cloning of [`NodeMetadata`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NodeClassification {
    Root,
    Literal,
    Command,
    Argument,
}

/// Represents a node not attached to a [`Tree`] yet.
#[derive(Clone)]
pub enum AttachedNode {
    Root(RootAttachedNode),
    Literal(LiteralAttachedNode),
    Command(CommandAttachedNode),
    Argument(ArgumentAttachedNode),
}

/// Returned when a literal could not be parsed.
pub struct CouldNotParseLiteral;

impl AttachedNode {
    /// Creates an [`AttachedNode`] from its properties allowing any [`NodeMetadata`].
    #[must_use]
    pub fn from_parts(
        owned: OwnedNodeData,
        children: FxHashMap<String, NodeId>,
        redirect: Option<Redirection>,
        meta: NodeMetadata,
    ) -> Self {
        match meta {
            NodeMetadata::Root => Self::Root(RootAttachedNode { owned, children }),
            NodeMetadata::Literal(meta) => Self::Literal(LiteralAttachedNode {
                owned,
                children,
                redirect,
                meta,
            }),
            NodeMetadata::Command(meta) => Self::Command(CommandAttachedNode {
                owned,
                children,
                redirect,
                meta,
            }),
            NodeMetadata::Argument(meta) => Self::Argument(ArgumentAttachedNode {
                owned,
                children,
                redirect,
                meta,
            }),
        }
    }

    /// Gets the classification of this node.
    /// This is a relatively cheap operation.
    #[must_use]
    pub const fn classification(&self) -> NodeClassification {
        match self {
            Self::Root(_) => NodeClassification::Root,
            Self::Literal(_) => NodeClassification::Literal,
            Self::Command(_) => NodeClassification::Command,
            Self::Argument(_) => NodeClassification::Argument,
        }
    }

    /// Gets the global ID from this node.
    #[must_use]
    pub const fn global_id(&self) -> GlobalNodeId {
        self.owned_node_data_ref().global_id
    }

    /// Gets a reference to the owned data of this node.
    #[must_use]
    pub const fn owned_node_data_ref(&self) -> &OwnedNodeData {
        match self {
            Self::Root(node) => &node.owned,
            Self::Literal(node) => &node.owned,
            Self::Command(node) => &node.owned,
            Self::Argument(node) => &node.owned,
        }
    }

    /// Gets a mutable reference to the owned data of this node.
    pub const fn owned_node_data_mut_ref(&mut self) -> &mut OwnedNodeData {
        match self {
            Self::Root(node) => &mut node.owned,
            Self::Literal(node) => &mut node.owned,
            Self::Command(node) => &mut node.owned,
            Self::Argument(node) => &mut node.owned,
        }
    }

    /// Gets a reference to the children IDs of this node.
    #[must_use]
    pub const fn children_ref(&self) -> &FxHashMap<String, NodeId> {
        match self {
            Self::Root(node) => &node.children,
            Self::Literal(node) => &node.children,
            Self::Command(node) => &node.children,
            Self::Argument(node) => &node.children,
        }
    }

    /// Gets a mutable reference to the children IDs of this node.
    pub const fn children_mut_ref(&mut self) -> &mut FxHashMap<String, NodeId> {
        match self {
            Self::Root(node) => &mut node.children,
            Self::Literal(node) => &mut node.children,
            Self::Command(node) => &mut node.children,
            Self::Argument(node) => &mut node.children,
        }
    }

    /// Gets the name of this node.
    #[must_use]
    pub fn name(&self) -> String {
        match self {
            Self::Root(_) => String::new(),
            Self::Literal(node) => node.meta.literal.to_string(),
            Self::Command(node) => node.meta.literal.to_string(),
            Self::Argument(node) => node.meta.name.to_string(),
        }
    }

    /// Gets the redirection of this node.
    #[must_use]
    pub const fn redirect(&self) -> Option<Redirection> {
        match self {
            Self::Root(_) => None,
            Self::Literal(node) => node.redirect,
            Self::Command(node) => node.redirect,
            Self::Argument(node) => node.redirect,
        }
    }

    /// Gets an [`Option`] of a mutable reference to the redirection of this node.
    pub const fn redirect_mut_ref(&mut self) -> Option<&mut Redirection> {
        match self {
            Self::Root(_) => None,
            Self::Literal(node) => node.redirect.as_mut(),
            Self::Command(node) => node.redirect.as_mut(),
            Self::Argument(node) => node.redirect.as_mut(),
        }
    }

    /// Gets the requirement for this node to be run.
    ///
    /// Note that this does not account for the permission required; that is separately
    /// stored. Use the [`permission`] method to index it.
    ///
    /// [`permission`]: AttachedNode::permission
    #[must_use]
    pub const fn requirement(&self) -> &Requirement {
        &self.owned_node_data_ref().requirement
    }

    /// Sets the requirement for this node to be run to a value.
    ///
    /// Note that this does not account for the permission required; that is separately
    /// stored. Use the [`set_permission`] method to set that field's value instead.
    ///
    /// [`set_permission`]: AttachedNode::set_permission
    pub fn set_requirement(&mut self, requirement: Requirement) {
        self.owned_node_data_mut_ref().requirement = requirement;
    }

    /// Gets the permission required for this node to be run.
    ///
    /// Note that this does not account for the extra requirement required; that is separately
    /// stored. Use the [`requirement`] method to index it.
    ///
    /// [`requirement`]: AttachedNode::requirement
    #[must_use]
    pub fn permission(&self) -> Option<&str> {
        self.owned_node_data_ref().permission.as_deref()
    }

    /// Sets the permission required for this node to be run to a value.
    ///
    /// Note that this does not account for the extra requirement required; that is separately
    /// stored. Use the [`set_requirement`] method to set that field's value instead.
    ///
    /// [`set_requirement`]: AttachedNode::set_requirement
    pub fn set_permission<P>(&mut self, permission: Option<P>)
    where
        P: Into<Cow<'static, str>>,
    {
        self.owned_node_data_mut_ref().permission = permission.map(Into::into);
    }

    /// Gets the modifier for this node to be run.
    #[must_use]
    pub const fn modifier(&self) -> &RedirectModifier {
        &self.owned_node_data_ref().modifier
    }

    /// Sets the modifier for this node to a value.
    pub fn set_modifier(&mut self, modifier: RedirectModifier) {
        self.owned_node_data_mut_ref().modifier = modifier;
    }

    /// Whether this node forks [`CommandSources`] or not.
    #[must_use]
    pub const fn forks(&self) -> bool {
        self.owned_node_data_ref().forks
    }

    /// Sets whether this node forks [`CommandSources`] or not.
    pub const fn set_forks(&mut self, forks: bool) {
        self.owned_node_data_mut_ref().forks = forks;
    }

    /// Gets the executable command for this node.
    #[must_use]
    pub fn command(&self) -> &Option<Command> {
        &self.owned_node_data_ref().command
    }

    /// Sets the executable command for this node.
    pub fn set_command(&mut self, command: Option<Command>) {
        self.owned_node_data_mut_ref().command = command;
    }

    /// Get the usage text of this node.
    #[must_use]
    pub fn usage_text(&self) -> String {
        match self {
            Self::Root(_) => String::new(),
            Self::Literal(node) => node.meta.literal.to_string(),
            Self::Command(node) => node.meta.literal.to_string(),
            Self::Argument(node) => format!("<{}>", node.meta.name),
        }
    }

    /// Checks if the given input is valid for this node.
    #[must_use]
    pub fn is_valid_input(&self, input: &str) -> bool {
        match self {
            Self::Root(_) => false,
            Self::Literal(node) => {
                let mut reader = StringReader::new(input);
                Self::parse_literal(&mut reader, &node.meta.literal).is_ok()
            }
            Self::Command(node) => {
                let mut reader = StringReader::new(input);
                Self::parse_literal(&mut reader, &node.meta.literal).is_ok()
            }
            Self::Argument(node) => {
                let mut reader = StringReader::new(input);
                let parsed = node.meta.argument_type.parse(&mut reader);
                if parsed.is_ok() {
                    matches!(reader.peek(), Some(' ') | None)
                } else {
                    false
                }
            }
        }
    }

    /// Parses the given input for this node.
    /// Prefer using a [`CommandDispatcher`] over this function directly.
    pub fn parse(
        &self,
        reader: &mut StringReader,
        literal: &str,
    ) -> Result<StringRange, CommandSyntaxError> {
        let start = reader.cursor();
        Self::parse_literal(reader, literal).map_or_else(
            |_| Err(LITERAL_INCORRECT.create(reader, TextComponent::text(literal.to_string()))),
            |end| Ok(StringRange::between(start, end)),
        )
    }

    /// Internal function to parse a literal. Used by [`Tree`].
    pub fn parse_literal(
        reader: &mut StringReader,
        literal: &str,
    ) -> Result<usize, CouldNotParseLiteral> {
        let start = reader.cursor();
        let len = literal.len();
        if reader.can_read_bytes(len) {
            let end = start + len;
            if &reader.string()[start..end] == literal {
                reader.set_cursor(end);
                if matches!(reader.peek(), Some(' ') | None) {
                    return Ok(end);
                }
                reader.set_cursor(start);
            }
        }
        Err(CouldNotParseLiteral)
    }

    /// Gets examples accepted by this node.
    #[must_use]
    pub fn examples(&self) -> Vec<String> {
        match self {
            Self::Root(_) => Vec::new(),
            Self::Literal(node) => vec![node.meta.literal.to_string()],
            Self::Command(node) => vec![node.meta.literal.to_string()],
            Self::Argument(node) => node.meta.argument_type.examples(),
        }
    }
}
