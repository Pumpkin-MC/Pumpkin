use pumpkin_util::text::TextComponent;
use pumpkin_util::text::color::NamedColor;

use crate::command::args::ConsumedArgs;
use crate::command::tree::CommandTree;
use crate::command::{CommandExecutor, CommandResult, CommandSender};
use crate::request_restart;

const NAMES: [&str; 1] = ["restart"];
const DESCRIPTION: &str = "Restart the server.";

struct Executor;

impl CommandExecutor for Executor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            sender
                .send_message(
                    TextComponent::translate("commands.restart.restarting", [])
                        .color_named(NamedColor::Red),
                )
                .await;
            request_restart();
            Ok(())
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).execute(Executor)
}
