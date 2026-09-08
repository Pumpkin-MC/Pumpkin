use std::fmt::Write;
use std::path::PathBuf;

use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::color::NamedColor;
use pumpkin_util::text::hover::HoverEvent;

use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::core::string::StringArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::context::command_source::CommandSource;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::command::suggestion::provider::{SuggestionProvider, SuggestionProviderResult};
use crate::command::suggestion::suggestions::SuggestionsBuilder;
use crate::plugin::{PluginEntry, PluginStatus};

const DESCRIPTION: &str = "Manage server plugins.";
const PERMISSION: &str = "pumpkin:command.plugin";

/// Which plugins a `<plugin>` argument completes to.
enum Suggested {
    /// Everything the manager knows, by name
    Known,
    /// Only what is running, for `unload`
    Active,
    /// Only what is not running, for `load`
    Loadable,
}

struct PluginSuggestions(Suggested);

impl SuggestionProvider for PluginSuggestions {
    fn suggest(
        &self,
        context: &CommandContext,
        builder: SuggestionsBuilder,
    ) -> SuggestionProviderResult {
        let manager = &context.server().plugin_manager;
        let candidates = match self.0 {
            Suggested::Known => manager.plugin_names(),
            Suggested::Active => manager
                .active_plugins()
                .into_iter()
                .map(|metadata| metadata.name)
                .collect(),
            Suggested::Loadable => manager.inactive_names(),
        };

        // The argument is a quotable phrase, so anything with a space has to come back quoted
        builder
            .filter_and_suggest_iter(candidates.into_iter().map(|candidate| {
                if candidate.contains(' ') {
                    format!("\"{candidate}\"")
                } else {
                    candidate
                }
            }))
            .build()
    }
}

/// Green while running, yellow while starting, red for everything that is not.
pub(super) const fn status_color(status: &PluginStatus) -> NamedColor {
    match status {
        PluginStatus::Active => NamedColor::Green,
        PluginStatus::Loading => NamedColor::Yellow,
        _ => NamedColor::Red,
    }
}

/// plugin Tooltip -> tells about itself, plus why it is not running.
pub(super) fn hover_text(entry: &PluginEntry) -> String {
    let mut text = entry.metadata.as_ref().map_or_else(
        || format!("File: {}", entry.path.display()),
        |metadata| {
            format!(
                "Version: {}\nAuthors: {}\nDescription: {}",
                metadata.version,
                metadata.authors.join(", "),
                metadata.description
            )
        },
    );

    if !entry.status.is_active() {
        let _ = write!(text, "\nStatus: {}", entry.status);
    }

    text
}

/// Sends the outcome of a plugin operation, green on success and red on failure.
fn send_result(source: &CommandSource, result: Result<(), String>, success: String) {
    match result {
        Ok(()) => source.send_feedback(
            TextComponent::text(success).color_named(NamedColor::Green),
            true,
        ),
        Err(message) => source.send_feedback(
            TextComponent::text(message).color_named(NamedColor::Red),
            false,
        ),
    }
}

struct ListExecutor;

impl CommandExecutor for ListExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let server_arc = context.server().clone();
        let source_clone = context.source.clone();
        let server_clone = server_arc.clone();

        server_arc.spawn_task(async move {
            let entries = server_clone.plugin_manager.plugin_entries().await;
            let active = entries.iter().filter(|e| e.status.is_active()).count();

            let mut message = TextComponent::text(format!("Plugins ({active}/{}):", entries.len()))
                .color_named(NamedColor::Gold)
                .add_child(TextComponent::text("\n"));

            for (i, entry) in entries.iter().enumerate() {
                let name = entry.name();
                let line = entry.metadata.as_ref().map_or_else(
                    || format!("- {name}"),
                    |metadata| {
                        let version = metadata
                            .version
                            .strip_prefix('v')
                            .unwrap_or(&metadata.version);
                        format!("- {name} (v{version})")
                    },
                );
                let line = if i == entries.len() - 1 {
                    line
                } else {
                    format!("{line}\n")
                };

                let mut plugin_component = TextComponent::text(line)
                    .color_named(status_color(&entry.status))
                    .hover_event(HoverEvent::show_text(TextComponent::text(hover_text(
                        entry,
                    ))));

                if let Some(metadata) = &entry.metadata
                    && !metadata.permissions.is_empty()
                {
                    plugin_component = plugin_component.add_child(
                        TextComponent::text(format!(
                            " (Permissions: {})",
                            metadata.permissions.join(", ")
                        ))
                            .color_named(NamedColor::Gray),
                    );
                }

                message = message.add_child(plugin_component);
            }

            source_clone.send_feedback(message, false);
        });

        Ok(1)
    }
}

struct LoadExecutor;

impl CommandExecutor for LoadExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let argument = StringArgumentType::get(context, "plugin")?.to_string();
        let server_arc = context.server().clone();

        let source_clone = context.source.clone();
        let server_clone = server_arc.clone();
        server_arc.spawn_task(async move {
            // Takes a name like the other subcommands, a path is only needed for a file the
            // manager has never seen
            let entry = server_clone.plugin_manager.plugin_entry(&argument).await;
            let (name, path) = entry.as_ref().map_or_else(
                || (argument.clone(), PathBuf::from(&argument)),
                |entry| (entry.name().into_owned(), entry.path.clone()),
            );

            if entry.is_some_and(|entry| entry.status.is_active()) {
                source_clone.send_feedback(
                    TextComponent::text(format!("Plugin {name} is already loaded")),
                    false,
                );
                return;
            }

            let result = server_clone
                .plugin_manager
                .try_load_plugin(&server_clone, &path)
                .await
                .map_err(|e| format!("Failed to load plugin {name}: {e}"));

            send_result(
                &source_clone,
                result,
                format!("Plugin {name} loaded successfully"),
            );
        });

        Ok(1)
    }
}

struct UnloadExecutor;

impl CommandExecutor for UnloadExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let plugin_name = StringArgumentType::get(context, "plugin")?.to_string();
        let server_arc = context.server().clone();

        if !server_arc.plugin_manager.is_plugin_active(&plugin_name) {
            context.source.send_feedback(
                TextComponent::text(format!("Plugin {plugin_name} is not loaded")),
                false,
            );
            return Ok(1);
        }

        let source_clone = context.source.clone();
        let plugin_name_clone = plugin_name;
        let server_clone = server_arc.clone();
        server_arc.spawn_task(async move {
            let result = server_clone
                .plugin_manager
                .unload_plugin(&plugin_name_clone)
                .await
                .map_err(|e| format!("Failed to unload plugin {plugin_name_clone}: {e}"));

            send_result(
                &source_clone,
                result,
                format!("Plugin {plugin_name_clone} unloaded successfully"),
            );
        });

        Ok(1)
    }
}

struct ReloadExecutor;

impl CommandExecutor for ReloadExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let plugin_name = StringArgumentType::get(context, "plugin")?.to_string();
        let server_arc = context.server().clone();

        let source_clone = context.source.clone();
        let server_clone = server_arc.clone();
        server_arc.spawn_task(async move {
            // The manager has no reload: unload takes a name, load takes the file it came from
            let Some(entry) = server_clone.plugin_manager.plugin_entry(&plugin_name).await else {
                source_clone.send_feedback(
                    TextComponent::text(format!("Plugin {plugin_name} is not known"))
                        .color_named(NamedColor::Red),
                    false,
                );
                return;
            };

            // Loading it a second time would leave two copies registered
            if !entry.can_unload {
                source_clone.send_feedback(
                    TextComponent::text(format!(
                        "Plugin {plugin_name} cannot be unloaded at runtime, restart the server"
                    ))
                    .color_named(NamedColor::Red),
                    false,
                );
                return;
            }

            if server_clone.plugin_manager.is_plugin_loaded(&plugin_name)
                && let Err(e) = server_clone
                    .plugin_manager
                    .unload_plugin(&plugin_name)
                    .await
            {
                source_clone.send_feedback(
                    TextComponent::text(format!("Failed to unload plugin {plugin_name}: {e}"))
                        .color_named(NamedColor::Red),
                    false,
                );
                return;
            }

            let result = server_clone
                .plugin_manager
                .try_load_plugin(&server_clone, &entry.path)
                .await
                // The plugin is unloaded at this point, so say so
                .map_err(|e| {
                    format!("Plugin {plugin_name} was unloaded but could not be loaded again: {e}")
                });

            send_result(
                &source_clone,
                result,
                format!("Plugin {plugin_name} reloaded successfully"),
            );
        });

        Ok(1)
    }
}

/// One gray detail line appended to a `/plugin info` message.
fn info_line(message: TextComponent, text: String) -> TextComponent {
    message
        .add_child(TextComponent::text("\n"))
        .add_child(TextComponent::text(text).color_named(NamedColor::Gray))
}

/// Full `/plugin info` message for one entry: name, status, path, and metadata when there is any.
fn info_message(entry: &PluginEntry) -> TextComponent {
    let mut message = TextComponent::text(entry.name().into_owned())
        .color_named(NamedColor::Gold)
        .add_child(TextComponent::text("\n"))
        .add_child(
            TextComponent::text(format!("Status: {}", entry.status))
                .color_named(status_color(&entry.status)),
        );
    message = info_line(message, format!("Path: {}", entry.path.display()));

    let Some(metadata) = &entry.metadata else {
        return message;
    };

    message = info_line(message, format!("Version: {}", metadata.version));
    message = info_line(message, format!("Authors: {}", metadata.authors.join(", ")));
    message = info_line(message, format!("Description: {}", metadata.description));

    if !metadata.dependencies.is_empty() {
        message = info_line(
            message,
            format!("Dependencies: {}", metadata.dependencies.join(", ")),
        );
    }
    if !metadata.permissions.is_empty() {
        message = info_line(
            message,
            format!("Permissions: {}", metadata.permissions.join(", ")),
        );
    }

    message
}

struct InfoExecutor;

impl CommandExecutor for InfoExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let plugin_name = StringArgumentType::get(context, "plugin")?.to_string();
        let server_arc = context.server().clone();

        let source_clone = context.source.clone();
        let server_clone = server_arc.clone();
        server_arc.spawn_task(async move {
            let Some(entry) = server_clone.plugin_manager.plugin_entry(&plugin_name).await else {
                source_clone.send_feedback(
                    TextComponent::text(format!("Plugin {plugin_name} is not known"))
                        .color_named(NamedColor::Red),
                    false,
                );
                return;
            };

            source_clone.send_feedback(info_message(&entry), false);
        });

        Ok(1)
    }
}

struct ReloadAllExecutor;

impl CommandExecutor for ReloadAllExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let server_arc = context.server().clone();

        let source_clone = context.source.clone();
        let server_clone = server_arc.clone();
        server_arc.spawn_task(async move {
            if let Err(e) = server_clone
                .plugin_manager
                .reload_all_plugins(&server_clone)
                .await
            {
                source_clone.send_feedback(
                    TextComponent::text(format!("Failed to reload plugins: {e}"))
                        .color_named(NamedColor::Red),
                    false,
                );
                return;
            }

            let entries = server_clone.plugin_manager.plugin_entries().await;
            let active = entries.iter().filter(|e| e.status.is_active()).count();
            let inactive = entries.len() - active;

            // Which ones are not running is what `/plugins` is for
            let (color, note) = if inactive == 0 {
                (NamedColor::Green, String::new())
            } else {
                (NamedColor::Yellow, format!(", {inactive} not running"))
            };

            source_clone.send_feedback(
                TextComponent::text(format!("Reloaded plugins: {active} active{note}"))
                    .color_named(color),
                true,
            );
        });

        Ok(1)
    }
}

struct HotReloadExecutor(bool);

impl CommandExecutor for HotReloadExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let enabled = self.0;
        let server_arc = context.server().clone();
        let source_clone = context.source.clone();
        let server_clone = server_arc.clone();

        if enabled {
            server_arc.spawn_task(async move {
                if let Err(e) = server_clone.plugin_manager.start_watcher(&server_clone).await {
                    source_clone.send_feedback(
                        TextComponent::text(format!("Failed to start plugin watcher: {e}")),
                        false,
                    );
                    return;
                }

                source_clone.send_feedback(
                    TextComponent::text("Hot reloading has been enabled.")
                        .color_named(NamedColor::Green),
                    true,
                );
                source_clone.send_feedback(
                    TextComponent::text(
                        "WARNING: Hot reloading can impact performance and should only be enabled during plugin development.",
                    )
                    .color_named(NamedColor::Red),
                    false,
                );
            });
        } else {
            server_arc.spawn_task(async move {
                server_clone.plugin_manager.stop_watcher().await;
                source_clone.send_feedback(
                    TextComponent::text("Hot reloading has been disabled.")
                        .color_named(NamedColor::Yellow),
                    true,
                );
            });
        }

        Ok(1)
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Three),
    ));

    dispatcher.register(
        command("plugin", DESCRIPTION)
            .requires(PERMISSION)
            .then(literal("list").executes(ListExecutor))
            // Quotable rather than a single word: `load` takes a path, and names can contain spaces
            .then(
                literal("load").then(
                    argument("plugin", StringArgumentType::QuotablePhrase)
                        .suggests(PluginSuggestions(Suggested::Loadable))
                        .executes(LoadExecutor),
                ),
            )
            .then(
                literal("unload").then(
                    argument("plugin", StringArgumentType::QuotablePhrase)
                        .suggests(PluginSuggestions(Suggested::Active))
                        .executes(UnloadExecutor),
                ),
            )
            .then(
                literal("reload").then(
                    argument("plugin", StringArgumentType::QuotablePhrase)
                        .suggests(PluginSuggestions(Suggested::Known))
                        .executes(ReloadExecutor),
                ),
            )
            .then(
                literal("info").then(
                    argument("plugin", StringArgumentType::QuotablePhrase)
                        .suggests(PluginSuggestions(Suggested::Known))
                        .executes(InfoExecutor),
                ),
            )
            .then(literal("reloadall").executes(ReloadAllExecutor))
            .then(
                literal("hotreload")
                    .then(literal("enable").executes(HotReloadExecutor(true)))
                    .then(literal("disable").executes(HotReloadExecutor(false))),
            ),
    );
}
