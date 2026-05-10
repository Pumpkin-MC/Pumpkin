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
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let targets = EntityArgumentType::get_entities(context, ARG_TARGETS).await?;

            let target_count = targets.len();
            for target in &targets {
                target.kill(target.as_ref()).await;
            }

            let msg = if target_count == 1 {
                TextComponent::translate_cross(
                    translation::java::COMMANDS_KILL_SUCCESS_SINGLE,
                    translation::java::COMMANDS_KILL_SUCCESS_SINGLE,
                    [targets[0].get_display_name().await],
                )
            } else {
                TextComponent::translate_cross(
                    translation::java::COMMANDS_KILL_SUCCESS_MULTIPLE,
                    translation::java::COMMANDS_KILL_SUCCESS_MULTIPLE,
                    [TextComponent::text(target_count.to_string())],
                )
            };

            context.source.send_feedback(msg, true).await;

            Ok(target_count as i32)
        })
    }
}

struct SelfExecutor;

impl CommandExecutor for SelfExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let target = context.source.entity_or_err()?;
            target.kill(&*target).await;

            context
                .source
                .send_feedback(
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_KILL_SUCCESS_SINGLE,
                        translation::java::COMMANDS_KILL_SUCCESS_SINGLE,
                        [target.get_display_name().await],
                    ),
                    true,
                )
                .await;

            Ok(1)
        })
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &mut PermissionRegistry) {
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
