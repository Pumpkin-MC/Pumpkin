use crate::command::CommandResult;
use crate::command::{
    CommandError, CommandExecutor, CommandSender,
    args::{
        Arg, ConsumedArgs,
        gameprofile::{GameProfileSuggestionMode, GameProfilesArgumentConsumer},
    },
    tree::CommandTree,
    tree::builder::argument,
};
use CommandError::InvalidConsumption;
use pumpkin_util::text::TextComponent;

const NAMES: [&str; 1] = ["deop"];
const DESCRIPTION: &str = "Revokes operator status from a player.";
const ARG_TARGETS: &str = "targets";

struct Executor;

impl CommandExecutor for Executor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::GameProfiles(targets)) = args.get(&ARG_TARGETS) else {
                return Err(InvalidConsumption(Some(ARG_TARGETS.into())));
            };

            let mut succeeded_deops: i32 = 0;
            for profile in targets {
                if !server.op_storage.is_op(profile.id).await.unwrap_or(false) {
                    continue;
                }
                if let Err(e) = server.op_storage.deop(profile.id).await {
                    tracing::error!("Failed to deop {}: {e}", profile.name);
                    continue;
                }
                succeeded_deops += 1;

                if let Some(player) = server.get_player_by_uuid(profile.id) {
                    let command_dispatcher = server.command_dispatcher.read().await;
                    player
                        .set_permission_lvl(
                            server,
                            pumpkin_util::PermissionLvl::Zero,
                            &command_dispatcher,
                        )
                        .await;
                }

                let msg = TextComponent::translate(
                    "commands.deop.success",
                    [TextComponent::text(profile.name.clone())],
                );
                sender.send_message(msg).await;
            }

            if succeeded_deops == 0 {
                Err(CommandError::CommandFailed(TextComponent::translate(
                    "commands.deop.failed",
                    [],
                )))
            } else {
                Ok(succeeded_deops)
            }
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).then(
        argument(
            ARG_TARGETS,
            GameProfilesArgumentConsumer::new(GameProfileSuggestionMode::OpNames, false),
        )
        .execute(Executor),
    )
}
