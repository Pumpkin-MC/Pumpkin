pub mod attached;
pub mod detached;
pub mod dispatcher;
pub mod tree;

use crate::command::argument_types::argument_type::AnyArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::context::command_source::CommandSource;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::node::attached::NodeId;
use crate::command::node::detached::GlobalNodeId;
use std::borrow::Cow;
use std::pin::Pin;
use std::sync::Arc;

/// Represents a [`CommandExecutor`]'s result.
pub type CommandExecutorResult<'a> =
    Pin<Box<dyn Future<Output = Result<i32, CommandSyntaxError>> + Send + 'a>>;

/// A struct implementing this trait is able to run with a given context.
pub trait CommandExecutor: Sync + Send {
    /// Executes this executor for a command.
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a>;
}

impl<F> CommandExecutor for F
where
    F: for<'c> Fn(&'c CommandContext) -> CommandExecutorResult<'c> + Send + Sync,
{
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        self(context)
    }
}

/// A function that takes a context and returns a command result.
pub type Command = Arc<dyn CommandExecutor>;

/// Represents the result of [`Arc<CommandSource>`]s from a [`CommandContext`].
pub type RedirectModifierResult<'a> =
    Pin<Box<dyn Future<Output = Result<Vec<Arc<CommandSource>>, CommandSyntaxError>> + Send + 'a>>;

/// A function that performs the required modification.
pub type RedirectModifierExecutor =
    dyn for<'c> Fn(&'c CommandContext) -> RedirectModifierResult<'c> + Send + Sync;

/// A function that returns a new collection of sources from a given context.
#[derive(Clone)]
pub enum RedirectModifier {
    /// Always returns only the source from the given context.
    KeepSource,

    /// Returns multiple [`CommandSource`]s from one context via
    /// custom behavior.
    Custom(Arc<RedirectModifierExecutor>),
}

impl RedirectModifier {
    /// Tries to provide a [`Vec`] of [`Arc<CommandSource>`] from a
    /// given [`CommandContext`].
    #[must_use]
    pub fn sources<'c>(&self, command_context: &'c CommandContext) -> RedirectModifierResult<'c> {
        match self {
            Self::KeepSource => Box::pin(async move { Ok(vec![command_context.source.clone()]) }),
            Self::Custom(function) => function(command_context),
        }
    }
}

/// Represents the result of a node requirement as a pinned boxed [`Future`].
pub type RequirementResult<'a> = Pin<Box<dyn Future<Output = bool> + Send + 'a>>;

/// A predicate that returns if the provided source satisfies it.
#[derive(Clone)]
pub struct Requirement(pub Arc<dyn Fn(&CommandSource) -> RequirementResult<'_> + Send + Sync>);

impl Requirement {
    /// Evaluates the given condition, returning whether the
    /// given [`CommandSource`] satisfies this requirement.
    #[must_use]
    pub fn evaluate<'a>(&'a self, command_source: &'a CommandSource) -> RequirementResult<'a> {
        self.0(command_source)
    }
}

impl<F> From<F> for Requirement
where
    F: Fn(&CommandSource) -> RequirementResult<'_> + Send + Sync + 'static,
{
    fn from(value: F) -> Self {
        Self(Arc::new(value))
    }
}

// Permissions
impl From<String> for Requirement {
    fn from(value: String) -> Self {
        Self(Arc::new({
            let permission = Arc::new(value);

            move |source| {
                let cloned_permission = permission.clone();
                Box::pin(async move { source.has_permission(&cloned_permission).await })
            }
        }))
    }
}

impl From<&'static str> for Requirement {
    fn from(value: &'static str) -> Self {
        Self(Arc::new(move |source| {
            Box::pin(async move { source.has_permission(value).await })
        }))
    }
}

/// A structure that returns if the source is qualified enough to run the command.
#[derive(Clone)]
pub struct Requirements(pub Vec<Requirement>);

impl Requirements {
    /// Creates a new `Requirements` with no requirements in it.
    /// If used, this will always return `true` when evaluated.
    #[must_use]
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    /// Evaluates the given condition, returning whether the
    /// given [`CommandSource`] satisfies all contained requirements.
    #[must_use]
    pub fn evaluate<'a>(&'a self, command_source: &'a CommandSource) -> RequirementResult<'a> {
        let futures = self
            .0
            .iter()
            .map(|predicate| predicate.evaluate(command_source));
        Box::pin(async move {
            for future in futures {
                if !future.await {
                    return false;
                }
            }
            true
        })
    }
}

impl Default for Requirements {
    fn default() -> Self {
        Self::new()
    }
}

/// Stores common owned data for a node.
#[derive(Clone)]
pub struct OwnedNodeData {
    pub global_id: GlobalNodeId,
    pub requirements: Requirements,
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
            literal_lowercase: literal.to_lowercase(),
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
            literal_lowercase: literal.to_lowercase(),
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
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Redirection {
    /// Leads to the root of the tree.
    Root,

    /// Leads to a node in the tree from its tree-local ID.
    Local(NodeId),

    /// Leads to a node in the tree from its global ID.
    Global(GlobalNodeId),
}

impl<T: Into<NodeId>> From<T> for Redirection {
    fn from(value: T) -> Self {
        Self::Local(value.into())
    }
}

impl From<GlobalNodeId> for Redirection {
    fn from(value: GlobalNodeId) -> Self {
        Self::Global(value)
    }
}
