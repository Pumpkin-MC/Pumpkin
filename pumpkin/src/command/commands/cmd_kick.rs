use async_trait::async_trait;
use pumpkin_util::text::color::NamedColor;
use pumpkin_util::text::TextComponent;

use crate::command::args::arg_players::PlayersArgumentConsumer;
use crate::command::args::{Arg, ConsumedArgs};
use crate::command::tree::CommandTree;
use crate::command::tree_builder::argument;
use crate::command::CommandError;
use crate::command::{CommandExecutor, CommandSender};
use CommandError::InvalidConsumption;

const NAMES: [&str; 1] = ["kick"];
const DESCRIPTION: &str = "Kicks the target player from the server.";

const ARG_TARGET: &str = "target";

struct KickExecutor;

#[async_trait]
impl CommandExecutor for KickExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let Some(Arg::Players(targets)) = args.get(&ARG_TARGET) else {
            return Err(InvalidConsumption(Some(ARG_TARGET.into())));
        };

        for target in targets {
            target
                .kick(TextComponent::translate(
                    "multiplayer.disconnect.kicked",
                    [],
                ))
                .await;
            let name = &target.gameprofile.name;
            let msg = TextComponent::text(format!("Kicked: {name}"));
            sender.send_message(msg.color_named(NamedColor::Blue)).await;
        }

        Ok(())
    }
}

// TODO: Permission
pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .with_child(argument(ARG_TARGET, PlayersArgumentConsumer).execute(KickExecutor))
}
