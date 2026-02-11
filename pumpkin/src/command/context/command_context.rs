use crate::command::context::command_source::{CommandSource, ReturnValue};
use crate::command::context::string_range::StringRange;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::node::attached::NodeId;
use crate::command::node::dispatcher::{CommandDispatcher, ResultConsumer};
use crate::command::node::tree::Tree;
use crate::command::node::{Command, RedirectModifier};
use rustc_hash::FxHashMap;
use std::any::Any;
use std::sync::Arc;

/// Represents the current stage of the chain.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Stage {
    MODIFY,
    EXECUTE,
}

/// Represents a parsed node.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct ParsedNode {
    pub node: NodeId,
    pub range: StringRange,
}

/// Represents a suggestional context involving a node.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct SuggestionContext {
    pub parent: NodeId,
    pub starting_position: usize,
}

/// Represents a parsed argument of any type.
pub struct ParsedArgument {
    /// The range of this parsed argument.
    pub range: StringRange,

    /// The result of this parsed argument.
    pub result: Box<dyn Any + Send + Sync>,
}

impl ParsedArgument {
    /// Creates a new [`ParsedArgument`] from its range and resultant value.
    #[must_use]
    pub fn new(range: StringRange, result: Box<dyn Any + Send + Sync>) -> Self {
        Self { range, result }
    }
}

/// Represents the context used when commands are run.
#[derive(Clone)]
pub struct CommandContext {
    /// The source running the commands.
    pub source: Arc<CommandSource>,

    /// The input string ran as the command.
    pub input: String,

    /// Arguments which have been parsed and
    /// can be fetched for command execution.
    pub arguments: FxHashMap<String, Arc<ParsedArgument>>,

    /// The tree this context is related to.
    pub tree: Arc<Tree>,

    /// The root that this context will use, bound to the tree
    /// Not necessarily the root node of the tree, however.
    pub root: NodeId,

    /// All the parsed nodes of the command.
    pub nodes: Vec<ParsedNode>,

    /// The string range of input.
    pub range: StringRange,

    /// The child context of this context.
    pub child: Option<Arc<Self>>,

    /// The redirect modifier of this context.
    pub modifier: RedirectModifier,

    /// Whether this context forks or not.
    pub forks: bool,

    /// The command stored in this context which
    /// is run to get a command result.
    pub command: Option<Command>,
}

impl CommandContext {
    /// Copies this context with the source provided.
    #[must_use]
    pub fn with_source(&self, source: Arc<CommandSource>) -> Self {
        Self {
            source,
            input: self.input.clone(),
            arguments: self.arguments.clone(),
            nodes: self.nodes.clone(),
            range: self.range,
            child: self.child.clone(),
            modifier: self.modifier.clone(),
            forks: self.forks,
            command: self.command.clone(),
            tree: self.tree.clone(),
            root: self.root,
        }
    }

    /// Returns the child immediately below this node.
    #[must_use]
    pub const fn get_child(&self) -> Option<&Arc<Self>> {
        self.child.as_ref()
    }

    /// Returns the child which does not have a child which originated from this node.
    /// This may return itself.
    #[must_use]
    pub fn get_last_child(&self) -> &Self {
        let mut current_child = self;
        while let Some(child) = &current_child.child {
            current_child = child;
        }
        current_child
    }
}

/// Represents a linked chain of [`CommandContext`]s, where the previous links to the next as a child.
#[derive(Clone)]
pub struct ContextChain {
    /// The modifiers of this context chain.
    modifiers: Vec<Arc<CommandContext>>,

    /// That specific [`CommandContext`] to execute.
    execute: Arc<CommandContext>,
}

impl ContextChain {
    /// Creates a new chain of contexts from a vector of them and one to execute.
    ///
    /// # Panics
    ///
    /// Panics if the `execute` given is non-executable.
    #[must_use]
    pub fn new(modifiers: Vec<Arc<CommandContext>>, execute: Arc<CommandContext>) -> Self {
        assert!(
            execute.command.is_some(),
            "Expected last command in chain to be executable"
        );
        Self { modifiers, execute }
    }

    /// Tries to flatten a [`CommandContext`] in a [`Box`]. If no command
    /// is available at the end of chain, [`None`] is returned.
    #[must_use]
    pub fn try_flatten(root: &CommandContext) -> Option<Self> {
        let mut modifiers = Vec::new();
        let mut current = root;

        loop {
            if let Some(child) = current.get_child() {
                modifiers.push(child.clone());
                current = child;
            } else {
                return current
                    .command
                    .is_some()
                    .then(|| Self::new(modifiers, Arc::new(current.clone())));
            }
        }
    }

    /// Runs the given modifier with provided details.
    pub async fn run_modifier(
        modifier: &CommandContext,
        source: &Arc<CommandSource>,
        result_consumer: &dyn ResultConsumer,
        forked_mode: bool,
    ) -> Result<Vec<Arc<CommandSource>>, CommandSyntaxError> {
        let source_modifier = &modifier.modifier;

        if matches!(source_modifier, RedirectModifier::OneSource) {
            return Ok(vec![source.clone()]);
        }

        let context_to_use = modifier.with_source(source.clone());
        let mut result = source_modifier.sources(&context_to_use).await;

        if result.is_err() {
            result_consumer.on_command_completion(&context_to_use, ReturnValue::Failure);
            if forked_mode {
                result = Ok(vec![]);
            }
        }

        result
    }

    /// Runs the given executable, returning an [`i32`] on success.
    ///
    /// # Panics
    ///
    /// Panics if the `executable` provided cannot be executed.
    pub async fn run_executable(
        executable: &CommandContext,
        source: &Arc<CommandSource>,
        result_consumer: &dyn ResultConsumer,
        forked_mode: bool,
    ) -> Result<i32, CommandSyntaxError> {
        let context_to_use = executable.with_source(source.clone());

        let mut result = match &executable.command {
            None => panic!("Expected `executable` to be executable"),
            Some(command) => command.execute(&context_to_use).await,
        };

        if let Ok(result) = result {
            result_consumer.on_command_completion(&context_to_use, ReturnValue::Success(result));
            Ok(if forked_mode { 1 } else { result })
        } else {
            result_consumer.on_command_completion(&context_to_use, ReturnValue::Failure);
            if forked_mode {
                result = Ok(0);
            }
            result
        }
    }

    /// Executes all contexts in the chain, returning the ultimate result.
    pub async fn execute_all(
        &self,
        source: &Arc<CommandSource>,
        result_consumer: &dyn ResultConsumer,
    ) -> Result<i32, CommandSyntaxError> {
        if self.modifiers.is_empty() {
            return Self::run_executable(&self.execute, source, result_consumer, false).await;
        }

        let mut forked_mode = false;
        let mut current_sources: Vec<Arc<CommandSource>> = vec![source.clone()];

        for modifier in &self.modifiers {
            forked_mode |= modifier.forks;

            let mut next_sources = Vec::new();
            for source in current_sources {
                let mut to_add =
                    Self::run_modifier(modifier, &source, result_consumer, forked_mode).await?;
                next_sources.append(&mut to_add);
            }
            if next_sources.is_empty() {
                return Ok(0);
            }
            current_sources = next_sources;
        }

        let mut result = 0;
        for execution_source in current_sources {
            result += Self::run_executable(
                &self.execute,
                &execution_source,
                result_consumer,
                forked_mode,
            )
            .await?;
        }

        Ok(result)
    }

    /// Gets the current stage of this context.
    #[must_use]
    pub const fn get_stage(&self) -> Stage {
        if self.modifiers.is_empty() {
            Stage::EXECUTE
        } else {
            Stage::MODIFY
        }
    }

    /// Gets a reference to the top context of this chain.
    #[must_use]
    pub fn get_top_context(&self) -> &Arc<CommandContext> {
        if self.modifiers.is_empty() {
            &self.execute
        } else {
            &self.modifiers[0]
        }
    }

    /// Gets a mutable reference to the top context of this chain.
    pub fn get_top_context_mut(&mut self) -> &mut Arc<CommandContext> {
        if self.modifiers.is_empty() {
            &mut self.execute
        } else {
            &mut self.modifiers[0]
        }
    }

    /// Gets the next stage of this chain.
    #[must_use]
    pub fn next_stage(&self) -> Option<Self> {
        if self.modifiers.is_empty() {
            None
        } else {
            Some(Self::new(
                self.modifiers[1..].to_vec(),
                self.execute.clone(),
            ))
        }
    }
}

/// A builder that helps to create a [`CommandContext`].
///
/// This builder's lifetime is bound to the dispatcher provided to it.
#[derive(Clone)]
pub struct CommandContextBuilder<'a> {
    /// The dispatcher this builder is related to.
    pub dispatcher: &'a CommandDispatcher,

    /// The source running the commands.
    pub source: Arc<CommandSource>,

    /// Arguments which have been parsed and
    /// can be fetched for command execution.
    pub arguments: FxHashMap<String, Arc<ParsedArgument>>,

    /// The tree this context is related to.
    pub tree: Arc<Tree>,

    /// The root that this context will use, bound to the tree
    /// Not necessarily the root node of the tree, however.
    pub root: NodeId,

    /// All the parsed nodes of the command.
    pub nodes: Vec<ParsedNode>,

    /// The string range of input.
    pub range: StringRange,

    /// The child context of this context.
    pub child: Option<Box<Self>>,

    /// The redirect modifier of this context.
    pub modifier: RedirectModifier,

    /// Whether this context forks or not.
    pub forks: bool,

    /// The command stored in this context which
    /// is run to get a command result.
    pub command: Option<Command>,
}

impl<'a> CommandContextBuilder<'a> {
    /// Creates a new [`CommandContextBuilder`] from the properties required to initialize one.
    ///
    /// Note that builder's lifetime is bound to the dispatcher provided to it.
    #[must_use]
    pub fn new(
        dispatcher: &'a CommandDispatcher,
        source: Arc<CommandSource>,
        tree: Arc<Tree>,
        root: NodeId,
        start: usize,
    ) -> Self {
        CommandContextBuilder {
            dispatcher,
            source,
            arguments: FxHashMap::default(),
            tree,
            root,
            nodes: Vec::new(),
            range: StringRange::at(start),
            child: None,
            modifier: RedirectModifier::OneSource,
            forks: false,
            command: None,
        }
    }

    /// Builds the required [`CommandContext`], consuming itself in the process.
    #[must_use]
    pub fn build(self, input: &str) -> CommandContext {
        CommandContext {
            source: self.source,
            input: input.to_string(),
            arguments: self.arguments,
            tree: self.tree,
            root: self.root,
            nodes: self.nodes,
            range: self.range,
            child: self.child.map(|child| Arc::new(child.build(input))),
            modifier: self.modifier,
            forks: self.forks,
            command: self.command,
        }
    }

    /// Returns itself with the new source set.
    pub fn with_source(&mut self, source: Arc<CommandSource>) {
        self.source = source;
    }

    /// Returns itself with a new argument added.
    pub fn with_argument(&mut self, name: String, argument: Arc<ParsedArgument>) {
        self.arguments.insert(name, argument);
    }

    /// Returns itself with the new command set.
    pub fn with_command(&mut self, command: Option<Command>) {
        self.command = command;
    }

    /// Returns itself with a new node added to this builder.
    pub fn with_node(&mut self, node: NodeId, range: StringRange) {
        self.nodes.push(ParsedNode { node, range });
        self.range = StringRange::encompass(self.range, range);
        self.modifier = self.tree[node].modifier().clone();
        self.forks = self.tree[node].forks();
    }

    /// Returns itself with the new child set.
    pub fn with_child(&mut self, child: Self) {
        self.child = Some(Box::new(child));
    }

    /// Returns the last child of this builder.
    #[must_use]
    pub fn last_child(&self) -> &Self {
        let mut result = self;
        while let Some(child) = &result.child {
            result = child;
        }
        result
    }

    /// Creates a [`SuggestionContext`] from the provided cursor position.
    ///
    /// # Panics
    ///
    /// Panics if the node couldn't be found before the cursor.
    #[must_use]
    pub fn find_suggestion_context(&self, cursor: usize) -> SuggestionContext {
        assert!(
            self.range.start <= cursor,
            "Could not find node before cursor"
        );
        if self.range.end < cursor {
            self.child.as_ref().map_or_else(|| {
                if let Some(child) = &self.child {
                    child.find_suggestion_context(cursor)
                } else if let Some(last_node) = self.nodes.last() {
                    SuggestionContext {
                        parent: last_node.node,
                        starting_position: last_node.range.end + 1,
                    }
                } else {
                    SuggestionContext {
                        parent: self.root,
                        starting_position: self.range.start,
                    }
                }
            }, |child| child.find_suggestion_context(cursor))
        } else {
            let mut previous = self.root;
            for node in &self.nodes {
                if (self.range.start..=self.range.end).contains(&cursor) {
                    return SuggestionContext {
                        parent: previous,
                        starting_position: self.range.start,
                    };
                }
                previous = node.node;
            }
            SuggestionContext {
                parent: previous,
                starting_position: self.range.start,
            }
        }
    }
}
