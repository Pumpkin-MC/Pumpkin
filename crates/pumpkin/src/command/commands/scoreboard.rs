use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::core::integer::IntegerArgumentType;
use crate::command::argument_types::core::string::StringArgumentType;
use crate::command::argument_types::objective::ObjectiveArgumentType;
use crate::command::argument_types::operation::{OperationArgumentType, ScoreboardOperation};
use crate::command::argument_types::score_holder::{ResolvedScoreHolder, ScoreHolderArgumentType};
use crate::command::context::command_context::CommandContext;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::world::scoreboard::{
    Scoreboard, ScoreboardObjective, ScoreboardScore, ScoreboardTarget,
};
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

const OBJECTIVE_READ_ONLY_ERROR: CommandErrorType<1> = CommandErrorType::new(
    translation::java::ARGUMENTS_OBJECTIVE_READONLY,
    translation::java::ARGUMENTS_OBJECTIVE_READONLY,
);

const NO_SCORE_ERROR: CommandErrorType<2> = CommandErrorType::new(
    translation::java::COMMANDS_SCOREBOARD_PLAYERS_GET_NULL,
    translation::java::COMMANDS_SCOREBOARD_PLAYERS_GET_NULL,
);

const OBJECTIVE_NOT_FOUND_ERROR: CommandErrorType<1> = CommandErrorType::new(
    translation::java::ARGUMENTS_OBJECTIVE_NOTFOUND,
    translation::java::ARGUMENTS_OBJECTIVE_NOTFOUND,
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
        let holders = ScoreHolderArgumentType::get_score_holders(context, ARG_TARGETS)?;
        let objective_name = ObjectiveArgumentType::get(context, ARG_OBJECTIVE)?;

        let world = context.world().clone();
        let objective_name_owned = objective_name.to_string();

        let mut scoreboard = world
            .scoreboard
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        let (enabled_count, msg) =
            Self::enable(&mut scoreboard, &world, &holders, &objective_name_owned)?;
        drop(scoreboard);
        context.source.send_feedback(msg, true);
        Ok(enabled_count)
    }
}

impl PlayersEnableExecutor {
    /// Unlocks only disabled trigger scores, preserving their values and formatting.
    fn enable(
        scoreboard: &mut Scoreboard,
        target: &impl ScoreboardTarget,
        holders: &[ResolvedScoreHolder],
        objective_name: &str,
    ) -> Result<(i32, TextComponent), CommandSyntaxError> {
        let objective = objective_or_error(scoreboard, objective_name)?;

        if objective.criterion != "trigger" {
            return Err(INVALID_ENABLE_ERROR.create_without_context());
        }

        let objective_display_name = objective.display_name.clone();
        let mut enabled_holders = Vec::new();
        for holder in holders {
            let player_name = &holder.name;
            let current_score = scoreboard.get_score(player_name, objective_name);

            let is_already_enabled = current_score.is_some_and(|s| !s.locked);

            if !is_already_enabled {
                let value = current_score.map_or(0, |s| s.value.0);
                let display_name = current_score.and_then(|s| s.display_name.clone());
                let number_format = current_score.and_then(|s| s.number_format.clone());

                let updated_score = ScoreboardScore {
                    entity_name: player_name.clone(),
                    objective_name: objective_name.to_string(),
                    value: VarInt(value),
                    display_name,
                    number_format,
                    locked: false,
                };

                scoreboard.update_score(target, updated_score);
                enabled_holders.push(holder);
            }
        }

        if enabled_holders.is_empty() {
            return Err(FAILED_ENABLE_ERROR.create_without_context());
        }

        let enabled_count = enabled_holders.len() as i32;
        let msg = if let [holder] = enabled_holders.as_slice() {
            TextComponent::translate_cross(
                translation::java::COMMANDS_SCOREBOARD_PLAYERS_ENABLE_SUCCESS_SINGLE,
                translation::java::COMMANDS_SCOREBOARD_PLAYERS_ENABLE_SUCCESS_SINGLE,
                [objective_display_name, holder.display_name.clone()],
            )
        } else {
            TextComponent::translate_cross(
                translation::java::COMMANDS_SCOREBOARD_PLAYERS_ENABLE_SUCCESS_MULTIPLE,
                translation::java::COMMANDS_SCOREBOARD_PLAYERS_ENABLE_SUCCESS_MULTIPLE,
                [
                    objective_display_name,
                    TextComponent::text(enabled_count.to_string()),
                ],
            )
        };

        Ok((enabled_count, msg))
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

        let display_name = objective_or_error(&scoreboard, &objective_name_owned)?
            .display_name
            .clone();

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

fn objective_or_error<'a>(
    scoreboard: &'a Scoreboard,
    name: &str,
) -> Result<&'a ScoreboardObjective, CommandSyntaxError> {
    scoreboard.get_objective(name).ok_or_else(|| {
        OBJECTIVE_NOT_FOUND_ERROR.create_without_context(TextComponent::text(name.to_string()))
    })
}

fn writable_objective_or_error<'a>(
    scoreboard: &'a Scoreboard,
    name: &str,
) -> Result<&'a ScoreboardObjective, CommandSyntaxError> {
    let objective = objective_or_error(scoreboard, name)?;
    // These are the six criteria registered as read-only by vanilla ObjectiveCriteria.
    let read_only = matches!(
        objective.criterion.as_str(),
        "health" | "food" | "air" | "armor" | "xp" | "level"
    );
    if read_only {
        return Err(
            OBJECTIVE_READ_ONLY_ERROR.create_without_context(TextComponent::text(name.to_string()))
        );
    }
    Ok(objective)
}

/// Vanilla prints the holder name for a single target and a count otherwise.
fn holder_component(holders: &[ResolvedScoreHolder]) -> (TextComponent, bool) {
    if holders.len() == 1 {
        (holders[0].display_name.clone(), true)
    } else {
        (TextComponent::text(holders.len().to_string()), false)
    }
}

struct PlayersSetExecutor;

impl CommandExecutor for PlayersSetExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let holders = ScoreHolderArgumentType::get_score_holders(context, ARG_TARGETS)?;
        let objective_name = ObjectiveArgumentType::get(context, ARG_OBJECTIVE)?.to_string();
        let value = IntegerArgumentType::get(context, ARG_SCORE)?;

        let world = context.world().clone();
        let mut scoreboard = world
            .scoreboard
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        let display_name = writable_objective_or_error(&scoreboard, &objective_name)?
            .display_name
            .clone();
        for holder in &holders {
            scoreboard.set_score_value(&world, holder.name.clone(), objective_name.clone(), value);
        }
        drop(scoreboard);

        // Java 26.3 uses the nonzero result count for `players set` feedback.
        let (holder_text, single) = holder_component(if value == 0 { &[] } else { &holders });
        let value_component = TextComponent::text(value.to_string());
        context.source.send_feedback(
            if single {
                TextComponent::translate_cross(
                    translation::java::COMMANDS_SCOREBOARD_PLAYERS_SET_SUCCESS_SINGLE,
                    translation::java::COMMANDS_SCOREBOARD_PLAYERS_SET_SUCCESS_SINGLE,
                    [display_name, holder_text, value_component],
                )
            } else {
                TextComponent::translate_cross(
                    translation::java::COMMANDS_SCOREBOARD_PLAYERS_SET_SUCCESS_MULTIPLE,
                    translation::java::COMMANDS_SCOREBOARD_PLAYERS_SET_SUCCESS_MULTIPLE,
                    [display_name, holder_text, value_component],
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
        let holder = ScoreHolderArgumentType::get_score_holder(context, ARG_TARGETS)?;
        let objective_name = ObjectiveArgumentType::get(context, ARG_OBJECTIVE)?.to_string();

        let world = context.world().clone();
        let scoreboard = world
            .scoreboard
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        let (value, feedback) = Self::get(&scoreboard, holder, &objective_name)?;
        drop(scoreboard);
        context.source.send_feedback(feedback, false);
        Ok(value)
    }
}

impl PlayersGetExecutor {
    /// Reads an existing score and builds the non-broadcast query feedback.
    fn get(
        scoreboard: &Scoreboard,
        holder: ResolvedScoreHolder,
        objective_name: &str,
    ) -> Result<(i32, TextComponent), CommandSyntaxError> {
        let objective = objective_or_error(scoreboard, objective_name)?;
        let Some(value) = scoreboard.get_score_value(&holder.name, objective_name) else {
            return Err(NO_SCORE_ERROR.create_without_context(
                TextComponent::text(objective_name.to_string()),
                holder.display_name,
            ));
        };

        Ok((
            value,
            TextComponent::translate_cross(
                translation::java::COMMANDS_SCOREBOARD_PLAYERS_GET_SUCCESS,
                translation::java::COMMANDS_SCOREBOARD_PLAYERS_GET_SUCCESS,
                [
                    holder.display_name,
                    TextComponent::text(value.to_string()),
                    objective.display_name.clone(),
                ],
            ),
        ))
    }
}

struct PlayersAddRemoveExecutor {
    remove: bool,
}

impl CommandExecutor for PlayersAddRemoveExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let holders = ScoreHolderArgumentType::get_score_holders(context, ARG_TARGETS)?;
        let objective_name = ObjectiveArgumentType::get(context, ARG_OBJECTIVE)?.to_string();
        let value = IntegerArgumentType::get(context, ARG_SCORE)?;
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

        let display_name = writable_objective_or_error(&scoreboard, &objective_name)?
            .display_name
            .clone();
        let mut result: i32 = 0;
        for holder in &holders {
            result = result.wrapping_add(scoreboard.add_score(
                &world,
                holder.name.clone(),
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

        let (holder_text, single) = holder_component(&holders);
        let value_component = TextComponent::text(value.to_string());
        context.source.send_feedback(
            if single {
                TextComponent::translate_cross(
                    single_key,
                    single_key,
                    [
                        value_component,
                        display_name,
                        holder_text,
                        TextComponent::text(result.to_string()),
                    ],
                )
            } else {
                TextComponent::translate_cross(
                    multiple_key,
                    multiple_key,
                    [value_component, display_name, holder_text],
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
        let holders = ScoreHolderArgumentType::get_score_holders(context, ARG_TARGETS)?;
        let world = context.world().clone();
        let mut scoreboard = world
            .scoreboard
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        if self.has_objective {
            let objective_name = ObjectiveArgumentType::get(context, ARG_OBJECTIVE)?.to_string();
            let display_name = objective_or_error(&scoreboard, &objective_name)?
                .display_name
                .clone();
            for holder in &holders {
                scoreboard.remove_score(&world, &holder.name, &objective_name);
            }
            drop(scoreboard);
            let (holder_text, single) = holder_component(&holders);
            context.source.send_feedback(
                if single {
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_SCOREBOARD_PLAYERS_RESET_SPECIFIC_SINGLE,
                        translation::java::COMMANDS_SCOREBOARD_PLAYERS_RESET_SPECIFIC_SINGLE,
                        [display_name, holder_text],
                    )
                } else {
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_SCOREBOARD_PLAYERS_RESET_SPECIFIC_MULTIPLE,
                        translation::java::COMMANDS_SCOREBOARD_PLAYERS_RESET_SPECIFIC_MULTIPLE,
                        [display_name, holder_text],
                    )
                },
                true,
            );
        } else {
            for holder in &holders {
                scoreboard.reset_scores_for_entity(&world, &holder.name);
            }
            drop(scoreboard);
            let (holder_text, single) = holder_component(&holders);
            context.source.send_feedback(
                if single {
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_SCOREBOARD_PLAYERS_RESET_ALL_SINGLE,
                        translation::java::COMMANDS_SCOREBOARD_PLAYERS_RESET_ALL_SINGLE,
                        [holder_text],
                    )
                } else {
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_SCOREBOARD_PLAYERS_RESET_ALL_MULTIPLE,
                        translation::java::COMMANDS_SCOREBOARD_PLAYERS_RESET_ALL_MULTIPLE,
                        [holder_text],
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
        let holders = ScoreHolderArgumentType::get_score_holders(context, ARG_TARGETS)?;
        let objective_name = ObjectiveArgumentType::get(context, ARG_OBJECTIVE)?.to_string();
        let operation = *context.get_argument::<ScoreboardOperation>(ARG_OPERATION)?;
        let sources = ScoreHolderArgumentType::get_score_holders(context, ARG_SOURCE_TARGETS)?;
        let source_objective =
            ObjectiveArgumentType::get(context, ARG_SOURCE_OBJECTIVE)?.to_string();

        let world = context.world().clone();
        let mut scoreboard = world
            .scoreboard
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        let display_name = writable_objective_or_error(&scoreboard, &objective_name)?
            .display_name
            .clone();
        objective_or_error(&scoreboard, &source_objective)?;

        let result = Self::apply(
            &mut scoreboard,
            &world,
            &holders,
            &objective_name,
            operation,
            &sources,
            &source_objective,
        )?;
        drop(scoreboard);

        let (holder_text, single) = holder_component(&holders);
        let result_component = TextComponent::text(result.to_string());
        context.source.send_feedback(
            if single {
                TextComponent::translate_cross(
                    translation::java::COMMANDS_SCOREBOARD_PLAYERS_OPERATION_SUCCESS_SINGLE,
                    translation::java::COMMANDS_SCOREBOARD_PLAYERS_OPERATION_SUCCESS_SINGLE,
                    [display_name, holder_text, result_component],
                )
            } else {
                TextComponent::translate_cross(
                    translation::java::COMMANDS_SCOREBOARD_PLAYERS_OPERATION_SUCCESS_MULTIPLE,
                    translation::java::COMMANDS_SCOREBOARD_PLAYERS_OPERATION_SUCCESS_MULTIPLE,
                    [display_name, holder_text],
                )
            },
            true,
        );

        Ok(result)
    }
}

impl PlayersOperationExecutor {
    /// Applies each source sequentially and returns the sum of final target scores.
    fn apply(
        scoreboard: &mut Scoreboard,
        target: &impl ScoreboardTarget,
        holders: &[ResolvedScoreHolder],
        objective_name: &str,
        operation: ScoreboardOperation,
        sources: &[ResolvedScoreHolder],
        source_objective: &str,
    ) -> CommandExecutorResult {
        let mut result: i32 = 0;
        for holder in holders {
            if scoreboard
                .get_score_value(&holder.name, objective_name)
                .is_none()
            {
                scoreboard.set_score_value(target, holder.name.clone(), objective_name, 0);
            }
            for source in sources {
                let source_value = scoreboard
                    .get_score_value(&source.name, source_objective)
                    .unwrap_or_else(|| {
                        scoreboard.set_score_value(
                            target,
                            source.name.clone(),
                            source_objective,
                            0,
                        );
                        0
                    });
                let target_value = scoreboard
                    .get_score_value(&holder.name, objective_name)
                    .unwrap_or(0);
                if operation == ScoreboardOperation::Swap {
                    scoreboard.set_score_value(
                        target,
                        holder.name.clone(),
                        objective_name,
                        source_value,
                    );
                    scoreboard.set_score_value(
                        target,
                        source.name.clone(),
                        source_objective,
                        target_value,
                    );
                } else {
                    let new_value = operation.apply(target_value, source_value)?;
                    scoreboard.set_score_value(
                        target,
                        holder.name.clone(),
                        objective_name,
                        new_value,
                    );
                }
            }
            // Vanilla counts each holder once, with the score left after all of
            // its source operations.
            result = result.wrapping_add(
                scoreboard
                    .get_score_value(&holder.name, objective_name)
                    .unwrap_or(0),
            );
        }
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
                            argument(ARG_TARGETS, ScoreHolderArgumentType::Multiple).then(
                                argument(ARG_OBJECTIVE, ObjectiveArgumentType).then(
                                    argument(ARG_SCORE, IntegerArgumentType::any())
                                        .executes(PlayersSetExecutor),
                                ),
                            ),
                        ),
                    )
                    .then(
                        literal("get").then(
                            argument(ARG_TARGETS, ScoreHolderArgumentType::Single).then(
                                argument(ARG_OBJECTIVE, ObjectiveArgumentType)
                                    .executes(PlayersGetExecutor),
                            ),
                        ),
                    )
                    .then(
                        literal("add").then(
                            argument(ARG_TARGETS, ScoreHolderArgumentType::Multiple).then(
                                argument(ARG_OBJECTIVE, ObjectiveArgumentType).then(
                                    argument(ARG_SCORE, IntegerArgumentType::with_min(0))
                                        .executes(PlayersAddRemoveExecutor { remove: false }),
                                ),
                            ),
                        ),
                    )
                    .then(
                        literal("remove").then(
                            argument(ARG_TARGETS, ScoreHolderArgumentType::Multiple).then(
                                argument(ARG_OBJECTIVE, ObjectiveArgumentType).then(
                                    argument(ARG_SCORE, IntegerArgumentType::with_min(0))
                                        .executes(PlayersAddRemoveExecutor { remove: true }),
                                ),
                            ),
                        ),
                    )
                    .then(
                        literal("reset").then(
                            argument(ARG_TARGETS, ScoreHolderArgumentType::Multiple)
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
                            argument(ARG_TARGETS, ScoreHolderArgumentType::Multiple).then(
                                argument(ARG_OBJECTIVE, ObjectiveArgumentType).then(
                                    argument(ARG_OPERATION, OperationArgumentType).then(
                                        argument(
                                            ARG_SOURCE_TARGETS,
                                            ScoreHolderArgumentType::Multiple,
                                        )
                                        .then(
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
                            argument(ARG_TARGETS, ScoreHolderArgumentType::Multiple).then(
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
    use crate::world::scoreboard::NoTarget;

    #[test]
    fn single_holder_does_not_expand_wildcards() {
        use crate::command::CommandSource;
        use std::sync::Arc;

        let mut dispatcher = CommandDispatcher::new();
        register(&mut dispatcher, &PermissionRegistry::default());
        let source = Arc::new(CommandSource::dummy());
        let input = "scoreboard players get * test";
        let parsed = dispatcher.parse_input(input, &source);
        let context = parsed.context.build(input);
        let error = PlayersGetExecutor.execute(&context).unwrap_err();
        assert_eq!(
            serde_json::to_value(error.message).unwrap()["translate"],
            translation::java::ARGUMENT_SCOREHOLDER_EMPTY
        );
    }

    fn holder(name: &str) -> ResolvedScoreHolder {
        ResolvedScoreHolder {
            name: name.to_string(),
            display_name: TextComponent::text(name.to_string()),
        }
    }

    fn scoreboard_with_objective(criterion: &str) -> Scoreboard {
        let mut scoreboard = Scoreboard::new();
        scoreboard.add_objective(
            &NoTarget,
            ScoreboardObjective::new(
                "test",
                TextComponent::text("Test"),
                RenderType::Integer,
                None,
                criterion,
            ),
        );
        scoreboard
    }

    #[test]
    fn get_returns_the_score_and_orders_feedback_arguments() {
        let mut scoreboard = scoreboard_with_objective("dummy");
        scoreboard.set_score_value(&NoTarget, "entity-uuid", "test", 7);
        let entity = || ResolvedScoreHolder {
            name: "entity-uuid".to_string(),
            display_name: TextComponent::text("Named Pig"),
        };

        let (value, feedback) = PlayersGetExecutor::get(&scoreboard, entity(), "test").unwrap();
        assert_eq!(value, 7);
        let feedback = serde_json::to_value(feedback).unwrap();
        assert_eq!(
            feedback["translate"],
            translation::java::COMMANDS_SCOREBOARD_PLAYERS_GET_SUCCESS
        );
        assert_eq!(
            feedback["with"],
            serde_json::json!([
                {"text": "Named Pig"}, {"text": "7"}, {"text": "Test"}
            ])
        );

        scoreboard.remove_score(&NoTarget, "entity-uuid", "test");
        let error = PlayersGetExecutor::get(&scoreboard, entity(), "test").unwrap_err();
        assert!(error.is(&NO_SCORE_ERROR));
        assert_eq!(
            serde_json::to_value(error.message).unwrap()["with"],
            serde_json::json!([
                {"text": "test"}, {"text": "Named Pig"}
            ])
        );
    }

    #[test]
    fn enable_counts_only_newly_unlocked_scores_and_preserves_values() {
        let mut scoreboard = scoreboard_with_objective("trigger");
        let mut enabled = ScoreboardScore::new("enabled", "test", VarInt(7), None, None);
        enabled.locked = false;
        scoreboard.update_score(&NoTarget, enabled);
        scoreboard.set_score_value(&NoTarget, "locked", "test", 5);
        let holders = [holder("enabled"), holder("locked"), holder("new")];

        let (count, feedback) =
            PlayersEnableExecutor::enable(&mut scoreboard, &NoTarget, &holders, "test").unwrap();
        assert_eq!(count, 2);
        let feedback = serde_json::to_value(feedback).unwrap();
        assert_eq!(
            feedback["translate"],
            translation::java::COMMANDS_SCOREBOARD_PLAYERS_ENABLE_SUCCESS_MULTIPLE
        );
        assert_eq!(feedback["with"][1]["text"], "2");
        for (name, value) in [("enabled", 7), ("locked", 5), ("new", 0)] {
            let score = scoreboard.get_score(name, "test").unwrap();
            assert_eq!(score.value.0, value);
            assert!(!score.locked);
        }
        let error = PlayersEnableExecutor::enable(&mut scoreboard, &NoTarget, &holders, "test")
            .unwrap_err();
        assert!(error.is(&FAILED_ENABLE_ERROR));

        scoreboard.set_score_value(&NoTarget, "only_new", "test", 9);
        let holders = [holder("enabled"), holder("locked"), holder("only_new")];
        let (count, feedback) =
            PlayersEnableExecutor::enable(&mut scoreboard, &NoTarget, &holders, "test").unwrap();
        assert_eq!(count, 1);
        let feedback = serde_json::to_value(feedback).unwrap();
        assert_eq!(
            feedback["translate"],
            translation::java::COMMANDS_SCOREBOARD_PLAYERS_ENABLE_SUCCESS_SINGLE
        );
        assert_eq!(feedback["with"][1]["text"], "only_new");
    }

    #[test]
    fn operations_track_missing_sources_and_propagate_zero_divisor_errors() {
        for operation in [
            ScoreboardOperation::Assign,
            ScoreboardOperation::Divide,
            ScoreboardOperation::Modulo,
        ] {
            let mut scoreboard = scoreboard_with_objective("dummy");
            scoreboard.set_score_value(&NoTarget, "target", "test", 7);
            let result = PlayersOperationExecutor::apply(
                &mut scoreboard,
                &NoTarget,
                &[holder("target")],
                "test",
                operation,
                &[holder("source")],
                "test",
            );
            assert_eq!(scoreboard.get_score_value("source", "test"), Some(0));
            if operation == ScoreboardOperation::Assign {
                assert_eq!(result.unwrap(), 0);
                assert_eq!(scoreboard.get_score_value("target", "test"), Some(0));
            } else {
                let error = result.unwrap_err();
                assert_eq!(
                    serde_json::to_value(error.message).unwrap()["translate"],
                    translation::java::ARGUMENTS_OPERATION_DIV0
                );
                assert_eq!(scoreboard.get_score_value("target", "test"), Some(7));
            }
        }
    }

    #[test]
    fn operations_sum_only_final_target_values_and_swap_sequentially() {
        let mut scoreboard = scoreboard_with_objective("dummy");
        for (name, value) in [("a", 1), ("b", 4), ("source1", 2), ("source2", 3)] {
            scoreboard.set_score_value(&NoTarget, name, "test", value);
        }
        let targets = [holder("a"), holder("b")];
        let result = PlayersOperationExecutor::apply(
            &mut scoreboard,
            &NoTarget,
            &targets,
            "test",
            ScoreboardOperation::Plus,
            &[holder("source1"), holder("source2")],
            "test",
        )
        .unwrap();
        assert_eq!(result, 15);
        assert_eq!(scoreboard.get_score_value("a", "test"), Some(6));
        assert_eq!(scoreboard.get_score_value("b", "test"), Some(9));

        let result = PlayersOperationExecutor::apply(
            &mut scoreboard,
            &NoTarget,
            &targets,
            "test",
            ScoreboardOperation::Swap,
            &[holder("source1")],
            "test",
        )
        .unwrap();
        assert_eq!(result, 8);
        assert_eq!(scoreboard.get_score_value("a", "test"), Some(2));
        assert_eq!(scoreboard.get_score_value("b", "test"), Some(6));
        assert_eq!(scoreboard.get_score_value("source1", "test"), Some(9));
    }

    #[test]
    fn holder_component_uses_the_name_for_one_and_a_count_for_many() {
        let one = vec![ResolvedScoreHolder {
            name: "entity-uuid".to_string(),
            display_name: TextComponent::text("Named Pig"),
        }];
        let (holder_text, single) = holder_component(&one);
        assert!(single);
        assert_eq!(holder_text.to_pretty_console(), "Named Pig");

        let mut many = one;
        many.push(ResolvedScoreHolder {
            name: "Coal".to_string(),
            display_name: TextComponent::text("Coal"),
        });
        let (holder_text, single) = holder_component(&many);
        assert!(!single);
        assert!(holder_text.to_pretty_console().contains('2'));
    }

    #[test]
    fn only_live_attribute_criteria_are_read_only() {
        let mut scoreboard = Scoreboard::new();
        for (criterion, writable) in [
            ("dummy", true),
            ("trigger", true),
            ("deathCount", true),
            ("playerKillCount", true),
            ("totalKillCount", true),
            ("teamkill.red", true),
            ("killedByTeam.blue", true),
            ("minecraft.mined:minecraft.stone", true),
            ("minecraft.custom:minecraft.jump", true),
            ("health", false),
            ("food", false),
            ("air", false),
            ("armor", false),
            ("xp", false),
            ("level", false),
        ] {
            scoreboard.add_objective(
                &NoTarget,
                ScoreboardObjective::new(
                    "test",
                    TextComponent::text("Test"),
                    RenderType::Integer,
                    None,
                    criterion,
                ),
            );
            let result = writable_objective_or_error(&scoreboard, "test");
            assert_eq!(result.is_ok(), writable, "{criterion}");
            if let Err(error) = result {
                assert!(error.is(&OBJECTIVE_READ_ONLY_ERROR));
                assert_eq!(
                    error.message.to_pretty_console(),
                    "Scoreboard objective 'test' is read-only"
                );
            }
            scoreboard.remove_objective(&NoTarget, "test");
        }
        let error = objective_or_error(&scoreboard, "missing").unwrap_err();
        assert!(error.is(&OBJECTIVE_NOT_FOUND_ERROR));
        assert_eq!(
            serde_json::to_value(&error.message).unwrap()["translate"],
            translation::java::ARGUMENTS_OBJECTIVE_NOTFOUND
        );
    }

    #[test]
    fn get_requires_a_single_holder_in_the_command_tree() {
        use crate::command::CommandSource;
        use std::sync::Arc;

        let mut dispatcher = CommandDispatcher::new();
        register(&mut dispatcher, &PermissionRegistry::default());
        let source = Arc::new(CommandSource::dummy());
        for input in [
            "scoreboard players get #temp test",
            "scoreboard players get @s test",
            "scoreboard players get @e[limit=1] test",
            "scoreboard players set $x test 4",
            "scoreboard players set @a test 4",
        ] {
            let result = dispatcher.parse_input(input, &source);
            assert!(result.errors.is_empty(), "{input}: {:?}", result.errors);
            assert_eq!(result.reader.remaining_length(), 0, "{input}");
        }
        for input in [
            "scoreboard players get @a test",
            "scoreboard players get @e test",
        ] {
            let result = dispatcher.parse_input(input, &source);
            assert!(!result.errors.is_empty(), "{input}");
        }
    }

    #[test]
    fn holder_suggestions_include_selectors_and_selector_options() {
        use crate::command::CommandSource;
        use std::sync::Arc;

        let mut dispatcher = CommandDispatcher::new();
        register(&mut dispatcher, &PermissionRegistry::default());
        let source = Arc::new(CommandSource::dummy());
        for (input, expected) in [
            ("scoreboard players set @", "@a"),
            ("scoreboard players set @", "@s"),
            ("scoreboard players set @e[", "type="),
        ] {
            let parsed = dispatcher.parse_input(input, &source);
            let suggestions = dispatcher.get_completion_suggestions_at_end(parsed);
            assert!(
                suggestions
                    .suggestions
                    .iter()
                    .any(|suggestion| { suggestion.text.cached_text() == expected }),
                "{input}: {suggestions:?}"
            );
        }
    }
}
