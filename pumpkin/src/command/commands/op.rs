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
            let mut config = OPERATOR_CONFIG.write().await;

            let Some(Arg::Players(targets)) = args.get(&ARG_TARGETS) else {
                return Err(InvalidConsumption(Some(ARG_TARGETS.into())));
            };

            for player in targets {
                let new_level = server
                    .basic_config
                    .op_permission_level
                    .min(sender.permission_lvl());

                if player.permission_lvl.load() == new_level {
                    sender
                        .send_message(TextComponent::translate("commands.op.failed", []))
                        .await;
                    continue;
                }

                if let Some(op) = config
                    .ops
                    .iter_mut()
                    .find(|o| o.uuid == player.gameprofile.id)
                {
                    op.level = new_level;
                } else {
                    let op_entry = Op::new(
                        player.gameprofile.id,
                        player.gameprofile.name.clone(),
                        new_level,
                        false,
                    );
                    config.ops.push(op_entry);
                }

                config.save();

                {
                    let command_dispatcher = server.command_dispatcher.read().await;
                    player
                        .set_permission_lvl(new_level, &command_dispatcher)
                        .await;
                };

                sender
                    .send_message(TextComponent::translate(
                        "commands.op.success",
                        [player.get_display_name().await],
                    ))
                    .await;
            }

            Ok(())
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(argument(ARG_TARGETS, PlayersArgumentConsumer).execute(Executor))
}
