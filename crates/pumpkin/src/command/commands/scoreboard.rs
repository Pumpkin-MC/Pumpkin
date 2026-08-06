use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::core::integer::IntegerArgumentType;
use crate::command::argument_types::core::string::StringArgumentType;
use crate::command::argument_types::display_slot::ScoreboardDisplaySlotArgumentType;
use crate::command::argument_types::entity::EntityArgumentType;
use crate::command::argument_types::objective::ObjectiveArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::world::scoreboard::{ScoreboardObjective, ScoreboardScore};
use pumpkin_data::scoreboard::ScoreboardDisplaySlot;
use pumpkin_data::translation;
use pumpkin_protocol::NumberFormat;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::RenderType;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;
use std::sync::Arc;

const DESCRIPTION: &str = "Manages scoreboard objectives and players.";
const PERMISSION: &str = "minecraft:command.scoreboard";

const ARG_OBJECTIVE: &str = "objective";
const ARG_CRITERION: &str = "criterion";
const ARG_DISPLAY_NAME: &str = "display_name";
const ARG_TARGETS: &str = "targets";
const ARG_SCORE: &str = "score";
const ARG_TARGET: &str = "target";
const ARG_SLOT: &str = "slot";

const DUPLICATE_OBJECTIVE_ERROR: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_SCOREBOARD_OBJECTIVES_ADD_DUPLICATE,
    translation::java::COMMANDS_SCOREBOARD_OBJECTIVES_ADD_DUPLICATE,
);

const INVALID_CRITERION_ERROR: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_SCOREBOARD_OBJECTIVES_ADD_DUPLICATE, // Approximate error, no exact key
    translation::java::COMMANDS_SCOREBOARD_OBJECTIVES_ADD_DUPLICATE,
);

const STYLE_PARSE_ERROR: crate::command::errors::error_types::LiteralCommandErrorType =
    crate::command::errors::error_types::LiteralCommandErrorType::new("Invalid style format");

const INVALID_ENABLE_ERROR: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_SCOREBOARD_PLAYERS_ENABLE_INVALID,
    translation::java::COMMANDS_SCOREBOARD_PLAYERS_ENABLE_INVALID,
);

const FAILED_ENABLE_ERROR: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_SCOREBOARD_PLAYERS_ENABLE_FAILED,
    translation::java::COMMANDS_SCOREBOARD_PLAYERS_ENABLE_FAILED,
);

const DISPLAY_SLOT_ALREADY_EMPTY_ERROR: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_SCOREBOARD_OBJECTIVES_DISPLAY_ALREADYEMPTY,
    translation::java::COMMANDS_SCOREBOARD_OBJECTIVES_DISPLAY_ALREADYEMPTY,
);

const DISPLAY_SLOT_ALREADY_SET_ERROR: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_SCOREBOARD_OBJECTIVES_DISPLAY_ALREADYSET,
    translation::java::COMMANDS_SCOREBOARD_OBJECTIVES_DISPLAY_ALREADYSET,
);

const NO_VALUE_ERROR: CommandErrorType<2> = CommandErrorType::new(
    translation::java::COMMANDS_SCOREBOARD_PLAYERS_GET_NULL,
    translation::java::COMMANDS_SCOREBOARD_PLAYERS_GET_NULL,
);

const OBJECTIVE_NOT_FOUND_ERROR: CommandErrorType<0> = CommandErrorType::new(
    translation::java::ARGUMENTS_OBJECTIVE_NOTFOUND,
    translation::java::ARGUMENTS_OBJECTIVE_NOTFOUND,
);

struct ObjectivesAddExecutor {
    has_display_name: bool,
}

fn obj_name<'a>(
    context: &'a CommandContext,
) -> Result<&'a str, crate::command::errors::command_syntax_error::CommandSyntaxError> {
    ObjectiveArgumentType::get(context, ARG_OBJECTIVE)
}

impl CommandExecutor for ObjectivesAddExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let objective_name = StringArgumentType::get(context, ARG_OBJECTIVE)?;
            let criterion = StringArgumentType::get(context, ARG_CRITERION)?;

            let display_name = if self.has_display_name {
                TextComponent::text(StringArgumentType::get(context, ARG_DISPLAY_NAME)?.to_string())
            } else {
                TextComponent::text(objective_name.to_string())
            };

            if !crate::world::scoreboard::is_valid_criterion(criterion) {
                return Err(INVALID_CRITERION_ERROR.create_without_context());
            }

            let world = context.world();
            let mut scoreboard = world.scoreboard.lock().await;

            if scoreboard.get_objectives().contains_key(objective_name) {
                return Err(DUPLICATE_OBJECTIVE_ERROR.create_without_context());
            }

            let render_type =
                crate::world::scoreboard::default_render_type_for_criterion(criterion);
            let new_objective = ScoreboardObjective::new(
                Arc::from(objective_name),
                display_name.clone(),
                render_type,
                None,
                Arc::from(criterion),
            );

            scoreboard.add_objective(world, new_objective);

            context
                .source
                .send_feedback(
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_SCOREBOARD_OBJECTIVES_ADD_SUCCESS,
                        translation::java::COMMANDS_SCOREBOARD_OBJECTIVES_ADD_SUCCESS,
                        [display_name],
                    ),
                    true,
                )
                .await;

            Ok(1)
        })
    }
}

struct PlayersEnableExecutor;

impl CommandExecutor for PlayersEnableExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let targets = EntityArgumentType::get_players(context, ARG_TARGETS).await?;
            let objective_name = ObjectiveArgumentType::get(context, ARG_OBJECTIVE)?;

            let world = context.world();
            let mut scoreboard = world.scoreboard.lock().await;

            let objective = scoreboard
                .get_objectives()
                .get(objective_name)
                .ok_or_else(|| OBJECTIVE_NOT_FOUND_ERROR.create_without_context())?;

            if &*objective.criterion != "trigger" {
                return Err(INVALID_ENABLE_ERROR.create_without_context());
            }

            let objective_display_name = objective.display_name.clone();

            let mut enabled_count = 0;
            for player in &targets {
                let player_name = &player.gameprofile.name;
                let current_score = scoreboard
                    .get_scores()
                    .get(objective_name)
                    .and_then(|m| m.get(player_name));

                let is_already_enabled = current_score.is_some_and(|s| !s.locked);

                if !is_already_enabled {
                    let value = current_score.map_or(0, |s| s.value.0);
                    let display_name = current_score.and_then(|s| s.display_name.clone());
                    let number_format = current_score.and_then(|s| s.number_format.clone());

                    let updated_score = ScoreboardScore {
                        entity_name: Arc::from(player_name.as_str()),
                        objective_name: Arc::from(objective_name),
                        value: VarInt(value),
                        display_name,
                        number_format,
                        locked: false,
                    };

                    scoreboard.update_score(world, updated_score).await;
                    enabled_count += 1;
                }
            }

            if enabled_count == 0 {
                return Err(FAILED_ENABLE_ERROR.create_without_context());
            }

            let msg = if targets.len() == 1 {
                TextComponent::translate_cross(
                    translation::java::COMMANDS_SCOREBOARD_PLAYERS_ENABLE_SUCCESS_SINGLE,
                    translation::java::COMMANDS_SCOREBOARD_PLAYERS_ENABLE_SUCCESS_SINGLE,
                    [
                        objective_display_name,
                        TextComponent::text(targets[0].gameprofile.name.clone()),
                    ],
                )
            } else {
                TextComponent::translate_cross(
                    translation::java::COMMANDS_SCOREBOARD_PLAYERS_ENABLE_SUCCESS_MULTIPLE,
                    translation::java::COMMANDS_SCOREBOARD_PLAYERS_ENABLE_SUCCESS_MULTIPLE,
                    [
                        objective_display_name,
                        TextComponent::text(targets.len().to_string()),
                    ],
                )
            };

            context.source.send_feedback(msg, true).await;

            Ok(enabled_count)
        })
    }
}

struct ObjectivesListExecutor;

impl CommandExecutor for ObjectivesListExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let scoreboard = context.world().scoreboard.lock().await;
            let objectives: Vec<&str> = scoreboard
                .get_objectives()
                .keys()
                .map(String::as_str)
                .collect();

            if objectives.is_empty() {
                context
                    .source
                    .send_feedback(
                        TextComponent::translate_cross(
                            translation::java::COMMANDS_SCOREBOARD_OBJECTIVES_LIST_EMPTY,
                            translation::java::COMMANDS_SCOREBOARD_OBJECTIVES_LIST_EMPTY,
                            [],
                        ),
                        false,
                    )
                    .await;
            } else {
                context
                    .source
                    .send_feedback(
                        TextComponent::translate_cross(
                            translation::java::COMMANDS_SCOREBOARD_OBJECTIVES_LIST_SUCCESS,
                            translation::java::COMMANDS_SCOREBOARD_OBJECTIVES_LIST_SUCCESS,
                            [
                                TextComponent::text(objectives.len().to_string()),
                                TextComponent::text(objectives.join(", ")),
                            ],
                        ),
                        false,
                    )
                    .await;
            }

            Ok(objectives.len() as i32)
        })
    }
}

struct ObjectivesRemoveExecutor;

impl CommandExecutor for ObjectivesRemoveExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let objective_name = obj_name(context)?;
            let world = context.world();
            let mut scoreboard = world.scoreboard.lock().await;

            if !scoreboard.get_objectives().contains_key(objective_name) {
                return Err(OBJECTIVE_NOT_FOUND_ERROR.create_without_context());
            }

            scoreboard.remove_objective(world, objective_name);

            context
                .source
                .send_feedback(
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_SCOREBOARD_OBJECTIVES_REMOVE_SUCCESS,
                        translation::java::COMMANDS_SCOREBOARD_OBJECTIVES_REMOVE_SUCCESS,
                        [TextComponent::text(objective_name.to_string())],
                    ),
                    true,
                )
                .await;

            Ok(0)
        })
    }
}

struct ModifyDisplayNameExecutor;

impl CommandExecutor for ModifyDisplayNameExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let objective_name = obj_name(context)?;
            let new_display = TextComponent::text(
                StringArgumentType::get(context, ARG_DISPLAY_NAME)?.to_string(),
            );

            let world = context.world();
            let mut scoreboard = world.scoreboard.lock().await;

            let Some(objective) = scoreboard.get_objective(objective_name) else {
                return Err(OBJECTIVE_NOT_FOUND_ERROR.create_without_context());
            };
            let render_type = objective.render_type;
            let number_format = objective.number_format.clone();
            let old_display = objective.display_name.clone();
            let _ = objective;

            if old_display == new_display {
                return Ok(0);
            }

            scoreboard.modify_objective(
                world,
                objective_name,
                new_display.clone(),
                render_type,
                number_format,
            );

            context
                .source
                .send_feedback(
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_SCOREBOARD_OBJECTIVES_MODIFY_DISPLAYNAME,
                        translation::java::COMMANDS_SCOREBOARD_OBJECTIVES_MODIFY_DISPLAYNAME,
                        [TextComponent::text(objective_name.to_string()), new_display],
                    ),
                    true,
                )
                .await;

            Ok(0)
        })
    }
}

struct ModifyRenderTypeExecutor {
    render_type: RenderType,
}

impl CommandExecutor for ModifyRenderTypeExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let objective_name = obj_name(context)?;
            let world = context.world();
            let mut scoreboard = world.scoreboard.lock().await;

            let Some(objective) = scoreboard.get_objective(objective_name) else {
                return Err(OBJECTIVE_NOT_FOUND_ERROR.create_without_context());
            };
            let display_name = objective.display_name.clone();
            let render_type = objective.render_type;
            let number_format = objective.number_format.clone();
            let _ = objective;

            if render_type as i32 == self.render_type as i32 {
                return Ok(0);
            }

            scoreboard.modify_objective(
                world,
                objective_name,
                display_name.clone(),
                self.render_type,
                number_format,
            );

            context
                .source
                .send_feedback(
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_SCOREBOARD_OBJECTIVES_MODIFY_RENDERTYPE,
                        translation::java::COMMANDS_SCOREBOARD_OBJECTIVES_MODIFY_RENDERTYPE,
                        [display_name],
                    ),
                    true,
                )
                .await;

            Ok(0)
        })
    }
}

struct ObjectivesSetDisplayExecutor;

impl CommandExecutor for ObjectivesSetDisplayExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let slot = ScoreboardDisplaySlotArgumentType::get(context, ARG_SLOT)?;
            let objective_name: Option<String> =
                context.get_argument::<String>(ARG_OBJECTIVE).ok().cloned();
            let slot_name = display_slot_name(slot);

            let world = context.world();
            let mut scoreboard = world.scoreboard.lock().await;

            if let Some(name) = &objective_name {
                if !scoreboard.get_objectives().contains_key(name) {
                    return Err(OBJECTIVE_NOT_FOUND_ERROR.create_without_context());
                }

                if scoreboard.get_display_objective(&slot) == Some(name.as_str()) {
                    return Err(DISPLAY_SLOT_ALREADY_SET_ERROR.create_without_context());
                }

                scoreboard.set_display_objective(world, slot, Some(name));

                context
                    .source
                    .send_feedback(
                        TextComponent::translate_cross(
                            translation::java::COMMANDS_SCOREBOARD_OBJECTIVES_DISPLAY_SET,
                            translation::java::COMMANDS_SCOREBOARD_OBJECTIVES_DISPLAY_SET,
                            [
                                TextComponent::text(slot_name),
                                TextComponent::text(name.clone()),
                            ],
                        ),
                        true,
                    )
                    .await;
            } else {
                if scoreboard.get_display_objective(&slot).is_none() {
                    return Err(DISPLAY_SLOT_ALREADY_EMPTY_ERROR.create_without_context());
                }

                scoreboard.set_display_objective(world, slot, None);

                context
                    .source
                    .send_feedback(
                        TextComponent::translate_cross(
                            translation::java::COMMANDS_SCOREBOARD_OBJECTIVES_DISPLAY_CLEARED,
                            translation::java::COMMANDS_SCOREBOARD_OBJECTIVES_DISPLAY_CLEARED,
                            [TextComponent::text(slot_name)],
                        ),
                        true,
                    )
                    .await;
            }

            Ok(0)
        })
    }
}

struct ObjectivesClearDisplayExecutor;

impl CommandExecutor for ObjectivesClearDisplayExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let slot = ScoreboardDisplaySlotArgumentType::get(context, ARG_SLOT)?;
            let slot_name = display_slot_name(slot);
            let world = context.world();
            let mut scoreboard = world.scoreboard.lock().await;

            if scoreboard.get_display_objective(&slot).is_none() {
                return Err(DISPLAY_SLOT_ALREADY_EMPTY_ERROR.create_without_context());
            }

            scoreboard.set_display_objective(world, slot, None);

            context
                .source
                .send_feedback(
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_SCOREBOARD_OBJECTIVES_DISPLAY_CLEARED,
                        translation::java::COMMANDS_SCOREBOARD_OBJECTIVES_DISPLAY_CLEARED,
                        [TextComponent::text(slot_name)],
                    ),
                    true,
                )
                .await;

            Ok(0)
        })
    }
}

struct PlayersSetExecutor;

impl CommandExecutor for PlayersSetExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let targets = EntityArgumentType::get_players(context, ARG_TARGETS).await?;
            let objective_name = obj_name(context)?;
            let value = IntegerArgumentType::get(context, ARG_SCORE)?;

            let world = context.world();
            let mut scoreboard = world.scoreboard.lock().await;

            if !scoreboard.get_objectives().contains_key(objective_name) {
                return Err(OBJECTIVE_NOT_FOUND_ERROR.create_without_context());
            }

            for player in &targets {
                let score = ScoreboardScore {
                    entity_name: Arc::from(player.gameprofile.name.as_str()),
                    objective_name: Arc::from(objective_name),
                    value: VarInt(value),
                    display_name: None,
                    number_format: None,
                    locked: false,
                };
                scoreboard.update_score(world, score).await;
            }

            if targets.len() == 1 {
                context
                    .source
                    .send_feedback(
                        TextComponent::translate_cross(
                            translation::java::COMMANDS_SCOREBOARD_PLAYERS_SET_SUCCESS_SINGLE,
                            translation::java::COMMANDS_SCOREBOARD_PLAYERS_SET_SUCCESS_SINGLE,
                            [
                                TextComponent::text(objective_name.to_string()),
                                TextComponent::text(targets[0].gameprofile.name.clone()),
                                TextComponent::text(value.to_string()),
                            ],
                        ),
                        true,
                    )
                    .await;
            } else {
                context
                    .source
                    .send_feedback(
                        TextComponent::translate_cross(
                            translation::java::COMMANDS_SCOREBOARD_PLAYERS_SET_SUCCESS_MULTIPLE,
                            translation::java::COMMANDS_SCOREBOARD_PLAYERS_SET_SUCCESS_MULTIPLE,
                            [
                                TextComponent::text(objective_name.to_string()),
                                TextComponent::text(targets.len().to_string()),
                                TextComponent::text(value.to_string()),
                            ],
                        ),
                        true,
                    )
                    .await;
            }

            Ok(value * targets.len() as i32)
        })
    }
}

struct PlayersGetExecutor;

impl CommandExecutor for PlayersGetExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let targets = EntityArgumentType::get_players(context, ARG_TARGET).await?;
            let objective_name = obj_name(context)?;

            // `players get` with empty targets is a parse error (EntityArgumentType requires at least 1)
            if targets.is_empty() {
                return Err(OBJECTIVE_NOT_FOUND_ERROR.create_without_context());
            }

            let player = &targets[0];
            let player_name = &player.gameprofile.name;

            let world = context.world();
            let scoreboard = world.scoreboard.lock().await;

            let Some(score_info) = scoreboard.get_player_score_info(player_name, objective_name)
            else {
                return Err(NO_VALUE_ERROR.create_without_context(
                    TextComponent::text(objective_name.to_string()),
                    TextComponent::text(player_name.clone()),
                ));
            };

            let value = score_info.value.0;

            context
                .source
                .send_feedback(
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_SCOREBOARD_PLAYERS_GET_SUCCESS,
                        translation::java::COMMANDS_SCOREBOARD_PLAYERS_GET_SUCCESS,
                        [
                            TextComponent::text(objective_name.to_string()),
                            TextComponent::text(player_name.clone()),
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

struct PlayersAddExecutor;

impl CommandExecutor for PlayersAddExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let targets = EntityArgumentType::get_players(context, ARG_TARGETS).await?;
            let objective_name = obj_name(context)?;
            let add_value = IntegerArgumentType::get(context, ARG_SCORE)?;

            let world = context.world();
            let mut scoreboard = world.scoreboard.lock().await;

            let objective = scoreboard
                .get_objectives()
                .get(objective_name)
                .ok_or_else(|| OBJECTIVE_NOT_FOUND_ERROR.create_without_context())?;
            let obj_display = objective.display_name.clone();
            let _ = objective;

            let mut result = 0;
            for player in &targets {
                let player_name = &player.gameprofile.name;
                let existing = scoreboard.get_player_score_info(player_name, objective_name);
                let current = existing.map_or(0, |s| s.value.0);
                let new_value = current + add_value;

                let score = ScoreboardScore {
                    entity_name: Arc::from(player_name.as_str()),
                    objective_name: Arc::from(objective_name),
                    value: VarInt(new_value),
                    display_name: existing.and_then(|s| s.display_name.clone()),
                    number_format: existing.and_then(|s| s.number_format.clone()),
                    locked: existing.is_none_or(|s| s.locked),
                };
                scoreboard.update_score(world, score).await;
                result += new_value;
            }

            if targets.len() == 1 {
                context
                    .source
                    .send_feedback(
                        TextComponent::translate_cross(
                            translation::java::COMMANDS_SCOREBOARD_PLAYERS_ADD_SUCCESS_SINGLE,
                            translation::java::COMMANDS_SCOREBOARD_PLAYERS_ADD_SUCCESS_SINGLE,
                            [
                                TextComponent::text(add_value.to_string()),
                                obj_display,
                                TextComponent::text(targets[0].gameprofile.name.clone()),
                                TextComponent::text(result.to_string()),
                            ],
                        ),
                        true,
                    )
                    .await;
            } else {
                context
                    .source
                    .send_feedback(
                        TextComponent::translate_cross(
                            translation::java::COMMANDS_SCOREBOARD_PLAYERS_ADD_SUCCESS_MULTIPLE,
                            translation::java::COMMANDS_SCOREBOARD_PLAYERS_ADD_SUCCESS_MULTIPLE,
                            [
                                TextComponent::text(add_value.to_string()),
                                obj_display,
                                TextComponent::text(targets.len().to_string()),
                            ],
                        ),
                        true,
                    )
                    .await;
            }

            Ok(result)
        })
    }
}

struct PlayersRemoveExecutor;

impl CommandExecutor for PlayersRemoveExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let targets = EntityArgumentType::get_players(context, ARG_TARGETS).await?;
            let objective_name = obj_name(context)?;
            let remove_value = IntegerArgumentType::get(context, ARG_SCORE)?;

            let world = context.world();
            let mut scoreboard = world.scoreboard.lock().await;

            let objective = scoreboard
                .get_objectives()
                .get(objective_name)
                .ok_or_else(|| OBJECTIVE_NOT_FOUND_ERROR.create_without_context())?;
            let obj_display = objective.display_name.clone();
            let _ = objective;

            let mut result = 0;
            for player in &targets {
                let player_name = &player.gameprofile.name;
                let existing = scoreboard.get_player_score_info(player_name, objective_name);
                let current = existing.map_or(0, |s| s.value.0);
                let new_value = current - remove_value;

                let score = ScoreboardScore {
                    entity_name: Arc::from(player_name.as_str()),
                    objective_name: Arc::from(objective_name),
                    value: VarInt(new_value),
                    display_name: existing.and_then(|s| s.display_name.clone()),
                    number_format: existing.and_then(|s| s.number_format.clone()),
                    locked: existing.is_none_or(|s| s.locked),
                };
                scoreboard.update_score(world, score).await;
                result += new_value;
            }

            if targets.len() == 1 {
                context
                    .source
                    .send_feedback(
                        TextComponent::translate_cross(
                            translation::java::COMMANDS_SCOREBOARD_PLAYERS_REMOVE_SUCCESS_SINGLE,
                            translation::java::COMMANDS_SCOREBOARD_PLAYERS_REMOVE_SUCCESS_SINGLE,
                            [
                                TextComponent::text(remove_value.to_string()),
                                obj_display,
                                TextComponent::text(targets[0].gameprofile.name.clone()),
                                TextComponent::text(result.to_string()),
                            ],
                        ),
                        true,
                    )
                    .await;
            } else {
                context
                    .source
                    .send_feedback(
                        TextComponent::translate_cross(
                            translation::java::COMMANDS_SCOREBOARD_PLAYERS_REMOVE_SUCCESS_MULTIPLE,
                            translation::java::COMMANDS_SCOREBOARD_PLAYERS_REMOVE_SUCCESS_MULTIPLE,
                            [
                                TextComponent::text(remove_value.to_string()),
                                obj_display,
                                TextComponent::text(targets.len().to_string()),
                            ],
                        ),
                        true,
                    )
                    .await;
            }

            Ok(result)
        })
    }
}

struct PlayersResetAllExecutor;

impl CommandExecutor for PlayersResetAllExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let targets = EntityArgumentType::get_players(context, ARG_TARGETS).await?;
            let world = context.world();
            let mut scoreboard = world.scoreboard.lock().await;

            for player in &targets {
                scoreboard.reset_all_player_scores(world, &player.gameprofile.name);
            }

            if targets.len() == 1 {
                context
                    .source
                    .send_feedback(
                        TextComponent::translate_cross(
                            translation::java::COMMANDS_SCOREBOARD_PLAYERS_RESET_ALL_SINGLE,
                            translation::java::COMMANDS_SCOREBOARD_PLAYERS_RESET_ALL_SINGLE,
                            [TextComponent::text(targets[0].gameprofile.name.clone())],
                        ),
                        true,
                    )
                    .await;
            } else {
                context
                    .source
                    .send_feedback(
                        TextComponent::translate_cross(
                            translation::java::COMMANDS_SCOREBOARD_PLAYERS_RESET_ALL_MULTIPLE,
                            translation::java::COMMANDS_SCOREBOARD_PLAYERS_RESET_ALL_MULTIPLE,
                            [TextComponent::text(targets.len().to_string())],
                        ),
                        true,
                    )
                    .await;
            }

            Ok(targets.len() as i32)
        })
    }
}

struct PlayersResetSingleExecutor;

impl CommandExecutor for PlayersResetSingleExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let targets = EntityArgumentType::get_players(context, ARG_TARGETS).await?;
            let objective_name = obj_name(context)?;
            let world = context.world();
            let mut scoreboard = world.scoreboard.lock().await;

            for player in &targets {
                scoreboard.reset_single_player_score(
                    world,
                    &player.gameprofile.name,
                    objective_name,
                );
            }

            let obj_display = TextComponent::text(objective_name.to_string());
            if targets.len() == 1 {
                context
                    .source
                    .send_feedback(
                        TextComponent::translate_cross(
                            translation::java::COMMANDS_SCOREBOARD_PLAYERS_RESET_SPECIFIC_SINGLE,
                            translation::java::COMMANDS_SCOREBOARD_PLAYERS_RESET_SPECIFIC_SINGLE,
                            [
                                obj_display,
                                TextComponent::text(targets[0].gameprofile.name.clone()),
                            ],
                        ),
                        true,
                    )
                    .await;
            } else {
                context
                    .source
                    .send_feedback(
                        TextComponent::translate_cross(
                            translation::java::COMMANDS_SCOREBOARD_PLAYERS_RESET_SPECIFIC_MULTIPLE,
                            translation::java::COMMANDS_SCOREBOARD_PLAYERS_RESET_SPECIFIC_MULTIPLE,
                            [obj_display, TextComponent::text(targets.len().to_string())],
                        ),
                        true,
                    )
                    .await;
            }

            Ok(targets.len() as i32)
        })
    }
}

struct PlayersListExecutor;

impl CommandExecutor for PlayersListExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let scoreboard = context.world().scoreboard.lock().await;
            let tracked = scoreboard.get_tracked_players();

            if tracked.is_empty() {
                context
                    .source
                    .send_feedback(
                        TextComponent::translate_cross(
                            translation::java::COMMANDS_SCOREBOARD_PLAYERS_LIST_EMPTY,
                            translation::java::COMMANDS_SCOREBOARD_PLAYERS_LIST_EMPTY,
                            [],
                        ),
                        false,
                    )
                    .await;
            } else {
                let names = tracked.clone();
                context
                    .source
                    .send_feedback(
                        TextComponent::translate_cross(
                            translation::java::COMMANDS_SCOREBOARD_PLAYERS_LIST_SUCCESS,
                            translation::java::COMMANDS_SCOREBOARD_PLAYERS_LIST_SUCCESS,
                            [
                                TextComponent::text(names.len().to_string()),
                                TextComponent::text(names.join(", ")),
                            ],
                        ),
                        false,
                    )
                    .await;
            }

            Ok(tracked.len() as i32)
        })
    }
}

struct PlayersListTargetExecutor;

impl CommandExecutor for PlayersListTargetExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let targets = EntityArgumentType::get_players(context, ARG_TARGETS).await?;
            if targets.is_empty() {
                return Err(INVALID_ENABLE_ERROR.create_without_context());
            }
            let player_name = &targets[0].gameprofile.name;

            let scoreboard = context.world().scoreboard.lock().await;
            let scores = scoreboard.list_scores_for_player(player_name);

            if scores.is_empty() {
                context
                    .source
                    .send_feedback(
                        TextComponent::translate_cross(
                            translation::java::COMMANDS_SCOREBOARD_PLAYERS_LIST_ENTITY_EMPTY,
                            translation::java::COMMANDS_SCOREBOARD_PLAYERS_LIST_ENTITY_EMPTY,
                            [TextComponent::text(player_name.clone())],
                        ),
                        false,
                    )
                    .await;
            } else {
                context
                    .source
                    .send_feedback(
                        TextComponent::translate_cross(
                            translation::java::COMMANDS_SCOREBOARD_PLAYERS_LIST_ENTITY_SUCCESS,
                            translation::java::COMMANDS_SCOREBOARD_PLAYERS_LIST_ENTITY_SUCCESS,
                            [
                                TextComponent::text(player_name.clone()),
                                TextComponent::text(scores.len().to_string()),
                            ],
                        ),
                        false,
                    )
                    .await;

                for (obj_name, value) in &scores {
                    context
                        .source
                        .send_feedback(
                            TextComponent::translate_cross(
                                translation::java::COMMANDS_SCOREBOARD_PLAYERS_LIST_ENTITY_ENTRY,
                                translation::java::COMMANDS_SCOREBOARD_PLAYERS_LIST_ENTITY_ENTRY,
                                [
                                    TextComponent::text(obj_name.to_string()),
                                    TextComponent::text(value.to_string()),
                                ],
                            ),
                            false,
                        )
                        .await;
                }
            }

            Ok(scores.len() as i32)
        })
    }
}

struct PlayersDisplayNameSetExecutor;

impl CommandExecutor for PlayersDisplayNameSetExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let targets = EntityArgumentType::get_players(context, ARG_TARGETS).await?;
            let objective_name = obj_name(context)?;
            let name = TextComponent::text(StringArgumentType::get(context, "name")?.to_string());

            let world = context.world();
            let mut scoreboard = world.scoreboard.lock().await;

            for player in &targets {
                scoreboard.set_score_display_name(
                    world,
                    &player.gameprofile.name,
                    objective_name,
                    Some(name.clone()),
                );
            }

            if targets.len() == 1 {
                context
                    .source
                    .send_feedback(
                        TextComponent::translate_cross(
                            translation::java::COMMANDS_SCOREBOARD_PLAYERS_DISPLAY_NAME_SET_SUCCESS_SINGLE,
                            translation::java::COMMANDS_SCOREBOARD_PLAYERS_DISPLAY_NAME_SET_SUCCESS_SINGLE,
                            [
                                name,
                                TextComponent::text(targets[0].gameprofile.name.clone()),
                                TextComponent::text(objective_name.to_string()),
                            ],
                        ),
                        true,
                    )
                    .await;
            } else {
                context
                    .source
                    .send_feedback(
                        TextComponent::translate_cross(
                            translation::java::COMMANDS_SCOREBOARD_PLAYERS_DISPLAY_NAME_SET_SUCCESS_MULTIPLE,
                            translation::java::COMMANDS_SCOREBOARD_PLAYERS_DISPLAY_NAME_SET_SUCCESS_MULTIPLE,
                            [
                                name,
                                TextComponent::text(targets.len().to_string()),
                                TextComponent::text(objective_name.to_string()),
                            ],
                        ),
                        true,
                    )
                    .await;
            }

            Ok(targets.len() as i32)
        })
    }
}

struct PlayersDisplayNameClearExecutor;

impl CommandExecutor for PlayersDisplayNameClearExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let targets = EntityArgumentType::get_players(context, ARG_TARGETS).await?;
            let objective_name = obj_name(context)?;

            let world = context.world();
            let mut scoreboard = world.scoreboard.lock().await;

            for player in &targets {
                scoreboard.set_score_display_name(
                    world,
                    &player.gameprofile.name,
                    objective_name,
                    None,
                );
            }

            if targets.len() == 1 {
                context
                    .source
                    .send_feedback(
                        TextComponent::translate_cross(
                            translation::java::COMMANDS_SCOREBOARD_PLAYERS_DISPLAY_NAME_CLEAR_SUCCESS_SINGLE,
                            translation::java::COMMANDS_SCOREBOARD_PLAYERS_DISPLAY_NAME_CLEAR_SUCCESS_SINGLE,
                            [
                                TextComponent::text(targets[0].gameprofile.name.clone()),
                                TextComponent::text(objective_name.to_string()),
                            ],
                        ),
                        true,
                    )
                    .await;
            } else {
                context
                    .source
                    .send_feedback(
                        TextComponent::translate_cross(
                            translation::java::COMMANDS_SCOREBOARD_PLAYERS_DISPLAY_NAME_CLEAR_SUCCESS_MULTIPLE,
                            translation::java::COMMANDS_SCOREBOARD_PLAYERS_DISPLAY_NAME_CLEAR_SUCCESS_MULTIPLE,
                            [
                                TextComponent::text(targets.len().to_string()),
                                TextComponent::text(objective_name.to_string()),
                            ],
                        ),
                        true,
                    )
                    .await;
            }

            Ok(targets.len() as i32)
        })
    }
}

const ARG_SOURCE: &str = "source";
const ARG_SOURCE_OBJECTIVE: &str = "sourceObjective";

fn operation_source_args(
    executor: impl CommandExecutor + 'static,
) -> crate::command::argument_builder::RequiredArgumentBuilder {
    argument(ARG_SOURCE, EntityArgumentType::Players)
        .then(argument(ARG_SOURCE_OBJECTIVE, ObjectiveArgumentType).executes(executor))
}

/// Applies an operation to all targets. `op` is `|a, b| -> i32`.
async fn apply_operation(
    context: &CommandContext<'_>,
    op: impl Fn(i32, i32) -> i32 + Send + Sync,
) -> Result<i32, crate::command::errors::command_syntax_error::CommandSyntaxError> {
    let targets = EntityArgumentType::get_players(context, ARG_TARGETS).await?;
    let objective_name = obj_name(context)?;
    let sources = EntityArgumentType::get_players(context, ARG_SOURCE).await?;
    let source_objective = obj_name(context)?;

    let world = context.world();
    let mut scoreboard = world.scoreboard.lock().await;

    let mut last_new_value = 0;
    for target in &targets {
        let target_name = &target.gameprofile.name;
        for source in &sources {
            let source_name = &source.gameprofile.name;
            let source_value = scoreboard
                .get_player_score_info(source_name, source_objective)
                .map_or(0, |s| s.value.0);
            let current = scoreboard
                .get_player_score_info(target_name, objective_name)
                .map_or(0, |s| s.value.0);
            let new_value = op(current, source_value);
            last_new_value = new_value;

            let score = ScoreboardScore {
                entity_name: Arc::from(target_name.as_str()),
                objective_name: Arc::from(objective_name),
                value: VarInt(new_value),
                display_name: None,
                number_format: None,
                locked: false,
            };
            scoreboard.update_score(world, score).await;
        }
    }

    if targets.len() == 1 {
        context
            .source
            .send_feedback(
                TextComponent::translate_cross(
                    translation::java::COMMANDS_SCOREBOARD_PLAYERS_OPERATION_SUCCESS_SINGLE,
                    translation::java::COMMANDS_SCOREBOARD_PLAYERS_OPERATION_SUCCESS_SINGLE,
                    [
                        TextComponent::text(objective_name.to_string()),
                        TextComponent::text(targets[0].gameprofile.name.clone()),
                        TextComponent::text(last_new_value.to_string()),
                    ],
                ),
                true,
            )
            .await;
    } else {
        context
            .source
            .send_feedback(
                TextComponent::translate_cross(
                    translation::java::COMMANDS_SCOREBOARD_PLAYERS_OPERATION_SUCCESS_MULTIPLE,
                    translation::java::COMMANDS_SCOREBOARD_PLAYERS_OPERATION_SUCCESS_MULTIPLE,
                    [
                        TextComponent::text(objective_name.to_string()),
                        TextComponent::text(targets.len().to_string()),
                    ],
                ),
                true,
            )
            .await;
    }

    Ok(targets.len() as i32)
}

macro_rules! make_operation_executor {
    ($name:ident, $op:expr) => {
        struct $name;
        impl CommandExecutor for $name {
            fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
                Box::pin(apply_operation(context, $op))
            }
        }
    };
}

make_operation_executor!(PlayersOperationAddExecutor, |a, b| a + b);
make_operation_executor!(PlayersOperationSubExecutor, |a, b| a - b);
make_operation_executor!(PlayersOperationMulExecutor, |a, b| a * b);
make_operation_executor!(PlayersOperationDivExecutor, |a, b| if b == 0 {
    0
} else {
    a / b
});
make_operation_executor!(PlayersOperationModExecutor, |a, b| if b == 0 {
    0
} else {
    a % b
});
make_operation_executor!(PlayersOperationAssignExecutor, |_a, b| b);
make_operation_executor!(PlayersOperationMinExecutor, Ord::min);
make_operation_executor!(PlayersOperationMaxExecutor, Ord::max);
make_operation_executor!(PlayersOperationIfExecutor, |_a, b| b);

struct ModifyObjectiveNumberFormatClearExecutor;
struct ModifyObjectiveNumberFormatBlankExecutor;
struct ModifyObjectiveNumberFormatFixedExecutor;
struct ModifyObjectiveNumberFormatStyledExecutor;

impl CommandExecutor for ModifyObjectiveNumberFormatClearExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let objective_name = obj_name(context)?;
            let world = context.world();
            let mut scoreboard = world.scoreboard.lock().await;

            if !scoreboard.set_objective_number_format(world, objective_name, None) {
                return Err(OBJECTIVE_NOT_FOUND_ERROR.create_without_context());
            }

            context
                .source
                .send_feedback(
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_SCOREBOARD_OBJECTIVES_MODIFY_OBJECTIVEFORMAT_CLEAR,
                        translation::java::COMMANDS_SCOREBOARD_OBJECTIVES_MODIFY_OBJECTIVEFORMAT_CLEAR,
                        [TextComponent::text(objective_name.to_string())],
                    ),
                    true,
                )
                .await;

            Ok(0)
        })
    }
}

impl CommandExecutor for ModifyObjectiveNumberFormatBlankExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let objective_name = obj_name(context)?;
            let world = context.world();
            let mut scoreboard = world.scoreboard.lock().await;

            if !scoreboard.set_objective_number_format(
                world,
                objective_name,
                Some(NumberFormat::Blank),
            ) {
                return Err(OBJECTIVE_NOT_FOUND_ERROR.create_without_context());
            }

            context
                .source
                .send_feedback(
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_SCOREBOARD_OBJECTIVES_MODIFY_OBJECTIVEFORMAT_SET,
                        translation::java::COMMANDS_SCOREBOARD_OBJECTIVES_MODIFY_OBJECTIVEFORMAT_SET,
                        [TextComponent::text(objective_name.to_string())],
                    ),
                    true,
                )
                .await;

            Ok(0)
        })
    }
}

impl CommandExecutor for ModifyObjectiveNumberFormatFixedExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let objective_name = obj_name(context)?;
            let contents =
                TextComponent::text(StringArgumentType::get(context, "contents")?.to_string());

            let world = context.world();
            let mut scoreboard = world.scoreboard.lock().await;

            if !scoreboard.set_objective_number_format(
                world,
                objective_name,
                Some(NumberFormat::Fixed(contents)),
            ) {
                return Err(OBJECTIVE_NOT_FOUND_ERROR.create_without_context());
            }

            context
                .source
                .send_feedback(
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_SCOREBOARD_OBJECTIVES_MODIFY_OBJECTIVEFORMAT_SET,
                        translation::java::COMMANDS_SCOREBOARD_OBJECTIVES_MODIFY_OBJECTIVEFORMAT_SET,
                        [TextComponent::text(objective_name.to_string())],
                    ),
                    true,
                )
                .await;

            Ok(0)
        })
    }
}

impl CommandExecutor for ModifyObjectiveNumberFormatStyledExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let objective_name = obj_name(context)?;
            let style_json = StringArgumentType::get(context, "style")?.to_string();
            let style: pumpkin_util::text::style::Style = serde_json::from_str(&style_json)
                .map_err(|_| STYLE_PARSE_ERROR.create_without_context())?;

            let world = context.world();
            let mut scoreboard = world.scoreboard.lock().await;

            if !scoreboard.set_objective_number_format(
                world,
                objective_name,
                Some(NumberFormat::Styled(style)),
            ) {
                return Err(OBJECTIVE_NOT_FOUND_ERROR.create_without_context());
            }

            context
                .source
                .send_feedback(
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_SCOREBOARD_OBJECTIVES_MODIFY_OBJECTIVEFORMAT_SET,
                        translation::java::COMMANDS_SCOREBOARD_OBJECTIVES_MODIFY_OBJECTIVEFORMAT_SET,
                        [TextComponent::text(objective_name.to_string())],
                    ),
                    true,
                )
                .await;

            Ok(0)
        })
    }
}

struct ModifyDisplayAutoUpdateExecutor;

impl CommandExecutor for ModifyDisplayAutoUpdateExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            use crate::command::argument_types::core::bool::BoolArgumentType;

            let objective_name = obj_name(context)?;
            let value = BoolArgumentType::get(context, "value")?;
            let world = context.world();
            let mut scoreboard = world.scoreboard.lock().await;

            if !scoreboard.set_display_auto_update(world, objective_name, value) {
                return Err(OBJECTIVE_NOT_FOUND_ERROR.create_without_context());
            }

            if value {
                context
                    .source
                    .send_feedback(
                        TextComponent::translate_cross(
                            translation::java::COMMANDS_SCOREBOARD_OBJECTIVES_MODIFY_DISPLAYAUTOUPDATE_ENABLE,
                            translation::java::COMMANDS_SCOREBOARD_OBJECTIVES_MODIFY_DISPLAYAUTOUPDATE_ENABLE,
                            [
                                TextComponent::text(objective_name.to_string()),
                                TextComponent::text(objective_name.to_string()),
                            ],
                        ),
                        true,
                    )
                    .await;
            } else {
                context
                    .source
                    .send_feedback(
                        TextComponent::translate_cross(
                            translation::java::COMMANDS_SCOREBOARD_OBJECTIVES_MODIFY_DISPLAYAUTOUPDATE_DISABLE,
                            translation::java::COMMANDS_SCOREBOARD_OBJECTIVES_MODIFY_DISPLAYAUTOUPDATE_DISABLE,
                            [TextComponent::text(objective_name.to_string())],
                        ),
                        true,
                    )
                    .await;
            }

            Ok(0)
        })
    }
}

struct PlayersDisplayNumberFormatClearExecutor;
struct PlayersDisplayNumberFormatBlankExecutor;
struct PlayersDisplayNumberFormatFixedExecutor;
struct PlayersDisplayNumberFormatStyledExecutor;

impl CommandExecutor for PlayersDisplayNumberFormatClearExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let targets = EntityArgumentType::get_players(context, ARG_TARGETS).await?;
            let objective_name = obj_name(context)?;

            let world = context.world();
            let mut scoreboard = world.scoreboard.lock().await;

            for player in &targets {
                scoreboard.set_score_number_format(
                    world,
                    &player.gameprofile.name,
                    objective_name,
                    None,
                );
            }

            if targets.len() == 1 {
                context
                    .source
                    .send_feedback(
                        TextComponent::translate_cross(
                            translation::java::COMMANDS_SCOREBOARD_PLAYERS_DISPLAY_NUMBERFORMAT_CLEAR_SUCCESS_SINGLE,
                            translation::java::COMMANDS_SCOREBOARD_PLAYERS_DISPLAY_NUMBERFORMAT_CLEAR_SUCCESS_SINGLE,
                            [
                                TextComponent::text(targets[0].gameprofile.name.clone()),
                                TextComponent::text(objective_name.to_string()),
                            ],
                        ),
                        true,
                    )
                    .await;
            } else {
                context
                    .source
                    .send_feedback(
                        TextComponent::translate_cross(
                            translation::java::COMMANDS_SCOREBOARD_PLAYERS_DISPLAY_NUMBERFORMAT_CLEAR_SUCCESS_MULTIPLE,
                            translation::java::COMMANDS_SCOREBOARD_PLAYERS_DISPLAY_NUMBERFORMAT_CLEAR_SUCCESS_MULTIPLE,
                            [
                                TextComponent::text(targets.len().to_string()),
                                TextComponent::text(objective_name.to_string()),
                            ],
                        ),
                        true,
                    )
                    .await;
            }

            Ok(targets.len() as i32)
        })
    }
}

impl CommandExecutor for PlayersDisplayNumberFormatBlankExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let targets = EntityArgumentType::get_players(context, ARG_TARGETS).await?;
            let objective_name = obj_name(context)?;

            let world = context.world();
            let mut scoreboard = world.scoreboard.lock().await;

            for player in &targets {
                scoreboard.set_score_number_format(
                    world,
                    &player.gameprofile.name,
                    objective_name,
                    Some(NumberFormat::Blank),
                );
            }

            if targets.len() == 1 {
                context
                    .source
                    .send_feedback(
                        TextComponent::translate_cross(
                            translation::java::COMMANDS_SCOREBOARD_PLAYERS_DISPLAY_NUMBERFORMAT_SET_SUCCESS_SINGLE,
                            translation::java::COMMANDS_SCOREBOARD_PLAYERS_DISPLAY_NUMBERFORMAT_SET_SUCCESS_SINGLE,
                            [
                                TextComponent::text(targets[0].gameprofile.name.clone()),
                                TextComponent::text(objective_name.to_string()),
                            ],
                        ),
                        true,
                    )
                    .await;
            } else {
                context
                    .source
                    .send_feedback(
                        TextComponent::translate_cross(
                            translation::java::COMMANDS_SCOREBOARD_PLAYERS_DISPLAY_NUMBERFORMAT_SET_SUCCESS_MULTIPLE,
                            translation::java::COMMANDS_SCOREBOARD_PLAYERS_DISPLAY_NUMBERFORMAT_SET_SUCCESS_MULTIPLE,
                            [
                                TextComponent::text(targets.len().to_string()),
                                TextComponent::text(objective_name.to_string()),
                            ],
                        ),
                        true,
                    )
                    .await;
            }

            Ok(targets.len() as i32)
        })
    }
}

impl CommandExecutor for PlayersDisplayNumberFormatFixedExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let targets = EntityArgumentType::get_players(context, ARG_TARGETS).await?;
            let objective_name = obj_name(context)?;
            let contents =
                TextComponent::text(StringArgumentType::get(context, "contents")?.to_string());

            let world = context.world();
            let mut scoreboard = world.scoreboard.lock().await;

            for player in &targets {
                scoreboard.set_score_number_format(
                    world,
                    &player.gameprofile.name,
                    objective_name,
                    Some(NumberFormat::Fixed(contents.clone())),
                );
            }

            if targets.len() == 1 {
                context
                    .source
                    .send_feedback(
                        TextComponent::translate_cross(
                            translation::java::COMMANDS_SCOREBOARD_PLAYERS_DISPLAY_NUMBERFORMAT_SET_SUCCESS_SINGLE,
                            translation::java::COMMANDS_SCOREBOARD_PLAYERS_DISPLAY_NUMBERFORMAT_SET_SUCCESS_SINGLE,
                            [
                                TextComponent::text(targets[0].gameprofile.name.clone()),
                                TextComponent::text(objective_name.to_string()),
                            ],
                        ),
                        true,
                    )
                    .await;
            } else {
                context
                    .source
                    .send_feedback(
                        TextComponent::translate_cross(
                            translation::java::COMMANDS_SCOREBOARD_PLAYERS_DISPLAY_NUMBERFORMAT_SET_SUCCESS_MULTIPLE,
                            translation::java::COMMANDS_SCOREBOARD_PLAYERS_DISPLAY_NUMBERFORMAT_SET_SUCCESS_MULTIPLE,
                            [
                                TextComponent::text(targets.len().to_string()),
                                TextComponent::text(objective_name.to_string()),
                            ],
                        ),
                        true,
                    )
                    .await;
            }

            Ok(targets.len() as i32)
        })
    }
}

impl CommandExecutor for PlayersDisplayNumberFormatStyledExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let targets = EntityArgumentType::get_players(context, ARG_TARGETS).await?;
            let objective_name = obj_name(context)?;
            let style_json = StringArgumentType::get(context, "style")?.to_string();
            let style: pumpkin_util::text::style::Style = serde_json::from_str(&style_json)
                .map_err(|_| STYLE_PARSE_ERROR.create_without_context())?;

            let world = context.world();
            let mut scoreboard = world.scoreboard.lock().await;

            for player in &targets {
                scoreboard.set_score_number_format(
                    world,
                    &player.gameprofile.name,
                    objective_name,
                    Some(NumberFormat::Styled(style.clone())),
                );
            }

            if targets.len() == 1 {
                context
                    .source
                    .send_feedback(
                        TextComponent::translate_cross(
                            translation::java::COMMANDS_SCOREBOARD_PLAYERS_DISPLAY_NUMBERFORMAT_SET_SUCCESS_SINGLE,
                            translation::java::COMMANDS_SCOREBOARD_PLAYERS_DISPLAY_NUMBERFORMAT_SET_SUCCESS_SINGLE,
                            [
                                TextComponent::text(targets[0].gameprofile.name.clone()),
                                TextComponent::text(objective_name.to_string()),
                            ],
                        ),
                        true,
                    )
                    .await;
            } else {
                context
                    .source
                    .send_feedback(
                        TextComponent::translate_cross(
                            translation::java::COMMANDS_SCOREBOARD_PLAYERS_DISPLAY_NUMBERFORMAT_SET_SUCCESS_MULTIPLE,
                            translation::java::COMMANDS_SCOREBOARD_PLAYERS_DISPLAY_NUMBERFORMAT_SET_SUCCESS_MULTIPLE,
                            [
                                TextComponent::text(targets.len().to_string()),
                                TextComponent::text(objective_name.to_string()),
                            ],
                        ),
                        true,
                    )
                    .await;
            }

            Ok(targets.len() as i32)
        })
    }
}

const fn display_slot_name(slot: ScoreboardDisplaySlot) -> &'static str {
    match slot {
        ScoreboardDisplaySlot::List => "list",
        ScoreboardDisplaySlot::Sidebar => "sidebar",
        ScoreboardDisplaySlot::BelowName => "below_name",
        ScoreboardDisplaySlot::TeamBlack => "sidebar.team.black",
        ScoreboardDisplaySlot::TeamDarkBlue => "sidebar.team.dark_blue",
        ScoreboardDisplaySlot::TeamDarkGreen => "sidebar.team.dark_green",
        ScoreboardDisplaySlot::TeamDarkAqua => "sidebar.team.dark_aqua",
        ScoreboardDisplaySlot::TeamDarkRed => "sidebar.team.dark_red",
        ScoreboardDisplaySlot::TeamDarkPurple => "sidebar.team.dark_purple",
        ScoreboardDisplaySlot::TeamGold => "sidebar.team.gold",
        ScoreboardDisplaySlot::TeamGray => "sidebar.team.gray",
        ScoreboardDisplaySlot::TeamDarkGray => "sidebar.team.dark_gray",
        ScoreboardDisplaySlot::TeamBlue => "sidebar.team.blue",
        ScoreboardDisplaySlot::TeamGreen => "sidebar.team.green",
        ScoreboardDisplaySlot::TeamAqua => "sidebar.team.aqua",
        ScoreboardDisplaySlot::TeamRed => "sidebar.team.red",
        ScoreboardDisplaySlot::TeamLightPurple => "sidebar.team.light_purple",
        ScoreboardDisplaySlot::TeamYellow => "sidebar.team.yellow",
        ScoreboardDisplaySlot::TeamWhite => "sidebar.team.white",
    }
}

#[allow(clippy::too_many_lines)]
pub fn register(dispatcher: &mut CommandDispatcher, registry: &mut PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    dispatcher.register(
        command("scoreboard", DESCRIPTION)
            .requires(PERMISSION)
            .then(
                literal("objectives")
                    .then(
                        literal("add").then(
                            argument(ARG_OBJECTIVE, StringArgumentType::SingleWord).then(
                                argument(ARG_CRITERION, StringArgumentType::SingleWord)
                                    .executes(ObjectivesAddExecutor {
                                        has_display_name: false,
                                    })
                                    .then(
                                        argument(
                                            ARG_DISPLAY_NAME,
                                            StringArgumentType::GreedyPhrase,
                                        )
                                        .executes(
                                            ObjectivesAddExecutor {
                                                has_display_name: true,
                                            },
                                        ),
                                    ),
                            ),
                        ),
                    )
                    .then(literal("list").executes(ObjectivesListExecutor))
                    .then(
                        literal("remove").then(
                            argument(ARG_OBJECTIVE, ObjectiveArgumentType)
                                .executes(ObjectivesRemoveExecutor),
                        ),
                    )
                    .then(
                        literal("modify").then(
                            argument(ARG_OBJECTIVE, ObjectiveArgumentType)
                                .then(
                                    literal("displayname").then(
                                        argument(
                                            ARG_DISPLAY_NAME,
                                            StringArgumentType::GreedyPhrase,
                                        )
                                        .executes(ModifyDisplayNameExecutor),
                                    ),
                                )
                                .then(
                                    literal("rendertype")
                                        .then(literal("hearts").executes(
                                            ModifyRenderTypeExecutor {
                                                render_type: RenderType::Hearts,
                                            },
                                        ))
                                        .then(literal("integer").executes(
                                            ModifyRenderTypeExecutor {
                                                render_type: RenderType::Integer,
                                            },
                                        )),
                                )
                                .then(
                                    literal("numberformat")
                                        .executes(ModifyObjectiveNumberFormatClearExecutor)
                                        .then(literal("blank").executes(
                                            ModifyObjectiveNumberFormatBlankExecutor,
                                        ))
                                        .then(
                                            literal("fixed").then(
                                                argument("contents", StringArgumentType::GreedyPhrase)
                                                    .executes(ModifyObjectiveNumberFormatFixedExecutor),
                                            ),
                                        )
                                        .then(
                                            literal("styled").then(
                                                argument("style", StringArgumentType::GreedyPhrase)
                                                    .executes(ModifyObjectiveNumberFormatStyledExecutor),
                                            ),
                                        ),
                                )
                                .then(
                                    literal("displayautoupdate").then(
                                        argument("value", crate::command::argument_types::core::bool::BoolArgumentType)
                                            .executes(ModifyDisplayAutoUpdateExecutor),
                                    ),
                                ),
                        ),
                    )
                    .then(
                        literal("setdisplay").then(
                            argument(ARG_SLOT, ScoreboardDisplaySlotArgumentType)
                                .executes(ObjectivesClearDisplayExecutor)
                                .then(
                                    argument(ARG_OBJECTIVE, ObjectiveArgumentType)
                                        .executes(ObjectivesSetDisplayExecutor),
                                ),
                        ),
                    ),
            )
            .then(
                literal("players")
                    .then(
                        literal("list").executes(PlayersListExecutor).then(
                            argument(ARG_TARGETS, EntityArgumentType::Players)
                                .executes(PlayersListTargetExecutor),
                        ),
                    )
                    .then(
                        literal("set").then(
                            argument(ARG_TARGETS, EntityArgumentType::Players).then(
                                argument(ARG_OBJECTIVE, ObjectiveArgumentType).then(
                                    argument(ARG_SCORE, IntegerArgumentType::any())
                                        .executes(PlayersSetExecutor),
                                ),
                            ),
                        ),
                    )
                    .then(
                        literal("get").then(
                            argument(ARG_TARGET, EntityArgumentType::Player).then(
                                argument(ARG_OBJECTIVE, ObjectiveArgumentType)
                                    .executes(PlayersGetExecutor),
                            ),
                        ),
                    )
                    .then(
                        literal("add").then(
                            argument(ARG_TARGETS, EntityArgumentType::Players).then(
                                argument(ARG_OBJECTIVE, ObjectiveArgumentType).then(
                                    argument(ARG_SCORE, IntegerArgumentType::with_min(0))
                                        .executes(PlayersAddExecutor),
                                ),
                            ),
                        ),
                    )
                    .then(
                        literal("remove").then(
                            argument(ARG_TARGETS, EntityArgumentType::Players).then(
                                argument(ARG_OBJECTIVE, ObjectiveArgumentType).then(
                                    argument(ARG_SCORE, IntegerArgumentType::with_min(0))
                                        .executes(PlayersRemoveExecutor),
                                ),
                            ),
                        ),
                    )
                    .then(
                        literal("reset").then(
                            argument(ARG_TARGETS, EntityArgumentType::Players)
                                .executes(PlayersResetAllExecutor)
                                .then(
                                    argument(ARG_OBJECTIVE, ObjectiveArgumentType)
                                        .executes(PlayersResetSingleExecutor),
                                ),
                        ),
                    )
                    .then(
                        literal("display").then(
                            literal("name").then(
                                argument(ARG_TARGETS, EntityArgumentType::Players).then(
                                    argument(ARG_OBJECTIVE, ObjectiveArgumentType)
                                        .executes(PlayersDisplayNameClearExecutor)
                                        .then(
                                            argument("name", StringArgumentType::GreedyPhrase)
                                                .executes(PlayersDisplayNameSetExecutor),
                                        ),
                                ),
                            ),
                        )
                        .then(
                            literal("numberformat").then(
                                argument(ARG_TARGETS, EntityArgumentType::Players).then(
                                    argument(ARG_OBJECTIVE, ObjectiveArgumentType)
                                        .executes(PlayersDisplayNumberFormatClearExecutor)
                                        .then(literal("blank").executes(
                                            PlayersDisplayNumberFormatBlankExecutor,
                                        ))
                                        .then(
                                            literal("fixed").then(
                                                argument("contents", StringArgumentType::GreedyPhrase)
                                                    .executes(PlayersDisplayNumberFormatFixedExecutor),
                                            ),
                                        )
                                        .then(
                                            literal("styled").then(
                                                argument("style", StringArgumentType::GreedyPhrase)
                                                    .executes(PlayersDisplayNumberFormatStyledExecutor),
                                            ),
                                        ),
                                ),
                            ),
                        ),
                    )
                    .then(
                        literal("operation").then(
                            argument(ARG_TARGETS, EntityArgumentType::Players).then(
                                argument(ARG_OBJECTIVE, ObjectiveArgumentType).then(
                                    literal("+=")
                                        .then(operation_source_args(PlayersOperationAddExecutor))
                                        .then(literal("-=").then(operation_source_args(
                                            PlayersOperationSubExecutor,
                                        )))
                                        .then(literal("*=").then(operation_source_args(
                                            PlayersOperationMulExecutor,
                                        )))
                                        .then(literal("/=").then(operation_source_args(
                                            PlayersOperationDivExecutor,
                                        )))
                                        .then(literal("%=").then(operation_source_args(
                                            PlayersOperationModExecutor,
                                        )))
                                        .then(literal("=").then(operation_source_args(
                                            PlayersOperationAssignExecutor,
                                        )))
                                        .then(literal("<").then(operation_source_args(
                                            PlayersOperationMinExecutor,
                                        )))
                                        .then(literal(">").then(operation_source_args(
                                            PlayersOperationMaxExecutor,
                                        )))
                                        .then(literal("?").then(operation_source_args(
                                            PlayersOperationIfExecutor,
                                        ))),
                                ),
                            ),
                        ),
                    )
                    .then(
                        literal("enable").then(
                            argument(ARG_TARGETS, EntityArgumentType::Players).then(
                                argument(ARG_OBJECTIVE, ObjectiveArgumentType)
                                    .executes(PlayersEnableExecutor),
                            ),
                        ),
                    ),
            ),
    );
}
