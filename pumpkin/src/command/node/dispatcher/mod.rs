use crate::command::argument_builder::{ArgumentBuilder, CommandArgumentBuilder};
use crate::command::context::command_context::{
    CommandContext, CommandContextBuilder, ContextChain,
};
use crate::command::context::command_source::{CommandSource, ReturnValue};
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::{
    DISPATCHER_EXPECTED_ARGUMENT_SEPARATOR, DISPATCHER_UNKNOWN_ARGUMENT,
    DISPATCHER_UNKNOWN_COMMAND, LiteralCommandErrorType,
};
use crate::command::node::Redirection;
use crate::command::node::attached::{CommandNodeId, NodeId};
use crate::command::node::detached::CommandDetachedNode;
use crate::command::node::tree::{ROOT_NODE_ID, Tree};
use crate::command::string_reader::StringReader;
use pumpkin_data::translation::java::COMMAND_CONTEXT_HERE;
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::click::ClickEvent;
use pumpkin_util::text::color::{Color, NamedColor};
use rustc_hash::FxHashMap;
use std::pin::Pin;
use std::sync::{Arc, LazyLock};

mod suggestions;
#[cfg(test)]
mod test;
mod usage;

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
pub trait ResultConsumer: Sync + Send {
    fn on_command_completion<'a>(
        &'a self,
        context: &'a CommandContext,
        result: ReturnValue,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>>;
}

/// A [`ResultConsumer`] which does nothing.
pub struct EmptyResultConsumer;

impl ResultConsumer for EmptyResultConsumer {
    fn on_command_completion<'a>(
        &self,
        _context: &'a CommandContext,
        _result: ReturnValue,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async {})
    }
}

pub static EMPTY_CONSUMER: LazyLock<Arc<EmptyResultConsumer>> =
    LazyLock::new(|| Arc::new(EmptyResultConsumer));

/// A [`ResultConsumer`] which defers the given result to the source provided.
pub struct ResultDeferrer;

impl ResultConsumer for ResultDeferrer {
    fn on_command_completion<'a>(
        &self,
        context: &'a CommandContext,
        result: ReturnValue,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            context.source.command_result_taker.call(result).await;
        })
    }
}

pub static RESULT_DEFERRER: LazyLock<Arc<ResultDeferrer>> =
    LazyLock::new(|| Arc::new(ResultDeferrer));

/// The core command dispatcher, used to register, parse and execute commands.
///
/// Internally, this dispatcher stores a [`Tree`]. Refer to its documentation
/// for more information about nodes.
pub struct CommandDispatcher {
    pub tree: Tree,
    pub consumer: Arc<dyn ResultConsumer>,

    // Temporary setup:
    // We add this because we have a lot of commands
    // still dependent on this dispatcher.
    pub fallback_dispatcher: crate::command::dispatcher::CommandDispatcher,
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
            tree,
            consumer: RESULT_DEFERRER.clone(),
            fallback_dispatcher: crate::command::dispatcher::CommandDispatcher::default(),
        }
    }

    /// Registers a command which can then be dispatched.
    /// Returns the local ID of the node attached to the tree.
    ///
    /// Note that, at least for now with this system, there is no way to
    /// unregister a command. This is due to redirection to
    /// potentially unregistered (freed) nodes.
    pub fn register(&mut self, command_node: impl Into<CommandDetachedNode>) -> CommandNodeId {
        self.tree.add_child_to_root(command_node)
    }

    /// Registers a command which can then be dispatched, along with its
    /// aliases as the second argument. Returns the local ID of the node attached to the tree.
    ///
    /// Behind the scenes, `redirect` and `executes_arc` calls are made
    /// for each provided alias. This method is for convenience.
    ///
    /// Note that, at least for now with this system, there is no way to
    /// unregister a command. This is due to redirection to
    /// potentially unregistered (freed) nodes.
    pub fn register_with_aliases<S: AsRef<str>>(
        &mut self,
        command_node: impl Into<CommandDetachedNode>,
        aliases: &[S],
    ) -> CommandNodeId {
        let main_node_id = self.register(command_node);

        let main_node = &self.tree[main_node_id];
        let description = &main_node.meta.description;

        let mut built_nodes = Vec::with_capacity(aliases.len());

        for alias in aliases {
            let mut alias =
                CommandArgumentBuilder::new(alias.as_ref().to_string(), description.clone());

            // We take a look at the original node's owned data.
            let reference = &main_node.owned;

            // If the reference contains an executor, we clone that over.
            // If not, we need not check for the permission, as it
            // will be done by the target node.
            if let Some(executor) = &reference.command {
                alias = alias.executes_arc(executor.clone());

                // We must add the appropriate requirements as well.
                // This is because if we just simply set an executor, then
                // any player can execute it without any requirements (including permissions)!
                //
                // For example, if an alias `/s` was added for `/stop` (hypothetically),
                // any player can stop the server with `/s`!
                alias = alias.overwrite_requirements(reference.requirements.clone());
            }

            // And we redirect to the node.
            alias = alias.redirect(Redirection::Local(main_node_id.into()));

            // Build the nodes.
            built_nodes.push(alias.build());
        }

        for alias in built_nodes {
            self.register(alias);
        }

        main_node_id
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
        let parsed = self.parse(reader, source).await;
        self.execute(parsed).await
    }

    /// Executes a given result that has already been parsed from an input.
    pub async fn execute(&self, parsed: ParsingResult<'_>) -> Result<i32, CommandSyntaxError> {
        if parsed.reader.peek().is_some() {
            return if parsed.errors.len() == 1 {
                Err(parsed
                    .errors
                    .values()
                    .next()
                    .expect("Errors length is 1, so next should exist")
                    .clone())
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
                    .on_command_completion(&original_context, ReturnValue::Failure)
                    .await;
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
    pub async fn parse_input(&self, command: &str, source: &CommandSource) -> ParsingResult<'_> {
        let mut reader = StringReader::new(command);
        self.parse(&mut reader, source).await
    }

    /// Parses a command owned by a [`StringReader`] with the provided source.
    pub async fn parse(
        &self,
        reader: &mut StringReader<'_>,
        source: &CommandSource,
    ) -> ParsingResult<'_> {
        let context = CommandContextBuilder::new(
            self,
            Arc::new(source.clone()),
            ROOT_NODE_ID,
            reader.cursor(),
        );
        self.parse_nodes(ROOT_NODE_ID, reader, &context).await
    }

    async fn parse_nodes<'a>(
        &'a self,
        node: NodeId,
        original_reader: &mut StringReader<'_>,
        context_so_far: &CommandContextBuilder<'a>,
    ) -> ParsingResult<'a> {
        let source = context_so_far.source.clone();
        let mut errors: FxHashMap<NodeId, CommandSyntaxError> = FxHashMap::default();
        let mut potentials: Vec<ParsingResult> = Vec::new();
        let cursor = original_reader.cursor();

        for child in self.tree.get_relevant_nodes(original_reader, node) {
            if !self.tree.can_use(child, &source).await {
                continue;
            }
            let mut context = context_so_far.clone();
            let mut reader = original_reader.clone();
            let parse_result = {
                if let Err(error) = self.tree.parse(child, &mut reader, &mut context).await {
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
                    let child_context =
                        CommandContextBuilder::new(self, source, redirect, reader.cursor());
                    let parsed =
                        Box::pin(self.parse_nodes(redirect, &mut reader, &child_context)).await;
                    context.with_child(parsed.context);
                    return ParsingResult {
                        context,
                        errors: parsed.errors,
                        reader: parsed.reader,
                    };
                }
                let parsed = Box::pin(self.parse_nodes(child, &mut reader, &context)).await;
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
                .expect("Potentials list is not empty")
        }
    }

    /// Handle the execution of a command by a given source (sender),
    /// returning appropriate error messages to it if necessary.
    ///
    /// If the input starts with one slash (`/`), it is removed
    /// inside the call itself.
    ///
    /// # Panics
    ///
    /// Panics if the source given to it is a dummy one.
    pub async fn handle_command<'a>(&'a self, source: &CommandSource, mut input: &'a str) {
        assert!(
            source.server.is_some(),
            "Source provided to this command was a dummy source"
        );

        if let Some(sliced) = input.strip_prefix("/") {
            input = sliced;
        }

        let output = self.execute_input(input, source).await;

        if let Err(error) = output {
            // We check if the error came because a command could not be found.
            // Note: 'Permission denied' also falls under this error as
            //       no executable node could be found.
            if error.is(&DISPATCHER_UNKNOWN_COMMAND) {
                // Run the fallback dispatcher instead.
                // It might have the command we're looking for.
                self.fallback_dispatcher
                    .handle_command(&source.output, source.server().as_ref(), input)
                    .await;
            } else {
                // Print the error to the output.
                Self::send_error_to_source(source, error, input).await;
            }
        }
    }

    /// Sends a command error to the provided source.
    /// This also shows the contextual information
    /// leading up to the error if necessary.
    pub async fn send_error_to_source(
        source: &CommandSource,
        error: CommandSyntaxError,
        command: &str,
    ) {
        source
            .send_message(error.message.color(Color::Named(NamedColor::Red)))
            .await;

        if let Some(context) = error.context {
            let i = context.input.len().min(context.cursor);

            let mut error_text = TextComponent::empty()
                .color(Color::Named(NamedColor::Gray))
                .click_event(ClickEvent::SuggestCommand {
                    command: format!("/{command}").into(),
                });

            if i > 10 {
                error_text = error_text.add_text("...");
            }

            let start = i.saturating_sub(10);

            let command_snippet = &context.input[start..i];
            error_text = error_text.add_text(command_snippet.to_owned());

            if i < context.input.len() {
                let errored_part = &context.input[i..];
                error_text = error_text.add_child(
                    TextComponent::text(errored_part.to_owned())
                        .color(Color::Named(NamedColor::Red))
                        .underlined(),
                );
            }

            error_text = error_text.add_child(
                TextComponent::translate_cross(COMMAND_CONTEXT_HERE, COMMAND_CONTEXT_HERE, &[])
                    .color(Color::Named(NamedColor::Red))
                    .italic(),
            );

            source.send_error(error_text).await;
        }
    }
}
