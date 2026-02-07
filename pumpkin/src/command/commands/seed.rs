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
                CommandSender::Player(player) => {
                    player.living_entity.entity.world.load().level.seed.0
                }
                // TODO: Maybe ask player for world, or get the current world
                _ => match server.worlds.load().first() {
                    Some(world) => world.level.seed.0,
                    None => {
                        return Err(CommandError::CommandFailed(TextComponent::text(
                            "Unable to get Seed",
                        )));
                    }
                },
            } as i64;
            let seed_string = seed.to_string();

            sender
                .send_message(TextComponent::translate(
                    "commands.seed.success",
                    [TextComponent::text(seed_string.clone())
                        .hover_event(HoverEvent::show_text(TextComponent::translate(
                            Cow::from("chat.copy.click"),
                            [],
                        )))
                        .click_event(ClickEvent::CopyToClipboard {
                            value: Cow::from(seed_string),
                        })
                        .color_named(NamedColor::Green)],
                ))
                .await;

            Ok(seed.clamp(i32::MIN as i64, i32::MAX as i64) as i32)
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).execute(Executor)
}
