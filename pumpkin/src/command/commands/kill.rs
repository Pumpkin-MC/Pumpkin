use pumpkin_util::text::TextComponent;

use crate::command::args::entities::EntitiesArgumentConsumer;
use crate::command::args::{Arg, ConsumedArgs};
use crate::command::tree::CommandTree;
use crate::command::tree::builder::{argument, require};
use crate::command::{CommandError, CommandExecutor, CommandResult, CommandSender};
use crate::entity::EntityBase;
use CommandError::InvalidConsumption;

const NAMES: [&str; 1] = ["kill"];
const DESCRIPTION: &str = "Kills all target entities.";

const ARG_TARGET: &str = "target";

struct Executor;

impl CommandExecutor for Executor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
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
                    [targets[0].get_display_name().await],
                )
            } else {
                TextComponent::translate(
                    "commands.kill.success.multiple",
                    [TextComponent::text(target_count.to_string())],
                )
            };

            sender.send_message(msg).await;

            Ok(())
        })
    }
}

struct SelfExecutor;

impl CommandExecutor for SelfExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let target = sender.as_player().ok_or(CommandError::InvalidRequirement)?;
            target.kill(target.clone()).await;

            sender
                .send_message(TextComponent::translate(
                    "commands.kill.success.single",
                    [target.get_display_name().await],
                ))
                .await;

            Ok(())
        })
    }
}

#[allow(clippy::redundant_closure_for_method_calls)] // causes lifetime issues
pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(argument(ARG_TARGET, EntitiesArgumentConsumer).execute(Executor))
        .then(require(|sender| sender.is_player()).execute(SelfExecutor))
}
