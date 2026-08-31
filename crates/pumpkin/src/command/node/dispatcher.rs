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
use crate::command::node::tree::{NodeIdClassification, ROOT_NODE_ID, Tree};
use crate::command::string_reader::StringReader;
use crate::command::suggestion::suggestions::{Suggestions, SuggestionsBuilder};
use pumpkin_data::translation::java::COMMAND_CONTEXT_HERE;
use pumpkin_protocol::java::client::play::CommandSuggestion;
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::click::ClickEvent;
use pumpkin_util::text::color::{Color, NamedColor};
use rustc_hash::{FxHashMap, FxHashSet};
use std::collections::BTreeMap;
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
pub trait ResultConsumer: Sync + Send {
    fn on_command_completion(&self, context: &CommandContext, result: ReturnValue);
}

/// A [`ResultConsumer`] which does nothing.
pub struct EmptyResultConsumer;

impl ResultConsumer for EmptyResultConsumer {
    fn on_command_completion(&self, _context: &CommandContext, _result: ReturnValue) {}
}

pub static EMPTY_CONSUMER: LazyLock<Arc<EmptyResultConsumer>> =
    LazyLock::new(|| Arc::new(EmptyResultConsumer));

/// A [`ResultConsumer`] which defers the given result to the source provided.
pub struct ResultDeferrer;

impl ResultConsumer for ResultDeferrer {
    fn on_command_completion(&self, context: &CommandContext, result: ReturnValue) {
        context.source.command_result_taker.call(result);
    }
}

pub static RESULT_DEFERRER: LazyLock<Arc<ResultDeferrer>> =
    LazyLock::new(|| Arc::new(ResultDeferrer));

/// The core command dispatcher, used to register, parse and execute commands.
///
/// Internally, this dispatcher stores a [`Tree`]. Refer to its documentation
/// for more information about nodes.
#[derive(Clone)]
pub struct CommandDispatcher {
    pub tree: Tree,
    pub consumer: Arc<dyn ResultConsumer>,

    /// Primary names of commands that have been turned off through the server
    /// configuration. A disabled command behaves as if it does not exist: it
    /// cannot be executed and is left out of listings and suggestions.
    disabled: FxHashSet<String>,
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
            disabled: FxHashSet::default(),
        }
    }

    /// Turns a command off. A disabled command's primary name is recorded here
    /// so that it can no longer be executed, listed, or suggested, regardless of
    /// which internal dispatcher it lives on.
    pub fn disable_command(&mut self, name: impl Into<String>) {
        self.disabled.insert(name.into());
    }

    /// Returns `true` if the command with the given primary name has been turned
    /// off through the server configuration.
    #[must_use]
    pub fn is_disabled(&self, name: &str) -> bool {
        if self.disabled.contains(name) {
            return true;
        }
        if name.starts_with('/') && self.disabled.contains(name.trim_start_matches('/')) {
            return true;
        }
        if !name.starts_with('/') {
            let single = format!("/{name}");
            let double = format!("//{name}");
            if self.disabled.contains(&single) || self.disabled.contains(&double) {
                return true;
            }
        }
        false
    }

    /// Returns `true` if a command (or alias) with the given name is registered
    /// on the node-based tree.
    #[must_use]
    pub fn has_command(&self, name: &str) -> bool {
        self.tree.get(name).is_some()
    }

    /// Collects the names of every root-level alias that redirects to the command
    /// with the given primary name.
    ///
    /// Node-based commands model their aliases as extra root literals that
    /// redirect to the primary node (see `register_with_aliases`). When a command
    /// is disabled we also need to turn those aliases off, otherwise a player
    /// could still reach the live executor through one of them.
    #[must_use]
    pub fn tree_alias_names(&self, primary: &str) -> Vec<String> {
        let Some(primary_id) = self.tree.get(primary) else {
            return Vec::new();
        };
        let primary_node: NodeId = primary_id.into();

        let mut names = Vec::new();
        for child in self.tree.get_root_children() {
            // Index by `NodeId` so we get the `AttachedNode` enum (which exposes
            // `redirect`/`name`) rather than the inner command node.
            let node_id: NodeId = child.into();
            let node = &self.tree[node_id];
            if let Some(redirect) = node.redirect()
                && self.tree.resolve(redirect) == Some(primary_node)
            {
                names.push(node.name().to_ascii_lowercase());
            }
        }
        names
    }

    /// Resolves the primary name of a command from either an alias or a primary name.
    #[must_use]
    pub fn primary_command_name(&self, name: &str) -> String {
        if let Some(node_id) = self.tree.get(name) {
            let node = &self.tree[NodeId::from(node_id)];
            if let Some(redirect) = node.redirect()
                && let Some(resolved) = self.tree.resolve(redirect)
            {
                return self.tree[resolved].name().to_ascii_lowercase();
            }
            return node.name().to_ascii_lowercase();
        }
        name.to_string()
    }

    /// Extracts the command name (the first whitespace-separated token) from a
    /// raw input string.
    fn command_name(input: &str) -> &str {
        input.split_whitespace().next().unwrap_or("")
    }

    /// Registers a command which can then be dispatched.
    /// Returns the local ID of the node attached to the tree.
    ///
    /// Note that, at least for now with this system, there is no way to
    /// unregister a command. This is due to redirection to
    /// potentially unregistered (freed) nodes.
    pub fn register(&mut self, command_node: impl Into<CommandDetachedNode>) -> CommandNodeId {
        let node = command_node.into();
        let name = node.meta.literal.to_string();
        let main_node_id = self.tree.add_child_to_root(node);

        // For double-slash or slash-prefixed commands (e.g. //set or /set),
        // automatically register the alternate slash variant as an alias.
        if let Some(stripped) = name.strip_prefix("//") {
            let single_slash = format!("/{stripped}");
            if self.tree.get(&single_slash).is_none() {
                let main_node = &self.tree[main_node_id];
                let description = main_node.meta.description.clone();
                let mut alias = crate::command::argument_builder::CommandArgumentBuilder::new(
                    single_slash,
                    description,
                );
                if let Some(executor) = &main_node.owned.command {
                    alias = alias.executes_arc(executor.clone());
                    alias = alias.overwrite_requirements(main_node.owned.requirements.clone());
                }
                alias = alias.redirect(crate::command::node::Redirection::Local(
                    main_node_id.into(),
                ));
                self.tree.add_child_to_root(alias.build());
            }
        } else if let Some(stripped) = name.strip_prefix('/') {
            let double_slash = format!("//{stripped}");
            if self.tree.get(&double_slash).is_none() {
                let main_node = &self.tree[main_node_id];
                let description = main_node.meta.description.clone();
                let mut alias = crate::command::argument_builder::CommandArgumentBuilder::new(
                    double_slash,
                    description,
                );
                if let Some(executor) = &main_node.owned.command {
                    alias = alias.executes_arc(executor.clone());
                    alias = alias.overwrite_requirements(main_node.owned.requirements.clone());
                }
                alias = alias.redirect(crate::command::node::Redirection::Local(
                    main_node_id.into(),
                ));
                self.tree.add_child_to_root(alias.build());
            }
        }

        main_node_id
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
    pub fn execute_input(
        &self,
        input: &str,
        source: &CommandSource,
    ) -> Result<i32, CommandSyntaxError> {
        let mut reader = StringReader::new(input);

        // A disabled command must behave as if it does not exist from every
        // execution path, not just `handle_command`. This backstop covers
        // programmatic callers such as `/execute run <command>`, which reach the
        // dispatcher here without going through `handle_command`.
        if self.is_disabled(Self::command_name(input)) {
            return Err(DISPATCHER_UNKNOWN_COMMAND.create(&reader));
        }

        self.execute_reader(&mut reader, source)
    }

    /// Executes the given command in a [`StringReader`] with the provided source, returning a result of execution.
    ///
    /// # Note
    /// This does not cache parsed input.
    pub fn execute_reader(
        &self,
        reader: &mut StringReader<'_>,
        source: &CommandSource,
    ) -> Result<i32, CommandSyntaxError> {
        let parsed = self.parse(reader, source);
        self.execute(parsed)
    }

    /// Executes a given result that has already been parsed from an input.
    pub fn execute(&self, parsed: ParsingResult<'_>) -> Result<i32, CommandSyntaxError> {
        if parsed.reader.peek().is_some() {
            return if let Some(err) = parsed.errors.values().next() {
                Err(err.clone())
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
                flat_context.execute_all(&original_context.source, self.consumer.as_ref())
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
    pub fn parse(
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
        self.parse_nodes(ROOT_NODE_ID, reader, &context)
    }

    fn parse_nodes<'a>(
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
                    let child_context =
                        CommandContextBuilder::new(self, source, redirect, reader.cursor());
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
    pub fn handle_command<'a>(&'a self, source: &CommandSource, mut input: &'a str) {
        assert!(
            source.server.is_some(),
            "Source provided to this command was a dummy source"
        );

        // If input starts with '/', but the command with that leading slash is NOT
        // registered, while the stripped command IS registered (or if it's an unknown command
        // starting with a single slash, e.g. from console input), strip one leading slash.
        // For double-slash commands like WorldEdit's `//set`, the client sends `/set`, which
        // matches a registered `/set` or `//set` command and preserves the slash.
        if let Some(sliced) = input.strip_prefix('/') {
            let first_token = input.split_whitespace().next().unwrap_or("");
            let sliced_token = sliced.split_whitespace().next().unwrap_or("");
            if !self.has_command(first_token)
                && (self.has_command(sliced_token) || !first_token.starts_with("//"))
            {
                input = sliced;
            }
        }

        // A command that has been turned off in the configuration must behave as
        // if it does not exist, so we report the usual "unknown command" error
        // before either dispatcher gets a chance to run it.
        if self.is_disabled(Self::command_name(input)) {
            let reader = StringReader::new(input);
            Self::send_error_to_source(source, DISPATCHER_UNKNOWN_COMMAND.create(&reader), input);
            return;
        }

        let output = self.execute_input(input, source);

        if let Err(error) = output {
            Self::send_error_to_source(source, error, input);
        }
    }

    /// Sends a command error to the provided source.
    /// This also shows the contextual information
    /// leading up to the error if necessary.
    pub fn send_error_to_source(source: &CommandSource, error: CommandSyntaxError, command: &str) {
        source.send_message(error.message.color(Color::Named(NamedColor::Red)));

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

            source.send_error(error_text);
        }
    }

    /// Returns a new [`Suggestions`] structure
    /// from the given parsing result, which was a command that was parsed,
    /// assuming the cursor is at the end.
    ///
    /// This is useful to tell the client on what suggestions are there next.
    #[must_use]
    pub fn get_completion_suggestions_at_end(
        &self,
        parsing_result: ParsingResult<'_>,
    ) -> Suggestions {
        let length = parsing_result.reader.total_length();
        self.get_completion_suggestions(parsing_result, length)
    }

    /// Returns a new [`Suggestions`] structure
    /// from the given parsing result, which was a command that was parsed.
    ///
    /// This is useful to tell the client on what suggestions are there next.
    #[must_use]
    pub fn get_completion_suggestions(
        &self,
        parsing_result: ParsingResult<'_>,
        cursor: usize,
    ) -> Suggestions {
        let context = parsing_result.context;
        let (parent, start) = {
            let node_before_cursor = context.find_suggestion_context(cursor);
            (
                node_before_cursor.parent,
                node_before_cursor.starting_position.min(cursor),
            )
        };

        let full_input = parsing_result.reader.string();

        let truncated_input = &full_input[0..cursor.min(full_input.len())];

        let children = self.tree.get_children(parent);
        let context = context.build(truncated_input);
        let mut suggestions = Vec::with_capacity(children.len());

        for child in children {
            let builder = SuggestionsBuilder::new(truncated_input, start);

            match self.tree.classify_id(child) {
                NodeIdClassification::Root => {}
                NodeIdClassification::Literal(literal_node_id) => {
                    let node = &self.tree[literal_node_id];
                    if node
                        .meta
                        .literal_lowercase
                        .starts_with(builder.remaining_lowercase())
                    {
                        suggestions.push(builder.suggest(&*node.meta.literal).build());
                    }
                }
                NodeIdClassification::Command(command_node_id) => {
                    let node = &self.tree[command_node_id];
                    if node
                        .meta
                        .literal_lowercase
                        .starts_with(builder.remaining_lowercase())
                    {
                        suggestions.push(builder.suggest(&*node.meta.literal).build());
                    }
                }
                NodeIdClassification::Argument(argument_node_id) => {
                    let node = &self.tree[argument_node_id];
                    if let Some(provider) = &node.meta.suggestion_provider {
                        suggestions.push(provider.suggest(&context, builder));
                    } else {
                        suggestions
                            .push(node.meta.argument_type.list_suggestions(&context, builder));
                    }
                }
            }
        }

        Suggestions::merge(full_input, suggestions)
    }

    /// Gets all the suggestions as a [`Vec`] of [`CommandSuggestion`].
    ///
    /// # Panics
    ///
    /// This function currently panics if the source provided was a dummy source.
    /// This is subject to change in the future.
    #[must_use]
    pub fn suggest(&self, input: &str, source: &CommandSource) -> Vec<CommandSuggestion> {
        self.suggest_with_range(input, source)
            .suggestions
            .into_iter()
            .map(|suggestion| CommandSuggestion {
                suggestion: suggestion.text.cached_text().clone(),
                tooltip: suggestion.tooltip,
            })
            .collect()
    }

    #[must_use]
    pub fn suggest_with_range(&self, input: &str, source: &CommandSource) -> Suggestions {
        // Never suggest arguments for a command that has been turned off.
        if self.is_disabled(Self::command_name(input)) {
            return Suggestions::empty();
        }

        let parsed = self.parse_input(input, source);
        self.get_completion_suggestions_at_end(parsed)
    }

    /// Gets all the commands usable in this dispatcher, sorted.
    /// The map returned has the key as the command name
    /// and the value as the command's description.
    #[must_use]
    pub fn get_all_commands(&self) -> BTreeMap<&str, &str> {
        let mut commands: BTreeMap<&str, &str> = BTreeMap::new();

        for command in self.tree.get_root_children() {
            let meta = &self.tree[command].meta;
            if self.is_disabled(&meta.literal_lowercase) {
                continue;
            }
            commands.insert(&meta.literal_lowercase, &meta.description);
        }

        commands
    }

    /// Gets all the commands usable in this dispatcher, which
    /// the given source is able to use.
    /// The map returned has the key as the command name
    /// and the value as the command's description.
    #[must_use]
    pub fn get_all_permitted_commands(&self, source: &CommandSource) -> BTreeMap<&str, &str> {
        let mut commands: BTreeMap<&str, &str> = BTreeMap::new();

        for command in self.tree.get_root_children() {
            if self.tree.can_use(command.into(), source) {
                let meta = &self.tree[command].meta;
                if self.is_disabled(&meta.literal_lowercase) {
                    continue;
                }
                commands.insert(&meta.literal_lowercase, &meta.description);
            }
        }

        commands
    }

    /// Gets the description and usage of each permitted command of the given source.
    ///
    /// The key is the command identifier,
    /// and the value is a tuple of `(description, usage)`.
    #[must_use]
    pub fn get_all_permitted_commands_usage(
        &self,
        source: &CommandSource,
    ) -> BTreeMap<&str, (&str, Box<str>)> {
        let mut commands: BTreeMap<&str, (&str, Box<str>)> = BTreeMap::new();

        for (command_node_id, usage) in self.get_usage_of_commands(source) {
            let meta = &self.tree[command_node_id].meta;
            let command_name = meta.literal.as_ref();
            if self.is_disabled(&meta.literal_lowercase) {
                continue;
            }
            let command_description = meta.description.as_ref();
            commands.insert(command_name, (command_description, usage.into_boxed_str()));
        }

        commands
    }

    /// Gets the description and usage of commands from a specific plugin.
    /// Only returns commands that the source has permission to use.
    #[must_use]
    pub fn get_all_permitted_commands_usage_by_plugin(
        &self,
        source: &CommandSource,
        plugin_name: &str,
    ) -> BTreeMap<&str, (&str, Box<str>)> {
        let mut commands: BTreeMap<&str, (&str, Box<str>)> = BTreeMap::new();

        for (command_node_id, usage) in self.get_usage_of_commands(source) {
            let meta = &self.tree[command_node_id].meta;
            if self.is_disabled(&meta.literal_lowercase) {
                continue;
            }
            if let Some(src) = &meta.source
                && src == plugin_name
            {
                let command_name = meta.literal.as_ref();
                let command_description = meta.description.as_ref();
                commands.insert(command_name, (command_description, usage.into_boxed_str()));
            }
        }

        commands
    }

    /// Gets the description and usage of a given command of the given source.
    /// Returns `None` if not found or the source has insufficient permissions.
    ///
    /// The key is the command identifier,
    /// and the value is a tuple of `(description, usage)`.
    #[must_use]
    pub fn get_permitted_command_usage(
        &self,
        source: &CommandSource,
        command: &str,
    ) -> Option<(&str, Box<str>)> {
        let command_node_id = self.tree.get(command)?;

        // This propagates `None` to the function result if permissions are insufficient.
        let usage = self.get_usage_of_command(command_node_id, source)?;

        let description = self.tree[command_node_id].meta.description.as_ref();

        Some((description, usage.into_boxed_str()))
    }

    /// Returns the usage of the given command node.
    #[must_use]
    pub fn get_usage_of_command(
        &self,
        command_node: CommandNodeId,
        source: &CommandSource,
    ) -> Option<String> {
        // We know the root DOES NOT have an executor, so we pass false to `is_optional`.
        self.get_usage_recursive(command_node.into(), source, false, false, None)
            .map(|mut usage| {
                // We add a slash as the prefix.
                usage.insert(0, '/');
                usage
            })
    }

    /// Returns the usage of each child of the given node (permitted for the given source).
    #[must_use]
    pub fn get_usage_of_children(
        &self,
        node: NodeId,
        source: &CommandSource,
    ) -> FxHashMap<NodeId, String> {
        let mut map = FxHashMap::default();

        let is_optional = self.tree[node].command().is_some();
        for child in self.tree.get_children(node) {
            if let Some(usage) = self.get_usage_recursive(child, source, is_optional, false, None) {
                map.insert(child, usage);
            }
        }

        map
    }

    /// Returns the usage of each command (permitted for the given source).
    #[must_use]
    pub fn get_usage_of_commands(
        &self,
        source: &CommandSource,
    ) -> FxHashMap<CommandNodeId, String> {
        self.get_usage_of_children(ROOT_NODE_ID, source)
            .into_iter()
            // This is safe because every child of the root child is a command node.
            .map(|(k, mut v)| {
                // We add a slash at the beginning for command usage.
                v.insert(0, '/');
                (CommandNodeId(k.0), v)
            })
            .collect()
    }

    /// Internal function to recurse usages.
    fn get_usage_recursive(
        &self,
        node: NodeId,
        source: &CommandSource,
        is_optional: bool,
        deep: bool,
        redirector_usage_text: Option<String>,
    ) -> Option<String> {
        if !self.tree.can_use(node, source) {
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
                    let target_usage = if target == node {
                        "...".to_string()
                    } else if self.tree.is_command_node(node) && self.tree.is_command_node(target) {
                        // We do this so for example it will show usage for /?:
                        //
                        // /? [<commandOrPage>]
                        //
                        // instead of
                        //
                        // /? -> help
                        return self.get_usage_recursive(
                            target,
                            source,
                            is_optional,
                            deep,
                            Some(usage_text),
                        );
                    } else {
                        format!("-> {}", self.tree[target].usage_text())
                    };
                    return Some(format!("{usage_text}{ARG_SEPARATOR}{target_usage}"));
                }
            } else {
                let mut children = Vec::new();
                for child in self.tree.get_children(node) {
                    if self.tree.can_use(child, source) {
                        children.push(child);
                    }
                }

                if children.len() == 1 {
                    let child = children[0];
                    if let Some(child_usage_text) =
                        self.get_usage_recursive(child, source, child_optional, true, None)
                    {
                        return Some(format!("{usage_text}{ARG_SEPARATOR}{child_usage_text}"));
                    }
                } else if !children.is_empty() {
                    let mut child_usages = Vec::new();
                    // TODO: Optimize this set algorithm while keeping insertion order.
                    for child in children {
                        if let Some(child_usage_text) =
                            self.get_usage_recursive(child, source, child_optional, true, None)
                            && !child_usages.contains(&child_usage_text)
                        {
                            child_usages.push(child_usage_text);
                        }
                    }
                    if child_usages.len() == 1 {
                        let mut child_usage = child_usages.pop().unwrap_or_default();
                        if is_optional {
                            child_usage =
                                format!("{USAGE_OPTIONAL_OPEN}{child_usage}{USAGE_OPTIONAL_CLOSE}");
                        }
                        return Some(format!("{usage_text}{ARG_SEPARATOR}{child_usage}"));
                    } else if !child_usages.is_empty() {
                        let (open, close) = if child_optional {
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

        Some(usage_text)
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

    #[test]
    fn unknown_command() {
        let mut dispatcher = CommandDispatcher::new();
        dispatcher.register(
            CommandArgumentBuilder::new("unknown", "A command without an executor").build(),
        );
        let source = CommandSource::dummy();
        let result = dispatcher.execute_input("unknown", &source);
        assert!(result.is_err_and(|error| error.error_type == &DISPATCHER_UNKNOWN_COMMAND));
    }

    #[test]
    fn simple_command() {
        let mut dispatcher = CommandDispatcher::new();
        let executor: fn(&CommandContext) -> CommandExecutorResult = |_| Ok(1);
        dispatcher
            .register(CommandArgumentBuilder::new("simple", "A simple command").executes(executor));
        let source = CommandSource::dummy();
        let result = dispatcher.execute_input("simple", &source);
        assert_eq!(result, Ok(1));
    }

    #[test]
    fn disabled_command_cannot_be_executed_directly() {
        // Guards the `/execute run <command>` bypass: a disabled command must be
        // rejected even when reached through `execute_input` rather than
        // `handle_command`.
        let mut dispatcher = CommandDispatcher::new();
        let executor: fn(&CommandContext) -> CommandExecutorResult = |_| Ok(1);
        dispatcher
            .register(CommandArgumentBuilder::new("simple", "A simple command").executes(executor));
        dispatcher.disable_command("simple");

        let source = CommandSource::dummy();
        let result = dispatcher.execute_input("simple", &source);
        assert!(result.is_err_and(|error| error.error_type == &DISPATCHER_UNKNOWN_COMMAND));
    }

    #[test]
    fn arithmetic_command() {
        enum Operation {
            Add,
            Subtract,
            Multiply,
            Divide,
        }

        struct Executor(Operation);
        impl CommandExecutor for Executor {
            fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
                let operand1: i32 = *context.get_argument("operand1")?;
                let operand2: i32 = *context.get_argument("operand2")?;
                Ok(match self.0 {
                    Operation::Add => operand1 + operand2,
                    Operation::Subtract => operand1 - operand2,
                    Operation::Multiply => operand1 * operand2,
                    Operation::Divide => operand1 / operand2,
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
            dispatcher.execute_input("arithmetic 3 + -7", &source),
            Ok(-4)
        );
        assert_eq!(
            dispatcher.execute_input("arithmetic 4 - -8", &source),
            Ok(12)
        );
        assert_eq!(
            dispatcher.execute_input("arithmetic 2 * 9", &source),
            Ok(18)
        );
        assert_eq!(dispatcher.execute_input("arithmetic 9 / 2", &source), Ok(4));
    }

    #[test]
    fn alias_simple() {
        let mut dispatcher = CommandDispatcher::new();
        let executor: fn(&CommandContext) -> CommandExecutorResult = |_| Ok(1);
        dispatcher.register(CommandArgumentBuilder::new("a", "A command").executes(executor));
        // Note that we CANNOT use redirect here as node itself needs to execute the command,
        // not its 'children'.
        dispatcher.register(CommandArgumentBuilder::new("b", "An alias for /a").executes(executor));
        let source = CommandSource::dummy();
        assert_eq!(dispatcher.execute_input("a", &source), Ok(1));
        assert_eq!(dispatcher.execute_input("b", &source), Ok(1));
    }

    #[test]
    fn alias_complex() {
        struct Executor;
        impl CommandExecutor for Executor {
            fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
                Ok(*context.get_argument("result")?)
            }
        }

        let mut dispatcher = CommandDispatcher::new();

        let a = dispatcher.register(CommandArgumentBuilder::new("a", "A command").then(
            RequiredArgumentBuilder::new("result", IntegerArgumentType::any()).executes(Executor),
        ));
        // Note that this time, we SHOULD use redirect - it is leading to another node having `command`.
        dispatcher.register(CommandArgumentBuilder::new("b", "An alias for /a").redirect(a));
        let source = CommandSource::dummy();
        assert_eq!(dispatcher.execute_input("a 5", &source), Ok(5));
        assert_eq!(dispatcher.execute_input("b 7", &source), Ok(7));
    }

    #[test]
    fn recurse() {
        struct Executor;
        impl CommandExecutor for Executor {
            fn execute(&self, _context: &CommandContext) -> CommandExecutorResult {
                Ok(1)
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
        assert_eq!(dispatcher.execute_input("recurse", &source), Ok(1));
        assert_eq!(dispatcher.execute_input("recurse 4", &source), Ok(1));
        assert_eq!(dispatcher.execute_input("recurse 9 -1", &source), Ok(1));
        assert_eq!(
            dispatcher.execute_input("recurse 9 7 -6 5 -4", &source),
            Ok(1)
        );
        assert_eq!(
            dispatcher.execute_input("recurse 1 2 4 8 16 32 64 128 256 512", &source),
            Ok(1)
        );
    }

    #[test]
    fn double_slash_command_execution() {
        let mut dispatcher = CommandDispatcher::new();
        let executor: fn(&CommandContext) -> CommandExecutorResult = |_| Ok(42);

        dispatcher.register(
            CommandArgumentBuilder::new("//set", "WorldEdit set command").executes(executor),
        );

        let source = CommandSource::dummy();
        // Direct execution with //set
        assert_eq!(dispatcher.execute_input("//set", &source), Ok(42));
        // Execution via /set alias (as sent by Java client for //set)
        assert_eq!(dispatcher.execute_input("/set", &source), Ok(42));
    }
}
