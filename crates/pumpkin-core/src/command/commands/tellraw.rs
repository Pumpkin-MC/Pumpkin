use std::sync::Arc;

use crate::command::text_resolve::resolve_component;
use crate::entity::EntityBase;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};

use crate::command::argument_builder::{ArgumentBuilder, argument, command};
use crate::command::argument_types::component::ComponentArgumentType;
use crate::command::argument_types::entity::EntityArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};

const DESCRIPTION: &str = "Send raw message to players.";
const PERMISSION: &str = "minecraft:command.tellraw";

struct TellRawExecutor;

impl CommandExecutor for TellRawExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let targets = EntityArgumentType::get_players(context, "targets")?;
        let text = ComponentArgumentType::get(context, "message")?;

        for target in &targets {
            let entity: Arc<dyn EntityBase> = target.clone();
            let text = resolve_component(&text, &context.source, Some(&entity));
            target.send_system_message(&text);
        }

        Ok(targets.len() as i32)
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    dispatcher.register(
        command("tellraw", DESCRIPTION).requires(PERMISSION).then(
            argument("targets", EntityArgumentType::Players)
                .then(argument("message", ComponentArgumentType).executes(TellRawExecutor)),
        ),
    );
}
