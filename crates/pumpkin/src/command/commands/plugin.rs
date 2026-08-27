use std::path::Path;

use pumpkin_util::text::hover::HoverEvent;
use pumpkin_util::text::{TextComponent, color::NamedColor};

use crate::command::args::simple::SimpleArgConsumer;
use crate::command::args::{Arg, ConsumedArgs};
use crate::command::dispatcher::CommandError::{self, InvalidConsumption};
use crate::command::tree::CommandTree;
use crate::command::tree::builder::{argument, literal};
use crate::command::{CommandExecutor, CommandResult, CommandSender};

const NAMES: [&str; 1] = ["plugin"];

const DESCRIPTION: &str = "Manage server plugins.";

const PLUGIN_NAME: &str = "plugin";

struct ListExecutor;

impl CommandExecutor for ListExecutor {
    fn execute(
        &self,
        sender: &CommandSender,
        server: &crate::server::Server,
        _args: &ConsumedArgs,
    ) -> CommandResult {
        let Some(server_arc) = sender
            .world_or_first(server)
            .and_then(|w| w.server.upgrade())
        else {
            return Err(CommandError::CommandFailed(TextComponent::text(
                "Failed to get server instance",
            )));
        };

        let plugins = futures::executor::block_on(server_arc.plugin_manager.active_plugins());
        let loaded_plugins =
            futures::executor::block_on(server_arc.plugin_manager.loaded_plugins());

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
            let component = TextComponent::text(line)
                .color_named(NamedColor::Green)
                .hover_event(HoverEvent::show_text(TextComponent::text(hover_text)));

            message = message.add_child(component);
        }

        sender.send_message(message);

        Ok(1)
    }
}

struct LoadExecutor;

impl CommandExecutor for LoadExecutor {
    fn execute(
        &self,
        sender: &CommandSender,
        server: &crate::server::Server,
        args: &ConsumedArgs,
    ) -> CommandResult {
        let Some(Arg::Simple(plugin_name)) = args.get(PLUGIN_NAME) else {
            return Err(InvalidConsumption(Some(PLUGIN_NAME.into())));
        };

        let Some(server_arc) = sender
            .world_or_first(server)
            .and_then(|w| w.server.upgrade())
        else {
            return Err(CommandError::CommandFailed(TextComponent::text(
                "Failed to get server instance",
            )));
        };

        let plugin_name = plugin_name.to_string();
        if futures::executor::block_on(server_arc.plugin_manager.is_plugin_active(&plugin_name)) {
            sender.send_message(TextComponent::text(format!(
                "Plugin {plugin_name} is already loaded"
            )));
            return Ok(1);
        }

        let result = futures::executor::block_on(
            server_arc
                .plugin_manager
                .try_load_plugin(&server_arc, Path::new(&plugin_name)),
        );

        match result {
            Ok(()) => {
                sender.send_message(
                    TextComponent::text(format!("Plugin {plugin_name} loaded successfully"))
                        .color_named(NamedColor::Green),
                );
            }
            Err(e) => {
                sender.send_message(TextComponent::text(format!(
                    "Failed to load plugin {plugin_name}: {e}"
                )));
            }
        }

        Ok(1)
    }
}

struct UnloadExecutor;

impl CommandExecutor for UnloadExecutor {
    fn execute(
        &self,
        sender: &CommandSender,
        server: &crate::server::Server,
        args: &ConsumedArgs,
    ) -> CommandResult {
        let Some(Arg::Simple(plugin_name)) = args.get(PLUGIN_NAME) else {
            return Err(InvalidConsumption(Some(PLUGIN_NAME.into())));
        };

        let Some(server_arc) = sender
            .world_or_first(server)
            .and_then(|w| w.server.upgrade())
        else {
            return Err(CommandError::CommandFailed(TextComponent::text(
                "Failed to get server instance",
            )));
        };

        let plugin_name = plugin_name.to_string();
        if !futures::executor::block_on(server_arc.plugin_manager.is_plugin_active(&plugin_name)) {
            sender.send_message(TextComponent::text(format!(
                "Plugin {plugin_name} is not loaded"
            )));
            return Ok(1);
        }

        let result =
            futures::executor::block_on(server_arc.plugin_manager.unload_plugin(&plugin_name));

        match result {
            Ok(()) => {
                sender.send_message(
                    TextComponent::text(format!("Plugin {plugin_name} unloaded successfully"))
                        .color_named(NamedColor::Green),
                );
            }
            Err(e) => {
                sender.send_message(TextComponent::text(format!(
                    "Failed to unload plugin {plugin_name}: {e}"
                )));
            }
        }

        Ok(1)
    }
}

struct HotReloadExecutor(bool);

impl CommandExecutor for HotReloadExecutor {
    fn execute(
        &self,
        sender: &CommandSender,
        server: &crate::server::Server,
        _args: &ConsumedArgs,
    ) -> CommandResult {
        let enabled = self.0;

        let Some(server_arc) = sender
            .world_or_first(server)
            .and_then(|w| w.server.upgrade())
        else {
            return Err(CommandError::CommandFailed(TextComponent::text(
                "Failed to get server instance",
            )));
        };

        if enabled {
            if let Err(e) =
                futures::executor::block_on(server_arc.plugin_manager.start_watcher(&server_arc))
            {
                sender.send_message(TextComponent::text(format!(
                    "Failed to start plugin watcher: {e}"
                )));
                return Ok(1);
            }

            sender.send_message(
                TextComponent::text("Hot reloading has been enabled.")
                    .color_named(NamedColor::Green),
            );
            sender.send_message(
                TextComponent::text(
                    "WARNING: Hot reloading can impact performance and should only be enabled during plugin development.",
                )
                .color_named(NamedColor::Red),
            );
        } else {
            futures::executor::block_on(server_arc.plugin_manager.stop_watcher());
            sender.send_message(
                TextComponent::text("Hot reloading has been disabled.")
                    .color_named(NamedColor::Yellow),
            );
        }

        Ok(1)
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(literal("list").execute(ListExecutor))
        .then(literal("load").then(argument(PLUGIN_NAME, SimpleArgConsumer).execute(LoadExecutor)))
        .then(
            literal("unload")
                .then(argument(PLUGIN_NAME, SimpleArgConsumer).execute(UnloadExecutor)),
        )
        .then(
            literal("hotreload")
                .then(literal("enable").execute(HotReloadExecutor(true)))
                .then(literal("disable").execute(HotReloadExecutor(false))),
        )
}
