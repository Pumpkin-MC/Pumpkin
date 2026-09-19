use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::core::integer::IntegerArgumentType;
use crate::command::argument_types::core::string::StringArgumentType;
use crate::command::argument_types::objective::ObjectiveArgumentType;
use crate::command::argument_types::operation::{OperationArgumentType, ScoreboardOperation};
use crate::command::argument_types::score_holder::ScoreHolderArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::world::scoreboard::{Scoreboard, ScoreboardObjective, ScoreboardScore};
use pumpkin_data::translation;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::RenderType;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

const DESCRIPTION: &str = "Manages scoreboard objectives and players.";
const PERMISSION: &str = "minecraft:command.scoreboard";

const ARG_OBJECTIVE: &str = "objective";
const ARG_CRITERION: &str = "criterion";
const ARG_DISPLAY_NAME: &str = "display_name";
const ARG_TARGETS: &str = "targets";
const ARG_SCORE: &str = "score";
const ARG_OPERATION: &str = "operation";
const ARG_SOURCE_TARGETS: &str = "source_targets";
const ARG_SOURCE_OBJECTIVE: &str = "source_objective";

const NO_HOLDERS_ERROR: CommandErrorType<0> = CommandErrorType::new(
    "commands.scoreboard.players.nameNotFound",
    "commands.scoreboard.players.nameNotFound",
);

const OBJECTIVE_READ_ONLY_ERROR: CommandErrorType<1> = CommandErrorType::new(
    "commands.scoreboard.objectiveReadOnly",
    "commands.scoreboard.objectiveReadOnly",
);

const NO_SCORE_ERROR: CommandErrorType<2> = CommandErrorType::new(
    "commands.scoreboard.players.get.null",
    "commands.scoreboard.players.get.null",
);

const OBJECTIVE_NOT_FOUND_ERROR: CommandErrorType<1> = CommandErrorType::new(
    "commands.scoreboard.objectiveNotFound",
    "commands.scoreboard.objectiveNotFound",
);

const DUPLICATE_OBJECTIVE_ERROR: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_SCOREBOARD_OBJECTIVES_ADD_DUPLICATE,
    translation::java::COMMANDS_SCOREBOARD_OBJECTIVES_ADD_DUPLICATE,
);

const INVALID_ENABLE_ERROR: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_SCOREBOARD_PLAYERS_ENABLE_INVALID,
    translation::java::COMMANDS_SCOREBOARD_PLAYERS_ENABLE_INVALID,
);

const FAILED_ENABLE_ERROR: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_SCOREBOARD_PLAYERS_ENABLE_FAILED,
    translation::java::COMMANDS_SCOREBOARD_PLAYERS_ENABLE_FAILED,
);

struct ObjectivesAddExecutor {
    has_display_name: bool,
}

impl CommandExecutor for ObjectivesAddExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let objective_name = StringArgumentType::get(context, ARG_OBJECTIVE)?;
        let criterion = StringArgumentType::get(context, ARG_CRITERION)?;

        let display_name = if self.has_display_name {
            TextComponent::text(StringArgumentType::get(context, ARG_DISPLAY_NAME)?.to_string())
        } else {
            TextComponent::text(objective_name.to_string())
        };

        let world = context.world().clone();
        let objective_name_owned = objective_name.to_string();
        let criterion_owned = criterion.to_string();
        let display_name_clone = display_name;

        let mut scoreboard = world
            .scoreboard
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        if scoreboard
            .get_objectives()
            .contains_key(&objective_name_owned)
        {
            return Err(DUPLICATE_OBJECTIVE_ERROR.create_without_context());
        }

        let new_objective = ScoreboardObjective::new(
            &objective_name_owned,
            display_name_clone.clone(),
            RenderType::Integer,
            None,
            &criterion_owned,
        );

        scoreboard.add_objective(&world, new_objective);

        context.source.send_feedback(
            TextComponent::translate_cross(
                translation::java::COMMANDS_SCOREBOARD_OBJECTIVES_ADD_SUCCESS,
                translation::java::COMMANDS_SCOREBOARD_OBJECTIVES_ADD_SUCCESS,
                [display_name_clone],
            ),
            true,
        );

        Ok(1)
    }
}

struct PlayersEnableExecutor;

impl CommandExecutor for PlayersEnableExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let holders = holders_or_error(context)?;
        let objective_name = ObjectiveArgumentType::get(context, ARG_OBJECTIVE)?;

        let world = context.world().clone();
        let objective_name_owned = objective_name.to_string();

        let mut scoreboard = world
            .scoreboard
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        let Some(objective) = scoreboard.get_objectives().get(&objective_name_owned) else {
            return Err(INVALID_ENABLE_ERROR.create_without_context());
        };

        if objective.criterion != "trigger" {
            return Err(INVALID_ENABLE_ERROR.create_without_context());
        }

        let objective_display_name = objective.display_name.clone();

        let mut enabled_count = 0;
        for player_name in &holders {
            let current_score = scoreboard
                .get_scores()
                .get(&objective_name_owned)
                .and_then(|m| m.get(player_name));

            let is_already_enabled = current_score.is_some_and(|s| !s.locked);

            if !is_already_enabled {
                let value = current_score.map_or(0, |s| s.value.0);
                let display_name = current_score.and_then(|s| s.display_name.clone());
                let number_format = current_score.and_then(|s| s.number_format.clone());

                let updated_score = ScoreboardScore {
                    entity_name: player_name.clone(),
                    objective_name: objective_name_owned.clone(),
                    value: VarInt(value),
                    display_name,
                    number_format,
                    locked: false,
                };

                scoreboard.update_score(&world, updated_score);
                enabled_count += 1;
            }
        }

        if enabled_count == 0 {
            return Err(FAILED_ENABLE_ERROR.create_without_context());
        }

        let (holder, single) = holder_component(&holders);
        let msg = if single {
            TextComponent::translate_cross(
                translation::java::COMMANDS_SCOREBOARD_PLAYERS_ENABLE_SUCCESS_SINGLE,
                translation::java::COMMANDS_SCOREBOARD_PLAYERS_ENABLE_SUCCESS_SINGLE,
                [objective_display_name, holder],
            )
        } else {
            TextComponent::translate_cross(
                translation::java::COMMANDS_SCOREBOARD_PLAYERS_ENABLE_SUCCESS_MULTIPLE,
                translation::java::COMMANDS_SCOREBOARD_PLAYERS_ENABLE_SUCCESS_MULTIPLE,
                [objective_display_name, holder],
            )
        };

        context.source.send_feedback(msg, true);

        Ok(1)
    }
}

struct ObjectivesRemoveExecutor;

impl CommandExecutor for ObjectivesRemoveExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let objective_name = ObjectiveArgumentType::get(context, ARG_OBJECTIVE)?;

        let world = context.world().clone();
        let objective_name_owned = objective_name.to_string();

        let mut scoreboard = world
            .scoreboard
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        let Some(objective) = scoreboard.get_objectives().get(&objective_name_owned) else {
            return Err(INVALID_ENABLE_ERROR.create_without_context());
        };

        let display_name = objective.display_name.clone();

        scoreboard.remove_objective(&world, &objective_name_owned);

        context.source.send_feedback(
            TextComponent::translate_cross(
                translation::java::COMMANDS_SCOREBOARD_OBJECTIVES_REMOVE_SUCCESS,
                translation::bedrock::COMMANDS_SCOREBOARD_OBJECTIVES_REMOVE_SUCCESS,
                [display_name],
            ),
            true,
        );

        Ok(1)
    }
}

/// Resolves an objective's display name, or reports that it does not exist.
fn objective_or_error(
    scoreboard: &Scoreboard,
    name: &str,
) -> Result<TextComponent, CommandSyntaxError> {
    scoreboard
        .get_objectives()
        .get(name)
        .map(|objective| objective.display_name.clone())
        .ok_or_else(|| {
            OBJECTIVE_NOT_FOUND_ERROR.create_without_context(TextComponent::text(name.to_string()))
        })
}

/// Resolves an objective's display name and rejects objectives that do not
/// accept writes, such as the ones backed by a stat.
fn writable_objective_or_error(
    scoreboard: &Scoreboard,
    name: &str,
) -> Result<TextComponent, CommandSyntaxError> {
    let objective = scoreboard.get_objectives().get(name).ok_or_else(|| {
        OBJECTIVE_NOT_FOUND_ERROR.create_without_context(TextComponent::text(name.to_string()))
    })?;
    // Compound criteria are written as a `+` separated list and are writable
    // while every part allows writes, matching vanilla.
    let writable = objective
        .criterion
        .split('+')
        .all(|criterion| criterion == "dummy" || criterion == "trigger");
    if !writable {
        return Err(
            OBJECTIVE_READ_ONLY_ERROR.create_without_context(TextComponent::text(name.to_string()))
        );
    }
    Ok(objective.display_name.clone())
}

fn holders_or_error(context: &CommandContext) -> Result<Vec<String>, CommandSyntaxError> {
    let holders = ScoreHolderArgumentType::get_score_holder_names(context, ARG_TARGETS)?;
    if holders.is_empty() {
        return Err(NO_HOLDERS_ERROR.create_without_context());
    }
    Ok(holders)
}

/// Vanilla prints the holder name for a single target and a count otherwise.
fn holder_component(holders: &[String]) -> (TextComponent, bool) {
    if holders.len() == 1 {
        (TextComponent::text(holders[0].clone()), true)
    } else {
        (TextComponent::text(holders.len().to_string()), false)
    }
}

struct PlayersSetExecutor;

impl CommandExecutor for PlayersSetExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let holders = holders_or_error(context)?;
        let objective_name = ObjectiveArgumentType::get(context, ARG_OBJECTIVE)?.to_string();
        let value = *context.get_argument::<i32>(ARG_SCORE)?;

        let world = context.world().clone();
        let mut scoreboard = world
            .scoreboard
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        let display_name = writable_objective_or_error(&scoreboard, &objective_name)?;
        for holder in &holders {
            scoreboard.set_score_value(&world, holder.clone(), objective_name.clone(), value);
        }
        drop(scoreboard);

        let (holder, single) = holder_component(&holders);
        let value_component = TextComponent::text(value.to_string());
        context.source.send_feedback(
            if single {
                TextComponent::translate_cross(
                    translation::java::COMMANDS_SCOREBOARD_PLAYERS_SET_SUCCESS_SINGLE,
                    translation::java::COMMANDS_SCOREBOARD_PLAYERS_SET_SUCCESS_SINGLE,
                    [display_name, holder, value_component],
                )
            } else {
                TextComponent::translate_cross(
                    translation::java::COMMANDS_SCOREBOARD_PLAYERS_SET_SUCCESS_MULTIPLE,
                    translation::java::COMMANDS_SCOREBOARD_PLAYERS_SET_SUCCESS_MULTIPLE,
                    [display_name, holder, value_component],
                )
            },
            true,
        );

        Ok((holders.len() as i32).wrapping_mul(value))
    }
}

struct PlayersGetExecutor;

impl CommandExecutor for PlayersGetExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let holders = holders_or_error(context)?;
        let objective_name = ObjectiveArgumentType::get(context, ARG_OBJECTIVE)?.to_string();

        let world = context.world().clone();
        let scoreboard = world
            .scoreboard
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        let display_name = objective_or_error(&scoreboard, &objective_name)?;
        if holders.len() != 1 {
            return Err(NO_HOLDERS_ERROR.create_without_context());
        }
        let holder = holders[0].clone();
        let Some(value) = scoreboard.get_score_value(&holder, &objective_name) else {
            return Err(NO_SCORE_ERROR.create_without_context(
                TextComponent::text(holder),
                TextComponent::text(objective_name),
            ));
        };

        context.source.send_feedback(
            TextComponent::translate_cross(
                translation::java::COMMANDS_SCOREBOARD_PLAYERS_GET_SUCCESS,
                translation::java::COMMANDS_SCOREBOARD_PLAYERS_GET_SUCCESS,
                [
                    TextComponent::text(holders[0].clone()),
                    display_name,
                    TextComponent::text(value.to_string()),
                ],
            ),
            true,
        );

        Ok(value)
    }
}

struct PlayersAddExecutor {
    remove: bool,
}

impl CommandExecutor for PlayersAddExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let holders = holders_or_error(context)?;
        let objective_name = ObjectiveArgumentType::get(context, ARG_OBJECTIVE)?.to_string();
        let value = *context.get_argument::<i32>(ARG_SCORE)?;
        let delta = if self.remove {
            value.wrapping_neg()
        } else {
            value
        };

        let world = context.world().clone();
        let mut scoreboard = world
            .scoreboard
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        let display_name = writable_objective_or_error(&scoreboard, &objective_name)?;
        let mut result: i32 = 0;
        for holder in &holders {
            result = result.wrapping_add(scoreboard.add_score(
                &world,
                holder.clone(),
                objective_name.clone(),
                delta,
            ));
        }
        drop(scoreboard);

        let (single_key, multiple_key) = if self.remove {
            (
                translation::java::COMMANDS_SCOREBOARD_PLAYERS_REMOVE_SUCCESS_SINGLE,
                translation::java::COMMANDS_SCOREBOARD_PLAYERS_REMOVE_SUCCESS_MULTIPLE,
            )
        } else {
            (
                translation::java::COMMANDS_SCOREBOARD_PLAYERS_ADD_SUCCESS_SINGLE,
                translation::java::COMMANDS_SCOREBOARD_PLAYERS_ADD_SUCCESS_MULTIPLE,
            )
        };

        let (holder, single) = holder_component(&holders);
        let value_component = TextComponent::text(value.to_string());
        context.source.send_feedback(
            if single {
                TextComponent::translate_cross(
                    single_key,
                    single_key,
                    [
                        value_component,
                        display_name,
                        holder,
                        TextComponent::text(result.to_string()),
                    ],
                )
            } else {
                TextComponent::translate_cross(
                    multiple_key,
                    multiple_key,
                    [value_component, display_name, holder],
                )
            },
            true,
        );

        Ok(result)
    }
}

struct PlayersResetExecutor {
    has_objective: bool,
}

impl CommandExecutor for PlayersResetExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let holders = holders_or_error(context)?;
        let world = context.world().clone();
        let mut scoreboard = world
            .scoreboard
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        if self.has_objective {
            let objective_name = ObjectiveArgumentType::get(context, ARG_OBJECTIVE)?.to_string();
            let display_name = objective_or_error(&scoreboard, &objective_name)?;
            for holder in &holders {
                scoreboard.remove_score(&world, holder, &objective_name);
            }
            drop(scoreboard);
            let (holder, single) = holder_component(&holders);
            context.source.send_feedback(
                if single {
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_SCOREBOARD_PLAYERS_RESET_SPECIFIC_SINGLE,
                        translation::java::COMMANDS_SCOREBOARD_PLAYERS_RESET_SPECIFIC_SINGLE,
                        [display_name, holder],
                    )
                } else {
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_SCOREBOARD_PLAYERS_RESET_SPECIFIC_MULTIPLE,
                        translation::java::COMMANDS_SCOREBOARD_PLAYERS_RESET_SPECIFIC_MULTIPLE,
                        [display_name, holder],
                    )
                },
                true,
            );
        } else {
            for holder in &holders {
                scoreboard.reset_scores_for_entity(&world, holder);
            }
            drop(scoreboard);
            let (holder, single) = holder_component(&holders);
            context.source.send_feedback(
                if single {
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_SCOREBOARD_PLAYERS_RESET_ALL_SINGLE,
                        translation::java::COMMANDS_SCOREBOARD_PLAYERS_RESET_ALL_SINGLE,
                        [holder],
                    )
                } else {
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_SCOREBOARD_PLAYERS_RESET_ALL_MULTIPLE,
                        translation::java::COMMANDS_SCOREBOARD_PLAYERS_RESET_ALL_MULTIPLE,
                        [holder],
                    )
                },
                true,
            );
        }

        Ok(holders.len() as i32)
    }
}

struct PlayersOperationExecutor;

impl CommandExecutor for PlayersOperationExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let holders = holders_or_error(context)?;
        let objective_name = ObjectiveArgumentType::get(context, ARG_OBJECTIVE)?.to_string();
        let operation = *context.get_argument::<ScoreboardOperation>(ARG_OPERATION)?;
        let sources = ScoreHolderArgumentType::get_score_holder_names(context, ARG_SOURCE_TARGETS)?;
        let source_objective =
            ObjectiveArgumentType::get(context, ARG_SOURCE_OBJECTIVE)?.to_string();

        if sources.is_empty() {
            return Err(NO_HOLDERS_ERROR.create_without_context());
        }

        let world = context.world().clone();
        let mut scoreboard = world
            .scoreboard
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        let display_name = writable_objective_or_error(&scoreboard, &objective_name)?;
        if operation == ScoreboardOperation::Swap {
            writable_objective_or_error(&scoreboard, &source_objective)?;
        } else {
            objective_or_error(&scoreboard, &source_objective)?;
        }

        let mut result: i32 = 0;
        for holder in &holders {
            for source in &sources {
                let source_value = scoreboard
                    .get_score_value(source, &source_objective)
                    .unwrap_or(0);
                let target_value = scoreboard
                    .get_score_value(holder, &objective_name)
                    .unwrap_or(0);
                if operation == ScoreboardOperation::Swap {
                    scoreboard.set_score_value(
                        &world,
                        holder.clone(),
                        objective_name.clone(),
                        source_value,
                    );
                    scoreboard.set_score_value(
                        &world,
                        source.clone(),
                        source_objective.clone(),
                        target_value,
                    );
                } else {
                    let new_value = operation.apply(target_value, source_value);
                    scoreboard.set_score_value(
                        &world,
                        holder.clone(),
                        objective_name.clone(),
                        new_value,
                    );
                }
            }
            // Vanilla counts each holder once, with the score left after all of
            // its source operations.
            result = result.wrapping_add(
                scoreboard
                    .get_score_value(holder, &objective_name)
                    .unwrap_or(0),
            );
        }
        drop(scoreboard);

        let (holder, single) = holder_component(&holders);
        let result_component = TextComponent::text(result.to_string());
        context.source.send_feedback(
            if single {
                TextComponent::translate_cross(
                    translation::java::COMMANDS_SCOREBOARD_PLAYERS_OPERATION_SUCCESS_SINGLE,
                    translation::java::COMMANDS_SCOREBOARD_PLAYERS_OPERATION_SUCCESS_SINGLE,
                    [display_name, holder, result_component],
                )
            } else {
                TextComponent::translate_cross(
                    translation::java::COMMANDS_SCOREBOARD_PLAYERS_OPERATION_SUCCESS_MULTIPLE,
                    translation::java::COMMANDS_SCOREBOARD_PLAYERS_OPERATION_SUCCESS_MULTIPLE,
                    [display_name, holder, result_component],
                )
            },
            true,
        );

        Ok(result)
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "the scoreboard command tree mirrors the vanilla structure"
)]
pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
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
                    .then(
                        literal("remove").then(
                            argument(ARG_OBJECTIVE, ObjectiveArgumentType)
                                .executes(ObjectivesRemoveExecutor),
                        ),
                    ),
            )
            .then(
                literal("players")
                    .then(
                        literal("set").then(
                            argument(ARG_TARGETS, ScoreHolderArgumentType).then(
                                argument(ARG_OBJECTIVE, ObjectiveArgumentType).then(
                                    argument(ARG_SCORE, IntegerArgumentType::any())
                                        .executes(PlayersSetExecutor),
                                ),
                            ),
                        ),
                    )
                    .then(
                        literal("get").then(
                            argument(ARG_TARGETS, ScoreHolderArgumentType).then(
                                argument(ARG_OBJECTIVE, ObjectiveArgumentType)
                                    .executes(PlayersGetExecutor),
                            ),
                        ),
                    )
                    .then(
                        literal("add").then(
                            argument(ARG_TARGETS, ScoreHolderArgumentType).then(
                                argument(ARG_OBJECTIVE, ObjectiveArgumentType).then(
                                    argument(ARG_SCORE, IntegerArgumentType::with_min(0))
                                        .executes(PlayersAddExecutor { remove: false }),
                                ),
                            ),
                        ),
                    )
                    .then(
                        literal("remove").then(
                            argument(ARG_TARGETS, ScoreHolderArgumentType).then(
                                argument(ARG_OBJECTIVE, ObjectiveArgumentType).then(
                                    argument(ARG_SCORE, IntegerArgumentType::with_min(0))
                                        .executes(PlayersAddExecutor { remove: true }),
                                ),
                            ),
                        ),
                    )
                    .then(
                        literal("reset").then(
                            argument(ARG_TARGETS, ScoreHolderArgumentType)
                                .executes(PlayersResetExecutor {
                                    has_objective: false,
                                })
                                .then(argument(ARG_OBJECTIVE, ObjectiveArgumentType).executes(
                                    PlayersResetExecutor {
                                        has_objective: true,
                                    },
                                )),
                        ),
                    )
                    .then(
                        literal("operation").then(
                            argument(ARG_TARGETS, ScoreHolderArgumentType).then(
                                argument(ARG_OBJECTIVE, ObjectiveArgumentType).then(
                                    argument(ARG_OPERATION, OperationArgumentType).then(
                                        argument(ARG_SOURCE_TARGETS, ScoreHolderArgumentType).then(
                                            argument(ARG_SOURCE_OBJECTIVE, ObjectiveArgumentType)
                                                .executes(PlayersOperationExecutor),
                                        ),
                                    ),
                                ),
                            ),
                        ),
                    )
                    .then(
                        literal("enable").then(
                            argument(ARG_TARGETS, ScoreHolderArgumentType).then(
                                argument(ARG_OBJECTIVE, ObjectiveArgumentType)
                                    .executes(PlayersEnableExecutor),
                            ),
                        ),
                    ),
            ),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn holder_component_uses_the_name_for_one_and_a_count_for_many() {
        let one = vec!["detectGen".to_string()];
        let (holder, single) = holder_component(&one);
        assert!(single);
        assert!(holder.to_pretty_console().contains("detectGen"));

        let many = vec!["detectGen".to_string(), "Coal".to_string()];
        let (holder, single) = holder_component(&many);
        assert!(!single);
        assert!(holder.to_pretty_console().contains('2'));
    }
}
