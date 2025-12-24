use pumpkin_data::world::{MSG_COMMAND_INCOMING, MSG_COMMAND_OUTGOING};
use pumpkin_util::text::TextComponent;

use crate::command::{
    CommandError, CommandExecutor, CommandResult, CommandSender,
    args::{
        Arg, ConsumedArgs, FindArgDefaultName, message::MsgArgConsumer,
        players::PlayersArgumentConsumer,
    },
    tree::{
        CommandTree,
        builder::{argument, argument_default_name},
    },
};
use crate::entity::EntityBase;
use CommandError::InvalidConsumption;

const NAMES: [&str; 3] = ["msg", "tell", "w"];

const DESCRIPTION: &str = "Sends a private message to one or more players.";

const ARG_MESSAGE: &str = "message";

struct Executor;

impl CommandExecutor for Executor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Msg(msg)) = args.get(ARG_MESSAGE) else {
                return Err(InvalidConsumption(Some(ARG_MESSAGE.into())));
            };
            let targets = PlayersArgumentConsumer.find_arg_default_name(args)?;
            let player = sender.as_player().ok_or(CommandError::InvalidRequirement)?;

            for target in targets {
                player
                    .send_message(
                        &TextComponent::text(msg.clone()),
                        MSG_COMMAND_OUTGOING,
                        &player.get_display_name().await,
                        Some(&target.get_display_name().await),
                    )
                    .await;
            }
            for target in targets {
                target
                    .send_message(
                        &TextComponent::text(msg.clone()),
                        MSG_COMMAND_INCOMING,
                        &player.get_display_name().await,
                        Some(&target.get_display_name().await),
                    )
                    .await;
            }

            Ok(())
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).then(
        argument_default_name(PlayersArgumentConsumer)
            .then(argument(ARG_MESSAGE, MsgArgConsumer).execute(Executor)),
    )
}
