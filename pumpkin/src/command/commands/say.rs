use pumpkin_data::world::SAY_COMMAND;
use pumpkin_util::text::TextComponent;

use crate::command::{
    CommandError, CommandExecutor, CommandResult, CommandSender,
    args::{Arg, ConsumedArgs, message::MsgArgConsumer},
    tree::{CommandTree, builder::argument},
};
use CommandError::InvalidConsumption;

const NAMES: [&str; 1] = ["say"];

const DESCRIPTION: &str = "Broadcast a message to all Players.";

const ARG_MESSAGE: &str = "message";

struct Executor;

impl CommandExecutor for Executor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Msg(msg)) = args.get(ARG_MESSAGE) else {
                return Err(InvalidConsumption(Some(ARG_MESSAGE.into())));
            };

            server
                .broadcast_message(
                    &TextComponent::text(msg.clone()),
                    &TextComponent::text(format!("{sender}")),
                    SAY_COMMAND,
                    None,
                )
                .await;
            Ok(())
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(argument(ARG_MESSAGE, MsgArgConsumer).execute(Executor))
}
