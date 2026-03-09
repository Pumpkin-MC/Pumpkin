use pumpkin_data::translation;
use pumpkin_protocol::java::client::play::RenderType;
use pumpkin_util::text::TextComponent;

use crate::command::args::simple::SimpleArgConsumer;
use crate::command::args::{Arg, ConsumedArgs};
use crate::command::dispatcher::CommandError;
use crate::command::tree::CommandTree;
use crate::command::tree::builder::{argument, literal};
use crate::command::{CommandExecutor, CommandResult, CommandSender};
use crate::world::scoreboard::ScoreboardObjective;

const NAMES: [&str; 1] = ["scoreboard"];

const DESCRIPTION: &str = "Manages scoreboard objectives and players.";

const ARG_OBJECTIVE: &str = "objective";
const ARG_CRITERIA: &str = "criteria";
const ARG_DISPLAY_NAME: &str = "displayName";
const ARG_TARGETS: &str = "targets";
const ARG_SCORE: &str = "score";

struct ObjectivesAddExecutor;

impl CommandExecutor for ObjectivesAddExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Simple(name)) = args.get(ARG_OBJECTIVE) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_OBJECTIVE.into())));
            };
            let Some(Arg::Simple(_criteria)) = args.get(ARG_CRITERIA) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_CRITERIA.into())));
            };

            let display_name = args
                .get(ARG_DISPLAY_NAME)
                .and_then(|a| {
                    if let Arg::Simple(s) = a {
                        Some(TextComponent::text(s.to_string()))
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| TextComponent::text(name.to_string()));

            let world = sender.world().ok_or(CommandError::InvalidRequirement)?;
            let mut scoreboard = world.scoreboard.lock().await;

            if scoreboard.has_objective(name) {
                return Err(CommandError::CommandFailed(TextComponent::translate(
                    translation::COMMANDS_SCOREBOARD_OBJECTIVES_ADD_DUPLICATE,
                    [TextComponent::text(name.to_string())],
                )));
            }

            let objective =
                ScoreboardObjective::new(name, display_name, RenderType::Integer, None);
            scoreboard.add_objective(&world, objective).await;

            sender
                .send_message(TextComponent::translate(
                    translation::COMMANDS_SCOREBOARD_OBJECTIVES_ADD_SUCCESS,
                    [TextComponent::text(name.to_string())],
                ))
                .await;
            Ok(1)
        })
    }
}

struct ObjectivesListExecutor;

impl CommandExecutor for ObjectivesListExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let world = sender.world().ok_or(CommandError::InvalidRequirement)?;
            let scoreboard = world.scoreboard.lock().await;
            let objectives = scoreboard.get_objectives();

            if objectives.is_empty() {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_SCOREBOARD_OBJECTIVES_LIST_EMPTY,
                        [],
                    ))
                    .await;
            } else {
                let names: Vec<String> = objectives.keys().cloned().collect();
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_SCOREBOARD_OBJECTIVES_LIST_SUCCESS,
                        [
                            TextComponent::text(objectives.len().to_string()),
                            TextComponent::text(names.join(", ")),
                        ],
                    ))
                    .await;
            }
            Ok(objectives.len() as i32)
        })
    }
}

struct ObjectivesRemoveExecutor;

impl CommandExecutor for ObjectivesRemoveExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Simple(name)) = args.get(ARG_OBJECTIVE) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_OBJECTIVE.into())));
            };

            let world = sender.world().ok_or(CommandError::InvalidRequirement)?;
            let mut scoreboard = world.scoreboard.lock().await;

            if !scoreboard.has_objective(name) {
                return Err(CommandError::CommandFailed(TextComponent::text(format!(
                    "Unknown scoreboard objective '{name}'"
                ))));
            }

            scoreboard.remove_objective(&world, name).await;

            sender
                .send_message(TextComponent::translate(
                    translation::COMMANDS_SCOREBOARD_OBJECTIVES_REMOVE_SUCCESS,
                    [TextComponent::text(name.to_string())],
                ))
                .await;
            Ok(1)
        })
    }
}

struct PlayersSetExecutor;

impl CommandExecutor for PlayersSetExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Simple(targets)) = args.get(ARG_TARGETS) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_TARGETS.into())));
            };
            let Some(Arg::Simple(objective)) = args.get(ARG_OBJECTIVE) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_OBJECTIVE.into())));
            };
            let Some(Arg::Simple(score_str)) = args.get(ARG_SCORE) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_SCORE.into())));
            };
            let score: i32 = score_str
                .parse()
                .map_err(|_| CommandError::InvalidConsumption(Some(ARG_SCORE.into())))?;

            let world = sender.world().ok_or(CommandError::InvalidRequirement)?;
            let mut scoreboard = world.scoreboard.lock().await;

            if !scoreboard.has_objective(objective) {
                return Err(CommandError::CommandFailed(TextComponent::text(format!(
                    "Unknown scoreboard objective '{objective}'"
                ))));
            }

            scoreboard.set_score(&world, targets, objective, score).await;

            sender
                .send_message(TextComponent::translate(
                    translation::COMMANDS_SCOREBOARD_PLAYERS_SET_SUCCESS_SINGLE,
                    [
                        TextComponent::text(objective.to_string()),
                        TextComponent::text(targets.to_string()),
                        TextComponent::text(score.to_string()),
                    ],
                ))
                .await;
            Ok(score)
        })
    }
}

struct PlayersAddExecutor;

impl CommandExecutor for PlayersAddExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Simple(targets)) = args.get(ARG_TARGETS) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_TARGETS.into())));
            };
            let Some(Arg::Simple(objective)) = args.get(ARG_OBJECTIVE) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_OBJECTIVE.into())));
            };
            let Some(Arg::Simple(score_str)) = args.get(ARG_SCORE) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_SCORE.into())));
            };
            let amount: i32 = score_str
                .parse()
                .map_err(|_| CommandError::InvalidConsumption(Some(ARG_SCORE.into())))?;

            let world = sender.world().ok_or(CommandError::InvalidRequirement)?;
            let mut scoreboard = world.scoreboard.lock().await;

            if !scoreboard.has_objective(objective) {
                return Err(CommandError::CommandFailed(TextComponent::text(format!(
                    "Unknown scoreboard objective '{objective}'"
                ))));
            }

            let current = scoreboard.get_score(targets, objective).unwrap_or(0);
            let new_score = current + amount;
            scoreboard
                .set_score(&world, targets, objective, new_score)
                .await;

            sender
                .send_message(TextComponent::translate(
                    translation::COMMANDS_SCOREBOARD_PLAYERS_ADD_SUCCESS_SINGLE,
                    [
                        TextComponent::text(objective.to_string()),
                        TextComponent::text(targets.to_string()),
                        TextComponent::text(new_score.to_string()),
                    ],
                ))
                .await;
            Ok(new_score)
        })
    }
}

struct PlayersResetExecutor;

impl CommandExecutor for PlayersResetExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Simple(targets)) = args.get(ARG_TARGETS) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_TARGETS.into())));
            };

            let world = sender.world().ok_or(CommandError::InvalidRequirement)?;
            let mut scoreboard = world.scoreboard.lock().await;

            let objective = args.get(ARG_OBJECTIVE).and_then(|a| {
                if let Arg::Simple(s) = a {
                    Some(*s)
                } else {
                    None
                }
            });

            scoreboard.reset_scores(targets, objective);

            sender
                .send_message(TextComponent::translate(
                    "commands.scoreboard.players.reset.all.single",
                    [TextComponent::text(targets.to_string())],
                ))
                .await;
            Ok(1)
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(
            literal("objectives")
                .then(
                    literal("add").then(
                        argument(ARG_OBJECTIVE, SimpleArgConsumer).then(
                            argument(ARG_CRITERIA, SimpleArgConsumer)
                                .then(
                                    argument(ARG_DISPLAY_NAME, SimpleArgConsumer)
                                        .execute(ObjectivesAddExecutor),
                                )
                                .execute(ObjectivesAddExecutor),
                        ),
                    ),
                )
                .then(literal("list").execute(ObjectivesListExecutor))
                .then(
                    literal("remove").then(
                        argument(ARG_OBJECTIVE, SimpleArgConsumer)
                            .execute(ObjectivesRemoveExecutor),
                    ),
                ),
        )
        .then(
            literal("players")
                .then(
                    literal("set").then(
                        argument(ARG_TARGETS, SimpleArgConsumer).then(
                            argument(ARG_OBJECTIVE, SimpleArgConsumer).then(
                                argument(ARG_SCORE, SimpleArgConsumer)
                                    .execute(PlayersSetExecutor),
                            ),
                        ),
                    ),
                )
                .then(
                    literal("add").then(
                        argument(ARG_TARGETS, SimpleArgConsumer).then(
                            argument(ARG_OBJECTIVE, SimpleArgConsumer).then(
                                argument(ARG_SCORE, SimpleArgConsumer)
                                    .execute(PlayersAddExecutor),
                            ),
                        ),
                    ),
                )
                .then(
                    literal("reset").then(
                        argument(ARG_TARGETS, SimpleArgConsumer)
                            .then(
                                argument(ARG_OBJECTIVE, SimpleArgConsumer)
                                    .execute(PlayersResetExecutor),
                            )
                            .execute(PlayersResetExecutor),
                    ),
                ),
        )
}
