use async_trait::async_trait;
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::color::NamedColor;

use crate::command::CommandError;
use crate::command::args::message::MsgArgConsumer;
use crate::command::args::players::PlayersArgumentConsumer;
use crate::command::args::{Arg, ConsumedArgs};
use crate::command::tree::CommandTree;
use crate::command::tree::builder::argument;
use crate::command::{CommandExecutor, CommandSender};
use CommandError::InvalidConsumption;

const NAMES: [&str; 1] = ["kick"];
const DESCRIPTION: &str = "Kicks the target player from the server.";

const ARG_TARGETS: &str = "targets";

const ARG_REASON: &str = "reason";

struct Executor;

#[async_trait]
impl CommandExecutor for Executor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let Some(Arg::Players(targets)) = args.get(&ARG_TARGETS) else {
            return Err(InvalidConsumption(Some(ARG_TARGETS.into())));
        };

        let reason = match args.get(&ARG_REASON) {
            Some(Arg::Msg(r)) => TextComponent::text(r.clone()),
            _ => TextComponent::translate("multiplayer.disconnect.kicked", []),
        };

        for target in targets {
            target.kick(reason.clone()).await;
            let name = &target.gameprofile.name;
            let msg = TextComponent::text(format!("Kicked: {name}"));
            sender.send_message(msg.color_named(NamedColor::Blue)).await;
        }

        Ok(())
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
