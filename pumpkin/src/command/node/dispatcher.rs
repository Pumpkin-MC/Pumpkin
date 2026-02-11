use crate::command::context::command_context::{
    CommandContext, CommandContextBuilder, ContextChain,
};
use crate::command::context::command_source::{CommandSource, ReturnValue};
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::{
    DISPATCHER_EXPECTED_ARGUMENT_SEPARATOR, DISPATCHER_UNKNOWN_ARGUMENT,
    DISPATCHER_UNKNOWN_COMMAND, LiteralCommandErrorType,
};
use crate::command::node::attached::NodeId;
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
    tree: Arc<Tree>,
    consumer: Arc<dyn ResultConsumer>,
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
    pub fn register(&mut self, command_node: CommandDetachedNode) {
        // TODO: Determine if this is well optimized enough for a dispatcher.
        //
        // Mutations in dispatchers are extremely rare after server boot.
        // Thus, the following code will have a very low chance of actually
        // cloning, and even if it did, the `Arc` will get copied again later.
        Arc::make_mut(&mut self.tree).add_child_to_root(command_node);
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
        if parsed.reader.peek().is_none() {
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
        let mut context = CommandContextBuilder::new(
            self,
            Arc::new(source.clone()),
            self.tree.clone(),
            ROOT_NODE_ID,
            reader.cursor(),
        );
        self.parse_nodes(ROOT_NODE_ID, reader, &mut context)
    }

    fn parse_nodes<'a>(
        &'a self,
        node: NodeId,
        original_reader: &mut StringReader,
        context_so_far: &mut CommandContextBuilder<'a>,
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
            let mut reader = StringReader::new(original_reader.string());
            let parse_result = {
                if let Err(error) = self.tree.parse(child, &mut reader, context_so_far) {
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

            context.with_command(self.tree[child].command().clone());
            let redirect = self.tree[child].redirect();
            if reader.can_read_chars(if redirect.is_some() { 2 } else { 1 }) {
                reader.skip();
                if let Some(redirect) = redirect {
                    let Some(redirect) = self.tree.resolve(redirect) else {
                        errors.insert(child, UNRESOLVED_REDIRECT.create(&reader));
                        reader.set_cursor(cursor);
                        continue;
                    };
                    let mut child_context = CommandContextBuilder::new(
                        self,
                        source,
                        self.tree.clone(),
                        redirect,
                        reader.cursor(),
                    );
                    let parsed = self.parse_nodes(redirect, &mut reader, &mut child_context);
                    context.with_child(parsed.context);
                    return ParsingResult {
                        context,
                        errors: parsed.errors,
                        reader: parsed.reader,
                    };
                }
                let parsed = self.parse_nodes(child, &mut reader, &mut context);
                potentials.push(parsed);
            } else {
                potentials.push(ParsingResult {
                    context,
                    errors: FxHashMap::default(),
                    reader: reader.clone_into_owned(),
                });
            }
        }

        if !potentials.is_empty() {
            potentials.sort_by(|a, b| {
                let a_reader_remaining = a.reader.peek().is_some();
                let b_reader_remaining = b.reader.peek().is_some();

                let a_has_errors = !a.errors.is_empty();
                let b_has_errors = !b.errors.is_empty();

                (a_reader_remaining, a_has_errors).cmp(&(b_reader_remaining, b_has_errors))
            });
        }

        ParsingResult {
            context: context_so_far.clone(),
            errors,
            reader: original_reader.clone_into_owned(),
        }
    }
}
