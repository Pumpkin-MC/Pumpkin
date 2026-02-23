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
use crate::command::node::tree::{NodeIdClassification, ROOT_NODE_ID, Tree};
use crate::command::string_reader::StringReader;
use crate::command::suggestion::suggestions::{Suggestions, SuggestionsBuilder};
use crate::command::tree::Command;
use futures::future;
use pumpkin_protocol::java::client::play::CommandSuggestion;
use rustc_hash::FxHashMap;
use std::collections::BTreeMap;
use std::pin::Pin;
use std::sync::{Arc, LazyLock};
use pumpkin_data::translation::COMMAND_CONTEXT_HERE;
use pumpkin_util::text::click::ClickEvent;
use pumpkin_util::text::color::{Color, NamedColor};
use pumpkin_util::text::TextComponent;

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
    pub fallback_dispatcher: crate::command::dispatcher::CommandDispatcher
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
            fallback_dispatcher: crate::command::dispatcher::CommandDispatcher::default()
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
                .unwrap()
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
    pub async fn handle_command<'a>(
        &'a self,
        source: &CommandSource,
        mut input: &'a str
    ) {
        assert!(source.server.is_some(), "Source provided to this command was a dummy source");

        if let Some(sliced) = input.strip_prefix("/") {
            input = sliced;
        }

        let output = self.execute_input(input, source).await;

        if let Err(error) = output {
            // We check if the error came because a command could not be found.
            if error.is(&DISPATCHER_UNKNOWN_COMMAND) {
                // Run the fallback dispatcher instead.
                // It might have the command we're looking for.
                self.fallback_dispatcher.handle_command(
                    &source.output,
                    source.server().as_ref(),
                    input
                ).await;
            } else {
                // Print the error to the output.
                Self::send_error_to_source(
                    source,
                    error,
                    input
                ).await;
            }
        }
    }

    /// Sends a command error to the provided source.
    /// This also shows the contextual information
    /// leading up to the error if necessary.
    pub async fn send_error_to_source(
        source: &CommandSource,
        error: CommandSyntaxError,
        command: &str
    ) {
        source.send_message(
            error
                .message
                .color(
                    Color::Named(NamedColor::Red)
                )
        ).await;

        if let Some(context) = error.context {
            let i = context.input.len().min(context.cursor);

            let mut error_text =
                TextComponent::text("")
                    .color(Color::Named(NamedColor::Gray))
                    .click_event(ClickEvent::SuggestCommand {
                        command: format!("/{command}").into()
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
                        .underlined()
                );
            }

            error_text = error_text.add_child(
                TextComponent::translate(COMMAND_CONTEXT_HERE, &[])
                    .color(Color::Named(NamedColor::Red))
                    .italic()
            );

            source.send_error(error_text).await;
        }
    }

    /// Returns a new [`Suggestions`] structure in the future
    /// from the given parsing result, which was a command that was parsed,
    /// assuming the cursor is at the end.
    /// 
    /// This is useful to tell the client on what suggestions are there next.
    pub async fn get_completion_suggestions_at_end(&self, parsing_result: ParsingResult<'_>) -> Suggestions {
        let length = parsing_result.reader.total_length();
        self.get_completion_suggestions(parsing_result, length).await
    }

    /// Returns a new [`Suggestions`] structure in the future
    /// from the given parsing result, which was a command that was parsed.
    /// 
    /// This is useful to tell the client on what suggestions are there next.
    pub async fn get_completion_suggestions(&self, parsing_result: ParsingResult<'_>, cursor: usize) -> Suggestions {    
        let context = parsing_result.context;
        let (parent, start) = {
            let node_before_cursor = context.find_suggestion_context(cursor);
            (node_before_cursor.parent, node_before_cursor.starting_position.min(cursor))
        };

        let full_input = parsing_result.reader.string();

        let truncated_input = &full_input[0..cursor.min(full_input.len())];

        let children = self.tree.get_children(parent);
        let capacity = children.len();
        let mut futures = Vec::with_capacity(capacity);

        let context = context.build(truncated_input);

        for child in children {
            let mut builder = SuggestionsBuilder::new(truncated_input, start);

            let future: Pin<Box<dyn Future<Output = Suggestions> + Send>> = match self.tree.classify_id(child) {
                NodeIdClassification::Root => Box::pin(async { Suggestions::empty() }),
                NodeIdClassification::Literal(literal_node_id) => {
                    Box::pin(async move {
                        let node = &self.tree[literal_node_id];
                        if node.meta.literal_lowercase.starts_with(builder.remaining_lowercase()) {
                            builder.suggest(&*node.meta.literal).build()
                        } else {
                            Suggestions::empty()
                        }
                    })
                },
                NodeIdClassification::Command(command_node_id) => {
                    Box::pin(async move {
                        let node = &self.tree[command_node_id];
                        if node.meta.literal_lowercase.starts_with(builder.remaining_lowercase()) {
                            builder.suggest(&*node.meta.literal).build()
                        } else {
                            Suggestions::empty()
                        }
                    })
                },
                NodeIdClassification::Argument(argument_node_id) => {
                    let node = &self.tree[argument_node_id];
                    node.meta.argument_type.list_suggestions(&context, &mut builder)
                },
            };

            futures.push(future);
        }

        let suggestions = future::join_all(futures).await;
        Suggestions::merge(full_input, suggestions)
    }

    /// Gets all the suggestions in the future as a [`Vec`] of [`CommandSuggestion`].
    /// 
    /// # Panics
    /// 
    /// This function currently panics if the source provided was a dummy source.
    /// This is subject to change in the future.
    pub async fn suggest(
        &self,
        input: &str,
        source: &CommandSource
    ) -> Vec<CommandSuggestion> {
        let future1 = async move {
            let parsed = self.parse_input(input, source).await;
            let suggestions = self.get_completion_suggestions_at_end(parsed).await;

            suggestions
                .suggestions
                .into_iter()
                .map(|suggestion| CommandSuggestion {
                    suggestion: suggestion.text.cached_text().clone(),
                    tooltip: suggestion.tooltip,
                })
                .collect::<Vec<CommandSuggestion>>()
        };

        let future2 = async move {
            self.fallback_dispatcher.find_suggestions(&source.output, &source.server(), input).await
        };

        let (mut a, mut b) = future::join(future1, future2).await;
        a.append(&mut b);
        a
    }

    /// Gets all the commands usable in this dispatcher, sorted.
    /// The map returned has the key as the command name
    /// and the value as the command's description.
    pub fn get_all_commands(&self) -> BTreeMap<&str, &str> {
        let mut commands: BTreeMap<&str, &str> = BTreeMap::new();

        for command in self.tree.get_root_children() {
            let meta = &self.tree[command].meta;
            commands.insert(&meta.literal_lowercase, &meta.description);
        }

        for fallback_command in self.fallback_dispatcher.commands.values() {
            if let Command::Tree(command_tree) = fallback_command {
                for name in &command_tree.names {
                    commands.insert(name, &command_tree.description);
                }
            }
        }

        commands
    }

    /// Gets all the commands usable in this dispatcher, which
    /// the given source is able to use.
    /// The map returned has the key as the command name
    /// and the value as the command's description.
    pub async fn get_all_permitted_commands(&self, source: &CommandSource) -> BTreeMap<&str, &str> {
        let mut commands: BTreeMap<&str, &str> = BTreeMap::new();

        for command in self.tree.get_root_children() {
            if self.tree.can_use(command.into(), source).await {
                let meta = &self.tree[command].meta;
                commands.insert(&meta.literal_lowercase, &meta.description);
            }
        }

        for fallback_command in self.fallback_dispatcher.commands.values() {
            if let Command::Tree(command_tree) = fallback_command {
                if let Some(permission) = self.fallback_dispatcher.permissions.get(&command_tree.names[0]) && source.has_permission(permission).await {
                    for name in &command_tree.names {
                        commands.insert(name, &command_tree.description);
                    }
                }
            }
        }

        commands
    }

    /// Gets the description and usage of each permitted command of the given source.
    ///
    /// The key is the command identifier,
    /// and the value is a tuple of `(description, usage)`.
    pub async fn get_all_permitted_commands_usage(&self, source: &CommandSource) -> BTreeMap<&str, (&str, Box<str>)> {
        let mut commands: BTreeMap<&str, (&str, Box<str>)> = BTreeMap::new();

        for (command_node_id, mut usage) in self.get_usage_of_commands(source).await {
            let meta = &self.tree[command_node_id].meta;
            let command_name = meta.literal.as_ref();
            let command_description = meta.description.as_ref();
            usage = format!("/{usage}");
            commands.insert(command_name, (command_description, usage.into_boxed_str()));
        }

        for fallback_command in self.fallback_dispatcher.commands.values() {
            if let Command::Tree(command_tree) = fallback_command {
                if let Some(permission) = self.fallback_dispatcher.permissions.get(&command_tree.names[0]) && source.has_permission(permission).await {
                    let usage = command_tree.to_string();
                    for name in &command_tree.names {
                        commands.insert(name, (command_tree.description.as_ref(), usage.clone().into_boxed_str()));
                    }
                }
            }
        }

        commands
    }

    /// Gets the description and usage of a given command of the given source.
    /// Returns `None` if not found or the source has insufficient permissions.
    ///
    /// The key is the command identifier,
    /// and the value is a tuple of `(description, usage)`.
    pub async fn get_permitted_command_usage(&self, source: &CommandSource, command: &str) -> Option<(&str, Box<str>)> {
        if let Some(output) = self.get_permitted_command_usage_non_fallback(source, command).await {
            Some(output)
        } else {
            let tree = self.fallback_dispatcher.get_tree(command).ok()?;
            if let Some(permission) = self.fallback_dispatcher.permissions.get(&tree.names[0]) && source.has_permission(permission).await {
                Some((tree.description.as_ref(), tree.to_string().into_boxed_str()))
            } else {
                None
            }
        }
    }

    async fn get_permitted_command_usage_non_fallback(&self, source: &CommandSource, command: &str) -> Option<(&str, Box<str>)> {
        let command_node_id = self.tree.get(command)?;

        // This propagates `None` to the function result if permissions are insufficient.
        let usage = self.get_usage_of_command(command_node_id, source).await?;

        let description = self.tree[command_node_id].meta.description.as_ref();

        Some((description, usage.into_boxed_str()))
    }

    /// Returns the usage of the given command node.
    pub async fn get_usage_of_command(&self, command_node: CommandNodeId, source: &CommandSource) -> Option<String> {
        // We know the root DOES NOT have an executor, so we pass false to `is_optional`.
        self.get_usage_recursive(command_node.into(), source, false, false, None).await
    }

    /// Returns the usage of each child of the given node (permitted for the given source).
    pub async fn get_usage_of_children(&self, node: NodeId, source: &CommandSource) -> FxHashMap<NodeId, String> {
        let mut map = FxHashMap::default();

        let is_optional = self.tree[node].command().is_some();
        for child in self.tree.get_children(node) {
            if let Some(usage) = self.get_usage_recursive(child, source, is_optional, false, None).await {
                map.insert(child, usage);
            }
        }

        map
    }

    /// Returns the usage of each command (permitted for the given source).
    pub async fn get_usage_of_commands(&self, source: &CommandSource) -> FxHashMap<CommandNodeId, String> {
        self.get_usage_of_children(ROOT_NODE_ID, source)
            .await
            .into_iter()
            // This is safe because every child of the root child is a command node.
            .map(|(k, v)| (CommandNodeId(k.0), v))
            .collect()
    }

    /// Internal function to recurse usages.
    fn get_usage_recursive<'a>(&'a self, node: NodeId, source: &'a CommandSource, is_optional: bool, deep: bool, redirector_usage_text: Option<String>) -> Pin<Box<dyn Future<Output = Option<String>> + Send + 'a>> {
        Box::pin(
            async move {
                if !self.tree.can_use(node, source).await {
                    return None;
                }

                let usage_text = redirector_usage_text.unwrap_or_else(|| {
                    let mut text = self.tree[node].usage_text();
                    if is_optional {
                        text = format!("{USAGE_OPTIONAL_OPEN}{text}{USAGE_OPTIONAL_CLOSE}");
                    }
                    text
                });
                let child_optional = self.tree[node].command().is_some();

                if !deep {
                    if let Some(redirect) = self.tree[node].redirect() {
                        if let Some(target) = self.tree.resolve(redirect) {
                            let target_usage =
                                if target == node {
                                    "...".to_string()
                                } else if self.tree.is_command_node(node) && self.tree.is_command_node(target) {
                                    // We do this so for example it will show usage for /?:
                                    //
                                    // /? [<commandOrPage>]
                                    //
                                    // instead of
                                    //
                                    // /? -> help
                                    return self.get_usage_recursive(target, source, is_optional, deep, Some(usage_text)).await;
                                } else {
                                    format!("-> {}", self.tree[target].usage_text())
                                };
                            return Some(format!("{usage_text}{ARG_SEPARATOR}{target_usage}"));
                        }
                    } else {
                        let mut children = Vec::new();
                        for child in self.tree.get_children(node) {
                            if self.tree.can_use(child, source).await {
                                children.push(child);
                            }
                        }

                        if children.len() == 1 {
                            let child = children[0];
                            if let Some(child_usage_text) = self.get_usage_recursive(child, source, child_optional, true, None).await {
                                return Some(format!("{usage_text}{ARG_SEPARATOR}{child_usage_text}"));
                            } else if !children.is_empty() {
                                let mut child_usages = Vec::new();
                                // TODO: Optimize this set algorithm while keeping insertion order.
                                for child in children {
                                    if let Some(child_usage_text) = self.get_usage_recursive(child, source, child_optional, true, None).await {
                                        if !child_usages.contains(&child_usage_text) {
                                            child_usages.push(child_usage_text);
                                        }
                                    }
                                }
                                if child_usages.len() == 1 {
                                    let mut child_usage =
                                        child_usages
                                            .into_iter()
                                            .next()
                                            .unwrap();
                                    if is_optional {
                                        child_usage = format!("{USAGE_OPTIONAL_OPEN}{child_usage}{USAGE_OPTIONAL_CLOSE}");
                                    }
                                    return Some(format!("{usage_text}{ARG_SEPARATOR}{child_usage}"))
                                } else if !child_usages.is_empty() {
                                    let (open, close) =
                                        if child_optional {
                                            (USAGE_OPTIONAL_OPEN, USAGE_OPTIONAL_CLOSE)
                                        } else {
                                            (USAGE_REQUIRED_OPEN, USAGE_REQUIRED_CLOSE)
                                        };

                                    let mut result_usage = usage_text;
                                    result_usage += ARG_SEPARATOR;
                                    result_usage += open;
                                    let mut first = true;
                                    for child_usage in child_usages {
                                        if !first {
                                            result_usage += USAGE_OR;
                                        }
                                        result_usage += &*child_usage;
                                        first = false;
                                    }
                                    result_usage += close;
                                    return Some(result_usage);
                                }
                            }
                        }
                    }
                }

                Some(usage_text)
            }
        )
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
        enum Operation {
            Add,
            Subtract,
            Multiply,
            Divide,
        }

        struct Executor(Operation);
        impl CommandExecutor for Executor {
            fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
                Box::pin(async move {
                    let operand1: i32 = *context.get_argument("operand1")?;
                    let operand2: i32 = *context.get_argument("operand2")?;
                    Ok(match self.0 {
                        Operation::Add => operand1 + operand2,
                        Operation::Subtract => operand1 - operand2,
                        Operation::Multiply => operand1 * operand2,
                        Operation::Divide => operand1 / operand2,
                    })
                })
            }
        }

        let mut dispatcher = CommandDispatcher::new();
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
                                .executes(Executor(Operation::Add)),
                        ),
                    )
                    .then(
                        LiteralArgumentBuilder::new("-").then(
                            RequiredArgumentBuilder::new("operand2", IntegerArgumentType::any())
                                .executes(Executor(Operation::Subtract)),
                        ),
                    )
                    .then(
                        LiteralArgumentBuilder::new("*").then(
                            RequiredArgumentBuilder::new("operand2", IntegerArgumentType::any())
                                .executes(Executor(Operation::Multiply)),
                        ),
                    )
                    .then(
                        LiteralArgumentBuilder::new("/").then(
                            RequiredArgumentBuilder::new("operand2", IntegerArgumentType::any())
                                .executes(Executor(Operation::Divide)),
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
        struct Executor;
        impl CommandExecutor for Executor {
            fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
                Box::pin(async move { Ok(*context.get_argument("result")?) })
            }
        }

        let mut dispatcher = CommandDispatcher::new();

        let a = dispatcher.register(CommandArgumentBuilder::new("a", "A command").then(
            RequiredArgumentBuilder::new("result", IntegerArgumentType::any()).executes(Executor),
        ));
        // Note that this time, we SHOULD use redirect - it is leading to another node having `command`.
        dispatcher.register(CommandArgumentBuilder::new("b", "An alias for /a").redirect(a));
        let source = CommandSource::dummy();
        assert_eq!(dispatcher.execute_input("a 5", &source).await, Ok(5));
        assert_eq!(dispatcher.execute_input("b 7", &source).await, Ok(7));
    }

    #[tokio::test]
    async fn recurse() {
        struct Executor;
        impl CommandExecutor for Executor {
            fn execute<'a>(&'a self, _context: &'a CommandContext) -> CommandExecutorResult<'a> {
                Box::pin(async move { Ok(1) })
            }
        }

        let mut dispatcher = CommandDispatcher::new();

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
