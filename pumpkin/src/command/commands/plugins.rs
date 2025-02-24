use async_trait::async_trait;
use pumpkin_util::text::{TextComponent, color::NamedColor, hover::HoverEvent};

use crate::{
    PLUGIN_MANAGER,
    command::{
        CommandError, CommandExecutor, CommandSender, args::ConsumedArgs, tree::CommandTree,
    },
};

const NAMES: [&str; 1] = ["plugins"];

const DESCRIPTION: &str = "List all available plugins.";

struct Executor;

#[async_trait]
impl CommandExecutor for Executor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &crate::server::Server,
        _args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let plugin_manager = PLUGIN_MANAGER.lock().await;
        let plugins = plugin_manager.list_plugins();

        let message_text = if plugins.is_empty() {
            "There are no loaded plugins.".to_string()
        } else if plugins.len() == 1 {
            "There is 1 plugin loaded:\n".to_string()
        } else {
            format!("There are {} plugins loaded:\n", plugins.len())
        };
        let mut message = TextComponent::text(message_text);

        for (i, (metadata, loaded)) in plugins.clone().into_iter().enumerate() {
            let fmt = if i == plugins.len() - 1 {
                metadata.name.to_string()
            } else {
                format!("{}, ", metadata.name)
            };
            let hover_text = format!(
                "Version: {}\nAuthors: {}\nDescription: {}",
                metadata.version, metadata.authors, metadata.description
            );
            let component = if *loaded {
                TextComponent::text(fmt)
                    .color_named(NamedColor::Green)
                    .hover_event(HoverEvent::show_text(TextComponent::text(hover_text)))
            } else {
                TextComponent::text(fmt)
                    .color_named(NamedColor::Red)
                    .hover_event(HoverEvent::show_text(TextComponent::text(hover_text)))
            };
            message = message.add_child(component);
        }

        sender.send_message(message).await;

        Ok(())
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).execute(Executor)
}
