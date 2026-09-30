pub mod attached;
pub mod detached;
pub mod dispatcher;
pub mod tree;

use crate::argument_types::argument_type::AnyArgumentType;
use crate::context::command_context::CommandContext;
use crate::errors::command_syntax_error::CommandSyntaxError;
use crate::node::attached::NodeId;
use crate::node::detached::GlobalNodeId;
use crate::source::{CommandSource, DummySource};
use crate::suggestion::provider::SuggestionProvider;
use std::borrow::Cow;
use std::sync::Arc;

/// Represents a [`CommandExecutor`]'s result.
///
/// If the command **ran successfully**, an [`Ok`] is returned containing an [`i32`].
/// This represents the 'output value' of the command, which is *homologous* to the
/// `int` that command executors in vanilla return **upon success**.
///
/// **You should choose the successful result as `1` if**:
/// - you don't know what value to use for a success for your
///   own commands, or
/// - you don't understand what this value means, or
/// - you just simply don't care about this value at all
///
/// If the command **fails**, an [`Err`] is returned, containing the [`CommandSyntaxError`]
/// that led to this result.
pub type CommandExecutorResult = Result<i32, CommandSyntaxError>;

/// A struct implementing this trait is able to run with a given context.
pub trait CommandExecutor<S: CommandSource = DummySource>: Sync + Send {
    /// Executes this executor for a command.
    fn execute(&self, context: &CommandContext<S>) -> CommandExecutorResult;
}

impl<F, S: CommandSource> CommandExecutor<S> for F
where
    F: Fn(&CommandContext<S>) -> CommandExecutorResult + Send + Sync,
{
    fn execute(&self, context: &CommandContext<S>) -> CommandExecutorResult {
        self(context)
    }
}

/// A function that takes a context and returns a command result.
pub type Command<S = DummySource> = Arc<dyn CommandExecutor<S>>;

/// Represents the result of [`Arc<S>`]s from a [`CommandContext`].
pub type RedirectModifierResult<S = DummySource> = Result<Vec<Arc<S>>, CommandSyntaxError>;

/// A function that performs the required modification.
pub type RedirectModifierExecutor<S = DummySource> =
    dyn Fn(&CommandContext<S>) -> RedirectModifierResult<S> + Send + Sync;

/// A function that returns a new collection of sources from a given context.
#[derive(Clone)]
pub enum RedirectModifier<S: CommandSource = DummySource> {
    /// Always returns only the source from the given context.
    KeepSource,

    /// Returns multiple [`CommandSource`]s from one context via
    /// custom behavior.
    Custom(Arc<RedirectModifierExecutor<S>>),
}

impl<S: CommandSource> RedirectModifier<S> {
    /// Tries to provide a [`Vec`] of [`Arc<S>`] from a
    /// given [`CommandContext`].
    pub fn sources(&self, command_context: &CommandContext<S>) -> RedirectModifierResult<S> {
        match self {
            Self::KeepSource => Ok(vec![command_context.source.clone()]),
            Self::Custom(function) => function(command_context),
        }
    }
}

/// Represents the result of a node requirement.
pub type RequirementResult = bool;

/// A predicate that returns if the provided source satisfies it.
#[derive(Clone)]
pub struct Requirement<S: CommandSource = DummySource>(
    pub Arc<dyn Fn(&S) -> RequirementResult + Send + Sync>,
);

impl<S: CommandSource> Requirement<S> {
    /// Evaluates the given condition, returning whether the
    /// given [`CommandSource`] satisfies this requirement.
    #[must_use]
    pub fn evaluate(&self, command_source: &S) -> RequirementResult {
        self.0(command_source)
    }
}

impl<F, S: CommandSource> From<F> for Requirement<S>
where
    F: Fn(&S) -> RequirementResult + Send + Sync + 'static,
{
    fn from(value: F) -> Self {
        Self(Arc::new(value))
    }
}

// Permissions
impl<S: CommandSource> From<String> for Requirement<S> {
    fn from(value: String) -> Self {
        Self(Arc::new({
            let permission = Arc::new(value);

            move |source: &S| source.has_permission(&permission)
        }))
    }
}

impl<S: CommandSource> From<&'static str> for Requirement<S> {
    fn from(value: &'static str) -> Self {
        Self(Arc::new(move |source: &S| source.has_permission(value)))
    }
}

/// A structure that returns if the source is qualified enough to run the command.
#[derive(Clone)]
pub struct Requirements<S: CommandSource = DummySource>(pub Vec<Requirement<S>>);

impl<S: CommandSource> Requirements<S> {
    /// Creates a new `Requirements` with no requirements in it.
    /// If used, this will always return `true` when evaluated.
    #[must_use]
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    /// Evaluates the given condition, returning whether the
    /// given [`CommandSource`] satisfies all contained requirements.
    #[must_use]
    pub fn evaluate(&self, command_source: &S) -> RequirementResult {
        for predicate in &self.0 {
            if !predicate.evaluate(command_source) {
                return false;
            }
        }

        true
    }
}

impl<S: CommandSource> Default for Requirements<S> {
    fn default() -> Self {
        Self::new()
    }
}

/// Stores common owned data for a node.
#[derive(Clone)]
pub struct OwnedNodeData<S: CommandSource = DummySource> {
    pub global_id: GlobalNodeId,
    pub requirements: Requirements<S>,
    pub modifier: RedirectModifier<S>,
    pub forks: bool,
    pub command: Option<Command<S>>,
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
    pub source: Option<String>,
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
            source: None,
        }
    }
}

/// Represents the extra metadata of an argument of any type.
#[derive(Clone)]
pub struct ArgumentNodeMetadata<S: CommandSource = DummySource> {
    pub name: Cow<'static, str>,
    pub argument_type: Arc<dyn AnyArgumentType<S>>,
    pub suggestion_provider: Option<Arc<dyn SuggestionProvider<S>>>,
}

impl<S: CommandSource> ArgumentNodeMetadata<S> {
    pub fn new(
        name: impl Into<Cow<'static, str>>,
        argument_type: Arc<dyn AnyArgumentType<S>>,
        suggestion_provider: Option<Arc<dyn SuggestionProvider<S>>>,
    ) -> Self {
        Self {
            name: name.into(),
            argument_type,
            suggestion_provider,
        }
    }
}

/// Represents the extra metadata for nodes of different types. Can be of the root, a literal, command or an argument.
pub enum NodeMetadata<S: CommandSource = DummySource> {
    /// Metadata of the root node.
    Root,

    /// Metadata of a literal node that doesn't start a command.
    Literal(LiteralNodeMetadata),

    /// Metadata of a literal node that starts a command.
    Command(CommandNodeMetadata),

    /// Metadata of an argument node.
    Argument(ArgumentNodeMetadata<S>),
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
