use crate::command::{
    CommandExecutor, CommandSender,
    args::{
        ConsumedArgs, FindArg, players::PlayersArgumentConsumer, sound::SoundArgumentConsumer,
        sound_category::SoundCategoryArgumentConsumer,
    },
    dispatcher::CommandError,
    tree::{CommandTree, builder::argument},
};
use async_trait::async_trait;
use pumpkin_protocol::codec::identifier::Identifier;
use pumpkin_util::text::TextComponent;

const NAMES: [&str; 1] = ["stopsound"];
const DESCRIPTION: &str = "Stops a currently playing sound.";

const ARG_TARGETS: &str = "targets";
const ARG_SOURCE: &str = "source";
const ARG_SOUND: &str = "sound";

pub struct Executor;

#[async_trait]
impl CommandExecutor for Executor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        _server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let targets = PlayersArgumentConsumer::find_arg(args, ARG_TARGETS)?;

        let category = SoundCategoryArgumentConsumer::find_arg(args, ARG_SOURCE);
        let sound = SoundArgumentConsumer::find_arg(args, ARG_SOUND);

        for target in targets {
            target
                .stop_sound(
                    sound
                        .as_ref()
                        .cloned()
                        .map(|s| Identifier::vanilla(s.to_name()))
                        .ok(),
                    category.as_ref().map(|s| **s).ok(),
                )
                .await;
        }
        let text = match (category, sound) {
            (Ok(c), Ok(s)) => TextComponent::translate(
                "commands.stopsound.success.source.sound",
                [
                    TextComponent::text(s.to_name()),
                    TextComponent::text(c.to_name()),
                ],
            ),
            (Ok(c), Err(_)) => TextComponent::translate(
                "commands.stopsound.success.source.any",
                [TextComponent::text(c.to_name())],
            ),
            (Err(_), Ok(s)) => TextComponent::translate(
                "commands.stopsound.success.sourceless.sound",
                [TextComponent::text(s.to_name())],
            ),
            (Err(_), Err(_)) => {
                TextComponent::translate("commands.stopsound.success.sourceless.any", [])
            }
        };
        sender.send_message(text).await;

        Ok(())
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).then(
        argument(ARG_TARGETS, PlayersArgumentConsumer)
            .execute(Executor)
            .then(
                argument(ARG_SOURCE, SoundCategoryArgumentConsumer)
                    .execute(Executor)
                    .then(argument(ARG_SOUND, SoundArgumentConsumer).execute(Executor)),
            ),
    )
}
