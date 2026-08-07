use crate::{
    command::{
        CommandError, CommandExecutor, CommandResult, CommandSender,
        args::{
            Arg, ConsumedArgs,
            gameprofile::{GameProfileSuggestionMode, GameProfilesArgumentConsumer},
        },
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
                        .send_message(TextComponent::translate_cross(
                            pumpkin_data::translation::java::COMMANDS_PARDON_SUCCESS,
                            pumpkin_data::translation::bedrock::COMMANDS_UNBAN_SUCCESS,
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
                let err_target = targets
                    .first()
                    .map_or_else(String::new, |first_target| first_target.name.clone());
                Err(CommandError::CommandFailed(TextComponent::translate_cross(
                    pumpkin_data::translation::java::COMMANDS_PARDON_FAILED,
                    pumpkin_data::translation::bedrock::COMMANDS_UNBAN_FAILED,
                    [TextComponent::text(err_target)],
                )))
            }
        })
    }
}

#[allow(clippy::too_many_lines)]
pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).then(
        argument(
            ARG_TARGET,
            GameProfilesArgumentConsumer::new(GameProfileSuggestionMode::BannedNames, false),
        )
        .execute(Executor),
    )
}
