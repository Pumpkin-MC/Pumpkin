use crate::command::context::command_context::{
    CommandContext, CommandContextBuilder, ContextChain,
};
use crate::command::context::command_source::{CommandSource, ReturnValue};
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::{
    DISPATCHER_EXPECTED_ARGUMENT_SEPARATOR, DISPATCHER_UNKNOWN_ARGUMENT,
    DISPATCHER_UNKNOWN_COMMAND, LiteralCommandErrorType,
};
use crate::command::node::attached::{CommandNodeId, NodeId};
use crate::command::node::detached::CommandDetachedNode;
use crate::command::node::tree::{ROOT_NODE_ID, Tree};
use crate::command::string_reader::StringReader;
use rustc_hash::FxHashMap;
use std::sync::{Arc, LazyLock};

pub const ARG_SEPARATOR: &str = " ";
pub const ARG_SEPARATOR_CHAR: char = ' ';

pub const USAGE_OPTIONAL_OPEN: &str = "[";
pub const USAGE_OPTIONAL_CLOSE: &str = "]";
pub const USAGE_REQUIRED_OPEN: &str = "(";
pub const USAGE_REQUIRED_CLOSE: &str = ")";
pub const USAGE_OR: &str = "|";

/// Thrown when redirection could not be resolved.
/// This shouldn't happen, and only happens when the command is incorrectly configured.
pub const UNRESOLVED_REDIRECT: LiteralCommandErrorType =
    LiteralCommandErrorType::new("Could not resolve redirect to node");

/// Represents the result of parsing.
pub struct ParsingResult<'a> {
    pub context: CommandContextBuilder<'a>,
    pub errors: FxHashMap<NodeId, CommandSyntaxError>,
    pub reader: StringReader<'static>,
}

/// Structs implementing this trait are able to execute upon command completion.
pub trait ResultConsumer {
    fn on_command_completion(&self, context: &CommandContext, result: ReturnValue);
}

/// A [`ResultConsumer`] which does nothing.
pub struct EmptyResultConsumer;

impl ResultConsumer for EmptyResultConsumer {
    fn on_command_completion(&self, _context: &CommandContext, _result: ReturnValue) {}
}

pub static EMPTY_CONSUMER: LazyLock<Arc<EmptyResultConsumer>> =
    LazyLock::new(|| Arc::new(EmptyResultConsumer));

/// The core command dispatcher, used to register, parse and execute commands.
pub struct CommandDispatcher {
    pub tree: Arc<Tree>,
    pub consumer: Arc<dyn ResultConsumer>,
}

impl Default for CommandDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandDispatcher {
    /// Creates a new [`CommandDispatcher`] with a new [`Tree`].
    #[must_use]
    pub fn new() -> Self {
        Self::from_existing_tree(Tree::new())
    }

    /// Creates this [`CommandDispatcher`] from a pre-existing tree.
    pub fn from_existing_tree(tree: Tree) -> Self {
        Self {
            tree: Arc::new(tree),
            consumer: EMPTY_CONSUMER.clone(),
        }
    }

    /// Registers a command which can then be dispatched.
    /// Returns the local ID of the node attached to the tree.
    pub fn register(&mut self, command_node: impl Into<CommandDetachedNode>) -> CommandNodeId {
        // TODO: Determine if this is well optimized enough for a dispatcher.
        //
        // Mutations in dispatchers are extremely rare after server boot.
        // Thus, the following code will have a very low chance of actually
        // cloning, and even if it did, the `Arc` will get copied again later
        // by the sources.
        let arc_mut = Arc::make_mut(&mut self.tree);
        arc_mut.add_child_to_root(command_node.into())
    }

    /// Executes the given command with the provided source, returning a result of execution.
    ///
    /// # Note
    /// This does not cache parsed input.
    pub async fn execute_input(
        &self,
        input: &str,
        source: &CommandSource,
    ) -> Result<i32, CommandSyntaxError> {
        let mut reader = StringReader::new(input);
        self.execute_reader(&mut reader, source).await
    }

    /// Executes the given command in a [`StringReader`] with the provided source, returning a result of execution.
    ///
    /// # Note
    /// This does not cache parsed input.
    pub async fn execute_reader(
        &self,
        reader: &mut StringReader<'_>,
        source: &CommandSource,
    ) -> Result<i32, CommandSyntaxError> {
        let parsed = self.parse(reader, source);
        self.execute(parsed).await
    }

    /// Executes a given result that has already been parsed from an input.
    pub async fn execute(&self, parsed: ParsingResult<'_>) -> Result<i32, CommandSyntaxError> {
        if parsed.reader.peek().is_some() {
            return if parsed.errors.len() == 1 {
                Err(parsed.errors.values().next().unwrap().clone())
            } else if parsed.context.range.is_empty() {
                Err(DISPATCHER_UNKNOWN_COMMAND.create(&parsed.reader))
            } else {
                Err(DISPATCHER_UNKNOWN_ARGUMENT.create(&parsed.reader))
            };
        }

        let command = parsed.reader.string();
        let original_context = parsed.context.build(command);

        match ContextChain::try_flatten(&original_context) {
            None => {
                self.consumer
                    .on_command_completion(&original_context, ReturnValue::Failure);
                Err(DISPATCHER_UNKNOWN_COMMAND.create(&parsed.reader))
            }
            Some(flat_context) => {
                flat_context
                    .execute_all(&original_context.source, self.consumer.as_ref())
                    .await
            }
        }
    }

    /// Only parses a given source with the specified source.
    #[must_use]
    pub fn parse_input(&self, command: &str, source: &CommandSource) -> ParsingResult<'_> {
        let mut reader = StringReader::new(command);
        self.parse(&mut reader, source)
    }

    /// Parses a command owned by a [`StringReader`] with the provided source.
    pub fn parse(&self, reader: &mut StringReader, source: &CommandSource) -> ParsingResult<'_> {
        let context = CommandContextBuilder::new(
            self,
            Arc::new(source.clone()),
            self.tree.clone(),
            ROOT_NODE_ID,
            reader.cursor(),
        );
        self.parse_nodes(ROOT_NODE_ID, reader, &context)
    }

    fn parse_nodes<'a>(
        &'a self,
        node: NodeId,
        original_reader: &mut StringReader,
        context_so_far: &CommandContextBuilder<'a>,
    ) -> ParsingResult<'a> {
        let source = context_so_far.source.clone();
        let mut errors: FxHashMap<NodeId, CommandSyntaxError> = FxHashMap::default();
        let mut potentials: Vec<ParsingResult> = Vec::new();
        let cursor = original_reader.cursor();

        for child in self.tree.get_relevant_nodes(original_reader, node) {
            if !self.tree.can_use(child, &source) {
                continue;
            }
            let mut context = context_so_far.clone();
            let mut reader = original_reader.clone();
            let parse_result = {
                if let Err(error) = self.tree.parse(child, &mut reader, &mut context) {
                    Err(error)
                } else {
                    let peek = reader.peek();
                    if peek.is_some() && peek != Some(ARG_SEPARATOR_CHAR) {
                        Err(DISPATCHER_EXPECTED_ARGUMENT_SEPARATOR.create(&reader))
                    } else {
                        Ok(())
                    }
                }
            };
            if let Err(parse_error) = parse_result {
                errors.insert(child, parse_error);
                reader.set_cursor(cursor);
                continue;
            }

            let child_node = &self.tree[child];
            context.with_command(child_node.command().clone());
            let redirect = self.tree[child].redirect();
            if reader.can_read_chars(if redirect.is_some() { 2 } else { 1 }) {
                reader.skip();
                if let Some(redirect) = redirect {
                    let Some(redirect) = self.tree.resolve(redirect) else {
                        errors.insert(child, UNRESOLVED_REDIRECT.create(&reader));
                        reader.set_cursor(cursor);
                        continue;
                    };
                    let child_context = CommandContextBuilder::new(
                        self,
                        source,
                        self.tree.clone(),
                        redirect,
                        reader.cursor(),
                    );
                    let parsed = self.parse_nodes(redirect, &mut reader, &child_context);
                    context.with_child(parsed.context);
                    return ParsingResult {
                        context,
                        errors: parsed.errors,
                        reader: parsed.reader,
                    };
                }
                let parsed = self.parse_nodes(child, &mut reader, &context);
                potentials.push(parsed);
            } else {
                potentials.push(ParsingResult {
                    context,
                    errors: FxHashMap::default(),
                    reader: reader.clone_into_owned(),
                });
            }
        }

        if potentials.is_empty() {
            ParsingResult {
                context: context_so_far.clone(),
                errors,
                reader: original_reader.clone_into_owned(),
            }
        } else {
            potentials
                .into_iter()
                .min_by(|a, b| {
                    let a_reader_remaining = a.reader.peek().is_some();
                    let b_reader_remaining = b.reader.peek().is_some();

                    let a_has_errors = !a.errors.is_empty();
                    let b_has_errors = !b.errors.is_empty();

                    (a_reader_remaining, a_has_errors).cmp(&(b_reader_remaining, b_has_errors))
                })
                .unwrap()
        }
    }
}

#[cfg(test)]
mod test {
    use crate::command::argument_builder::{
        ArgumentBuilder, CommandArgumentBuilder, LiteralArgumentBuilder, RequiredArgumentBuilder,
    };
    use crate::command::argument_types::core::integer::IntegerArgumentType;
    use crate::command::context::command_context::CommandContext;
    use crate::command::context::command_source::CommandSource;
    use crate::command::errors::error_types::DISPATCHER_UNKNOWN_COMMAND;
    use crate::command::node::dispatcher::CommandDispatcher;
    use crate::command::node::{CommandExecutor, CommandExecutorResult};

    #[tokio::test]
    async fn unknown_command() {
        let mut dispatcher = CommandDispatcher::new();
        dispatcher.register(
            CommandArgumentBuilder::new("unknown", "A command without an executor").build(),
        );
        let source = CommandSource::dummy();
        let result = dispatcher.execute_input("unknown", &source).await;
        assert!(result.is_err_and(|error| error.error_type == &DISPATCHER_UNKNOWN_COMMAND));
    }

    #[tokio::test]
    async fn simple_command() {
        let mut dispatcher = CommandDispatcher::new();
        let executor: for<'c> fn(&'c CommandContext) -> CommandExecutorResult<'c> =
            |_| Box::pin(async move { Ok(1) });
        dispatcher
            .register(CommandArgumentBuilder::new("simple", "A simple command").executes(executor));
        let source = CommandSource::dummy();
        let result = dispatcher.execute_input("simple", &source).await;
        assert_eq!(result, Ok(1));
    }

    #[tokio::test]
    async fn arithmetic_command() {
        let mut dispatcher = CommandDispatcher::new();

        enum Operation {
            ADD,
            SUBTRACT,
            MULTIPLY,
            DIVIDE,
        }
        struct Executor(Operation);

        impl CommandExecutor for Executor {
            fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
                Box::pin(async move {
                    let operand1: i32 = context.get_argument("operand1")?;
                    let operand2: i32 = context.get_argument("operand2")?;
                    Ok(match self.0 {
                        Operation::ADD => operand1 + operand2,
                        Operation::SUBTRACT => operand1 - operand2,
                        Operation::MULTIPLY => operand1 * operand2,
                        Operation::DIVIDE => operand1 / operand2,
                    })
                })
            }
        }

        dispatcher.register(
            CommandArgumentBuilder::new(
                "arithmetic",
                "A command which adds two integers, returning the result",
            )
            .then(
                RequiredArgumentBuilder::new("operand1", IntegerArgumentType::any())
                    .then(
                        LiteralArgumentBuilder::new("+").then(
                            RequiredArgumentBuilder::new("operand2", IntegerArgumentType::any())
                                .executes(Executor(Operation::ADD)),
                        ),
                    )
                    .then(
                        LiteralArgumentBuilder::new("-").then(
                            RequiredArgumentBuilder::new("operand2", IntegerArgumentType::any())
                                .executes(Executor(Operation::SUBTRACT)),
                        ),
                    )
                    .then(
                        LiteralArgumentBuilder::new("*").then(
                            RequiredArgumentBuilder::new("operand2", IntegerArgumentType::any())
                                .executes(Executor(Operation::MULTIPLY)),
                        ),
                    )
                    .then(
                        LiteralArgumentBuilder::new("/").then(
                            RequiredArgumentBuilder::new("operand2", IntegerArgumentType::any())
                                .executes(Executor(Operation::DIVIDE)),
                        ),
                    ),
            ),
        );
        let source = CommandSource::dummy();
        assert_eq!(
            dispatcher.execute_input("arithmetic 3 + -7", &source).await,
            Ok(-4)
        );
        assert_eq!(
            dispatcher.execute_input("arithmetic 4 - -8", &source).await,
            Ok(12)
        );
        assert_eq!(
            dispatcher.execute_input("arithmetic 2 * 9", &source).await,
            Ok(18)
        );
        assert_eq!(
            dispatcher.execute_input("arithmetic 9 / 2", &source).await,
            Ok(4)
        );
    }

    #[tokio::test]
    async fn alias_simple() {
        let mut dispatcher = CommandDispatcher::new();
        let executor: for<'c> fn(&'c CommandContext) -> CommandExecutorResult<'c> =
            |_| Box::pin(async move { Ok(1) });
        dispatcher.register(CommandArgumentBuilder::new("a", "A command").executes(executor));
        // Note that we CANNOT use redirect here as node itself needs to execute the command,
        // not its 'children'.
        dispatcher.register(CommandArgumentBuilder::new("b", "An alias for /a").executes(executor));
        let source = CommandSource::dummy();
        assert_eq!(dispatcher.execute_input("a", &source).await, Ok(1));
        assert_eq!(dispatcher.execute_input("b", &source).await, Ok(1));
    }

    #[tokio::test]
    async fn alias_complex() {
        let mut dispatcher = CommandDispatcher::new();

        struct Executor;

        impl CommandExecutor for Executor {
            fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
                Box::pin(async move { context.get_argument("result") })
            }
        }

        let a = dispatcher.register(CommandArgumentBuilder::new("a", "A command").then(
            RequiredArgumentBuilder::new("result", IntegerArgumentType::any()).executes(Executor),
        ));
        // Note that this time, we SHOULD use redirect - it is leading to another node having command.
        dispatcher.register(CommandArgumentBuilder::new("b", "An alias for /a").redirect(a));
        let source = CommandSource::dummy();
        assert_eq!(dispatcher.execute_input("a 5", &source).await, Ok(5));
        assert_eq!(dispatcher.execute_input("b 7", &source).await, Ok(7));
    }

    #[tokio::test]
    async fn recurse() {
        let mut dispatcher = CommandDispatcher::new();

        struct Executor;

        impl CommandExecutor for Executor {
            fn execute<'a>(&'a self, _context: &'a CommandContext) -> CommandExecutorResult<'a> {
                Box::pin(async move { Ok(1) })
            }
        }

        let mut builder = CommandArgumentBuilder::new(
            "recurse",
            "Recurses itself, doing nothing with the numbers provided",
        )
        .executes(Executor);

        let id = builder.id();
        builder = builder.then(
            RequiredArgumentBuilder::new("value", IntegerArgumentType::any())
                .executes(Executor)
                .redirect(id),
        );

        dispatcher.register(builder);

        let source = CommandSource::dummy();
        assert_eq!(dispatcher.execute_input("recurse", &source).await, Ok(1));
        assert_eq!(dispatcher.execute_input("recurse 4", &source).await, Ok(1));
        assert_eq!(
            dispatcher.execute_input("recurse 9 -1", &source).await,
            Ok(1)
        );
        assert_eq!(
            dispatcher
                .execute_input("recurse 9 7 -6 5 -4", &source)
                .await,
            Ok(1)
        );
        assert_eq!(
            dispatcher
                .execute_input("recurse 1 2 4 8 16 32 64 128 256 512", &source)
                .await,
            Ok(1)
        );
    }
}
