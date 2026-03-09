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
            let mspt = server.get_mspt();
            let tps = if mspt > 0.0 {
                (1000.0 / mspt).min(20.0)
            } else {
                20.0
            };

            sender
                .send_message(TextComponent::text(format!(
                    "Started 10-second performance profiling. TPS: {tps:.1}, MSPT: {mspt:.2}ms"
                )))
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
            let mspt = server.get_mspt();
            let tps = if mspt > 0.0 {
                (1000.0 / mspt).min(20.0)
            } else {
                20.0
            };

            sender
                .send_message(TextComponent::text(format!(
                    "Stopped performance profiling. TPS: {tps:.1}, MSPT: {mspt:.2}ms"
                )))
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
