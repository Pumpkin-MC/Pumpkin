use async_trait::async_trait;
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::color::NamedColor;

use crate::command::args::ConsumedArgs;
use crate::command::tree::CommandTree;
use crate::command::{CommandError, CommandExecutor, CommandSender};
use crate::{HEAP_PROFILER, stop_server};

const NAMES: [&str; 1] = ["mem_profile"];

const DESCRIPTION: &str = "Stop the server and dump a memory profile.";

struct Executor;

#[async_trait]
impl CommandExecutor for Executor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        _server: &crate::server::Server,
        _args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        sender
            .send_message(
                TextComponent::translate("commands.stop.stopping", []).color_named(NamedColor::Red),
            )
            .await;

        let mut profiler = HEAP_PROFILER.lock().await;
        let p = profiler.take().unwrap();
        // Do the actual profiling
        drop(p);

        stop_server();
        Ok(())
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).execute(Executor)
}
