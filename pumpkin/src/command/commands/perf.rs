use pumpkin_data::translation;
use pumpkin_util::text::TextComponent;

use crate::command::args::ConsumedArgs;
use crate::command::tree::CommandTree;
use crate::command::tree::builder::literal;
use crate::command::{CommandExecutor, CommandResult, CommandSender};

const NAMES: [&str; 1] = ["perf"];

const DESCRIPTION: &str = "Captures info and metrics about the server for 10 seconds.";

struct StartExecutor;

impl CommandExecutor for StartExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let _mspt = server.get_mspt();

            sender
                .send_message(TextComponent::translate(
                    translation::COMMANDS_PERF_STARTED,
                    [],
                ))
                .await;
            Ok(1)
        })
    }
}

struct StopExecutor;

impl CommandExecutor for StopExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let _mspt = server.get_mspt();

            sender
                .send_message(TextComponent::translate(
                    translation::COMMANDS_PERF_STOPPED,
                    [],
                ))
                .await;
            Ok(1)
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(literal("start").execute(StartExecutor))
        .then(literal("stop").execute(StopExecutor))
}
