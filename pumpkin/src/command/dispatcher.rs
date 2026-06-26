use pumpkin_data::translation;
use pumpkin_i18n::server_command_locale;
use pumpkin_protocol::java::client::play::CommandSuggestion;
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::click::ClickEvent;
use pumpkin_util::text::color::NamedColor;
use rustc_hash::FxHashMap;
use tracing::{debug, error, warn};

use super::args::ConsumedArgs;
use super::errors::command_syntax_error::{CommandSyntaxError, CommandSyntaxErrorContext};
use super::errors::error_types;
use super::{tr, tr_format, tr_plain};

use crate::command::CommandSender;
use crate::command::dispatcher::CommandError::{
    CommandFailed, InvalidConsumption, InvalidRequirement, PermissionDenied, SyntaxError,
};
use crate::command::tree::{Command, CommandTree, NodeType, RawArg, RawArgs};
use crate::server::Server;
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub enum CommandError {
    /// This error means that there was an error while parsing a previously consumed argument.
    /// That only happens when consumption is wrongly implemented, as it should ensure parsing may
    /// never fail.
    InvalidConsumption(Option<String>),
    /// Return this if a condition that a [`Node::Require`] should ensure is met is not met.
    InvalidRequirement,
    /// The command could not be executed due to insufficient permissions.
    /// The user attempting to run the command lacks the necessary authorization.
    PermissionDenied,
    /// A general error occurred during command execution that doesn't fit into
    /// more specific `CommandError` variants.
    CommandFailed(TextComponent),
    SyntaxError(CommandSyntaxError),
}

impl CommandError {
    #[must_use]
    pub fn into_messages(self, cmd: &str, locale: pumpkin_i18n::Locale) -> Vec<TextComponent> {
        match self {
            InvalidConsumption(s) => {
                error!(
                    "{}",
                    tr_format(
                        "commands.dispatcher.log.invalid_consumption",
                        server_command_locale(),
                        &[cmd.to_owned(), format!("{s:?}")]
                    )
                );
                vec![tr("commands.dispatcher.internal_error", locale, [])]
            }
            InvalidRequirement => {
                error!(
                    "{}",
                    tr_format(
                        "commands.dispatcher.log.invalid_requirement",
                        server_command_locale(),
                        &[cmd.to_owned()]
                    )
                );
                vec![tr("commands.dispatcher.internal_error", locale, [])]
            }
            PermissionDenied => {
                warn!(
                    "{}",
                    tr_format(
                        "commands.dispatcher.log.permission_denied",
                        server_command_locale(),
                        &[cmd.to_owned()]
                    )
                );
                vec![tr("commands.dispatcher.permission_denied", locale, [])]
            }
            CommandFailed(s) => vec![s],
            SyntaxError(s) => render_syntax_error_messages(s),
        }
    }
}

#[derive(Debug)]
struct PathParsingFailure {
    cursor: usize,
    consumed_tokens: usize,
    matched_any_node: bool,
    syntax_error: Option<CommandSyntaxError>,
}

enum PathResult {
    Matched,
    Failed(PathParsingFailure),
}

fn render_syntax_error_messages(syntax_error: CommandSyntaxError) -> Vec<TextComponent> {
    let Some(context) = syntax_error.context else {
        return vec![syntax_error.message];
    };

    let input = context.input;
    let cursor = clamp_cursor_to_char_boundary(&input, context.cursor);
    let context_start = last_n_char_start(&input, cursor, 10);
    let before_cursor = &input[context_start..cursor];
    let remaining_input = &input[cursor..];

    let mut context_message = TextComponent::text("")
        .color_named(NamedColor::Gray)
        .click_event(ClickEvent::SuggestCommand {
            command: format!("/{input}").into(),
        });
    if context_start > 0 {
        context_message = context_message.add_child(TextComponent::text("..."));
    }
    context_message = context_message.add_child(TextComponent::text(before_cursor.to_string()));

    if !remaining_input.is_empty() {
        context_message = context_message.add_child(
            TextComponent::text(remaining_input.to_string())
                .color_named(NamedColor::Red)
                .underlined(),
        );
    }

    context_message = context_message.add_child(
        TextComponent::translate_cross(
            translation::java::COMMAND_CONTEXT_HERE,
            translation::java::COMMAND_CONTEXT_HERE,
            [],
        )
        .color_named(NamedColor::Red)
        .italic(),
    );

    vec![syntax_error.message, context_message]
}

fn clamp_cursor_to_char_boundary(input: &str, cursor: usize) -> usize {
    let mut clamped = cursor.min(input.len());
    while clamped > 0 && !input.is_char_boundary(clamped) {
        clamped -= 1;
    }
    clamped
}

fn last_n_char_start(input: &str, cursor: usize, char_count: usize) -> usize {
    let mut start = cursor;
    let mut seen = 0usize;
    for (index, _) in input[..cursor].char_indices().rev() {
        start = index;
        seen += 1;
        if seen == char_count {
            break;
        }
    }
    if seen < char_count { 0 } else { start }
}

fn unknown_command_syntax_error(input: &str, cursor: usize) -> CommandSyntaxError {
    let context = CommandSyntaxErrorContext {
        input: input.to_string(),
        cursor: clamp_cursor_to_char_boundary(input, cursor),
    };
    error_types::DISPATCHER_UNKNOWN_COMMAND.create(&context)
}

fn unknown_argument_syntax_error(input: &str, cursor: usize) -> CommandSyntaxError {
    let context = CommandSyntaxErrorContext {
        input: input.to_string(),
        cursor: clamp_cursor_to_char_boundary(input, cursor),
    };
    error_types::DISPATCHER_UNKNOWN_ARGUMENT.create(&context)
}

fn select_parse_error(
    input: &str,
    failures: &[PathParsingFailure],
    known_command: bool,
) -> CommandSyntaxError {
    if failures.is_empty() {
        return if known_command {
            unknown_argument_syntax_error(input, input.len())
        } else {
            unknown_command_syntax_error(input, input.len())
        };
    }

    let farthest_cursor = failures
        .iter()
        .map(|failure| failure.cursor)
        .max()
        .unwrap_or(input.len());

    let best_progress = failures
        .iter()
        .filter(|failure| failure.cursor == farthest_cursor)
        .map(|failure| failure.consumed_tokens)
        .max()
        .unwrap_or(0);

    let finalists = failures
        .iter()
        .filter(|failure| {
            failure.cursor == farthest_cursor && failure.consumed_tokens == best_progress
        })
        .collect::<Vec<_>>();

    let syntax_errors = finalists
        .iter()
        .filter_map(|failure| failure.syntax_error.clone())
        .collect::<Vec<_>>();
    if syntax_errors.len() == 1 {
        return syntax_errors[0].clone();
    }

    let matched_any_node = finalists.iter().any(|failure| failure.matched_any_node);
    if matched_any_node || known_command {
        unknown_argument_syntax_error(input, farthest_cursor)
    } else {
        unknown_command_syntax_error(input, farthest_cursor)
    }
}

fn next_unread_cursor(raw_args: &RawArgs<'_>, input_len: usize) -> usize {
    raw_args.last().map_or(input_len, |arg| arg.start)
}

fn path_failure(
    raw_args: &RawArgs<'_>,
    total_args: usize,
    input_len: usize,
    matched_any_node: bool,
    syntax_error: Option<CommandSyntaxError>,
) -> PathParsingFailure {
    let syntax_error_cursor = syntax_error
        .as_ref()
        .and_then(|error| error.context.as_ref().map(|context| context.cursor))
        .unwrap_or_else(|| next_unread_cursor(raw_args, input_len));

    PathParsingFailure {
        cursor: syntax_error_cursor,
        consumed_tokens: total_args.saturating_sub(raw_args.len()),
        matched_any_node,
        syntax_error,
    }
}

#[derive(Default)]
pub struct CommandDispatcher {
    pub commands: FxHashMap<String, Command>,
    pub permissions: FxHashMap<String, String>,
}

/// Stores registered [`CommandTree`]s and dispatches commands to them.
impl CommandDispatcher {
    pub async fn handle_command<'a>(
        &'a self,
        sender: &CommandSender,
        server: &'a Server,
        cmd: &'a str,
    ) {
        let result = self.dispatch(sender, server, cmd).await;
        sender.set_success_count(u32::from(result.is_ok()));

        if let Err(e) = result {
            let locale = sender.get_locale();
            for text in e.into_messages(cmd, locale) {
                sender
                    .send_message(
                        TextComponent::text("")
                            .add_child(text)
                            .color_named(pumpkin_util::text::color::NamedColor::Red),
                    )
                    .await;
            }
        }
    }

    /// server side suggestions (client side suggestions work independently)
    ///
    /// # todo
    /// - make this less ugly
    /// - do not query suggestions for the same consumer multiple times just because they are on different paths through the tree
    pub(crate) async fn find_suggestions<'a>(
        &'a self,
        src: &'a CommandSender,
        server: &'a Server,
        cmd: &'a str,
    ) -> Vec<CommandSuggestion> {
        let mut parts = cmd.split_whitespace();
        let Some(key) = parts.next() else {
            return Vec::new();
        };
        let mut raw_args: RawArgs<'a> = parts
            .rev()
            .map(|value| RawArg {
                value,
                start: 0,
                end: 0,
                input: cmd,
            })
            .collect();

        let Ok(tree) = self.get_tree(key) else {
            return Vec::new();
        };

        let mut suggestions = HashSet::new();

        // try paths and collect the nodes that fail
        // todo: make this more fine-grained
        for path in tree.iter_paths() {
            match Self::try_find_suggestions_on_path(src, server, &path, tree, &mut raw_args, cmd)
                .await
            {
                Err(InvalidConsumption(s)) => {
                    debug!(
                        "{}",
                        tr_format(
                            "commands.dispatcher.log.invalid_consumption",
                            server_command_locale(),
                            &[cmd.to_owned(), format!("{s:?}")],
                        )
                    );
                    return Vec::new();
                }
                Err(InvalidRequirement) => {
                    debug!(
                        "{}",
                        tr_format(
                            "commands.dispatcher.log.invalid_requirement",
                            server_command_locale(),
                            &[cmd.to_owned()],
                        )
                    );
                    return Vec::new();
                }
                Err(PermissionDenied) => {
                    debug!(
                        "{}",
                        tr_format(
                            "commands.dispatcher.log.permission_denied",
                            server_command_locale(),
                            &[cmd.to_owned()],
                        )
                    );
                    return Vec::new();
                }
                Err(CommandFailed(_)) => {
                    debug!(
                        "{}",
                        tr_plain("server.log.command_failed", server_command_locale())
                    );
                    return Vec::new();
                }
                Err(SyntaxError(_)) => {
                    return Vec::new();
                }
                Ok(Some(new_suggestions)) => {
                    suggestions.extend(new_suggestions);
                }
                Ok(None) => {
                    debug!(
                        "{}",
                        tr_plain("server.log.command_none", server_command_locale())
                    );
                }
            }
        }

        let mut suggestions = Vec::from_iter(suggestions);
        suggestions.sort_by(|a, b| a.suggestion.cmp(&b.suggestion));
        suggestions
    }

    #[allow(clippy::too_many_lines)]
    pub(crate) fn split_parts(cmd: &str) -> Result<(&str, RawArgs<'_>), CommandError> {
        if cmd.is_empty() {
            return Err(CommandFailed(tr(
                "commands.dispatcher.empty_command",
                server_command_locale(),
                [],
            )));
        }
        let mut args = Vec::new();
        let mut current_arg_start = 0usize;
        let mut in_single_quotes = false;
        let mut in_double_quotes = false;
        let mut in_braces = 0u32;
        let mut in_brackets = 0u32;
        let mut is_escaping = false;
        for (i, c) in cmd.char_indices() {
            if c == '\\' {
                is_escaping = !is_escaping;
                continue;
            }
            if is_escaping {
                is_escaping = false;
                continue;
            }
            match c {
                '{' if !in_single_quotes && !in_double_quotes => {
                    in_braces += 1;
                }
                '}' if !in_single_quotes && !in_double_quotes => {
                    if in_braces == 0 {
                        return Err(CommandFailed(tr(
                            "commands.dispatcher.unmatched_braces",
                            server_command_locale(),
                            [],
                        )));
                    }
                    in_braces -= 1;
                }
                '[' if !in_single_quotes && !in_double_quotes => {
                    in_brackets += 1;
                }
                ']' if !in_single_quotes && !in_double_quotes => {
                    if in_brackets == 0 {
                        return Err(CommandFailed(tr(
                            "commands.dispatcher.unmatched_brackets",
                            server_command_locale(),
                            [],
                        )));
                    }
                    in_brackets -= 1;
                }
                '\'' if !in_double_quotes => {
                    in_single_quotes = !in_single_quotes;
                }
                '"' if !in_single_quotes => {
                    in_double_quotes = !in_double_quotes;
                }
                ' ' if !in_single_quotes
                    && !in_double_quotes
                    && in_braces == 0
                    && in_brackets == 0 =>
                {
                    if current_arg_start != i {
                        args.push(RawArg {
                            value: &cmd[current_arg_start..i],
                            start: current_arg_start,
                            end: i,
                            input: cmd,
                        });
                    }
                    current_arg_start = i + 1;
                }
                _ => {}
            }
        }
        if current_arg_start != cmd.len() {
            args.push(RawArg {
                value: &cmd[current_arg_start..],
                start: current_arg_start,
                end: cmd.len(),
                input: cmd,
            });
        }
        if in_single_quotes || in_double_quotes {
            return Err(CommandFailed(tr(
                "commands.dispatcher.unmatched_quotes",
                server_command_locale(),
                [],
            )));
        }
        if in_braces != 0 {
            return Err(CommandFailed(tr(
                "commands.dispatcher.unmatched_braces_end",
                server_command_locale(),
                [],
            )));
        }
        if in_brackets != 0 {
            return Err(CommandFailed(tr(
                "commands.dispatcher.unmatched_brackets_end",
                server_command_locale(),
                [],
            )));
        }
        if args.is_empty() {
            return Err(CommandFailed(tr(
                "commands.dispatcher.empty_command",
                server_command_locale(),
                [],
            )));
        }
        let key = args.remove(0).value;
        Ok((key, args.into_iter().rev().collect()))
    }

    /// Execute a command using its corresponding [`CommandTree`].
    pub(crate) async fn dispatch<'a>(
        &'a self,
        src: &CommandSender,
        server: &'a Server,
        cmd: &'a str,
    ) -> Result<(), CommandError> {
        let (key, raw_args) = Self::split_parts(cmd)?;

        if !self.commands.contains_key(key) {
            return Err(SyntaxError(unknown_command_syntax_error(cmd, 0)));
        }

        let Some(permission) = self.permissions.get(key) else {
            return Err(CommandFailed(tr(
                "commands.dispatcher.permission_not_found",
                server_command_locale(),
                [],
            )));
        };

        if !src.has_permission(server, permission.as_str()).await {
            return Err(PermissionDenied);
        }

        let tree = self.get_tree(key)?;

        let mut path_failures = Vec::new();

        // try paths until fitting path is found
        for path in tree.iter_paths() {
            match Self::try_is_fitting_path(src, server, &path, tree, &mut raw_args.clone(), cmd)
                .await
            {
                Ok(PathResult::Matched) => return Ok(()),
                Ok(PathResult::Failed(failure)) => path_failures.push(failure),
                Err(error) => return Err(error),
            }
        }

        Err(SyntaxError(select_parse_error(cmd, &path_failures, true)))
    }

    pub fn get_tree<'a>(&'a self, key: &str) -> Result<&'a CommandTree, CommandError> {
        let command = self.commands.get(key).ok_or(CommandFailed(tr(
            "commands.dispatcher.command_not_found",
            server_command_locale(),
            [],
        )))?;

        match command {
            Command::Tree(tree) => Ok(tree),
            Command::Alias(target) => {
                let Some(Command::Tree(tree)) = self.commands.get(target) else {
                    error!(
                        "Error while parsing command alias \"{key}\": pointing to \"{target}\" which is not a valid tree"
                    );
                    return Err(CommandFailed(tr(
                        "commands.dispatcher.internal_error",
                        server_command_locale(),
                        [],
                    )));
                };
                Ok(tree)
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    async fn try_is_fitting_path<'a>(
        src: &'a CommandSender,
        server: &'a Server,
        path: &[usize],
        tree: &'a CommandTree,
        raw_args: &mut RawArgs<'a>,
        input: &str,
    ) -> Result<PathResult, CommandError> {
        let mut parsed_args: ConsumedArgs = HashMap::new();
        let total_args = raw_args.len();
        let mut matched_any_node = false;
        let input_len = input.len();

        for node in path.iter().map(|&i| &tree.nodes[i]) {
            match &node.node_type {
                NodeType::ExecuteLeaf { executor } => {
                    return if raw_args.is_empty() {
                        executor.execute(src, server, &parsed_args).await?;
                        Ok(PathResult::Matched)
                    } else {
                        debug!(
                            "Error while parsing command: {raw_args:?} was not consumed, but should have been"
                        );
                        Ok(PathResult::Failed(path_failure(
                            raw_args,
                            total_args,
                            input_len,
                            matched_any_node,
                            None,
                        )))
                    };
                }
                NodeType::Literal { string, .. } => {
                    let Some(raw_arg) = raw_args.last() else {
                        return Ok(PathResult::Failed(path_failure(
                            raw_args,
                            total_args,
                            input_len,
                            matched_any_node,
                            None,
                        )));
                    };
                    if raw_arg.value != string.as_str() {
                        debug!(
                            "{}",
                            tr_format(
                                "server.log.command_expected_argument",
                                server_command_locale(),
                                &[format!("{raw_args:?}"), string.clone()],
                            )
                        );
                        return Ok(PathResult::Failed(path_failure(
                            raw_args,
                            total_args,
                            input_len,
                            matched_any_node,
                            None,
                        )));
                    }
                    raw_args.pop();
                    matched_any_node = true;
                }
                NodeType::Argument { consumer, name, .. } => {
                    match consumer.consume_with_syntax(src, server, raw_args).await {
                        Ok(Some(consumed)) => {
                            parsed_args.insert(name, consumed);
                            matched_any_node = true;
                        }
                        Ok(None) => {
                            debug!(
                                "{}",
                                tr_format(
                                    "server.log.command_cannot_parse_argument",
                                    server_command_locale(),
                                    &[format!("{raw_args:?}"), name.clone()],
                                )
                            );
                            return Ok(PathResult::Failed(path_failure(
                                raw_args,
                                total_args,
                                input_len,
                                matched_any_node,
                                None,
                            )));
                        }
                        Err(error) => {
                            return Ok(PathResult::Failed(path_failure(
                                raw_args,
                                total_args,
                                input_len,
                                matched_any_node,
                                Some(error),
                            )));
                        }
                    }
                }
                NodeType::Require { predicate, .. } => {
                    if !predicate(src) {
                        debug!(
                            "{}",
                            tr_format(
                                "server.log.command_requirement_not_met",
                                server_command_locale(),
                                &[format!("{raw_args:?}")],
                            )
                        );
                        return Ok(PathResult::Failed(path_failure(
                            raw_args,
                            total_args,
                            input_len,
                            matched_any_node,
                            None,
                        )));
                    }
                    matched_any_node = true;
                }
            }
        }

        debug!(
            "{}",
            tr_format(
                "server.log.command_unconsumed_args",
                server_command_locale(),
                &[format!("{raw_args:?}")],
            )
        );
        Ok(PathResult::Failed(path_failure(
            raw_args,
            total_args,
            input_len,
            matched_any_node,
            None,
        )))
    }

    async fn try_find_suggestions_on_path<'a>(
        src: &'a CommandSender,
        server: &'a Server,
        path: &[usize],
        tree: &'a CommandTree,
        raw_args: &mut RawArgs<'a>,
        input: &'a str,
    ) -> Result<Option<Vec<CommandSuggestion>>, CommandError> {
        //let mut parsed_args: ConsumedArgs = HashMap::new();

        for node in path.iter().map(|&i| &tree.nodes[i]) {
            match &node.node_type {
                NodeType::ExecuteLeaf { .. } => {
                    return Ok(None);
                }
                NodeType::Literal { string, .. } => {
                    if raw_args.pop().map(|arg| arg.value) != Some(string.as_str()) {
                        return Ok(None);
                    }
                }
                NodeType::Argument { consumer, name: _ } => {
                    match consumer.consume_with_syntax(src, server, raw_args).await {
                        Ok(Some(_consumed)) => {
                            //parsed_args.insert(name, consumed);
                        }
                        Ok(None) => {
                            return if raw_args.is_empty() {
                                let suggestions = consumer.suggest(src, server, input).await?;
                                Ok(suggestions)
                            } else {
                                Ok(None)
                            };
                        }
                        Err(_) => return Ok(None),
                    }
                }
                NodeType::Require { predicate, .. } => {
                    if !predicate(src) {
                        return Ok(None);
                    }
                }
            }
        }

        Ok(None)
    }

    /// Register a command with the dispatcher.
    pub fn register<P: Into<String>>(&mut self, tree: CommandTree, permission: P) {
        let mut names = tree.names.iter();
        let permission = permission.into();

        let primary_name = names.next().unwrap_or_else(|| {
            panic!(
                "{}",
                tr_plain(
                    "debug.expect.name_must_be_provided",
                    server_command_locale(),
                )
            )
        });

        for name in names {
            self.commands
                .insert(name.clone(), Command::Alias(primary_name.clone()));
            self.permissions.insert(name.clone(), permission.clone());
        }

        self.permissions.insert(primary_name.clone(), permission);
        self.commands
            .insert(primary_name.clone(), Command::Tree(tree));
    }

    /// Remove a command from the dispatcher by its primary name.
    pub fn unregister(&mut self, name: &str) {
        let mut to_remove = Vec::new();
        for (key, value) in &self.commands {
            if key == name {
                to_remove.push(key.clone());
            } else if let Command::Alias(target) = value
                && target == name
            {
                to_remove.push(key.clone());
            }
        }

        for key in to_remove {
            self.commands.remove(&key);
            self.permissions.remove(&key);
        }
    }
}

#[cfg(test)]
mod test {
    use pumpkin_config::BasicConfiguration;
    use pumpkin_data::translation;
    use pumpkin_util::permission::PermissionRegistry;
    use pumpkin_util::text::TextContent;
    use pumpkin_util::text::click::ClickEvent;
    use pumpkin_util::text::color::{Color, NamedColor};
    use tokio::sync::RwLock;

    use super::{
        PathParsingFailure, render_syntax_error_messages, select_parse_error,
        unknown_argument_syntax_error,
    };
    use crate::command::errors::error_types;
    use crate::command::{commands::default_dispatcher, tree::CommandTree};

    fn component_plain_text(component: &pumpkin_util::text::TextComponentBase) -> Option<&str> {
        if let TextContent::Text { text } = component.content.as_ref() {
            Some(text.as_ref())
        } else {
            None
        }
    }

    #[tokio::test]
    async fn dynamic_command() {
        let config = BasicConfiguration::default();
        let registry = RwLock::new(PermissionRegistry::new());
        let mut dispatcher = default_dispatcher(&registry, &config)
            .await
            .fallback_dispatcher;
        let tree = CommandTree::new(["test"], "test_desc");
        dispatcher.register(tree, "minecraft:test");
    }

    #[test]
    fn syntax_renderer_outputs_two_messages_with_context_styling() {
        let input = "0123456789abcdefghij";
        let error = unknown_argument_syntax_error(input, 15);
        let messages = render_syntax_error_messages(error);

        assert_eq!(messages.len(), 2);

        let context = &messages[1].0;
        assert_eq!(context.style.color, Some(Color::Named(NamedColor::Gray)));
        assert_eq!(
            context.style.click_event,
            Some(ClickEvent::SuggestCommand {
                command: format!("/{input}").into()
            })
        );
        assert_eq!(context.extra.len(), 4);

        assert_eq!(component_plain_text(&context.extra[0]), Some("..."));
        assert_eq!(component_plain_text(&context.extra[1]), Some("56789abcde"));
        assert_eq!(component_plain_text(&context.extra[2]), Some("fghij"));
        assert_eq!(
            context.extra[2].style.color,
            Some(Color::Named(NamedColor::Red))
        );
        assert_eq!(context.extra[2].style.underlined, Some(true));

        let here_component = &context.extra[3];
        if let TextContent::Translate { translate, .. } = here_component.content.as_ref() {
            assert_eq!(translate, translation::java::COMMAND_CONTEXT_HERE);
        } else {
            panic!("expected translate component for command.context.here");
        }
        assert_eq!(
            here_component.style.color,
            Some(Color::Named(NamedColor::Red))
        );
        assert_eq!(here_component.style.italic, Some(true));
    }

    #[test]
    fn parse_error_selection_prefers_farthest_cursor_then_progress() {
        let fallback_error = unknown_argument_syntax_error("test one two", 5);
        let preferred_error = unknown_argument_syntax_error("test one two", 9);

        let selected = select_parse_error(
            "test one two",
            &[
                PathParsingFailure {
                    cursor: 5,
                    consumed_tokens: 2,
                    matched_any_node: true,
                    syntax_error: Some(fallback_error),
                },
                PathParsingFailure {
                    cursor: 9,
                    consumed_tokens: 1,
                    matched_any_node: true,
                    syntax_error: None,
                },
                PathParsingFailure {
                    cursor: 9,
                    consumed_tokens: 2,
                    matched_any_node: true,
                    syntax_error: Some(preferred_error.clone()),
                },
            ],
            true,
        );

        assert_eq!(selected.context, preferred_error.context);
    }

    #[test]
    fn parse_error_selection_synthesizes_unknown_argument_for_tied_syntax_errors() {
        let selected = select_parse_error(
            "alpha beta gamma",
            &[
                PathParsingFailure {
                    cursor: 12,
                    consumed_tokens: 2,
                    matched_any_node: true,
                    syntax_error: Some(unknown_argument_syntax_error("alpha beta gamma", 12)),
                },
                PathParsingFailure {
                    cursor: 12,
                    consumed_tokens: 2,
                    matched_any_node: true,
                    syntax_error: Some(unknown_argument_syntax_error("alpha beta gamma", 12)),
                },
            ],
            true,
        );

        assert!(selected.is(&error_types::DISPATCHER_UNKNOWN_ARGUMENT));
    }

    #[test]
    fn parse_error_selection_can_synthesize_unknown_command_when_not_matched() {
        let selected = select_parse_error(
            "abc",
            &[PathParsingFailure {
                cursor: 0,
                consumed_tokens: 0,
                matched_any_node: false,
                syntax_error: None,
            }],
            false,
        );

        assert!(selected.is(&error_types::DISPATCHER_UNKNOWN_COMMAND));
    }
}
