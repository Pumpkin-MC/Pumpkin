use crate::command::args::simple::SimpleArgConsumer;
use crate::command::args::{Arg, ConsumedArgs};
use crate::command::dispatcher::CommandError;
use crate::command::tree::CommandTree;
use crate::command::tree::builder::argument;
use crate::command::{CommandExecutor, CommandResult, CommandSender};
use pumpkin_util::text::TextComponent;

const NAMES: [&str; 1] = ["function"];
const DESCRIPTION: &str = "Execute a function.";
const ARG_NAME: &str = "name";

struct FunctionExecutor;

impl CommandExecutor for FunctionExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Simple(name)) = args.get(ARG_NAME) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_NAME.into())));
            };

            let Some(func_id) = pumpkin_datapack::Identifier::parse(name).ok() else {
                sender
                    .send_message(TextComponent::text(format!("§cUnknown function: {name}")))
                    .await;
                return Err(CommandError::CommandFailed(TextComponent::text(format!(
                    "§cUnknown function: {name}"
                ))));
            };

            // Read function from the datapack manager
            let function = {
                let functions = server.datapack_manager.functions.read().await;
                functions.get(&func_id).cloned()
            };

            let Some(function) = function else {
                sender
                    .send_message(TextComponent::text(format!("§cUnknown function: {name}")))
                    .await;
                return Err(CommandError::CommandFailed(TextComponent::text(format!(
                    "§cUnknown function: {name}"
                ))));
            };

            let cmd_count = match function {
                pumpkin_datapack::function::parser::MCFunction::PlainText { commands } => {
                    let count = commands.len();

                    // Get the global Arc<Server> to create a CommandSource for dispatch.
                    let Some(global_server) = crate::server::global_server() else {
                        sender
                            .send_message(TextComponent::text(
                                "§cServer not fully initialized yet.",
                            ))
                            .await;
                        return Err(CommandError::CommandFailed(TextComponent::text(
                            "§cServer not fully initialized yet.",
                        )));
                    };

                    let source = sender.clone().into_source(global_server).await;
                    for cmd in &commands {
                        server
                            .command_dispatcher
                            .read()
                            .await
                            .handle_command(&source, cmd)
                            .await;
                    }
                    count
                }
                pumpkin_datapack::function::parser::MCFunction::Macro { .. } => {
                    sender
                        .send_message(TextComponent::text(
                            "§cMacro functions require arguments; use /function <id> <args>.",
                        ))
                        .await;
                    return Err(CommandError::CommandFailed(TextComponent::text(
                        "§cMacro functions require arguments",
                    )));
                }
            };

            sender
                .send_message(TextComponent::text(format!(
                    "§aExecuted {cmd_count} command(s) from '{name}'"
                )))
                .await;
            Ok(cmd_count as i32)
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(argument(ARG_NAME, SimpleArgConsumer).execute(FunctionExecutor))
}
