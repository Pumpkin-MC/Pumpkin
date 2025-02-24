use async_trait::async_trait;
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::color::NamedColor;

use crate::command::args::ConsumedArgs;
use crate::command::tree::CommandTree;
use crate::command::{CommandError, CommandExecutor, CommandSender};
use crate::stop_server;

const NAMES: [&str; 1] = ["stop"];

const DESCRIPTION: &str = "Stop the server.";

struct Executor;

#[async_trait]
impl CommandExecutor for Executor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &crate::server::Server,
        _args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        sender
            .send_message(
                TextComponent::translate("commands.stop.stopping", []).color_named(NamedColor::Red),
            )
            .await;
        stop_server();
        Ok(())
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).execute(Executor)
}
