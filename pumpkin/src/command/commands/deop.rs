use crate::command::CommandResult;
use crate::{
    command::{
        CommandError, CommandExecutor, CommandSender,
        args::{
            Arg, ConsumedArgs,
            gameprofile::{GameProfileSuggestionMode, GameProfilesArgumentConsumer},
        },
        tree::CommandTree,
        tree::builder::argument,
    },
    data::SaveJSONConfiguration,
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
            let mut config = server.data.operator_config.write().await;

            let Some(Arg::GameProfiles(targets)) = args.get(&ARG_TARGETS) else {
                return Err(InvalidConsumption(Some(ARG_TARGETS.into())));
            };

            let mut succeeded_deops: i32 = 0;
            for profile in targets {
                if let Some(op_index) = config.ops.iter().position(|o| o.uuid == profile.id) {
                    config.ops.remove(op_index);
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
            }

            if succeeded_deops > 0 {
                config.save();
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
