use crate::command::argument_builder::{ArgumentBuilder, argument, command};
use crate::command::argument_types::entity::EntityArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

const DESCRIPTION: &str = "Kills all target entities.";

const PERMISSION: &str = "minecraft:command.kill";

const ARG_TARGETS: &str = "targets";

struct TargetsExecutor;

impl CommandExecutor for TargetsExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let targets = EntityArgumentType::get_entities(context, ARG_TARGETS)?;

        let target_count = targets.len();
        for target in &targets {
            target.kill(target.as_ref());
        }

        let msg = if target_count == 1 {
            TextComponent::translate_cross(
                translation::java::COMMANDS_KILL_SUCCESS_SINGLE,
                translation::bedrock::COMMANDS_KILL_SUCCESSFUL,
                [targets[0].get_display_name()],
            )
        } else {
            TextComponent::translate_cross(
                translation::java::COMMANDS_KILL_SUCCESS_MULTIPLE,
                translation::bedrock::COMMANDS_KILL_SUCCESSFUL,
                [TextComponent::text(target_count.to_string())],
            )
        };

        context.source.send_feedback(msg, true);

        Ok(target_count as i32)
    }
}

struct SelfExecutor;

impl CommandExecutor for SelfExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let target = context.source.entity_or_err()?;
        target.kill(&*target);

        context.source.send_feedback(
            TextComponent::translate_cross(
                translation::java::COMMANDS_KILL_SUCCESS_SINGLE,
                translation::bedrock::COMMANDS_KILL_SUCCESSFUL,
                [target.get_display_name()],
            ),
            true,
        );

        Ok(1)
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Four),
    ));

    dispatcher.register(
        command("kill", DESCRIPTION)
            .requires(PERMISSION)
            .then(argument(ARG_TARGETS, EntityArgumentType::Entities).executes(TargetsExecutor))
            .executes(SelfExecutor),
    );
}
