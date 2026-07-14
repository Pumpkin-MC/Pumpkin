use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::core::string::StringArgumentType;
use crate::command::argument_types::entity::EntityArgumentType;
use crate::command::argument_types::objective::ObjectiveArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::world::scoreboard::{ScoreboardObjective, ScoreboardScore};
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
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let objective_name = StringArgumentType::get(context, ARG_OBJECTIVE)?;
            let criterion = StringArgumentType::get(context, ARG_CRITERION)?;

            let display_name = if self.has_display_name {
                TextComponent::text(StringArgumentType::get(context, ARG_DISPLAY_NAME)?.to_string())
            } else {
                TextComponent::text(objective_name.to_string())
            };

            let world = context.world();
            let mut scoreboard = world.scoreboard.lock().await;

            if scoreboard.get_objectives().contains_key(objective_name) {
                return Err(DUPLICATE_OBJECTIVE_ERROR.create_without_context());
            }

            let obj_name_static = Box::leak(objective_name.to_string().into_boxed_str());
            let criterion_static = Box::leak(criterion.to_string().into_boxed_str());

            let new_objective = ScoreboardObjective::new(
                obj_name_static,
                display_name.clone(),
                RenderType::Integer,
                None,
                criterion_static,
            );

            scoreboard.add_objective(world, new_objective).await;

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
                .ok_or_else(|| INVALID_ENABLE_ERROR.create_without_context())?;

            if objective.criterion != "trigger" {
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
                        entity_name: Box::leak(player_name.clone().into_boxed_str()),
                        objective_name: Box::leak(objective_name.to_string().into_boxed_str()),
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
                literal("objectives").then(
                    literal("add").then(
                        argument(ARG_OBJECTIVE, StringArgumentType::SingleWord).then(
                            argument(ARG_CRITERION, StringArgumentType::SingleWord)
                                .executes(ObjectivesAddExecutor {
                                    has_display_name: false,
                                })
                                .then(
                                    argument(ARG_DISPLAY_NAME, StringArgumentType::GreedyPhrase)
                                        .executes(ObjectivesAddExecutor {
                                            has_display_name: true,
                                        }),
                                ),
                        ),
                    ),
                ),
            )
            .then(literal("players").then(literal("enable").then(
                argument(ARG_TARGETS, EntityArgumentType::Players).then(
                    argument(ARG_OBJECTIVE, ObjectiveArgumentType).executes(PlayersEnableExecutor),
                ),
            ))),
    );
}
