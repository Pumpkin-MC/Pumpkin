use crate::{
    command::{
        CommandError, CommandExecutor, CommandResult, CommandSender,
        args::{Arg, ConsumedArgs, simple::SimpleArgConsumer},
        tree::{CommandTree, builder::argument},
    },
    data::{SaveJSONConfiguration, banned_player_data::BANNED_PLAYER_LIST},
};
use CommandError::InvalidConsumption;
use pumpkin_util::text::TextComponent;

const NAMES: [&str; 1] = ["pardon"];
const DESCRIPTION: &str = "unbans a player";

const ARG_TARGET: &str = "player";

struct Executor;

impl CommandExecutor for Executor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Simple(target)) = args.get(&ARG_TARGET) else {
                return Err(InvalidConsumption(Some(ARG_TARGET.into())));
            };
            let target = (*target).to_string();

            let mut lock = BANNED_PLAYER_LIST.write().await;

            if let Some(idx) = lock
                .banned_players
                .iter()
                .position(|entry| entry.name == target)
            {
                lock.banned_players.remove(idx);
            } else {
                sender
                    .send_message(TextComponent::translate("commands.pardon.failed", []))
                    .await;
                return Ok(());
            }

            lock.save();

            sender
                .send_message(TextComponent::translate(
                    "commands.pardon.success",
                    [TextComponent::text(target)],
                ))
                .await;
            Ok(())
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(argument(ARG_TARGET, SimpleArgConsumer).execute(Executor))
}
