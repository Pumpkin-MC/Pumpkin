use crate::command::{
    CommandExecutor, CommandResult, CommandSender,
    args::{
        ConsumedArgs, FindArg, players::PlayersArgumentConsumer, sound::SoundArgumentConsumer,
        sound_category::SoundCategoryArgumentConsumer,
    },
    tree::{CommandTree, builder::argument},
};
use pumpkin_data::translation;
use pumpkin_util::text::TextComponent;

const NAMES: [&str; 1] = ["stopsound"];
const DESCRIPTION: &str = "Stops a currently playing sound.";

const ARG_TARGETS: &str = "targets";
const ARG_SOURCE: &str = "source";
const ARG_SOUND: &str = "sound";

pub struct Executor;

impl CommandExecutor for Executor {
    fn execute(
        &self,
        sender: &CommandSender,
        _server: &crate::server::Server,
        args: &ConsumedArgs,
    ) -> CommandResult {
        let targets = PlayersArgumentConsumer::find_arg(args, ARG_TARGETS)?;

        let category = SoundCategoryArgumentConsumer::find_arg(args, ARG_SOURCE);
        let sound = SoundArgumentConsumer::find_arg(args, ARG_SOUND);

        let sound_name = sound
            .as_ref()
            .cloned()
            .map(|s| format!("minecraft:{}", s.to_name()))
            .ok();
        let cat = category.as_ref().map(|s| **s).ok();

        for target in targets {
            target.stop_sound(sound_name.clone(), cat);
        }

        let text = match (category, sound) {
            (Ok(c), Ok(s)) => TextComponent::translate_cross(
                translation::java::COMMANDS_STOPSOUND_SUCCESS_SOURCE_SOUND,
                translation::java::COMMANDS_STOPSOUND_SUCCESS_SOURCE_SOUND,
                [
                    TextComponent::text(s.to_name()),
                    TextComponent::text(c.to_name()),
                ],
            ),
            (Ok(c), Err(_)) => TextComponent::translate_cross(
                translation::java::COMMANDS_STOPSOUND_SUCCESS_SOURCE_ANY,
                translation::java::COMMANDS_STOPSOUND_SUCCESS_SOURCE_ANY,
                [TextComponent::text(c.to_name())],
            ),
            (Err(_), Ok(s)) => TextComponent::translate_cross(
                translation::java::COMMANDS_STOPSOUND_SUCCESS_SOURCELESS_SOUND,
                translation::java::COMMANDS_STOPSOUND_SUCCESS_SOURCELESS_SOUND,
                [TextComponent::text(s.to_name())],
            ),
            (Err(_), Err(_)) => TextComponent::translate_cross(
                translation::java::COMMANDS_STOPSOUND_SUCCESS_SOURCELESS_ANY,
                translation::java::COMMANDS_STOPSOUND_SUCCESS_SOURCELESS_ANY,
                [],
            ),
        };
        sender.send_message(text);

        Ok(targets.len() as i32)
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
