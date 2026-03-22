use pumpkin_data::translation;
use pumpkin_util::text::TextComponent;

use crate::command::args::simple::SimpleArgConsumer;
use crate::command::args::{Arg, ConsumedArgs};
use crate::command::dispatcher::CommandError;
use crate::command::tree::CommandTree;
use crate::command::tree::builder::{argument, literal};
use crate::command::{CommandExecutor, CommandResult, CommandSender};

const NAMES: [&str; 1] = ["team"];

const DESCRIPTION: &str = "Controls teams.";

const ARG_TEAM: &str = "team";
const ARG_MEMBERS: &str = "members";

struct AddExecutor;

impl CommandExecutor for AddExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Simple(team_name)) = args.get(ARG_TEAM) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_TEAM.into())));
            };

            let world = sender.world().ok_or(CommandError::InvalidRequirement)?;
            let mut teams = world.teams.lock().await;

            if teams.has_team(team_name) {
                return Err(CommandError::CommandFailed(TextComponent::translate(
                    translation::COMMANDS_TEAM_ADD_DUPLICATE,
                    [],
                )));
            }

            teams.add_team(&world, team_name).await;

            sender
                .send_message(TextComponent::translate(
                    translation::COMMANDS_TEAM_ADD_SUCCESS,
                    [TextComponent::text(team_name.to_string())],
                ))
                .await;
            Ok(1)
        })
    }
}

struct RemoveExecutor;

impl CommandExecutor for RemoveExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Simple(team_name)) = args.get(ARG_TEAM) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_TEAM.into())));
            };

            let world = sender.world().ok_or(CommandError::InvalidRequirement)?;
            let mut teams = world.teams.lock().await;

            if teams.remove_team(&world, team_name).await.is_none() {
                return Err(CommandError::CommandFailed(TextComponent::text(format!(
                    "Unknown team '{team_name}'"
                ))));
            }

            sender
                .send_message(TextComponent::translate(
                    translation::COMMANDS_TEAM_REMOVE_SUCCESS,
                    [TextComponent::text(team_name.to_string())],
                ))
                .await;
            Ok(1)
        })
    }
}

struct ListExecutor;

impl CommandExecutor for ListExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let world = sender.world().ok_or(CommandError::InvalidRequirement)?;
            let teams = world.teams.lock().await;
            let all_teams = teams.get_teams();

            if all_teams.is_empty() {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_TEAM_LIST_TEAMS_EMPTY,
                        [],
                    ))
                    .await;
                return Ok(0);
            }

            let team_names: Vec<&String> = all_teams.keys().collect();
            let count = team_names.len();
            let team_list = team_names
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ");

            sender
                .send_message(TextComponent::translate(
                    translation::COMMANDS_TEAM_LIST_TEAMS_SUCCESS,
                    [
                        TextComponent::text(count.to_string()),
                        TextComponent::text(team_list),
                    ],
                ))
                .await;
            Ok(count as i32)
        })
    }
}

struct ListMembersExecutor;

impl CommandExecutor for ListMembersExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Simple(team_name)) = args.get(ARG_TEAM) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_TEAM.into())));
            };

            let world = sender.world().ok_or(CommandError::InvalidRequirement)?;
            let teams = world.teams.lock().await;

            let team = teams.get_team(team_name).ok_or_else(|| {
                CommandError::CommandFailed(TextComponent::text(format!(
                    "Unknown team '{team_name}'"
                )))
            })?;

            if team.members.is_empty() {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_TEAM_LIST_MEMBERS_EMPTY,
                        [TextComponent::text(team_name.to_string())],
                    ))
                    .await;
                return Ok(0);
            }

            let members: Vec<&String> = team.members.iter().collect();
            let count = members.len();
            let member_list = members
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ");

            sender
                .send_message(TextComponent::translate(
                    translation::COMMANDS_TEAM_LIST_MEMBERS_SUCCESS,
                    [
                        TextComponent::text(team_name.to_string()),
                        TextComponent::text(count.to_string()),
                        TextComponent::text(member_list),
                    ],
                ))
                .await;
            Ok(count as i32)
        })
    }
}

struct JoinExecutor;

impl CommandExecutor for JoinExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Simple(team_name)) = args.get(ARG_TEAM) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_TEAM.into())));
            };

            let world = sender.world().ok_or(CommandError::InvalidRequirement)?;
            let mut teams = world.teams.lock().await;

            if !teams.has_team(team_name) {
                return Err(CommandError::CommandFailed(TextComponent::text(format!(
                    "Unknown team '{team_name}'"
                ))));
            }

            let members = if let Some(Arg::Simple(member)) = args.get(ARG_MEMBERS) {
                vec![member.to_string()]
            } else {
                match sender {
                    CommandSender::Player(p) => vec![p.gameprofile.name.clone()],
                    _ => return Err(CommandError::InvalidRequirement),
                }
            };

            let added = teams.add_members(&world, team_name, &members).await;

            if members.len() == 1 {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_TEAM_JOIN_SUCCESS_SINGLE,
                        [
                            TextComponent::text(members[0].clone()),
                            TextComponent::text(team_name.to_string()),
                        ],
                    ))
                    .await;
            } else {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_TEAM_JOIN_SUCCESS_MULTIPLE,
                        [
                            TextComponent::text(added.to_string()),
                            TextComponent::text(team_name.to_string()),
                        ],
                    ))
                    .await;
            }
            Ok(added as i32)
        })
    }
}

struct LeaveExecutor;

impl CommandExecutor for LeaveExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let world = sender.world().ok_or(CommandError::InvalidRequirement)?;
            let mut teams = world.teams.lock().await;

            let members = if let Some(Arg::Simple(member)) = args.get(ARG_MEMBERS) {
                vec![member.to_string()]
            } else {
                match sender {
                    CommandSender::Player(p) => vec![p.gameprofile.name.clone()],
                    _ => return Err(CommandError::InvalidRequirement),
                }
            };

            let removed = teams.leave_members(&world, &members).await;

            if members.len() == 1 {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_TEAM_LEAVE_SUCCESS_SINGLE,
                        [TextComponent::text(members[0].clone())],
                    ))
                    .await;
            } else {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_TEAM_LEAVE_SUCCESS_MULTIPLE,
                        [TextComponent::text(removed.to_string())],
                    ))
                    .await;
            }
            Ok(removed as i32)
        })
    }
}

struct EmptyExecutor;

impl CommandExecutor for EmptyExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Simple(team_name)) = args.get(ARG_TEAM) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_TEAM.into())));
            };

            let world = sender.world().ok_or(CommandError::InvalidRequirement)?;
            let mut teams = world.teams.lock().await;

            if !teams.has_team(team_name) {
                return Err(CommandError::CommandFailed(TextComponent::text(format!(
                    "Unknown team '{team_name}'"
                ))));
            }

            let count = teams.empty_team(&world, team_name).await;

            if count == 0 {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_TEAM_EMPTY_UNCHANGED,
                        [],
                    ))
                    .await;
            } else {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_TEAM_EMPTY_SUCCESS,
                        [
                            TextComponent::text(count.to_string()),
                            TextComponent::text(team_name.to_string()),
                        ],
                    ))
                    .await;
            }
            Ok(count as i32)
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(literal("add").then(argument(ARG_TEAM, SimpleArgConsumer).execute(AddExecutor)))
        .then(literal("remove").then(argument(ARG_TEAM, SimpleArgConsumer).execute(RemoveExecutor)))
        .then(
            literal("list")
                .then(argument(ARG_TEAM, SimpleArgConsumer).execute(ListMembersExecutor))
                .execute(ListExecutor),
        )
        .then(
            literal("join").then(
                argument(ARG_TEAM, SimpleArgConsumer)
                    .then(argument(ARG_MEMBERS, SimpleArgConsumer).execute(JoinExecutor))
                    .execute(JoinExecutor),
            ),
        )
        .then(
            literal("leave")
                .then(argument(ARG_MEMBERS, SimpleArgConsumer).execute(LeaveExecutor))
                .execute(LeaveExecutor),
        )
        .then(literal("empty").then(argument(ARG_TEAM, SimpleArgConsumer).execute(EmptyExecutor)))
}
