use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::{TextComponent, color::NamedColor};

use crate::command::argument_builder::{ArgumentBuilder, command};
use crate::command::context::command_context::CommandContext;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};

const DESCRIPTION: &str = "Displays the server TPS and MSPT.";
const PERMISSION: &str = "pumpkin:command.tps";

struct TpsExecutor;

impl CommandExecutor for TpsExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let server = context.source.server();
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

        let message = TextComponent::text("TPS: ")
            .add_child(TextComponent::text(format!("{tps:.1}")).color_named(tps_color))
            .add_child(TextComponent::text(" MSPT: "))
            .add_child(TextComponent::text(format!("{mspt:.2}ms")).color_named(tps_color));

        context.source.send_message(message);

        Ok(tps as i32)
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    dispatcher.register(
        command("tps", DESCRIPTION)
            .requires(PERMISSION)
            .executes(TpsExecutor),
    );
}
