use pumpkin_util::text::TextComponent;

use crate::command::args::message::MsgArgConsumer;
use crate::command::args::{Arg, ConsumedArgs};
use crate::command::dispatcher::CommandError;
use crate::command::tree::CommandTree;
use crate::command::tree::builder::argument;
use crate::command::{CommandExecutor, CommandResult, CommandSender};

const NAMES: [&str; 2] = ["teammsg", "tm"];

const DESCRIPTION: &str = "Sends a message to all players on the sender's team.";

const ARG_MESSAGE: &str = "message";

struct TeamMsgExecutor;

impl CommandExecutor for TeamMsgExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Msg(message)) = args.get(ARG_MESSAGE) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_MESSAGE.into())));
            };

            // TODO: Send only to team members when team system is built
            // For now, just echo back to sender
            let sender_name = match sender {
                CommandSender::Player(p) => p.gameprofile.name.clone(),
                _ => "Server".to_string(),
            };

            sender
                .send_message(TextComponent::text(format!(
                    "[Team] <{sender_name}> {message}"
                )))
                .await;
            Ok(1)
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(argument(ARG_MESSAGE, MsgArgConsumer).execute(TeamMsgExecutor))
}
