use crate::command::CommandResult;
use crate::command::{
    CommandExecutor, CommandSender, args::ConsumedArgs, translate_component, tree::CommandTree,
};
use pumpkin_util::text::{TextComponent, color::NamedColor};

const NAMES: [&str; 1] = ["tps"];

const DESCRIPTION: &str = "commands.tps.description";

struct Executor;

impl CommandExecutor for Executor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let tps = server.get_tps().min(server.basic_config.tps as f64);
            let mspt = server.get_mspt();

            let max_tps = server.basic_config.tps as f64;
            let tps_color = if tps >= max_tps * 0.9 {
                NamedColor::Green
            } else if tps >= max_tps * 0.75 {
                NamedColor::Yellow
            } else {
                NamedColor::Red
            };
            let locale = sender.get_locale();

            let message = translate_component("commands.tps.tps_label", locale, [])
                .add_child(TextComponent::text(format!("{tps:.1}")).color_named(tps_color))
                .add_child(translate_component("commands.tps.mspt_label", locale, []))
                .add_child(TextComponent::text(format!("{mspt:.2}")).color_named(tps_color))
                .add_child(
                    translate_component("commands.tps.ms_unit", locale, []).color_named(tps_color),
                );

            sender.send_message(message).await;

            Ok(tps as i32)
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).execute(Executor)
}
