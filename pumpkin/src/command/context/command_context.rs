use crate::command::context::command_source::{CommandSource, ReturnValue};
use crate::command::context::string_range::StringRange;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::DISPATCHER_PARSE_EXCEPTION;
use crate::command::node::attached::NodeId;
use crate::command::node::dispatcher::{CommandDispatcher, ResultConsumer};
use crate::command::node::tree::Tree;
use crate::command::node::{Command, RedirectModifier};
use pumpkin_util::text::TextComponent;
use rustc_hash::FxHashMap;
use std::any::Any;
use std::sync::Arc;

/// Represents the current stage of the chain.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Stage {
    MODIFY,
    EXECUTE,
}

/// Represents a parsed node.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct ParsedNode {
    pub node: NodeId,
    pub range: StringRange,
}

/// Represents a suggestional context involving a node.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
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
pub struct CommandContext<'a> {
    /// The source running the commands.
    pub source: Arc<CommandSource>,

    /// The input string ran as the command.
    pub input: String,

    /// Arguments which have been parsed and
    /// can be fetched for command execution.
    pub arguments: FxHashMap<String, Arc<ParsedArgument>>,

    /// The tree this context is related to.
    pub tree: &'a Tree,

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

impl CommandContext<'_> {
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
            tree: self.tree,
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

    /// Returns a reference to a particular argument with type `T`.
    /// If it fails, an error with the appropriate message is returned.
    ///
    /// Ideally should be used with the `?` operator.
    ///
    /// # Example
    /// A simple example that takes two arguments specified from the node
    /// and returns their sum as the status output of the `Executor`:
    /// ```
    /// use pumpkin::command::context::command_context::CommandContext;
    /// use pumpkin::command::node::{CommandExecutor, CommandExecutorResult};
    ///
    /// struct Executor;
    /// impl CommandExecutor for Executor {
    ///     fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
    ///         Box::pin(async move {
    ///             // The `get_argument` method returns a `Result<&i32, CommandSyntaxError>`.
    ///             // We apply the `?` operator first, propagating the `CommandSyntaxError` if contained.
    ///             // Finally, we dereference the `&i32`, as `i32` implements Copy.
    ///             let operand1: i32 = *context.get_argument("operand1")?;
    ///             let operand2: i32 = *context.get_argument("operand2")?;
    ///             Ok(operand1 + operand2)
    ///         })
    ///     }
    /// }
    /// ```
    pub fn get_argument<T: 'static>(&self, name: &str) -> Result<&T, CommandSyntaxError> {
        // The errors below should never happen due to user input.
        // If an error below this comment is returned, that means the command was
        // improperly defined.
        //
        // Still, we provide helpful errors instead of panicking.

        let arg = self.arguments.get(name).ok_or_else(|| {
            DISPATCHER_PARSE_EXCEPTION.create_without_context(TextComponent::text(format!(
                "Could not find argument with name '{name}'"
            )))
        })?;
        let dyn_ref = &*arg.result;
        dyn_ref.downcast_ref::<T>().ok_or_else(|| {
            DISPATCHER_PARSE_EXCEPTION.create_without_context(TextComponent::text(format!(
                "Could not downcast argument '{name}'"
            )))
        })
    }
}

/// Represents a linked chain of [`CommandContext`]s, where the previous links to the next as a child.
#[derive(Clone)]
pub struct ContextChain<'a> {
    /// The modifiers of this context chain.
    modifiers: Vec<Arc<CommandContext<'a>>>,

    /// That specific [`CommandContext`] to execute.
    execute: Arc<CommandContext<'a>>,
}

impl<'a> ContextChain<'a> {
    /// Creates a new chain of contexts from a vector of them and one to execute.
    ///
    /// # Panics
    ///
    /// Panics if the `execute` given is non-executable.
    #[must_use]
    pub fn new(modifiers: Vec<Arc<CommandContext<'a>>>, execute: Arc<CommandContext<'a>>) -> Self {
        assert!(
            execute.command.is_some(),
            "Expected last command in chain to be executable"
        );
        Self { modifiers, execute }
    }

    /// Tries to flatten a [`CommandContext`]. If no command
    /// is available at the end of chain, [`None`] is returned.
    #[must_use]
    pub fn try_flatten(root: &CommandContext<'a>) -> Option<Self> {
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
        modifier: &CommandContext<'a>,
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
            result_consumer
                .on_command_completion(&context_to_use, ReturnValue::Failure)
                .await;
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
        executable: &CommandContext<'a>,
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
            result_consumer
                .on_command_completion(&context_to_use, ReturnValue::Success(result))
                .await;
            Ok(if forked_mode { 1 } else { result })
        } else {
            result_consumer
                .on_command_completion(&context_to_use, ReturnValue::Failure)
                .await;
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
    pub fn get_top_context(&'_ self) -> &'_ Arc<CommandContext<'_>> {
        if self.modifiers.is_empty() {
            &self.execute
        } else {
            &self.modifiers[0]
        }
    }

    /// Gets a mutable reference to the top context of this chain.
    pub fn get_top_context_mut(&'_ mut self) -> &mut Arc<CommandContext<'a>> {
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
        root: NodeId,
        start: usize,
    ) -> Self {
        CommandContextBuilder {
            dispatcher,
            source,
            arguments: FxHashMap::default(),
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
    pub fn build(self, input: &str) -> CommandContext<'a> {
        CommandContext {
            source: self.source,
            input: input.to_string(),
            arguments: self.arguments,
            tree: &self.dispatcher.tree,
            root: self.root,
            nodes: self.nodes,
            range: self.range,
            child: self.child.map(|child| Arc::new(child.build(input))),
            modifier: self.modifier,
            forks: self.forks,
            command: self.command,
        }
    }

    /// Mutates itself with the new source set.
    pub fn with_source(&mut self, source: Arc<CommandSource>) {
        self.source = source;
    }

    /// Mutates itself with a new argument added.
    pub fn with_argument(&mut self, name: String, argument: Arc<ParsedArgument>) {
        self.arguments.insert(name, argument);
    }

    /// Mutates itself with the new command set.
    pub fn with_command(&mut self, command: Option<Command>) {
        self.command = command;
    }

    /// Mutates itself with a new node added to this builder.
    pub fn with_node(&mut self, node: NodeId, range: StringRange) {
        self.nodes.push(ParsedNode { node, range });
        self.range = StringRange::encompass(self.range, range);
        self.modifier = self.dispatcher.tree[node].modifier().clone();
        self.forks = self.dispatcher.tree[node].forks();
    }

    /// Mutates itself with the new child set.
    pub fn with_child(&mut self, child: Self) {
        self.child = Some(Box::new(child));
    }

    /// Mutates the last child of this builder.
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
            self.child.as_ref().map_or_else(
                || {
                    self.nodes.last().as_ref().map_or_else(
                        || SuggestionContext {
                            parent: self.root,
                            starting_position: self.range.start,
                        },
                        |last_node| SuggestionContext {
                            parent: last_node.node,
                            starting_position: last_node.range.end + 1,
                        },
                    )
                },
                |child| child.find_suggestion_context(cursor),
            )
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

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use crate::command::argument_builder::{ArgumentBuilder, CommandArgumentBuilder};
    use crate::command::context::command_context::{
        CommandContext, CommandContextBuilder, ContextChain, ParsedArgument, Stage,
    };
    use crate::command::context::command_source::CommandSource;
    use crate::command::context::string_range::StringRange;
    use crate::command::errors::command_syntax_error::CommandSyntaxError;
    use crate::command::node::dispatcher::{CommandDispatcher, EmptyResultConsumer};
    use crate::command::node::tree::ROOT_NODE_ID;
    use crate::command::node::{CommandExecutor, CommandExecutorResult, Redirection};

    struct TenExecutor;
    impl CommandExecutor for TenExecutor {
        fn execute<'a>(&'a self, _context: &'a CommandContext) -> CommandExecutorResult<'a> {
            Box::pin(async move { Ok(10) })
        }
    }

    // For testing purposes
    fn builder(dispatcher: &'_ CommandDispatcher) -> CommandContextBuilder<'_> {
        let mut builder = CommandContextBuilder::new(
            dispatcher,
            Arc::new(CommandSource::dummy()),
            ROOT_NODE_ID,
            0,
        );

        let parsed_argument = ParsedArgument::new(StringRange::between(0, 1), Box::new(6789i32));

        builder.with_argument("foo".to_string(), Arc::new(parsed_argument));

        builder
    }

    #[test]
    fn get_argument() -> Result<(), CommandSyntaxError> {
        let dispatcher = CommandDispatcher::new();
        let builder = builder(&dispatcher);

        let context = builder.build("6789");
        assert_eq!(context.get_argument::<i32>("foo")?, &6789);

        Ok(())
    }

    #[test]
    fn get_nonexistent_argument() {
        let dispatcher = CommandDispatcher::new();
        let builder = builder(&dispatcher);

        let context = builder.build("6789");
        assert!(context.get_argument::<i32>("bar").is_err());
    }

    #[test]
    fn get_different_type_argument() {
        let dispatcher = CommandDispatcher::new();
        let builder = builder(&dispatcher);

        let context = builder.build("6789");
        assert!(context.get_argument::<f32>("foo").is_err());
    }

    #[tokio::test]
    async fn execute_single_command_chain() {
        let mut dispatcher = CommandDispatcher::new();
        dispatcher
            .register(CommandArgumentBuilder::new("foo", "A test command").executes(TenExecutor));

        let source = Arc::new(CommandSource::dummy());
        let result = dispatcher.parse_input("foo", &source).await;
        let top_context = result.context.build("foo");
        let chain = ContextChain::try_flatten(&top_context)
            .expect("The context should have properly flattened, as it has a command to execute");

        assert_eq!(
            chain.execute_all(&source, &EmptyResultConsumer).await,
            Ok(10)
        );
    }

    #[tokio::test]
    async fn execute_redirected_command_chain() {
        let mut dispatcher = CommandDispatcher::new();
        dispatcher
            .register(CommandArgumentBuilder::new("foo", "A test command").executes(TenExecutor));
        dispatcher.register(
            CommandArgumentBuilder::new("bar", "Another test command").redirect(Redirection::Root),
        );

        let source = Arc::new(CommandSource::dummy());
        let result = dispatcher.parse_input("bar foo", &source).await;
        let top_context = result.context.build("bar foo");
        let chain = ContextChain::try_flatten(&top_context)
            .expect("The context should have properly flattened, as it has a command to execute");

        assert_eq!(
            chain.execute_all(&source, &EmptyResultConsumer).await,
            Ok(10)
        );
    }

    #[tokio::test]
    async fn single_stage_execution() {
        let mut dispatcher = CommandDispatcher::new();
        dispatcher
            .register(CommandArgumentBuilder::new("foo", "A test command").executes(TenExecutor));

        let source = Arc::new(CommandSource::dummy());
        let result = dispatcher.parse_input("foo", &source).await;
        let top_context = result.context.build("foo");
        let chain = ContextChain::try_flatten(&top_context)
            .expect("The context should have properly flattened, as it has a command to execute");

        assert_eq!(chain.get_stage(), Stage::EXECUTE);
        assert!(chain.next_stage().is_none());
    }

    #[tokio::test]
    async fn multi_stage_execution() {
        let mut dispatcher = CommandDispatcher::new();
        dispatcher
            .register(CommandArgumentBuilder::new("foo", "A test command").executes(TenExecutor));
        dispatcher.register(
            CommandArgumentBuilder::new("bar", "Another test command").redirect(Redirection::Root),
        );
        dispatcher.register(
            CommandArgumentBuilder::new("qux", "Yet another test command")
                .redirect(Redirection::Root),
        );

        let source = Arc::new(CommandSource::dummy());
        let result = dispatcher.parse_input("bar qux foo", &source).await;
        let top_context = result.context.build("bar qux foo");
        let chain = ContextChain::try_flatten(&top_context)
            .expect("The context should have properly flattened, as it has a command to execute");
        assert_eq!(chain.get_stage(), Stage::MODIFY);

        let chain2 = chain
            .next_stage()
            .expect("There should have been the next stage");
        assert_eq!(chain2.get_stage(), Stage::MODIFY);

        let chain3 = chain2
            .next_stage()
            .expect("There should have been the next stage");
        assert_eq!(chain3.get_stage(), Stage::EXECUTE);
        assert!(chain3.next_stage().is_none());
    }

    #[tokio::test]
    async fn missing_command() {
        let mut dispatcher = CommandDispatcher::new();
        dispatcher.register(CommandArgumentBuilder::new("foo", "A test command"));

        let source = Arc::new(CommandSource::dummy());
        let result = dispatcher.parse_input("foo", &source).await;
        let top_context = result.context.build("foo");

        assert!(ContextChain::try_flatten(&top_context).is_none());
    }
}
