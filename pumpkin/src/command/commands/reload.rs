use pumpkin_util::text::TextComponent;

use crate::command::args::ConsumedArgs;
use crate::command::tree::CommandTree;
use crate::command::{CommandExecutor, CommandResult, CommandSender};

const NAMES: [&str; 1] = ["reload"];

const DESCRIPTION: &str = "Reloads loot tables, advancements, and functions from disk.";

struct ReloadExecutor;

impl CommandExecutor for ReloadExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            // TODO: Actually reload datapacks when datapack system is implemented
            sender
                .send_message(TextComponent::translate("commands.reload.success", []))
                .await;
            Ok(1)
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).execute(ReloadExecutor)
}
