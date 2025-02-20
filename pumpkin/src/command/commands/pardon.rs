use crate::{
    command::{
        args::{simple::SimpleArgConsumer, Arg, ConsumedArgs},
        tree::builder::argument,
        tree::CommandTree,
        CommandError, CommandExecutor, CommandSender,
    },
    data::{banned_player_data::BANNED_PLAYER_LIST, SaveJSONConfiguration},
};
use async_trait::async_trait;
use pumpkin_util::text::TextComponent;
use CommandError::InvalidConsumption;

const NAMES: [&str; 1] = ["pardon"];
const DESCRIPTION: &str = "unbans a player";

const ARG_TARGET: &str = "player";

struct PardonExecutor;

#[async_trait]
impl CommandExecutor for PardonExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
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
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(argument(ARG_TARGET, SimpleArgConsumer).execute(PardonExecutor))
}
