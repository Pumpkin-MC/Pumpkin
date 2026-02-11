pub mod attached;
pub mod detached;
pub mod dispatcher;
pub mod tree;

use crate::command::argument_types::argument_type::AnyArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::context::command_source::CommandSource;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::node::detached::GlobalNodeId;
use std::borrow::Cow;
use std::pin::Pin;
use std::sync::Arc;

/// Represents a [`CommandExecutor`]'s result.
pub type CommandExecutorResult =
    Pin<Box<dyn Future<Output = Result<i32, CommandSyntaxError>> + Send>>;

/// A struct implementing this trait is able to run with a given context.
pub trait CommandExecutor: Sync + Send {
    /// Executes this executor for a command.
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult;
}

/// A function that takes a context and returns a command result.
pub type Command = Arc<dyn CommandExecutor>;

/// Represents the result of [`Arc<CommandSource>`]s from a [`CommandContext`].
pub type RedirectModifierResult<'a> =
    Pin<Box<dyn Future<Output = Result<Vec<Arc<CommandSource>>, CommandSyntaxError>> + Send + 'a>>;

/// A function that returns a new collection of sources from a given context.
#[derive(Clone)]
pub enum RedirectModifier {
    /// Always returns only the source from the given context.
    OneSource,

    /// Returns multiple [`CommandSource`]s from one context via
    /// custom behavior.
    Custom(Arc<dyn Fn(&CommandContext) -> RedirectModifierResult<'_> + Send + Sync>),
}

impl RedirectModifier {
    /// Tries to provide a [`Vec`] of [`Arc<CommandSource>`] from a
    /// given [`CommandContext`].
    #[must_use]
    pub fn sources<'c>(&self, command_context: &'c CommandContext) -> RedirectModifierResult<'c> {
        match self {
            Self::OneSource => Box::pin(async move { Ok(vec![command_context.source.clone()]) }),
            Self::Custom(function) => function(command_context),
        }
    }
}

/// A structure that returns if the source is qualified enough to run the command.
#[derive(Clone)]
pub enum Requirement {
    /// Always returns `true`, i.e. no matter the source,
    /// it will always be qualified enough to run the command.
    AlwaysQualified,

    /// The given source must satisfy the condition to
    /// be allowed to run the command.
    Condition(Arc<dyn Fn(&CommandSource) -> bool + Send + Sync>),
}

impl Requirement {
    /// Evaluates the given condition, returning whether the
    /// given [`CommandSource`] satisfies this requirement.
    #[must_use]
    pub fn evaluate(&self, command_source: &CommandSource) -> bool {
        match self {
            Self::AlwaysQualified => true,
            Self::Condition(condition) => condition(command_source),
        }
    }
}

/// Stores common owned data for a node.
#[derive(Clone)]
pub struct OwnedNodeData {
    pub global_id: GlobalNodeId,
    pub requirement: Requirement,
    pub modifier: RedirectModifier,
    pub forks: bool,
    pub command: Option<Command>,
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
        Self {
            literal: literal.clone(),
            literal_lowercase: literal.to_uppercase(),
        }
    }
}

/// A special type of [`LiteralNodeMetadata`], containing
/// a description for the command as well.
#[derive(Clone)]
pub struct CommandNodeMetadata {
    pub literal: Cow<'static, str>,
    pub literal_lowercase: String,
    pub description: Cow<'static, str>,
}

impl CommandNodeMetadata {
    pub fn new(
        literal: impl Into<Cow<'static, str>>,
        description: impl Into<Cow<'static, str>>,
    ) -> Self {
        let literal = literal.into();
        Self {
            literal: literal.clone(),
            literal_lowercase: literal.to_uppercase(),
            description: description.into(),
        }
    }
}

/// Represents the extra metadata of an argument of any type.
#[derive(Clone)]
pub struct ArgumentNodeMetadata {
    pub name: Cow<'static, str>,
    pub argument_type: Arc<dyn AnyArgumentType>,
}

impl ArgumentNodeMetadata {
    pub fn new(
        name: impl Into<Cow<'static, str>>,
        argument_type: Arc<dyn AnyArgumentType>,
    ) -> Self {
        Self {
            name: name.into(),
            argument_type,
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
    Argument(ArgumentNodeMetadata),
}

/// Stores where this redirection would lead to.
#[derive(Clone)]
pub enum Redirection {
    Root,
    Global(GlobalNodeId),
}
