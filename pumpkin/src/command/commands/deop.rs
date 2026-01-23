use crate::command::CommandResult;
use crate::entity::EntityBase;
use crate::{
    command::{
        CommandError, CommandExecutor, CommandSender,
        args::{Arg, ConsumedArgs, players::PlayersArgumentConsumer},
        tree::CommandTree,
        tree::builder::argument,
    },
    data::{SaveJSONConfiguration, op_data::OPERATOR_CONFIG},
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
            let mut config = OPERATOR_CONFIG.write().await;

            let Some(Arg::Players(targets)) = args.get(&ARG_TARGETS) else {
                return Err(InvalidConsumption(Some(ARG_TARGETS.into())));
            };

            for player in targets {
                if let Some(op_index) = config
                    .ops
                    .iter()
                    .position(|o| o.uuid == player.gameprofile.id)
                {
                    config.ops.remove(op_index);
                }
                config.save();

                {
                    let command_dispatcher = server.command_dispatcher.read().await;
                    player
                        .set_permission_lvl(pumpkin_util::PermissionLvl::Zero, &command_dispatcher)
                        .await;
                };

                let msg = TextComponent::translate(
                    "commands.deop.success",
                    [player.get_display_name().await],
                );
                sender.send_message(msg).await;
            }
            Ok(())
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(argument(ARG_TARGETS, PlayersArgumentConsumer).execute(Executor))
}
