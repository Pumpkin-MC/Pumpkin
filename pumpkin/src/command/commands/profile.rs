use pumpkin_util::text::TextComponent;
use pumpkin_util::text::color::NamedColor;

use crate::command::CommandResult;

use crate::command::args::ConsumedArgs;
use crate::command::tree::CommandTree;
use crate::command::{CommandExecutor, CommandSender};
use crate::{HEAP_PROFILER, stop_server};

const NAMES: [&str; 1] = ["mem_profile"];

const DESCRIPTION: &str = "Stop the server and dump a memory profile.";

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
                    TextComponent::translate("commands.stop.stopping", [])
                        .color_named(NamedColor::Red),
                )
                .await;

            let mut profiler = HEAP_PROFILER.lock().await;
            let p = profiler.take().unwrap();
            drop(p);

            stop_server();

            Ok(())
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).execute(Executor)
}
