use pumpkin_data::translation;
use pumpkin_util::text::TextComponent;

use crate::command::args::simple::SimpleArgConsumer;
use crate::command::args::{Arg, ConsumedArgs};
use crate::command::dispatcher::CommandError;
use crate::command::tree::CommandTree;
use crate::command::tree::builder::{argument, literal};
use crate::command::{CommandExecutor, CommandResult, CommandSender};

const NAMES: [&str; 1] = ["trigger"];

const DESCRIPTION: &str = "Sets a trigger to be activated.";

const ARG_OBJECTIVE: &str = "objective";
const ARG_VALUE: &str = "value";

struct TriggerSimpleExecutor;

impl CommandExecutor for TriggerSimpleExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Simple(objective)) = args.get(ARG_OBJECTIVE) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_OBJECTIVE.into())));
            };

            let player = sender
                .as_player()
                .ok_or(CommandError::InvalidRequirement)?;
            let world = sender.world().ok_or(CommandError::InvalidRequirement)?;
            let mut scoreboard = world.scoreboard.lock().await;

            if !scoreboard.has_objective(objective) {
                return Err(CommandError::CommandFailed(TextComponent::translate(
                    translation::COMMANDS_TRIGGER_FAILED_INVALID,
                    [],
                )));
            }

            let current = scoreboard
                .get_score(&player.gameprofile.name, objective)
                .unwrap_or(0);
            let new_val = current + 1;
            scoreboard
                .set_score(&world, &player.gameprofile.name, objective, new_val)
                .await;

            sender
                .send_message(TextComponent::translate(
                    translation::COMMANDS_TRIGGER_SIMPLE_SUCCESS,
                    [TextComponent::text(objective.to_string())],
                ))
                .await;
            Ok(new_val)
        })
    }
}

struct TriggerAddExecutor;

impl CommandExecutor for TriggerAddExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Simple(objective)) = args.get(ARG_OBJECTIVE) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_OBJECTIVE.into())));
            };
            let Some(Arg::Simple(val_str)) = args.get(ARG_VALUE) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_VALUE.into())));
            };
            let value: i32 = val_str
                .parse()
                .map_err(|_| CommandError::InvalidConsumption(Some(ARG_VALUE.into())))?;

            let player = sender
                .as_player()
                .ok_or(CommandError::InvalidRequirement)?;
            let world = sender.world().ok_or(CommandError::InvalidRequirement)?;
            let mut scoreboard = world.scoreboard.lock().await;

            if !scoreboard.has_objective(objective) {
                return Err(CommandError::CommandFailed(TextComponent::translate(
                    translation::COMMANDS_TRIGGER_FAILED_INVALID,
                    [],
                )));
            }

            let current = scoreboard
                .get_score(&player.gameprofile.name, objective)
                .unwrap_or(0);
            let new_val = current + value;
            scoreboard
                .set_score(&world, &player.gameprofile.name, objective, new_val)
                .await;

            sender
                .send_message(TextComponent::translate(
                    translation::COMMANDS_TRIGGER_ADD_SUCCESS,
                    [
                        TextComponent::text(value.to_string()),
                        TextComponent::text(objective.to_string()),
                    ],
                ))
                .await;
            Ok(new_val)
        })
    }
}

struct TriggerSetExecutor;

impl CommandExecutor for TriggerSetExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Simple(objective)) = args.get(ARG_OBJECTIVE) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_OBJECTIVE.into())));
            };
            let Some(Arg::Simple(val_str)) = args.get(ARG_VALUE) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_VALUE.into())));
            };
            let value: i32 = val_str
                .parse()
                .map_err(|_| CommandError::InvalidConsumption(Some(ARG_VALUE.into())))?;

            let player = sender
                .as_player()
                .ok_or(CommandError::InvalidRequirement)?;
            let world = sender.world().ok_or(CommandError::InvalidRequirement)?;
            let mut scoreboard = world.scoreboard.lock().await;

            if !scoreboard.has_objective(objective) {
                return Err(CommandError::CommandFailed(TextComponent::translate(
                    translation::COMMANDS_TRIGGER_FAILED_INVALID,
                    [],
                )));
            }

            scoreboard
                .set_score(&world, &player.gameprofile.name, objective, value)
                .await;

            sender
                .send_message(TextComponent::translate(
                    translation::COMMANDS_TRIGGER_SET_SUCCESS,
                    [
                        TextComponent::text(objective.to_string()),
                        TextComponent::text(value.to_string()),
                    ],
                ))
                .await;
            Ok(value)
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).then(
        argument(ARG_OBJECTIVE, SimpleArgConsumer)
            .then(
                literal("add")
                    .then(argument(ARG_VALUE, SimpleArgConsumer).execute(TriggerAddExecutor)),
            )
            .then(
                literal("set")
                    .then(argument(ARG_VALUE, SimpleArgConsumer).execute(TriggerSetExecutor)),
            )
            .execute(TriggerSimpleExecutor),
    )
}
