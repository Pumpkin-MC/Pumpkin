use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::entity::EntityArgumentType;
use crate::command::argument_types::identifier::IdentifierArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::entity::EntityBase;

const DESCRIPTION: &str = "Adds, removes, or lists screen shaders (post-processing effects).";
const PERMISSION: &str = "minecraft:command.posteffect";

const ARG_PLAYER: &str = "player";
const ARG_TARGET: &str = "target";
const ARG_POSTEFFECT: &str = "posteffect";

const ERROR_ADD_FAILED: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_POSTEFFECT_ADD_FAILED,
    translation::java::COMMANDS_POSTEFFECT_ADD_FAILED,
);

const ERROR_CLEAR_FAILED: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_POSTEFFECT_CLEAR_FAILED,
    translation::java::COMMANDS_POSTEFFECT_CLEAR_FAILED,
);

const ERROR_REMOVE_FAILED: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_POSTEFFECT_REMOVE_FAILED,
    translation::java::COMMANDS_POSTEFFECT_REMOVE_FAILED,
);

struct AddExecutor;

impl CommandExecutor for AddExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let players = EntityArgumentType::get_players(context, ARG_PLAYER)?;
        let effect_id = IdentifierArgumentType::get(context, ARG_POSTEFFECT)?;
        let effect_str = effect_id.to_string();

        let mut added_count = 0;
        let mut first_player = None;

        for player in &players {
            if player.add_post_effect(effect_str.clone()) {
                if first_player.is_none() {
                    first_player = Some(player.clone());
                }
                added_count += 1;
            }
        }

        if added_count == 0 {
            return Err(ERROR_ADD_FAILED.create_without_context());
        }

        let msg = if added_count == 1 {
            let player = first_player.unwrap_or_else(|| players[0].clone());
            TextComponent::translate_cross(
                translation::java::COMMANDS_POSTEFFECT_ADD_SUCCESS_SINGLE,
                translation::java::COMMANDS_POSTEFFECT_ADD_SUCCESS_SINGLE,
                [TextComponent::text(effect_str), player.get_display_name()],
            )
        } else {
            TextComponent::translate_cross(
                translation::java::COMMANDS_POSTEFFECT_ADD_SUCCESS_MULTIPLE,
                translation::java::COMMANDS_POSTEFFECT_ADD_SUCCESS_MULTIPLE,
                [
                    TextComponent::text(effect_str),
                    TextComponent::text(added_count.to_string()),
                ],
            )
        };

        context.source.send_feedback(msg, true);
        Ok(added_count)
    }
}

struct ClearExecutor;

impl CommandExecutor for ClearExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let players = EntityArgumentType::get_players(context, ARG_PLAYER)?;

        let mut cleared_players = 0;
        let mut first_player = None;

        for player in &players {
            if player.clear_post_effects() > 0 {
                if first_player.is_none() {
                    first_player = Some(player.clone());
                }
                cleared_players += 1;
            }
        }

        if cleared_players == 0 {
            return Err(ERROR_CLEAR_FAILED.create_without_context());
        }

        let msg = if cleared_players == 1 {
            let player = first_player.unwrap_or_else(|| players[0].clone());
            TextComponent::translate_cross(
                translation::java::COMMANDS_POSTEFFECT_CLEAR_SUCCESS_SINGLE,
                translation::java::COMMANDS_POSTEFFECT_CLEAR_SUCCESS_SINGLE,
                [player.get_display_name()],
            )
        } else {
            TextComponent::translate_cross(
                translation::java::COMMANDS_POSTEFFECT_CLEAR_SUCCESS_MULTIPLE,
                translation::java::COMMANDS_POSTEFFECT_CLEAR_SUCCESS_MULTIPLE,
                [TextComponent::text(cleared_players.to_string())],
            )
        };

        context.source.send_feedback(msg, true);
        Ok(cleared_players)
    }
}

struct ListExecutor;

impl CommandExecutor for ListExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let player = EntityArgumentType::get_player(context, ARG_TARGET)?;
        let effects = player.get_post_effects();
        let name = player.get_display_name();

        if effects.is_empty() {
            let msg = TextComponent::translate_cross(
                translation::java::COMMANDS_POSTEFFECT_LIST_EMPTY,
                translation::java::COMMANDS_POSTEFFECT_LIST_EMPTY,
                [name],
            );
            context.source.send_feedback(msg, false);
            Ok(0)
        } else {
            let count = effects.len();
            let effects_str = effects.join(", ");
            let msg = TextComponent::translate_cross(
                translation::java::COMMANDS_POSTEFFECT_LIST_SUCCESS,
                translation::java::COMMANDS_POSTEFFECT_LIST_SUCCESS,
                [
                    name,
                    TextComponent::text(count.to_string()),
                    TextComponent::text(effects_str),
                ],
            );
            context.source.send_feedback(msg, false);
            Ok(count as i32)
        }
    }
}

struct RemoveExecutor;

impl CommandExecutor for RemoveExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let players = EntityArgumentType::get_players(context, ARG_PLAYER)?;
        let effect_id = IdentifierArgumentType::get(context, ARG_POSTEFFECT)?;
        let effect_str = effect_id.to_string();

        let mut removed_count = 0;
        let mut first_player = None;

        for player in &players {
            if player.remove_post_effect(&effect_str) {
                if first_player.is_none() {
                    first_player = Some(player.clone());
                }
                removed_count += 1;
            }
        }

        if removed_count == 0 {
            return Err(ERROR_REMOVE_FAILED.create_without_context());
        }

        let msg = if removed_count == 1 {
            let player = first_player.unwrap_or_else(|| players[0].clone());
            TextComponent::translate_cross(
                translation::java::COMMANDS_POSTEFFECT_REMOVE_SUCCESS_SINGLE,
                translation::java::COMMANDS_POSTEFFECT_REMOVE_SUCCESS_SINGLE,
                [TextComponent::text(effect_str), player.get_display_name()],
            )
        } else {
            TextComponent::translate_cross(
                translation::java::COMMANDS_POSTEFFECT_REMOVE_SUCCESS_MULTIPLE,
                translation::java::COMMANDS_POSTEFFECT_REMOVE_SUCCESS_MULTIPLE,
                [
                    TextComponent::text(effect_str),
                    TextComponent::text(removed_count.to_string()),
                ],
            )
        };

        context.source.send_feedback(msg, true);
        Ok(removed_count)
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    dispatcher.register(
        command("posteffect", DESCRIPTION)
            .requires(PERMISSION)
            .then(
                literal("add").then(
                    argument(ARG_PLAYER, EntityArgumentType::Players).then(
                        argument(ARG_POSTEFFECT, IdentifierArgumentType).executes(AddExecutor),
                    ),
                ),
            )
            .then(
                literal("clear").then(
                    argument(ARG_PLAYER, EntityArgumentType::Players).executes(ClearExecutor),
                ),
            )
            .then(
                literal("list")
                    .then(argument(ARG_TARGET, EntityArgumentType::Player).executes(ListExecutor)),
            )
            .then(literal("remove").then(
                argument(ARG_PLAYER, EntityArgumentType::Players).then(
                    argument(ARG_POSTEFFECT, IdentifierArgumentType).executes(RemoveExecutor),
                ),
            )),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use crate::command::context::command_source::CommandSource;

    #[test]
    fn parse_posteffect() {
        let mut dispatcher = CommandDispatcher::new();
        let registry = PermissionRegistry::default();
        register(&mut dispatcher, &registry);

        let source = Arc::new(CommandSource::dummy());

        let result = dispatcher.parse_input("posteffect add @a minecraft:blur", &source);
        assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);

        let result = dispatcher.parse_input("posteffect clear @a", &source);
        assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);

        let result = dispatcher.parse_input("posteffect list @p", &source);
        assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);

        let result = dispatcher.parse_input("posteffect remove @a minecraft:blur", &source);
        assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);
    }
}
