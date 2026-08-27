use pumpkin_util::text::hover::HoverEvent;
use pumpkin_util::text::{TextComponent, color::NamedColor};

use crate::command::args::ConsumedArgs;
use crate::command::dispatcher::CommandError;
use crate::command::tree::CommandTree;
use crate::command::{CommandExecutor, CommandResult, CommandSender};

const NAMES: [&str; 1] = ["plugins"];

const DESCRIPTION: &str = "Lists all plugins loaded on the server.";

struct PluginsExecutor;

impl CommandExecutor for PluginsExecutor {
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

        let message_text = if plugins.is_empty() {
            TextComponent::text("No plugins are loaded on the server.").color_named(NamedColor::Red)
        } else {
            let mut base_message = TextComponent::text(format!("Plugins ({}): ", plugins.len()))
                .color_named(NamedColor::White);

            for (i, plugin) in plugins.iter().enumerate() {
                let name = &plugin.name;
                let version = plugin.version.strip_prefix('v').unwrap_or(&plugin.version);
                let author_text = plugin.authors.join(", ");
                let description = &plugin.description;

                let hover_text = format!(
                    "Version: {version}\nAuthors: {author_text}\nDescription: {description}"
                );

                let plugin_component = TextComponent::text(name.clone())
                    .color_named(NamedColor::Green)
                    .hover_event(HoverEvent::show_text(TextComponent::text(hover_text)));

                if i > 0 {
                    base_message = base_message
                        .add_child(TextComponent::text(", ").color_named(NamedColor::White));
                } else {
                    base_message = base_message
                        .add_child(TextComponent::text(" ").color_named(NamedColor::White));
                }
                base_message = base_message.add_child(plugin_component);
            }

            base_message
        };

        sender.send_message(message_text);

        Ok(1)
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).execute(PluginsExecutor)
}
