use async_trait::async_trait;
use pumpkin_data::sound::SoundCategory;
use rand::{thread_rng, Rng};

use crate::command::{
    args::{sound::SoundArgumentConsumer, ConsumedArgs, FindArg},
    tree::CommandTree,
    tree_builder::argument,
    CommandError, CommandExecutor, CommandSender,
};
const NAMES: [&str; 1] = ["playsound"];

const DESCRIPTION: &str = "Plays a sound at a position.";

const ARG_SOUND: &str = "sound";

struct SoundExecutor;

#[async_trait]
impl CommandExecutor for SoundExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let sound = SoundArgumentConsumer::find_arg(args, ARG_SOUND)?;

        if let Some(player) = sender.as_player() {
            let seed = thread_rng().gen::<f64>();
            player
                .play_sound(
                    sound as u16,
                    SoundCategory::Master,
                    &player.living_entity.entity.pos.load(),
                    1.0,
                    1.0,
                    seed,
                )
                .await;
        }
        Ok(())
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(argument(ARG_SOUND, SoundArgumentConsumer).execute(SoundExecutor))
}
