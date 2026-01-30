use crate::command::CommandResult;
use crate::command::args::difficulty::DifficultyArgumentConsumer;
use crate::command::args::{Arg, GetCloned};
use crate::command::dispatcher::CommandError::{self, InvalidConsumption};
use crate::command::tree::builder::argument;
use crate::command::{CommandExecutor, CommandSender, args::ConsumedArgs, tree::CommandTree};

use pumpkin_util::text::TextComponent;

const NAMES: [&str; 1] = ["difficulty"];

const DESCRIPTION: &str = "Change the difficulty of the world.";

pub const ARG_DIFFICULTY: &str = "difficulty";
struct DifficultyExecutor;

impl CommandExecutor for DifficultyExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Difficulty(difficulty)) = args.get_cloned(&ARG_DIFFICULTY) else {
                return Err(InvalidConsumption(Some(ARG_DIFFICULTY.into())));
            };

            let difficulty_string = format!("{difficulty:?}").to_lowercase();
            let translation_key = format!("options.difficulty.{difficulty_string}");

            {
                let level_info = server.level_info.load();

                if level_info.difficulty == difficulty {
                    return Err(
                        CommandError::CommandFailed(
                            TextComponent::translate(
                                "commands.difficulty.failure",
                                [TextComponent::translate(translation_key, [])],
                            )
                        )
                    );
                }
            }

            server.set_difficulty(difficulty, Some(true)).await;

            sender
                .send_message(TextComponent::translate(
                    "commands.difficulty.success",
                    [TextComponent::translate(translation_key, [])],
                ))
                .await;

            Ok(0)
        })
    }
}

#[must_use]
pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(argument(ARG_DIFFICULTY, DifficultyArgumentConsumer).execute(DifficultyExecutor))
}
