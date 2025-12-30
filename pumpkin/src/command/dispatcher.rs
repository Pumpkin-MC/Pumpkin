use pumpkin_protocol::java::client::play::CommandSuggestion;
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::color::NamedColor;

use super::args::ConsumedArgs;

use crate::command::CommandSender;
use crate::command::dispatcher::CommandError::{
    CommandFailed, InvalidConsumption, InvalidRequirement, PermissionDenied,
};
use crate::command::tree::{Command, CommandTree, NodeType, RawArgs};
use crate::server::Server;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Maximum commands per second per player
/// Requirements: 7.1
pub const MAX_COMMANDS_PER_SECOND: u32 = 10;

/// Warning threshold - warn player when they reach this percentage of the limit
/// Requirements: 7.3
pub const COMMAND_RATE_WARNING_THRESHOLD: u32 = 8;

/// Entry tracking command usage from a specific player
#[derive(Debug, Clone)]
struct CommandRateLimitEntry {
    /// Number of commands in current window
    count: u32,
    /// Start of the current time window
    window_start: Instant,
    /// Whether we've already warned the player this window
    warned: bool,
}

/// A thread-safe rate limiter for player commands.
/// Tracks command usage per player UUID.
/// Requirements: 7.1, 7.2, 7.3
pub struct CommandRateLimiter {
    /// Command counts per player UUID
    entries: RwLock<HashMap<Uuid, CommandRateLimitEntry>>,
    /// Maximum commands allowed per second
    max_commands: u32,
    /// Warning threshold (number of commands before warning)
    warning_threshold: u32,
}

impl Default for CommandRateLimiter {
    fn default() -> Self {
        Self::new(MAX_COMMANDS_PER_SECOND, COMMAND_RATE_WARNING_THRESHOLD)
    }
}

impl CommandRateLimiter {
    /// Creates a new command rate limiter.
    ///
    /// # Arguments
    /// * `max_commands` - Maximum commands allowed per second
    /// * `warning_threshold` - Number of commands before warning the player
    #[must_use]
    pub fn new(max_commands: u32, warning_threshold: u32) -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            max_commands,
            warning_threshold,
        }
    }

    /// Checks if a player is allowed to execute a command and records the attempt.
    /// Returns a tuple of (allowed, should_warn).
    /// Requirements: 7.1, 7.2, 7.3
    pub async fn check_and_record(&self, player_id: &Uuid) -> (bool, bool) {
        let now = Instant::now();
        let window = Duration::from_secs(1);
        let mut entries = self.entries.write().await;

        let entry = entries.entry(*player_id).or_insert(CommandRateLimitEntry {
            count: 0,
            window_start: now,
            warned: false,
        });

        // Reset window if expired
        if now.duration_since(entry.window_start) >= window {
            entry.count = 0;
            entry.window_start = now;
            entry.warned = false;
        }

        // Check if rate limited
        if entry.count >= self.max_commands {
            log::debug!(
                "Player {} exceeded command rate limit ({}/{})",
                player_id,
                entry.count,
                self.max_commands
            );
            return (false, false);
        }

        entry.count += 1;

        // Check if we should warn
        let should_warn = entry.count >= self.warning_threshold && !entry.warned;
        if should_warn {
            entry.warned = true;
        }

        (true, should_warn)
    }

    /// Gets the current command count for a player (useful for testing/monitoring).
    #[cfg(test)]
    pub async fn get_count(&self, player_id: &Uuid) -> u32 {
        let entries = self.entries.read().await;
        entries.get(player_id).map(|e| e.count).unwrap_or(0)
    }

    /// Resets the rate limiter state for a player (useful for testing).
    #[cfg(test)]
    pub async fn reset(&self, player_id: &Uuid) {
        self.entries.write().await.remove(player_id);
    }

    /// Cleans up expired entries to prevent memory growth.
    /// Should be called periodically.
    pub async fn cleanup(&self) {
        let now = Instant::now();
        let window = Duration::from_secs(2); // Keep entries for 2 seconds
        let mut entries = self.entries.write().await;
        entries.retain(|_, entry| now.duration_since(entry.window_start) < window);
    }
}

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
    /// The command was rate limited - player is sending too many commands.
    /// Requirements: 7.2
    RateLimited,
    /// A general error occurred during command execution that doesn't fit into
    /// more specific `CommandError` variants.
    CommandFailed(TextComponent),
}

impl CommandError {
    #[must_use]
    pub fn into_component(self, cmd: &str) -> TextComponent {
        match self {
            InvalidConsumption(s) => {
                log::error!(
                    "Error while parsing command \"{cmd}\": {s:?} was consumed, but couldn't be parsed"
                );
                TextComponent::text("Internal error (See logs for details)")
            }
            InvalidRequirement => {
                log::error!(
                    "Error while parsing command \"{cmd}\": a requirement that was expected was not met."
                );
                TextComponent::text("Internal error (See logs for details)")
            }
            PermissionDenied => {
                log::warn!("Permission denied for command \"{cmd}\"");
                TextComponent::text(
                    "I'm sorry, but you do not have permission to perform this command. Please contact the server administrator if you believe this is an error.",
                )
            }
            CommandError::RateLimited => {
                // Silent - we don't send an error message for rate limited commands
                // The command is simply ignored per Requirements 7.2
                TextComponent::text("")
            }
            CommandFailed(s) => s,
        }
    }
}

#[derive(Default)]
pub struct CommandDispatcher {
    pub commands: HashMap<String, Command>,
    pub permissions: HashMap<String, String>,
}

/// Stores registered [`CommandTree`]s and dispatches commands to them.
impl CommandDispatcher {
    pub async fn handle_command<'a>(
        &'a self,
        sender: &CommandSender,
        server: &'a Server,
        cmd: &'a str,
    ) {
        if let Err(e) = self.dispatch(sender, server, cmd).await {
            let text = e.into_component(cmd);
            sender
                .send_message(text.color_named(pumpkin_util::text::color::NamedColor::Red))
                .await;
        }
    }

    /// server side suggestions (client side suggestions work independently)
    ///
    /// # todo
    /// - make this less ugly
    /// - do not query suggestions for the same consumer multiple times just because they are on different paths through the tree
    pub(crate) async fn find_suggestions<'a>(
        &'a self,
        src: &CommandSender,
        server: &'a Server,
        cmd: &'a str,
    ) -> Vec<CommandSuggestion> {
        let mut parts = cmd.split_whitespace();
        let Some(key) = parts.next() else {
            return Vec::new();
        };
        let mut raw_args: Vec<&str> = parts.rev().collect();

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
                    log::trace!(
                        "Error while parsing command \"{cmd}\": {s:?} was consumed, but couldn't be parsed"
                    );
                    return Vec::new();
                }
                Err(InvalidRequirement) => {
                    log::trace!(
                        "Error while parsing command \"{cmd}\": a requirement that was expected was not met."
                    );
                    return Vec::new();
                }
                Err(PermissionDenied) => {
                    log::trace!("Permission denied for command \"{cmd}\"");
                    return Vec::new();
                }
                Err(CommandError::RateLimited) => {
                    log::trace!("Rate limited for command \"{cmd}\"");
                    return Vec::new();
                }
                Err(CommandFailed(_)) => {
                    return Vec::new();
                }
                Ok(Some(new_suggestions)) => {
                    suggestions.extend(new_suggestions);
                }
                Ok(None) => {}
            }
        }

        let mut suggestions = Vec::from_iter(suggestions);
        suggestions.sort_by(|a, b| a.suggestion.cmp(&b.suggestion));
        suggestions
    }

    pub(crate) fn split_parts(cmd: &str) -> Result<(&str, Vec<&str>), CommandError> {
        if cmd.is_empty() {
            return Err(CommandFailed(TextComponent::text("Empty Command")));
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
                '{' => {
                    if !in_single_quotes && !in_double_quotes {
                        in_braces += 1;
                    }
                }
                '}' => {
                    if !in_single_quotes && !in_double_quotes {
                        if in_braces == 0 {
                            return Err(CommandFailed(TextComponent::text("Unmatched braces")));
                        }
                        in_braces -= 1;
                    }
                }
                '[' => {
                    if !in_single_quotes && !in_double_quotes {
                        in_brackets += 1;
                    }
                }
                ']' => {
                    if !in_single_quotes && !in_double_quotes {
                        if in_brackets == 0 {
                            return Err(CommandFailed(TextComponent::text("Unmatched brackets")));
                        }
                        in_brackets -= 1;
                    }
                }
                '\'' => {
                    if !in_double_quotes {
                        in_single_quotes = !in_single_quotes;
                    }
                }
                '"' => {
                    if !in_single_quotes {
                        in_double_quotes = !in_double_quotes;
                    }
                }
                ' ' if !in_single_quotes
                    && !in_double_quotes
                    && in_braces == 0
                    && in_brackets == 0 =>
                {
                    if current_arg_start != i {
                        args.push(&cmd[current_arg_start..i]);
                    }
                    current_arg_start = i + 1;
                }
                _ => {}
            }
        }
        if current_arg_start != cmd.len() {
            args.push(&cmd[current_arg_start..]);
        }
        if in_single_quotes || in_double_quotes {
            return Err(CommandFailed(TextComponent::text(
                "Unmatched quotes at the end",
            )));
        }
        if in_braces != 0 {
            return Err(CommandFailed(TextComponent::text(
                "Unmatched braces at the end",
            )));
        }
        if in_brackets != 0 {
            return Err(CommandFailed(TextComponent::text(
                "Unmatched brackets at the end",
            )));
        }
        if args.is_empty() {
            return Err(CommandFailed(TextComponent::text("Empty Command")));
        }
        let key = args.remove(0);
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
            return Err(CommandFailed(TextComponent::text(format!(
                "Command {key} does not exist"
            ))));
        }

        let Some(permission) = self.permissions.get(key) else {
            return Err(CommandFailed(TextComponent::text(
                "Permission for Command not found".to_string(),
            )));
        };

        if !src.has_permission(permission.as_str()).await {
            return Err(PermissionDenied);
        }

        let tree = self.get_tree(key)?;

        // try paths until fitting path is found
        for path in tree.iter_paths() {
            if Self::try_is_fitting_path(src, server, &path, tree, &mut raw_args.clone()).await? {
                return Ok(());
            }
        }
        Err(CommandFailed(TextComponent::text(format!(
            "Invalid Syntax. Usage: {tree}"
        ))))
    }

    pub fn get_tree<'a>(&'a self, key: &str) -> Result<&'a CommandTree, CommandError> {
        let command = self
            .commands
            .get(key)
            .ok_or(CommandFailed(TextComponent::text("Command not found")))?;

        match command {
            Command::Tree(tree) => Ok(tree),
            Command::Alias(target) => {
                let Some(Command::Tree(tree)) = self.commands.get(target) else {
                    log::error!(
                        "Error while parsing command alias \"{key}\": pointing to \"{target}\" which is not a valid tree"
                    );
                    return Err(CommandFailed(TextComponent::text(
                        "Internal Error (See logs for details)",
                    )));
                };
                Ok(tree)
            }
        }
    }

    async fn try_is_fitting_path<'a>(
        src: &'a CommandSender,
        server: &'a Server,
        path: &[usize],
        tree: &'a CommandTree,
        raw_args: &mut RawArgs<'a>,
    ) -> Result<bool, CommandError> {
        let mut parsed_args: ConsumedArgs = HashMap::new();

        for node in path.iter().map(|&i| &tree.nodes[i]) {
            match &node.node_type {
                NodeType::ExecuteLeaf { executor } => {
                    return if raw_args.is_empty() {
                        executor.execute(src, server, &parsed_args).await?;
                        Ok(true)
                    } else {
                        log::debug!(
                            "Error while parsing command: {raw_args:?} was not consumed, but should have been"
                        );
                        Ok(false)
                    };
                }
                NodeType::Literal { string, .. } => {
                    if raw_args.pop() != Some(string) {
                        log::debug!("Error while parsing command: {raw_args:?}: expected {string}");
                        return Ok(false);
                    }
                }
                NodeType::Argument { consumer, name, .. } => {
                    if let Some(consumed) = consumer.consume(src, server, raw_args).await {
                        parsed_args.insert(name, consumed);
                    } else {
                        log::debug!(
                            "Error while parsing command: {raw_args:?}: cannot parse argument {name}"
                        );
                        return Ok(false);
                    }
                }
                NodeType::Require { predicate, .. } => {
                    if !predicate(src) {
                        log::debug!(
                            "Error while parsing command: {raw_args:?} does not meet the requirement"
                        );
                        return Ok(false);
                    }
                }
            }
        }

        log::debug!(
            "Error while parsing command: {raw_args:?} was not consumed, but should have been"
        );
        Ok(false)
    }

    async fn try_find_suggestions_on_path<'a>(
        src: &'a CommandSender,
        server: &'a Server,
        path: &[usize],
        tree: &'a CommandTree,
        raw_args: &mut RawArgs<'a>,
        input: &'a str,
    ) -> Result<Option<Vec<CommandSuggestion>>, CommandError> {
        let mut parsed_args: ConsumedArgs = HashMap::new();

        for node in path.iter().map(|&i| &tree.nodes[i]) {
            match &node.node_type {
                NodeType::ExecuteLeaf { .. } => {
                    return Ok(None);
                }
                NodeType::Literal { string, .. } => {
                    if raw_args.pop() != Some(string) {
                        return Ok(None);
                    }
                }
                NodeType::Argument { consumer, name } => {
                    match consumer.consume(src, server, raw_args).await {
                        Some(consumed) => {
                            parsed_args.insert(name, consumed);
                        }
                        None => {
                            return if raw_args.is_empty() {
                                let suggestions = consumer.suggest(src, server, input).await?;
                                Ok(suggestions)
                            } else {
                                Ok(None)
                            };
                        }
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

        let primary_name = names.next().expect("at least one name must be provided");

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

    use crate::command::{commands::default_dispatcher, tree::CommandTree};
    #[tokio::test]
    async fn test_dynamic_command() {
        let config = BasicConfiguration::default();
        let mut dispatcher = default_dispatcher(&config).await;
        let tree = CommandTree::new(["test"], "test_desc");
        dispatcher.register(tree, "minecraft:test");
    }
}
