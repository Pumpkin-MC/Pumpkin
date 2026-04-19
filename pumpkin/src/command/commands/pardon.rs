use crate::command::{
    CommandError, CommandExecutor, CommandResult, CommandSender,
    args::{
        Arg, ConsumedArgs,
        gameprofile::{GameProfileSuggestionMode, GameProfilesArgumentConsumer},
    },
    tree::{CommandTree, builder::argument},
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

            let mut successes = 0;
            for target in targets {
                if server
                    .banned_player_storage
                    .is_banned(target.id)
                    .await
                    .unwrap_or(false)
                {
                    if let Err(e) = server.banned_player_storage.unban(target.id).await {
                        tracing::error!("Failed to unban {}: {e}", target.name);
                        continue;
                    }
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
