use std::collections::HashSet;

use pumpkin_data::translation;
use pumpkin_util::text::TextComponent;

use crate::command::args::message::MsgArgConsumer;
use crate::command::args::{Arg, ConsumedArgs};
use crate::command::dispatcher::CommandError;
use crate::command::tree::CommandTree;
use crate::command::tree::builder::argument;
use crate::command::{CommandExecutor, CommandResult, CommandSender};

const NAMES: [&str; 2] = ["teammsg", "tm"];

const DESCRIPTION: &str = "Sends a message to all players on the sender's team.";

const ARG_MESSAGE: &str = "message";

struct TeamMsgExecutor;

impl CommandExecutor for TeamMsgExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Msg(message)) = args.get(ARG_MESSAGE) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_MESSAGE.into())));
            };

            let CommandSender::Player(player) = sender else {
                return Err(CommandError::InvalidRequirement);
            };

            let sender_name = player.gameprofile.name.clone();
            let world = player.living_entity.entity.world.load_full();

            // Get team info and member list, then drop the lock
            let (team_name, members): (String, HashSet<String>) = {
                let teams = world.teams.lock().await;
                let Some(name) = teams.get_member_team(&sender_name) else {
                    return Err(CommandError::CommandFailed(TextComponent::translate(
                        translation::COMMANDS_TEAMMSG_FAILED_NOTEAM,
                        [],
                    )));
                };
                let team = teams.get_team(name).unwrap();
                (name.to_string(), team.members.clone())
            };

            let msg = TextComponent::text(format!("[{team_name}] <{sender_name}> {message}"));

            let players = world.players.load();
            for p in players.iter() {
                if members.contains(&p.gameprofile.name) {
                    p.send_system_message(&msg).await;
                }
            }

            Ok(1)
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(argument(ARG_MESSAGE, MsgArgConsumer).execute(TeamMsgExecutor))
}
