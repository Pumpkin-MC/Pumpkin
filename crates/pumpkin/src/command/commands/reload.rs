use crate::command::args::ConsumedArgs;
use crate::command::dispatcher::CommandError;
use crate::command::tree::CommandTree;
use crate::command::tree::builder::literal;
use crate::command::{CommandExecutor, CommandResult, CommandSender};
use pumpkin_util::text::TextComponent;

const NAMES: [&str; 1] = ["reload"];
const DESCRIPTION: &str = "Reload datapacks and other server data.";

struct ReloadExecutor;

impl CommandExecutor for ReloadExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            match server.reload_datapacks().await {
                Ok(()) => {
                    sender
                        .send_message(TextComponent::text("§aDatapacks reloaded successfully."))
                        .await;
                    Ok(1)
                }
                Err(errors) => {
                    for e in &errors {
                        sender
                            .send_message(TextComponent::text(format!("§c{e}")))
                            .await;
                    }
                    Err(CommandError::CommandFailed(TextComponent::text(
                        "§cDatapack reload completed with errors.",
                    )))
                }
            }
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .execute(ReloadExecutor)
        .then(literal("datapacks").execute(ReloadExecutor))
}
