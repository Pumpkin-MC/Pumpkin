use pumpkin_util::text::TextComponent;

use crate::command::args::ConsumedArgs;
use crate::command::tree::CommandTree;
use crate::command::tree::builder::literal;
use crate::command::{CommandExecutor, CommandResult, CommandSender};

const NAMES: [&str; 1] = ["debug"];

const DESCRIPTION: &str = "Starts or stops a debug profiling session.";

struct StartExecutor;

impl CommandExecutor for StartExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            sender
                .send_message(TextComponent::translate(
                    "commands.debug.started",
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
        _server: &'a crate::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            sender
                .send_message(TextComponent::translate(
                    "commands.debug.stopped",
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
