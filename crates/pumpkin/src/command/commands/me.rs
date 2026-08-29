use pumpkin_data::world::EMOTE_COMMAND;
use pumpkin_util::text::TextComponent;

use crate::command::{
    CommandError, CommandExecutor, CommandResult, CommandSender,
    args::{Arg, ConsumedArgs, message::MsgArgConsumer},
    tree::{CommandTree, builder::argument},
};
use CommandError::InvalidConsumption;

const NAMES: [&str; 1] = ["me"];

const DESCRIPTION: &str = "Broadcasts a narrative message about yourself.";

const ARG_MESSAGE: &str = "action";

struct Executor;

impl CommandExecutor for Executor {
    fn execute(
        &self,
        sender: &CommandSender,
        server: &crate::server::Server,
        args: &ConsumedArgs,
    ) -> CommandResult {
        let Some(Arg::Msg(msg)) = args.get(ARG_MESSAGE) else {
            return Err(InvalidConsumption(Some(ARG_MESSAGE.into())));
        };

        let Some(server_arc) = sender
            .world_or_first(server)
            .and_then(|w| w.server.upgrade())
        else {
            return Err(CommandError::CommandFailed(TextComponent::text(
                "Failed to get server instance",
            )));
        };

        server_arc.broadcast_message(
            &TextComponent::text(msg.clone()),
            &TextComponent::text(format!("{sender}")),
            EMOTE_COMMAND,
            None,
        );

        Ok(1)
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(argument(ARG_MESSAGE, MsgArgConsumer).execute(Executor))
}
