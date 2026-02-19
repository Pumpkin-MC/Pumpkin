use crate::command::CommandResult;
use crate::command::{
    CommandExecutor, CommandSender, args::ConsumedArgs, tree::CommandTree,
};
use pumpkin_util::text::{TextComponent, color::NamedColor};

const NAMES: [&str; 1] = ["tps"];

const DESCRIPTION: &str = "Displays the server TPS and MSPT.";

struct Executor;

impl CommandExecutor for Executor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let tps = server.get_tps().min(20.0);
            let mspt = server.get_mspt();

            let tps_color = if tps >= 18.0 {
                NamedColor::Green
            } else if tps >= 15.0 {
                NamedColor::Yellow
            } else {
                NamedColor::Red
            };

            let message = TextComponent::text("TPS: ")
                .add_child(
                    TextComponent::text(format!("{tps:.1}"))
                        .color_named(tps_color),
                )
                .add_child(TextComponent::text(" MSPT: "))
                .add_child(
                    TextComponent::text(format!("{mspt:.2}ms"))
                        .color_named(tps_color),
                );

            sender.send_message(message).await;

            Ok(tps as i32)
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).execute(Executor)
}
