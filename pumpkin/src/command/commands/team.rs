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
            // TODO: Implement team storage when team system is built
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
            // TODO: Implement team removal when team system is built
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
            // TODO: List actual teams when team system is built
            sender
                .send_message(TextComponent::translate(
                    translation::COMMANDS_TEAM_LIST_TEAMS_EMPTY,
                    [],
                ))
                .await;
            Ok(0)
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
            // TODO: Join team when team system is built
            let member_name = match sender {
                CommandSender::Player(p) => p.gameprofile.name.clone(),
                _ => "Unknown".to_string(),
            };
            sender
                .send_message(TextComponent::translate(
                    translation::COMMANDS_TEAM_JOIN_SUCCESS_SINGLE,
                    [
                        TextComponent::text(member_name),
                        TextComponent::text(team_name.to_string()),
                    ],
                ))
                .await;
            Ok(1)
        })
    }
}

struct LeaveExecutor;

impl CommandExecutor for LeaveExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            // TODO: Leave team when team system is built
            let member_name = match sender {
                CommandSender::Player(p) => p.gameprofile.name.clone(),
                _ => "Unknown".to_string(),
            };
            sender
                .send_message(TextComponent::translate(
                    translation::COMMANDS_TEAM_LEAVE_SUCCESS_SINGLE,
                    [TextComponent::text(member_name)],
                ))
                .await;
            Ok(1)
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
            // TODO: Empty team when team system is built
            sender
                .send_message(TextComponent::translate(
                    translation::COMMANDS_TEAM_EMPTY_SUCCESS,
                    [
                        TextComponent::text("0".to_string()),
                        TextComponent::text(team_name.to_string()),
                    ],
                ))
                .await;
            Ok(1)
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(literal("add").then(argument(ARG_TEAM, SimpleArgConsumer).execute(AddExecutor)))
        .then(literal("remove").then(argument(ARG_TEAM, SimpleArgConsumer).execute(RemoveExecutor)))
        .then(literal("list").execute(ListExecutor))
        .then(
            literal("join").then(
                argument(ARG_TEAM, SimpleArgConsumer)
                    .then(argument(ARG_MEMBERS, SimpleArgConsumer).execute(JoinExecutor))
                    .execute(JoinExecutor),
            ),
        )
        .then(
            literal("leave").then(argument(ARG_MEMBERS, SimpleArgConsumer).execute(LeaveExecutor)),
        )
        .then(literal("empty").then(argument(ARG_TEAM, SimpleArgConsumer).execute(EmptyExecutor)))
}
