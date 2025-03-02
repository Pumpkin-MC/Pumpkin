use async_trait::async_trait;
use pumpkin_data::world::EMOTE_COMMAND;
use pumpkin_util::text::TextComponent;

use crate::command::{
    CommandError, CommandExecutor, CommandSender,
    args::{Arg, ConsumedArgs, message::MsgArgConsumer},
    tree::CommandTree,
    tree::builder::argument,
};
use CommandError::InvalidConsumption;

const NAMES: [&str; 1] = ["me"];

const DESCRIPTION: &str = "Broadcasts a narrative message about yourself.";

const ARG_MESSAGE: &str = "action";

struct Executor;

#[async_trait]
impl CommandExecutor for Executor {
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
                EMOTE_COMMAND,
                None,
            )
            .await;
        Ok(())
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(argument(ARG_MESSAGE, MsgArgConsumer).execute(Executor))
}
