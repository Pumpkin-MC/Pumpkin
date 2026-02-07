use std::sync::atomic::Ordering;

use crate::command::CommandResult;
use crate::command::{
    CommandExecutor, CommandSender, args::ConsumedArgs, tree::CommandTree,
};
use pumpkin_util::text::TextComponent;

const NAMES: [&str; 1] = ["save-off"];

const DESCRIPTION: &str = "Disables automatic saving.";

struct Executor;

impl CommandExecutor for Executor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            server.autosave_enabled.store(false, Ordering::Relaxed);
            sender
                .send_message(TextComponent::translate(
                    "commands.save.disabled",
                    [],
                ))
                .await;
            Ok(1)
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).execute(Executor)
}
