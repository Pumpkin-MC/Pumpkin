use async_trait::async_trait;
use pumpkin_data::world::SAY_COMMAND;
use pumpkin_util::text::TextComponent;

use crate::command::{
    args::{arg_message::MsgArgConsumer, Arg, ConsumedArgs},
    tree::CommandTree,
    tree_builder::argument,
    CommandError, CommandExecutor, CommandSender,
};
use CommandError::InvalidConsumption;

const NAMES: [&str; 1] = ["say"];

const DESCRIPTION: &str = "Broadcast a message to all Players.";

const ARG_MESSAGE: &str = "message";

struct SayExecutor;

#[async_trait]
impl CommandExecutor for SayExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
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
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .with_child(argument(ARG_MESSAGE, MsgArgConsumer).execute(SayExecutor))
}
