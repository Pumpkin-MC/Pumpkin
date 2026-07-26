use super::{
    ARG_SEPARATOR, CommandDispatcher, USAGE_OPTIONAL_CLOSE, USAGE_OPTIONAL_OPEN, USAGE_OR,
    USAGE_REQUIRED_CLOSE, USAGE_REQUIRED_OPEN,
};
use crate::command::context::command_source::CommandSource;
use crate::command::node::attached::{CommandNodeId, NodeId};
use crate::command::node::tree::ROOT_NODE_ID;
use crate::command::tree::Command;
use rustc_hash::FxHashMap;
use std::collections::BTreeMap;
use std::pin::Pin;
use tracing::warn;

impl CommandDispatcher {
    /// Gets all the commands usable in this dispatcher, sorted.
    /// The map returned has the key as the command name
    /// and the value as the command's description.
    #[must_use]
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
    #[must_use]
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
                if let Some(permission) = self
                    .fallback_dispatcher
                    .permissions
                    .get(&command_tree.names[0])
                    && source.has_permission(permission).await
                {
                    for name in &command_tree.names {
                        commands.insert(name, &command_tree.description);
                    }
                } else {
                    warn!(
                        "Command /{} does not have a permission set up",
                        &command_tree.names[0]
                    );
                }
            }
        }

        commands
    }

    /// Gets the description and usage of each permitted command of the given source.
    ///
    /// The key is the command identifier,
    /// and the value is a tuple of `(description, usage)`.
    #[must_use]
    pub async fn get_all_permitted_commands_usage(
        &self,
        source: &CommandSource,
    ) -> BTreeMap<&str, (&str, Box<str>)> {
        let mut commands: BTreeMap<&str, (&str, Box<str>)> = BTreeMap::new();

        for (command_node_id, usage) in self.get_usage_of_commands(source).await {
            let meta = &self.tree[command_node_id].meta;
            let command_name = meta.literal.as_ref();
            let command_description = meta.description.as_ref();
            commands.insert(command_name, (command_description, usage.into_boxed_str()));
        }

        for fallback_command in self.fallback_dispatcher.commands.values() {
            if let Command::Tree(command_tree) = fallback_command
                && let Some(permission) = self
                    .fallback_dispatcher
                    .permissions
                    .get(&command_tree.names[0])
                && source.has_permission(permission).await
            {
                let usage = command_tree.to_string();
                for name in &command_tree.names {
                    commands.insert(
                        name,
                        (
                            command_tree.description.as_ref(),
                            usage.clone().into_boxed_str(),
                        ),
                    );
                }
            }
        }

        commands
    }

    /// Gets the description and usage of commands from a specific plugin.
    /// Only returns commands that the source has permission to use.
    pub async fn get_all_permitted_commands_usage_by_plugin(
        &self,
        source: &CommandSource,
        plugin_name: &str,
    ) -> BTreeMap<&str, (&str, Box<str>)> {
        let mut commands: BTreeMap<&str, (&str, Box<str>)> = BTreeMap::new();

        for fallback_command in self.fallback_dispatcher.commands.values() {
            if let Command::Tree(command_tree) = fallback_command
                && let Some(source_name) = &command_tree.source
                && source_name == plugin_name
                && let Some(permission) = self
                    .fallback_dispatcher
                    .permissions
                    .get(&command_tree.names[0])
                && source.has_permission(permission).await
            {
                let usage = command_tree.to_string();
                for name in &command_tree.names {
                    commands.insert(
                        name,
                        (
                            command_tree.description.as_ref(),
                            usage.clone().into_boxed_str(),
                        ),
                    );
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
    pub async fn get_permitted_command_usage(
        &self,
        source: &CommandSource,
        command: &str,
    ) -> Option<(&str, Box<str>)> {
        if let Some(output) = self
            .get_permitted_command_usage_non_fallback(source, command)
            .await
        {
            Some(output)
        } else {
            let tree = self.fallback_dispatcher.get_tree(command).ok()?;
            if let Some(permission) = self.fallback_dispatcher.permissions.get(&tree.names[0])
                && source.has_permission(permission).await
            {
                Some((tree.description.as_ref(), tree.to_string().into_boxed_str()))
            } else {
                None
            }
        }
    }

    async fn get_permitted_command_usage_non_fallback(
        &self,
        source: &CommandSource,
        command: &str,
    ) -> Option<(&str, Box<str>)> {
        let command_node_id = self.tree.get(command)?;

        // This propagates `None` to the function result if permissions are insufficient.
        let usage = self.get_usage_of_command(command_node_id, source).await?;

        let description = self.tree[command_node_id].meta.description.as_ref();

        Some((description, usage.into_boxed_str()))
    }

    /// Returns the usage of the given command node.
    pub async fn get_usage_of_command(
        &self,
        command_node: CommandNodeId,
        source: &CommandSource,
    ) -> Option<String> {
        // We know the root DOES NOT have an executor, so we pass false to `is_optional`.
        self.get_usage_recursive(command_node.into(), source, false, false, None)
            .await
            .map(|mut usage| {
                // We add a slash as the prefix.
                usage.insert(0, '/');
                usage
            })
    }

    /// Returns the usage of each child of the given node (permitted for the given source).
    pub async fn get_usage_of_children(
        &self,
        node: NodeId,
        source: &CommandSource,
    ) -> FxHashMap<NodeId, String> {
        let mut map = FxHashMap::default();

        let is_optional = self.tree[node].command().is_some();
        for child in self.tree.get_children(node) {
            if let Some(usage) = self
                .get_usage_recursive(child, source, is_optional, false, None)
                .await
            {
                map.insert(child, usage);
            }
        }

        map
    }

    /// Returns the usage of each command (permitted for the given source).
    pub async fn get_usage_of_commands(
        &self,
        source: &CommandSource,
    ) -> FxHashMap<CommandNodeId, String> {
        self.get_usage_of_children(ROOT_NODE_ID, source)
            .await
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
    fn get_usage_recursive<'a>(
        &'a self,
        node: NodeId,
        source: &'a CommandSource,
        is_optional: bool,
        deep: bool,
        redirector_usage_text: Option<String>,
    ) -> Pin<Box<dyn Future<Output = Option<String>> + Send + 'a>> {
        Box::pin(async move {
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
                        let target_usage = if target == node {
                            "...".to_string()
                        } else if self.tree.is_command_node(node)
                            && self.tree.is_command_node(target)
                        {
                            // We do this so for example it will show usage for /?:
                            //
                            // /? [<commandOrPage>]
                            //
                            // instead of
                            //
                            // /? -> help
                            return self
                                .get_usage_recursive(
                                    target,
                                    source,
                                    is_optional,
                                    deep,
                                    Some(usage_text),
                                )
                                .await;
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
                        if let Some(child_usage_text) = self
                            .get_usage_recursive(child, source, child_optional, true, None)
                            .await
                        {
                            return Some(format!("{usage_text}{ARG_SEPARATOR}{child_usage_text}"));
                        }
                    } else if !children.is_empty() {
                        let mut child_usages = Vec::new();
                        // TODO: Optimize this set algorithm while keeping insertion order.
                        for child in children {
                            if let Some(child_usage_text) = self
                                .get_usage_recursive(child, source, child_optional, true, None)
                                .await
                                && !child_usages.contains(&child_usage_text)
                            {
                                child_usages.push(child_usage_text);
                            }
                        }
                        if child_usages.len() == 1 {
                            let mut child_usage = child_usages
                                .into_iter()
                                .next()
                                .expect("Child usages length is 1, so next should exist");
                            if is_optional {
                                child_usage = format!(
                                    "{USAGE_OPTIONAL_OPEN}{child_usage}{USAGE_OPTIONAL_CLOSE}"
                                );
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
        })
    }
}
