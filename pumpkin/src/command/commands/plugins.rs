use pumpkin_i18n::PUMPKIN_NAMESPACE;
use pumpkin_util::text::{TextComponent, color::NamedColor, hover::HoverEvent};

use crate::command::{
    CommandExecutor, CommandResult, CommandSender, args::ConsumedArgs, tree::CommandTree,
};

const NAMES: [&str; 2] = ["pl", "plugins"];

const DESCRIPTION: &str = "commands.plugins.description";

struct Executor;

impl CommandExecutor for Executor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let plugins = server.plugin_manager.active_plugins().await;
            let locale = sender.get_locale();

            let mut message = if plugins.is_empty() {
                TextComponent::custom(PUMPKIN_NAMESPACE, "commands.plugins.no_plugins", locale, [])
            } else if plugins.len() == 1 {
                TextComponent::custom(PUMPKIN_NAMESPACE, "commands.plugins.one_plugin", locale, [])
            } else {
                TextComponent::custom(
                    PUMPKIN_NAMESPACE,
                    "commands.plugins.multiple_plugins",
                    locale,
                    [TextComponent::text(plugins.len().to_string())],
                )
            };

            for (i, metadata) in plugins.clone().into_iter().enumerate() {
                let mut component = TextComponent::text(metadata.name.clone())
                    .color_named(NamedColor::Green)
                    .hover_event(HoverEvent::show_text(TextComponent::custom(
                        PUMPKIN_NAMESPACE,
                        "commands.plugins.hover_text",
                        locale,
                        [
                            TextComponent::text(metadata.version.clone()),
                            TextComponent::text(metadata.authors.join(", ")),
                            TextComponent::text(metadata.description),
                        ],
                    )));
                if i != plugins.len() - 1 {
                    component = component.add_child(TextComponent::custom(
                        PUMPKIN_NAMESPACE,
                        "commands.plugins.list.separator",
                        locale,
                        [],
                    ));
                }
                message = message.add_child(component);
            }

            sender.send_message(message).await;

            Ok(plugins.len() as i32)
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).execute(Executor)
}
