use crate::command::CommandResult;
use crate::command::{
    CommandError, CommandExecutor, CommandSender, args::ConsumedArgs, tree::CommandTree,
};
use pumpkin_util::text::click::ClickEvent;
use pumpkin_util::text::hover::HoverEvent;
use pumpkin_util::text::{TextComponent, color::NamedColor};
use std::borrow::Cow;

const NAMES: [&str; 1] = ["seed"];

const DESCRIPTION: &str = "Displays the world seed.";

struct Executor;

impl CommandExecutor for Executor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let seed = match sender {
                CommandSender::Player(player) => player.living_entity.entity.world.level.seed.0,
                // TODO: Maybe ask player for world, or get the current world
                _ => match server.worlds.read().await.first() {
                    Some(world) => world.level.seed.0,
                    None => {
                        return Err(CommandError::CommandFailed(TextComponent::text(
                            "Unable to get Seed",
                        )));
                    }
                },
            };
            let seed = (seed as i64).to_string();

            sender
                .send_message(TextComponent::translate(
                    "commands.seed.success",
                    [TextComponent::text(seed.clone())
                        .hover_event(HoverEvent::show_text(TextComponent::translate(
                            Cow::from("chat.copy.click"),
                            [],
                        )))
                        .click_event(ClickEvent::CopyToClipboard {
                            value: Cow::from(seed),
                        })
                        .color_named(NamedColor::Green)],
                ))
                .await;
            Ok(())
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).execute(Executor)
}
