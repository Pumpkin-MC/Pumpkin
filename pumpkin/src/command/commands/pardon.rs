use crate::{
    command::{
        CommandError, CommandExecutor, CommandResult, CommandSender,
        args::{Arg, ConsumedArgs, gameprofile::GameProfilesArgumentConsumer},
        tree::{CommandTree, builder::argument},
    },
    data::SaveJSONConfiguration,
};
use CommandError::InvalidConsumption;
use pumpkin_util::text::TextComponent;

const NAMES: [&str; 1] = ["pardon"];
const DESCRIPTION: &str = "unbans a player";

const ARG_TARGET: &str = "targets";

struct Executor;

impl CommandExecutor for Executor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::GameProfiles(targets)) = args.get(&ARG_TARGET) else {
                return Err(InvalidConsumption(Some(ARG_TARGET.into())));
            };

            let mut lock = server.data.banned_player_list.write().await;
            let mut successes = 0;

            for target in targets {
                let idx = lock
                    .banned_players
                    .iter()
                    .position(|entry| entry.uuid == target.id);

                if let Some(idx) = idx {
                    lock.banned_players.remove(idx);
                    sender
                        .send_message(TextComponent::translate(
                            "commands.pardon.success",
                            [TextComponent::text(target.name.clone())],
                        ))
                        .await;
                    successes += 1;
                }
            }

            if successes > 0 {
                lock.save();
                Ok(successes)
            } else {
                Err(CommandError::CommandFailed(TextComponent::translate(
                    "commands.pardon.failed",
                    [],
                )))
            }
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(argument(ARG_TARGET, GameProfilesArgumentConsumer).execute(Executor))
}
