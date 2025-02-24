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
use async_trait::async_trait;
use pumpkin_util::text::TextComponent;

const NAMES: [&str; 1] = ["deop"];
const DESCRIPTION: &str = "Revokes operator status from a player.";
const ARG_TARGETS: &str = "targets";

struct Executor;

#[async_trait]
impl CommandExecutor for Executor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
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

            player
                .set_permission_lvl(
                    pumpkin_util::PermissionLvl::Zero,
                    &server.command_dispatcher,
                )
                .await;

            let player_name = &player.gameprofile.name;
            let msg = TextComponent::translate(
                "commands.deop.success",
                [TextComponent::text(player_name.clone())],
            );
            sender.send_message(msg).await;
        }
        Ok(())
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(argument(ARG_TARGETS, PlayersArgumentConsumer).execute(Executor))
}
