use std::sync::atomic::Ordering::Relaxed;

use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

use crate::command::argument_builder::{ArgumentBuilder, command};
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};

const DESCRIPTION: &str = "Disables automatic server saves.";

const PERMISSION: &str = "minecraft:command.save-off";

const ALREADY_OFF_ERROR_TYPE: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_SAVE_ALREADYOFF,
    translation::bedrock::COMMANDS_SAVE_OFF_ALREADYOFF,
);

struct SaveOffExecutor;

impl CommandExecutor for SaveOffExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let mut any_disabled = false;
            for world in context.server().worlds.load().iter() {
                if world.level.save_enabled.swap(false, Relaxed) {
                    any_disabled = true;
                }
            }

            if !any_disabled {
                return Err(ALREADY_OFF_ERROR_TYPE.create_without_context());
            }

            context
                .source
                .send_feedback(
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_SAVE_DISABLED,
                        translation::bedrock::COMMANDS_SAVE_DISABLED,
                        [],
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
        command("save-off", DESCRIPTION)
            .requires(PERMISSION)
            .executes(SaveOffExecutor),
    );
}
