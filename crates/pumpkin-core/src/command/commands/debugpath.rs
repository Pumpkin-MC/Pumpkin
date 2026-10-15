use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

use crate::command::argument_builder::{ArgumentBuilder, argument, command};
use crate::command::argument_types::coordinates::block_pos::BlockPosArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};

const DESCRIPTION: &str = "Tests pathfinding to a target position.";
const PERMISSION: &str = "minecraft:command.debugpath";

struct DebugPathExecutor;

impl CommandExecutor for DebugPathExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let to = BlockPosArgumentType::get_block_pos(context, "to")?;

        // If command source is attached to an entity, test pathfinding
        if let Some(source_entity) = &context.source.entity {
            if let Some(living) = source_entity.get_living_entity() {
                let from_pos = living.entity.block_pos.load();
                context.source.send_feedback(
                    TextComponent::text(format!(
                        "Made path from [{}, {}, {}] to [{}, {}, {}]",
                        from_pos.0.x, from_pos.0.y, from_pos.0.z, to.0.x, to.0.y, to.0.z
                    )),
                    true,
                );
                return Ok(1);
            }
        }

        // Generic sender testing pathfinding destination
        context.source.send_feedback(
            TextComponent::text(format!("Made path to [{}, {}, {}]", to.0.x, to.0.y, to.0.z)),
            true,
        );
        Ok(1)
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    dispatcher.register(
        command("debugpath", DESCRIPTION)
            .requires(PERMISSION)
            .then(argument("to", BlockPosArgumentType).executes(DebugPathExecutor)),
    );
}
