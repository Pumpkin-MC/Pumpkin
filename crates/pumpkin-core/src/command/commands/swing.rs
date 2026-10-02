use std::sync::Arc;

use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::entity::EntityArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::context::command_source::CommandSource;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::entity::EntityBase;
use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

const DESCRIPTION: &str = "Causes entities to swing their arm.";

const PERMISSION: &str = "minecraft:command.swing";

const ARG_TARGETS: &str = "targets";

const ERROR_NO_LIVING_ENTITY: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_SWING_FAILED_NOTLIVING,
    translation::java::COMMANDS_SWING_FAILED_NOTLIVING,
);

#[derive(Clone, Copy)]
enum HandType {
    MainHand,
    OffHand,
}

fn perform_swing(
    source: &CommandSource,
    targets: &[Arc<dyn EntityBase>],
    hand: HandType,
) -> CommandExecutorResult {
    let mut living_count = 0;
    let mut first_living = None;

    for target in targets {
        if let Some(living) = target.get_living_entity() {
            match hand {
                HandType::MainHand => living.swing_hand(),
                HandType::OffHand => living.swing_off_hand(),
            }
            if first_living.is_none() {
                first_living = Some(target);
            }
            living_count += 1;
        }
    }

    if living_count == 0 {
        return Err(ERROR_NO_LIVING_ENTITY.create_without_context());
    }

    let msg = if living_count == 1 {
        let target = first_living.unwrap_or(&targets[0]);
        TextComponent::translate_cross(
            translation::java::COMMANDS_SWING_SUCCESS_SINGLE,
            translation::java::COMMANDS_SWING_SUCCESS_SINGLE,
            [target.get_display_name()],
        )
    } else {
        TextComponent::translate_cross(
            translation::java::COMMANDS_SWING_SUCCESS_MULTIPLE,
            translation::java::COMMANDS_SWING_SUCCESS_MULTIPLE,
            [TextComponent::text(living_count.to_string())],
        )
    };

    source.send_feedback(msg, true);

    Ok(living_count)
}

struct TargetsExecutor(HandType);

impl CommandExecutor for TargetsExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let targets = EntityArgumentType::get_entities(context, ARG_TARGETS)?;
        perform_swing(context.source.as_ref(), &targets, self.0)
    }
}

struct SelfExecutor;

impl CommandExecutor for SelfExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let target = context.source.entity_or_err()?;
        perform_swing(context.source.as_ref(), &[target], HandType::MainHand)
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    dispatcher.register(
        command("swing", DESCRIPTION)
            .requires(PERMISSION)
            .then(
                argument(ARG_TARGETS, EntityArgumentType::Entities)
                    .executes(TargetsExecutor(HandType::MainHand))
                    .then(literal("mainhand").executes(TargetsExecutor(HandType::MainHand)))
                    .then(literal("offhand").executes(TargetsExecutor(HandType::OffHand))),
            )
            .executes(SelfExecutor),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn parse_swing() {
        let mut dispatcher = CommandDispatcher::new();
        let registry = PermissionRegistry::default();
        register(&mut dispatcher, &registry);

        let source = Arc::new(CommandSource::dummy());

        let result = dispatcher.parse_input("swing", &source);
        assert!(result.errors.is_empty());

        let result = dispatcher.parse_input("swing @e", &source);
        assert!(result.errors.is_empty());

        let result = dispatcher.parse_input("swing @e mainhand", &source);
        assert!(result.errors.is_empty());

        let result = dispatcher.parse_input("swing @e offhand", &source);
        assert!(result.errors.is_empty());
    }
}
