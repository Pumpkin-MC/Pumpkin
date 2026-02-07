use crate::command::CommandResult;
use crate::command::{
    CommandExecutor, CommandSender, args::ConsumedArgs, tree::CommandTree,
    tree::builder::literal,
};
use pumpkin_util::text::TextComponent;

const NAMES: [&str; 1] = ["save-all"];

const DESCRIPTION: &str = "Saves the server to disk.";

struct SaveExecutor;

impl CommandExecutor for SaveExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            sender
                .send_message(TextComponent::translate("commands.save.saving", []))
                .await;
            server.save_all(false).await;
            sender
                .send_message(TextComponent::translate("commands.save.success", []))
                .await;
            Ok(1)
        })
    }
}

struct FlushExecutor;

impl CommandExecutor for FlushExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            sender
                .send_message(TextComponent::translate("commands.save.saving", []))
                .await;
            server.save_all(true).await;
            sender
                .send_message(TextComponent::translate("commands.save.success", []))
                .await;
            Ok(1)
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .execute(SaveExecutor)
        .then(literal("flush").execute(FlushExecutor))
}
