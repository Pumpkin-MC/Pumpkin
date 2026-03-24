use pumpkin_data::translation;
use pumpkin_util::text::TextComponent;

use crate::command::args::simple::SimpleArgConsumer;
use crate::command::args::{Arg, ConsumedArgs};
use crate::command::dispatcher::CommandError;
use crate::command::tree::CommandTree;
use crate::command::tree::builder::{argument, literal};
use crate::command::{CommandExecutor, CommandResult, CommandSender};

const NAMES: [&str; 1] = ["random"];

const DESCRIPTION: &str = "Draw a random value or control random sequences.";

const ARG_RANGE: &str = "range";

fn parse_range(s: &str) -> Result<(i32, i32), CommandError> {
    if let Some(rest) = s.strip_prefix("..") {
        let max: i32 = rest
            .parse()
            .map_err(|_| CommandError::InvalidConsumption(Some(ARG_RANGE.into())))?;
        Ok((i32::MIN, max))
    } else if let Some(rest) = s.strip_suffix("..") {
        let min: i32 = rest
            .parse()
            .map_err(|_| CommandError::InvalidConsumption(Some(ARG_RANGE.into())))?;
        Ok((min, i32::MAX))
    } else if let Some((min_s, max_s)) = s.split_once("..") {
        let min: i32 = min_s
            .parse()
            .map_err(|_| CommandError::InvalidConsumption(Some(ARG_RANGE.into())))?;
        let max: i32 = max_s
            .parse()
            .map_err(|_| CommandError::InvalidConsumption(Some(ARG_RANGE.into())))?;
        Ok((min, max))
    } else {
        let val: i32 = s
            .parse()
            .map_err(|_| CommandError::InvalidConsumption(Some(ARG_RANGE.into())))?;
        Ok((val, val))
    }
}

struct ValueExecutor;

impl CommandExecutor for ValueExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Simple(range_str)) = args.get(ARG_RANGE) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_RANGE.into())));
            };

            let (min, max) = parse_range(range_str)?;

            if min == max {
                return Err(CommandError::CommandFailed(TextComponent::translate(
                    translation::COMMANDS_RANDOM_ERROR_RANGE_TOO_SMALL,
                    [],
                )));
            }

            let range_size = (max as i64) - (min as i64) + 1;
            if !(2..=2_147_483_646).contains(&range_size) {
                return Err(CommandError::CommandFailed(TextComponent::translate(
                    translation::COMMANDS_RANDOM_ERROR_RANGE_TOO_LARGE,
                    [],
                )));
            }

            let result = rand::random_range(min..=max);

            sender
                .send_message(TextComponent::translate(
                    translation::COMMANDS_RANDOM_SAMPLE_SUCCESS,
                    [TextComponent::text(result.to_string())],
                ))
                .await;
            Ok(result)
        })
    }
}

struct RollExecutor;

impl CommandExecutor for RollExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Simple(range_str)) = args.get(ARG_RANGE) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_RANGE.into())));
            };

            let (min, max) = parse_range(range_str)?;

            if min == max {
                return Err(CommandError::CommandFailed(TextComponent::translate(
                    translation::COMMANDS_RANDOM_ERROR_RANGE_TOO_SMALL,
                    [],
                )));
            }

            let range_size = (max as i64) - (min as i64) + 1;
            if !(2..=2_147_483_646).contains(&range_size) {
                return Err(CommandError::CommandFailed(TextComponent::translate(
                    translation::COMMANDS_RANDOM_ERROR_RANGE_TOO_LARGE,
                    [],
                )));
            }

            let result = rand::random_range(min..=max);

            let sender_name = match sender {
                CommandSender::Player(p) => p.gameprofile.name.clone(),
                _ => "Server".to_string(),
            };

            // Broadcast to all players
            let msg = TextComponent::translate(
                translation::COMMANDS_RANDOM_ROLL,
                [
                    TextComponent::text(sender_name),
                    TextComponent::text(result.to_string()),
                    TextComponent::text(min.to_string()),
                    TextComponent::text(max.to_string()),
                ],
            );

            for world in server.worlds.load().iter() {
                for player in world.players.load().iter() {
                    player.send_system_message(&msg).await;
                }
            }

            Ok(result)
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(literal("value").then(argument(ARG_RANGE, SimpleArgConsumer).execute(ValueExecutor)))
        .then(literal("roll").then(argument(ARG_RANGE, SimpleArgConsumer).execute(RollExecutor)))
}
