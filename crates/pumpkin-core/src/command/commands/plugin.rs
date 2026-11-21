use std::path::Path;

use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::color::NamedColor;
use pumpkin_util::text::hover::HoverEvent;

use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::core::string::StringArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};

const DESCRIPTION: &str = "Manage server plugins.";
const PERMISSION: &str = "pumpkin:command.plugin";

struct ListExecutor;

impl CommandExecutor for ListExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let server_arc = context.server().clone();
        let plugins = server_arc.plugin_manager.active_plugins();
        let loaded_plugins = server_arc.plugin_manager.loaded_plugins();

        let mut message = TextComponent::text(format!("Plugins ({}):", loaded_plugins.len()))
            .color_named(NamedColor::Gold)
            .add_child(TextComponent::text("\n"));

        for (i, plugin) in plugins.iter().enumerate() {
            let metadata = plugin;
            let version = metadata
                .version
                .strip_prefix('v')
                .unwrap_or(&metadata.version);
            let line = if i == plugins.len() - 1 {
                format!("- {} (v{version})", metadata.name)
            } else {
                format!("- {} (v{version})\n", metadata.name)
            };
            let hover_text = format!(
                "Version: {}\nAuthors: {}\nDescription: {}",
                metadata.version,
                metadata.authors.join(", "),
                metadata.description
            );
            let mut plugin_component = TextComponent::text(line)
                .color_named(NamedColor::Green)
                .hover_event(HoverEvent::show_text(TextComponent::text(hover_text)));

            if !metadata.permissions.is_empty() {
                plugin_component = plugin_component.add_child(
                    TextComponent::text(format!(" (Permissions: {:?})", metadata.permissions))
                        .color_named(NamedColor::Gray),
                );
            }

            message = message.add_child(plugin_component);
        }

        context.source.send_feedback(message, false);

        Ok(1)
    }
}

struct LoadExecutor;

impl CommandExecutor for LoadExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let plugin_name = StringArgumentType::get(context, "plugin")?.to_string();
        let server_arc = context.server().clone();

        if server_arc.plugin_manager.is_plugin_active(&plugin_name) {
            context.source.send_feedback(
                TextComponent::text(format!("Plugin {plugin_name} is already loaded")),
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
                .try_load_plugin(&server_clone, Path::new(&plugin_name_clone))
                .await;

            match result {
                Ok(()) => {
                    source_clone.send_feedback(
                        TextComponent::text(format!(
                            "Plugin {plugin_name_clone} loaded successfully"
                        ))
                        .color_named(NamedColor::Green),
                        true,
                    );
                }
                Err(e) => {
                    source_clone.send_feedback(
                        TextComponent::text(format!(
                            "Failed to load plugin {plugin_name_clone}: {e}"
                        )),
                        false,
                    );
                }
            }
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
                .await;

            match result {
                Ok(()) => {
                    source_clone.send_feedback(
                        TextComponent::text(format!(
                            "Plugin {plugin_name_clone} unloaded successfully"
                        ))
                        .color_named(NamedColor::Green),
                        true,
                    );
                }
                Err(e) => {
                    source_clone.send_feedback(
                        TextComponent::text(format!(
                            "Failed to unload plugin {plugin_name_clone}: {e}"
                        )),
                        false,
                    );
                }
            }
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
            .then(
                literal("load").then(
                    argument("plugin", StringArgumentType::SingleWord).executes(LoadExecutor),
                ),
            )
            .then(
                literal("unload").then(
                    argument("plugin", StringArgumentType::SingleWord).executes(UnloadExecutor),
                ),
            )
            .then(
                literal("hotreload")
                    .then(literal("enable").executes(HotReloadExecutor(true)))
                    .then(literal("disable").executes(HotReloadExecutor(false))),
            ),
    );
}
