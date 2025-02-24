use crate::command::{
    CommandError, CommandExecutor, CommandSender, args::ConsumedArgs, tree::CommandTree,
};
use async_trait::async_trait;
use pumpkin_util::text::click::ClickEvent;
use pumpkin_util::text::hover::HoverEvent;
use pumpkin_util::text::{TextComponent, color::NamedColor};
use std::borrow::Cow;

const NAMES: [&str; 1] = ["seed"];

const DESCRIPTION: &str = "Displays the world seed.";

struct Executor;

#[async_trait]
impl CommandExecutor for Executor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &crate::server::Server,
        _args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let seed = match sender {
            CommandSender::Player(player) => {
                player.living_entity.entity.world.read().await.level.seed.0
            }
            // TODO: Maybe ask player for world, or get the current world
            _ => match server.worlds.read().await.first() {
                Some(world) => world.level.seed.0,
                None => {
                    return Err(CommandError::GeneralCommandIssue(
                        "Unable to get Seed".to_string(),
                    ));
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
                    .click_event(ClickEvent::CopyToClipboard(Cow::from(seed)))
                    .color_named(NamedColor::Green)],
            ))
            .await;
        Ok(())
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).execute(Executor)
}
