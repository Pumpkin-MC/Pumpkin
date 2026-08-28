use pumpkin_data::translation;
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::color::NamedColor;

use crate::command::args::message::MsgArgConsumer;
use crate::command::args::players::PlayersArgumentConsumer;
use crate::command::args::{Arg, ConsumedArgs};
use crate::command::tree::CommandTree;
use crate::command::tree::builder::argument;
use crate::command::{CommandError, CommandResult};
use crate::command::{CommandExecutor, CommandSender};
use crate::entity::EntityBase;
use crate::net::DisconnectReason;
use CommandError::InvalidConsumption;

const NAMES: [&str; 1] = ["kick"];
const DESCRIPTION: &str = "Kicks the target player from the server.";

const ARG_TARGETS: &str = "targets";

const ARG_REASON: &str = "reason";

struct Executor;

impl CommandExecutor for Executor {
    fn execute(
        &self,
        sender: &CommandSender,
        _server: &crate::server::Server,
        args: &ConsumedArgs,
    ) -> CommandResult {
        let Some(Arg::Players(targets)) = args.get(&ARG_TARGETS) else {
            return Err(InvalidConsumption(Some(ARG_TARGETS.into())));
        };

        let custom_reason = args.get(&ARG_REASON);
        let reason = match custom_reason {
            Some(Arg::Msg(r)) => TextComponent::text(r.clone()),
            _ => TextComponent::translate_cross(
                translation::java::MULTIPLAYER_DISCONNECT_KICKED,
                translation::bedrock::DISCONNECT_KICKED,
                [],
            ),
        };

        for target in targets {
            target.kick(DisconnectReason::Kicked, &reason);

            let feedback = if custom_reason.is_some() {
                TextComponent::translate_cross(
                    translation::java::COMMANDS_KICK_SUCCESS,
                    translation::bedrock::COMMANDS_KICK_SUCCESS_REASON,
                    [target.get_display_name(), reason.clone()],
                )
            } else {
                TextComponent::translate_cross(
                    translation::java::COMMANDS_KICK_SUCCESS,
                    translation::bedrock::COMMANDS_KICK_SUCCESS,
                    [target.get_display_name(), reason.clone()],
                )
            };

            sender.send_message(feedback.color_named(NamedColor::Blue));
        }

        Ok(targets.len() as i32)
    }
}

// TODO: Permission
pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).then(
        argument(ARG_TARGETS, PlayersArgumentConsumer)
            .execute(Executor)
            .then(argument(ARG_REASON, MsgArgConsumer).execute(Executor)),
    )
}
