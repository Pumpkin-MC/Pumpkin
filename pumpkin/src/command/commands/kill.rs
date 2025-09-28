use async_trait::async_trait;
use pumpkin_util::text::TextComponent;

use crate::command::args::entities::EntitiesArgumentConsumer;
use crate::command::args::{Arg, ConsumedArgs};
use crate::command::tree::CommandTree;
use crate::command::tree::builder::{argument, require};
use crate::command::{CommandError, CommandExecutor, CommandSender};
use crate::entity::EntityBase;
use CommandError::InvalidConsumption;

const NAMES: [&str; 1] = ["kill"];
const DESCRIPTION: &str = "Kills all target entities.";

const ARG_TARGET: &str = "target";

struct Executor;

#[async_trait]
impl CommandExecutor for Executor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        _server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let Some(Arg::Entities(targets)) = args.get(&ARG_TARGET) else {
            return Err(InvalidConsumption(Some(ARG_TARGET.into())));
        };

        let target_count = targets.len();
        for target in targets {
            target.kill(target.clone()).await;
        }

        let msg = if target_count == 1 {
            TextComponent::translate(
                "commands.kill.success.single",
                None,
                [targets[0].get_display_name().await],
            )
        } else {
            TextComponent::translate(
                "commands.kill.success.multiple",
                None,
                [TextComponent::text(target_count.to_string())],
            )
        };

        sender.send_message(msg).await;

        Ok(())
    }
}

struct SelfExecutor;

#[async_trait]
impl CommandExecutor for SelfExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        _server: &crate::server::Server,
        _args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let target = sender.as_player().ok_or(CommandError::InvalidRequirement)?;
        target.kill(target.clone()).await;

        sender
            .send_message(TextComponent::translate(
                "commands.kill.success.single",
                None,
                [target.get_display_name().await],
            ))
            .await;

        Ok(())
    }
}

#[allow(clippy::redundant_closure_for_method_calls)] // causes lifetime issues
pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(argument(ARG_TARGET, EntitiesArgumentConsumer).execute(Executor))
        .then(require(|sender| sender.is_player()).execute(SelfExecutor))
}
