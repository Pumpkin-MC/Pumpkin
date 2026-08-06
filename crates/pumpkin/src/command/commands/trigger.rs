use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::core::integer::IntegerArgumentType;
use crate::command::argument_types::objective::ObjectiveArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::world::scoreboard::ScoreboardScore;
use pumpkin_data::translation;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

const DESCRIPTION: &str = "Modifies a trigger scoreboard objective.";
const PERMISSION: &str = "minecraft:command.trigger";

const ARG_OBJECTIVE: &str = "objective";
const ARG_VALUE: &str = "value";

const INVALID_TRIGGER_ERROR: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_TRIGGER_FAILED_INVALID,
    translation::java::COMMANDS_TRIGGER_FAILED_INVALID,
);

const UNPRIMED_TRIGGER_ERROR: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_TRIGGER_FAILED_UNPRIMED,
    translation::java::COMMANDS_TRIGGER_FAILED_UNPRIMED,
);

struct SimpleTriggerExecutor;

impl CommandExecutor for SimpleTriggerExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let player = context.source.player_or_err()?;
            let player_name = &player.gameprofile.name;
            let objective_name = ObjectiveArgumentType::get(context, ARG_OBJECTIVE)?;

            let world = context.world();
            let mut scoreboard = world.scoreboard.lock().await;

            let objective = scoreboard
                .get_objectives()
                .get(objective_name)
                .ok_or_else(|| INVALID_TRIGGER_ERROR.create_without_context())?;

            if objective.criterion != "trigger" {
                return Err(INVALID_TRIGGER_ERROR.create_without_context());
            }

            let objective_display_name = objective.display_name.clone();

            let is_locked = scoreboard
                .get_scores()
                .get(objective_name)
                .and_then(|m| m.get(player_name))
                .is_none_or(|s| s.locked);

            if is_locked {
                return Err(UNPRIMED_TRIGGER_ERROR.create_without_context());
            }

            let current_value = scoreboard
                .get_scores()
                .get(objective_name)
                .and_then(|m| m.get(player_name))
                .map_or(0, |s| s.value.0);

            let new_value = current_value + 1;

            let updated_score = ScoreboardScore {
                entity_name: Box::leak(player_name.clone().into_boxed_str()),
                objective_name: Box::leak(objective_name.to_string().into_boxed_str()),
                value: VarInt(new_value),
                display_name: None,
                number_format: None,
                locked: true,
            };

            scoreboard.update_score(world, updated_score).await;

            context
                .source
                .send_feedback(
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_TRIGGER_SIMPLE_SUCCESS,
                        translation::java::COMMANDS_TRIGGER_SIMPLE_SUCCESS,
                        [objective_display_name],
                    ),
                    false,
                )
                .await;

            Ok(new_value)
        })
    }
}

struct AddTriggerExecutor;

impl CommandExecutor for AddTriggerExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let player = context.source.player_or_err()?;
            let player_name = &player.gameprofile.name;
            let objective_name = ObjectiveArgumentType::get(context, ARG_OBJECTIVE)?;
            let value = IntegerArgumentType::get(context, ARG_VALUE)?;

            let world = context.world();
            let mut scoreboard = world.scoreboard.lock().await;

            let objective = scoreboard
                .get_objectives()
                .get(objective_name)
                .ok_or_else(|| INVALID_TRIGGER_ERROR.create_without_context())?;

            if objective.criterion != "trigger" {
                return Err(INVALID_TRIGGER_ERROR.create_without_context());
            }

            let objective_display_name = objective.display_name.clone();

            let is_locked = scoreboard
                .get_scores()
                .get(objective_name)
                .and_then(|m| m.get(player_name))
                .is_none_or(|s| s.locked);

            if is_locked {
                return Err(UNPRIMED_TRIGGER_ERROR.create_without_context());
            }

            let current_value = scoreboard
                .get_scores()
                .get(objective_name)
                .and_then(|m| m.get(player_name))
                .map_or(0, |s| s.value.0);

            let new_value = current_value + value;

            let updated_score = ScoreboardScore {
                entity_name: Box::leak(player_name.clone().into_boxed_str()),
                objective_name: Box::leak(objective_name.to_string().into_boxed_str()),
                value: VarInt(new_value),
                display_name: None,
                number_format: None,
                locked: true,
            };

            scoreboard.update_score(world, updated_score).await;

            context
                .source
                .send_feedback(
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_TRIGGER_ADD_SUCCESS,
                        translation::java::COMMANDS_TRIGGER_ADD_SUCCESS,
                        [
                            objective_display_name,
                            TextComponent::text(value.to_string()),
                        ],
                    ),
                    false,
                )
                .await;

            Ok(new_value)
        })
    }
}

struct SetTriggerExecutor;

impl CommandExecutor for SetTriggerExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let player = context.source.player_or_err()?;
            let player_name = &player.gameprofile.name;
            let objective_name = ObjectiveArgumentType::get(context, ARG_OBJECTIVE)?;
            let value = IntegerArgumentType::get(context, ARG_VALUE)?;

            let world = context.world();
            let mut scoreboard = world.scoreboard.lock().await;

            let objective = scoreboard
                .get_objectives()
                .get(objective_name)
                .ok_or_else(|| INVALID_TRIGGER_ERROR.create_without_context())?;

            if objective.criterion != "trigger" {
                return Err(INVALID_TRIGGER_ERROR.create_without_context());
            }

            let objective_display_name = objective.display_name.clone();

            let is_locked = scoreboard
                .get_scores()
                .get(objective_name)
                .and_then(|m| m.get(player_name))
                .is_none_or(|s| s.locked);

            if is_locked {
                return Err(UNPRIMED_TRIGGER_ERROR.create_without_context());
            }

            let updated_score = ScoreboardScore {
                entity_name: Box::leak(player_name.clone().into_boxed_str()),
                objective_name: Box::leak(objective_name.to_string().into_boxed_str()),
                value: VarInt(value),
                display_name: None,
                number_format: None,
                locked: true,
            };

            scoreboard.update_score(world, updated_score).await;

            context
                .source
                .send_feedback(
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_TRIGGER_SET_SUCCESS,
                        translation::java::COMMANDS_TRIGGER_SET_SUCCESS,
                        [
                            objective_display_name,
                            TextComponent::text(value.to_string()),
                        ],
                    ),
                    false,
                )
                .await;

            Ok(value)
        })
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &mut PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Allow,
    ));

    dispatcher.register(
        command("trigger", DESCRIPTION).requires(PERMISSION).then(
            argument(ARG_OBJECTIVE, ObjectiveArgumentType)
                .executes(SimpleTriggerExecutor)
                .then(literal("add").then(
                    argument(ARG_VALUE, IntegerArgumentType::any()).executes(AddTriggerExecutor),
                ))
                .then(literal("set").then(
                    argument(ARG_VALUE, IntegerArgumentType::any()).executes(SetTriggerExecutor),
                )),
        ),
    );
}
