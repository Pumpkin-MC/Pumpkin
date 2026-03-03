use crate::command::CommandResult;
use crate::{
    command::{
        CommandError, CommandExecutor, CommandSender,
        args::{Arg, ConsumedArgs, gameprofile::GameProfilesArgumentConsumer},
        tree::CommandTree,
        tree::builder::argument,
    },
    data::SaveJSONConfiguration,
};
use CommandError::InvalidConsumption;
use pumpkin_config::op::Op;
use pumpkin_util::text::TextComponent;

const NAMES: [&str; 1] = ["op"];
const DESCRIPTION: &str = "Grants operator status to a player.";
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

            let mut successes: i32 = 0;
            let new_level = server
                .basic_config
                .op_permission_level
                .min(sender.permission_lvl());

            for profile in targets {
                let maybe_existing_entry = config.ops.iter_mut().find(|o| o.uuid == profile.id);

                if let Some(op) = maybe_existing_entry {
                    if op.level == new_level {
                        continue;
                    }

                    op.level = new_level;
                    op.name.clone_from(&profile.name);
                } else {
                    let op_entry = Op::new(profile.id, profile.name.clone(), new_level, false);
                    config.ops.push(op_entry);
                }

                if let Some(player) = server.get_player_by_uuid(profile.id) {
                    let command_dispatcher = server.command_dispatcher.read().await;
                    player
                        .set_permission_lvl(server, new_level, &command_dispatcher)
                        .await;
                }

                sender
                    .send_message(TextComponent::translate(
                        "commands.op.success",
                        [TextComponent::text(profile.name.clone())],
                    ))
                    .await;

                successes += 1;
            }

            if successes > 0 {
                config.save();
            }

            if successes == 0 {
                Err(CommandError::CommandFailed(TextComponent::translate(
                    "commands.op.failed",
                    [],
                )))
            } else {
                Ok(successes)
            }
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(argument(ARG_TARGETS, GameProfilesArgumentConsumer).execute(Executor))
}
