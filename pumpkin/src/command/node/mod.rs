pub mod tree;
pub mod dispatcher;
pub mod detached;
pub mod attached;

use std::borrow::Cow;
use std::sync::{Arc, LazyLock};
use crate::command::argument_types::argument_type::{AnyArgumentType, ArgumentType};
use crate::command::context::command_context::{CommandContext};
use crate::command::context::command_source::CommandSource;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::node::attached::NodeId;
use crate::command::node::detached::GlobalNodeId;

/// A function that takes a context and returns a command result.
pub type Command = Arc<dyn Fn(&CommandContext) -> Result<i32, CommandSyntaxError>>;

/// A function that returns a new collection of sources from a given context.
#[derive(Clone)]
pub enum RedirectModifier {
    /// Always returns only the source from the given context.
    OneSource,

    /// Returns multiple [`CommandSource`]s from one context via
    /// custom behavior.
    Custom(Arc<dyn Fn(&CommandContext) -> Vec<Arc<CommandSource>>>)
}

impl RedirectModifier {
    /// Returns a [`Vec`] of [`Arc<CommandSource>`] from a
    /// given [`CommandContext`].
    pub fn sources(&self, command_context: &CommandContext) -> Vec<Arc<CommandSource>> {
        match self {
            Self::OneSource => vec![command_context.source.clone()],
            Self::Custom(function) => function(command_context)
        }
    }
}

/// A structure that returns if the source is qualified enough to run the command.
pub enum Requirement {
    /// Always returns `true`, i.e. no matter the source,
    /// it will always be qualified enough to run the command.
    AlwaysQualified,

    /// The given source must satisfy the condition to
    /// be allowed to run the command.
    Condition(Arc<dyn Fn(&CommandSource) -> bool>)
}

impl Requirement {
    /// Evaluates the given condition, returning whether the
    /// given [`CommandSource`] satisfies this requirement.
    pub fn evaluate(&self, command_source: &CommandSource) -> bool {
        match self {
            Self::AlwaysQualified => true,
            Self::Condition(condition) => condition(command_source)
        }
    }
}

/// Stores common owned data for a node.
pub struct OwnedNodeData {
    pub global_id: GlobalNodeId,
    pub requirement: Requirement,
    pub modifier: RedirectModifier,
    pub forks: bool,
    pub command: Option<Command>
}

/// Represents the extra metadata of a node storing a literal.
#[derive(Clone)]
pub struct LiteralNodeMetadata {
    pub literal: Cow<'static, str>,
    pub literal_lowercase: String,
}

impl LiteralNodeMetadata {
    pub fn new(literal: impl Into<Cow<'static, str>>) -> Self {
        let literal = literal.into();
        LiteralNodeMetadata {
            literal: literal.clone(),
            literal_lowercase: literal.to_uppercase()
        }
    }
}

/// A special type of [`LiteralNodeMetadata`], containing
/// a description for the command as well.
#[derive(Clone)]
pub struct CommandNodeMetadata {
    pub literal: Cow<'static, str>,
    pub literal_lowercase: String,
    pub description: Cow<'static, str>
}

impl CommandNodeMetadata {
    pub fn new(literal: impl Into<Cow<'static, str>>, description: impl Into<Cow<'static, str>>) -> Self {
        let literal = literal.into();
        CommandNodeMetadata {
            literal: literal.clone(),
            literal_lowercase: literal.to_uppercase(),
            description: description.into()
        }
    }
}

/// Represents the extra metadata of an argument of any type.
#[derive(Clone)]
pub struct ArgumentNodeMetadata {
    pub name: Cow<'static, str>,
    pub argument_type: Arc<dyn AnyArgumentType>
}

impl ArgumentNodeMetadata {
    pub fn new(name: impl Into<Cow<'static, str>>, argument_type: Arc<dyn AnyArgumentType>) -> Self {
        ArgumentNodeMetadata {
            name: name.into(),
            argument_type
        }
    }
}

/// Represents the extra metadata for nodes of different types. Can be of the root, a literal, command or an argument.
pub enum NodeMetadata {
    /// Metadata of the root node.
    Root,

    /// Metadata of a literal node that doesn't start a command.
    Literal(LiteralNodeMetadata),

    /// Metadata of a literal node that starts a command.
    Command(CommandNodeMetadata),

    /// Metadata of an argument node.
    Argument(ArgumentNodeMetadata)
}

/// Stores where this redirection would lead to.
#[derive(Clone)]
pub enum Redirection {
    Root,
    Global(GlobalNodeId),
    Local(NodeId)
}

/*
impl Node {
    /// Gets a reference to the common data of this node that all nodes can have.
    pub fn common(&self) -> &CommonNode {
        match self {
            Node::Root(node) => &node.common_data,
            Node::Literal(node) => &node.common_data,
            Node::Command(node) => &node.common_data,
            Node::Argument(node) => &node.common_data
        }
    }

    /// Gets a mutable reference to the common data of this node that all nodes can have.
    pub fn common_mut(&mut self) -> &mut CommonNode {
        match self {
            Node::Root(node) => &mut node.common_data,
            Node::Literal(node) => &mut node.common_data,
            Node::Command(node) => &mut node.common_data,
            Node::Argument(node) => &mut node.common_data
        }
    }

    /// Returns the optional command of this node.
    pub fn command(&self) -> &Option<Command> {
        &self.common().command
    }

    /// Get all children of this node.
    pub fn children(&self) -> &FxHashMap<String, NodeId> {
        &self.common().children
    }

    /// Get the requirement of this node.
    pub fn requirement(&self) -> &dyn Fn(&CommandSource) -> bool {
        &*self.common().requirement
    }

    /// Get a child of this node by name.
    pub fn child(&self, name: &str) -> Option<NodeId> {
        self.common().children.get(name).copied()
    }

    /// Get the node this node redirects to.
    pub fn redirect(&self) -> Option<NodeId> {
        self.common().redirect
    }

    /// Get the redirect modifier of this node.
    pub fn modifier(&self) -> &RedirectModifier {
        &self.common().modifier
    }

    /// Sets the optional command of this node.
    pub fn set_command(&mut self, command: Option<Command>) {
         self.common_mut().command = command;
    }

    /// Sets the requirement of this node.
    pub fn set_requirement(&mut self, requirement: Requirement) {
        self.common_mut().requirement = requirement;
    }

    /// Sets the node this node redirects to.
    pub fn set_redirect(&mut self, redirect: Option<NodeId>) {
        self.common_mut().redirect = redirect;
    }

    /// Sets the redirect modifier of this node.
    pub fn set_modifier(&mut self, modifier: RedirectModifier) {
        self.common_mut().modifier = modifier;
    }

    /// Returns whether the source provided can use this node.
    pub fn can_be_used(&self, source: &CommandSource) -> bool {
        (self.common().requirement)(source)
    }

    /// Returns whether the given input is valid for this node.
    pub fn is_valid_input(&self, input: &str) -> bool {
        match self {
            Node::Root(_) => false,
            Node::Literal(node) => {
                let mut reader = StringReader::new(input);
                Self::parse_literal(&mut reader, &node.literal).is_ok()
            }
            Node::Command(node) => {
                let mut reader = StringReader::new(input);
                Self::parse_literal(&mut reader, &node.literal).is_ok()
            }
            Node::Argument(node) => {
                let mut reader = StringReader::new(input);
                let parsed = node.argument_type.parse(&mut reader);
                if parsed.is_ok() {
                    matches!(reader.peek(), Some(' ') | None)
                } else {
                    false
                }
            }
        }
    }

    pub fn parse(reader: &mut StringReader, context_builder: &mut ContextBuilder) {

    }

    /// Internal function for assisting in parsing a literal.
    fn parse_literal(reader: &mut StringReader, literal: &str) -> Result<usize, ()> {
        let start = reader.cursor();
        let len = literal.len();
        if reader.can_read_bytes(len) {
            let end = start + len;
            if &reader.string()[start..end] == literal {
                reader.set_cursor(end);
                if matches!(reader.peek(), Some(' ') | None) {
                    return Ok(end);
                } else {
                    reader.set_cursor(start);
                }
            }
        }
        Err(())
    }

    /// Gets the name of this node.
    pub fn name(&self) -> String {
        match self {
            Node::Root(_) => String::new(),
            Node::Literal(node) => node.literal.to_string(),
            Node::Command(node) => node.literal.to_string(),
            Node::Argument(node) => node.name.to_string()
        }
    }

    /// Gets the usage text of this node.
    pub fn usage_text(&self) -> String {
        match self {
            Node::Root(_) => String::new(),
            Node::Literal(node) => node.literal.to_string(),
            Node::Command(node) => node.literal.to_string(),
            Node::Argument(node) => format!("<{}>", node.name.to_string())
        }
    }
}
*/