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
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Players(targets)) = args.get(&ARG_TARGETS) else {
                return Err(InvalidConsumption(Some(ARG_TARGETS.into())));
            };

            let reason = match args.get(&ARG_REASON) {
                Some(Arg::Msg(r)) => TextComponent::text(r.clone()),
                _ => TextComponent::translate("multiplayer.disconnect.kicked", []),
            };

            for target in targets {
                target.kick(DisconnectReason::Kicked, reason.clone()).await;
                let mut msg = TextComponent::text("Kicked: ");
                msg = msg.add_child(target.get_display_name().await);
                sender.send_message(msg.color_named(NamedColor::Blue)).await;
            }

            Ok(())
        })
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
