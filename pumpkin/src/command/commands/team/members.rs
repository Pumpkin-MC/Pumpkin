use super::{
    ARG_DISPLAY_NAME, ARG_MEMBERS, ARG_TEAM, ARG_TEAM_NAME, DUPLICATE_TEAM_ERROR,
    EMPTY_UNCHANGED_ERROR, TEAM_NOT_FOUND_ERROR, get_entity_scoreboard_name,
};
use crate::command::argument_builder::{
    ArgumentBuilder, LiteralArgumentBuilder, argument, literal,
};
use crate::command::argument_types::core::string::StringArgumentType;
use crate::command::argument_types::entity::EntityArgumentType;
use crate::command::argument_types::team::TeamArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::world::scoreboard::{CollisionRule, NameTagVisibility, Team};
use pumpkin_data::translation;
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::color::NamedColor;

struct TeamAddExecutor {
    has_display_name: bool,
}

impl CommandExecutor for TeamAddExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let team_name = StringArgumentType::get(context, ARG_TEAM_NAME)?;
            let display_name = if self.has_display_name {
                TextComponent::text(StringArgumentType::get(context, ARG_DISPLAY_NAME)?.to_string())
            } else {
                TextComponent::text(team_name.to_string())
            };

            let world = context.world();
            let mut scoreboard = world.scoreboard.lock().await;

            if scoreboard.get_teams().contains_key(team_name) {
                return Err(DUPLICATE_TEAM_ERROR.create_without_context());
            }

            let new_team = Team {
                name: team_name.to_string(),
                display_name: display_name.clone(),
                options: 0,
                nametag_visibility: NameTagVisibility::Always,
                collision_rule: CollisionRule::Always,
                color: NamedColor::White,
                player_prefix: TextComponent::empty(),
                player_suffix: TextComponent::empty(),
                players: vec![],
            };

            scoreboard.add_team(world, new_team);

            context
                .source
                .send_feedback(
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_TEAM_ADD_SUCCESS,
                        translation::java::COMMANDS_TEAM_ADD_SUCCESS,
                        [display_name],
                    ),
                    true,
                )
                .await;

            Ok(1)
        })
    }
}

struct TeamRemoveExecutor;

impl CommandExecutor for TeamRemoveExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let team_name = TeamArgumentType::get(context, ARG_TEAM)?;

            let world = context.world();
            let mut scoreboard = world.scoreboard.lock().await;

            let team = scoreboard.get_teams().get(team_name).ok_or_else(|| {
                TEAM_NOT_FOUND_ERROR
                    .create_without_context(TextComponent::text(team_name.to_string()))
            })?;

            let team_display_name = team.display_name.clone();

            scoreboard.remove_team(world, team_name);

            context
                .source
                .send_feedback(
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_TEAM_REMOVE_SUCCESS,
                        translation::java::COMMANDS_TEAM_REMOVE_SUCCESS,
                        [team_display_name],
                    ),
                    true,
                )
                .await;

            Ok(1)
        })
    }
}

struct TeamEmptyExecutor;

impl CommandExecutor for TeamEmptyExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let team_name = TeamArgumentType::get(context, ARG_TEAM)?;

            let world = context.world();
            let mut scoreboard = world.scoreboard.lock().await;

            let team = scoreboard.get_teams().get(team_name).ok_or_else(|| {
                TEAM_NOT_FOUND_ERROR
                    .create_without_context(TextComponent::text(team_name.to_string()))
            })?;

            let team_display_name = team.display_name.clone();
            let players_to_remove = team.players.clone();

            if players_to_remove.is_empty() {
                return Err(EMPTY_UNCHANGED_ERROR.create_without_context(team_display_name));
            }

            for player in &players_to_remove {
                scoreboard.remove_player_from_team(world, team_name, player);
            }

            context
                .source
                .send_feedback(
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_TEAM_EMPTY_SUCCESS,
                        translation::java::COMMANDS_TEAM_EMPTY_SUCCESS,
                        [
                            TextComponent::text(players_to_remove.len().to_string()),
                            team_display_name,
                        ],
                    ),
                    true,
                )
                .await;

            Ok(players_to_remove.len() as i32)
        })
    }
}

struct TeamJoinExecutor {
    has_members: bool,
}

impl CommandExecutor for TeamJoinExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let team_name = TeamArgumentType::get(context, ARG_TEAM)?;

            let world = context.world();
            let mut scoreboard = world.scoreboard.lock().await;

            let team = scoreboard.get_teams().get(team_name).ok_or_else(|| {
                TEAM_NOT_FOUND_ERROR
                    .create_without_context(TextComponent::text(team_name.to_string()))
            })?;

            let team_display_name = team.display_name.clone();

            let entity_names = if self.has_members {
                let targets =
                    EntityArgumentType::get_optional_entities(context, ARG_MEMBERS).await?;
                if targets.is_empty() {
                    return Err(
                        crate::command::argument_types::entity::NO_ENTITIES_ERROR_TYPE
                            .create_without_context(),
                    );
                }
                targets
                    .into_iter()
                    .map(|e| get_entity_scoreboard_name(&*e))
                    .collect::<Vec<_>>()
            } else {
                let sender_name = context.source.name.clone();
                vec![sender_name]
            };

            let count = entity_names.len();
            for name in &entity_names {
                scoreboard.add_player_to_team(world, team_name, name.clone());
            }

            let msg = if count == 1 {
                TextComponent::translate_cross(
                    translation::java::COMMANDS_TEAM_JOIN_SUCCESS_SINGLE,
                    translation::java::COMMANDS_TEAM_JOIN_SUCCESS_SINGLE,
                    [
                        TextComponent::text(entity_names[0].clone()),
                        team_display_name,
                    ],
                )
            } else {
                TextComponent::translate_cross(
                    translation::java::COMMANDS_TEAM_JOIN_SUCCESS_MULTIPLE,
                    translation::java::COMMANDS_TEAM_JOIN_SUCCESS_MULTIPLE,
                    [TextComponent::text(count.to_string()), team_display_name],
                )
            };

            context.source.send_feedback(msg, true).await;

            Ok(count as i32)
        })
    }
}

struct TeamLeaveExecutor {
    has_members: bool,
}

impl CommandExecutor for TeamLeaveExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let world = context.world();
            let mut scoreboard = world.scoreboard.lock().await;

            let entity_names = if self.has_members {
                let targets =
                    EntityArgumentType::get_optional_entities(context, ARG_MEMBERS).await?;
                if targets.is_empty() {
                    return Err(
                        crate::command::argument_types::entity::NO_ENTITIES_ERROR_TYPE
                            .create_without_context(),
                    );
                }
                targets
                    .into_iter()
                    .map(|e| get_entity_scoreboard_name(&*e))
                    .collect::<Vec<_>>()
            } else {
                let sender_name = context.source.name.clone();
                vec![sender_name]
            };

            let mut removed_count = 0;
            for name in &entity_names {
                let mut found_team = None;
                for team in scoreboard.get_teams().values() {
                    if team.players.contains(name) {
                        found_team = Some(team.name.clone());
                        break;
                    }
                }
                if let Some(team_name) = found_team {
                    scoreboard.remove_player_from_team(world, &team_name, name);
                    removed_count += 1;
                }
            }

            let msg = if entity_names.len() == 1 {
                TextComponent::translate_cross(
                    translation::java::COMMANDS_TEAM_LEAVE_SUCCESS_SINGLE,
                    translation::java::COMMANDS_TEAM_LEAVE_SUCCESS_SINGLE,
                    [TextComponent::text(entity_names[0].clone())],
                )
            } else {
                TextComponent::translate_cross(
                    translation::java::COMMANDS_TEAM_LEAVE_SUCCESS_MULTIPLE,
                    translation::java::COMMANDS_TEAM_LEAVE_SUCCESS_MULTIPLE,
                    [TextComponent::text(removed_count.to_string())],
                )
            };

            context.source.send_feedback(msg, true).await;

            Ok(removed_count)
        })
    }
}

struct TeamListExecutor {
    has_team: bool,
}

impl CommandExecutor for TeamListExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let world = context.world();
            let scoreboard = world.scoreboard.lock().await;

            if self.has_team {
                let team_name = TeamArgumentType::get(context, ARG_TEAM)?;
                let team = scoreboard.get_teams().get(team_name).ok_or_else(|| {
                    TEAM_NOT_FOUND_ERROR
                        .create_without_context(TextComponent::text(team_name.to_string()))
                })?;

                if team.players.is_empty() {
                    context
                        .source
                        .send_feedback(
                            TextComponent::translate_cross(
                                translation::java::COMMANDS_TEAM_LIST_MEMBERS_EMPTY,
                                translation::java::COMMANDS_TEAM_LIST_MEMBERS_EMPTY,
                                [team.display_name.clone()],
                            ),
                            false,
                        )
                        .await;
                } else {
                    let mut list_comp = TextComponent::empty();
                    for (i, player) in team.players.iter().enumerate() {
                        if i > 0 {
                            list_comp = list_comp.add_child(TextComponent::text(", "));
                        }
                        list_comp = list_comp.add_child(TextComponent::text(player.clone()));
                    }

                    context
                        .source
                        .send_feedback(
                            TextComponent::translate_cross(
                                translation::java::COMMANDS_TEAM_LIST_MEMBERS_SUCCESS,
                                translation::java::COMMANDS_TEAM_LIST_MEMBERS_SUCCESS,
                                [
                                    team.display_name.clone(),
                                    TextComponent::text(team.players.len().to_string()),
                                    list_comp,
                                ],
                            ),
                            false,
                        )
                        .await;
                }
                Ok(team.players.len() as i32)
            } else {
                let teams = scoreboard.get_teams();
                if teams.is_empty() {
                    context
                        .source
                        .send_feedback(
                            TextComponent::translate_cross(
                                translation::java::COMMANDS_TEAM_LIST_TEAMS_EMPTY,
                                translation::java::COMMANDS_TEAM_LIST_TEAMS_EMPTY,
                                [],
                            ),
                            false,
                        )
                        .await;
                } else {
                    let mut list_comp = TextComponent::empty();
                    for (i, team) in teams.values().enumerate() {
                        if i > 0 {
                            list_comp = list_comp.add_child(TextComponent::text(", "));
                        }
                        list_comp = list_comp.add_child(team.display_name.clone());
                    }

                    context
                        .source
                        .send_feedback(
                            TextComponent::translate_cross(
                                translation::java::COMMANDS_TEAM_LIST_TEAMS_SUCCESS,
                                translation::java::COMMANDS_TEAM_LIST_TEAMS_SUCCESS,
                                [TextComponent::text(teams.len().to_string()), list_comp],
                            ),
                            false,
                        )
                        .await;
                }
                Ok(teams.len() as i32)
            }
        })
    }
}

pub(super) fn add_branch() -> LiteralArgumentBuilder {
    literal("add").then(
        argument(ARG_TEAM_NAME, StringArgumentType::SingleWord)
            .executes(TeamAddExecutor {
                has_display_name: false,
            })
            .then(
                argument(ARG_DISPLAY_NAME, StringArgumentType::GreedyPhrase).executes(
                    TeamAddExecutor {
                        has_display_name: true,
                    },
                ),
            ),
    )
}

pub(super) fn remove_branch() -> LiteralArgumentBuilder {
    literal("remove").then(argument(ARG_TEAM, TeamArgumentType).executes(TeamRemoveExecutor))
}

pub(super) fn empty_branch() -> LiteralArgumentBuilder {
    literal("empty").then(argument(ARG_TEAM, TeamArgumentType).executes(TeamEmptyExecutor))
}

pub(super) fn join_branch() -> LiteralArgumentBuilder {
    literal("join").then(
        argument(ARG_TEAM, TeamArgumentType)
            .executes(TeamJoinExecutor { has_members: false })
            .then(
                argument(ARG_MEMBERS, EntityArgumentType::Entities)
                    .executes(TeamJoinExecutor { has_members: true }),
            ),
    )
}

pub(super) fn leave_branch() -> LiteralArgumentBuilder {
    literal("leave")
        .executes(TeamLeaveExecutor { has_members: false })
        .then(
            argument(ARG_MEMBERS, EntityArgumentType::Entities)
                .executes(TeamLeaveExecutor { has_members: true }),
        )
}

pub(super) fn list_branch() -> LiteralArgumentBuilder {
    literal("list")
        .executes(TeamListExecutor { has_team: false })
        .then(argument(ARG_TEAM, TeamArgumentType).executes(TeamListExecutor { has_team: true }))
}
