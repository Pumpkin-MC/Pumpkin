use async_trait::async_trait;
use pumpkin_util::text::color::NamedColor;
use pumpkin_util::text::TextComponent;

use crate::command::args::entities::EntitiesArgumentConsumer;
use crate::command::args::{Arg, ConsumedArgs};
use crate::command::tree::CommandTree;
use crate::command::tree_builder::{argument, require};
use crate::command::{CommandError, CommandExecutor, CommandSender};
use CommandError::InvalidConsumption;

const NAMES: [&str; 1] = ["kill"];
const DESCRIPTION: &str = "Kills all target entities.";

const ARG_TARGET: &str = "target";

struct KillExecutor;

#[async_trait]
impl CommandExecutor for KillExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let Some(Arg::Entities(targets)) = args.get(&ARG_TARGET) else {
            return Err(InvalidConsumption(Some(ARG_TARGET.into())));
        };

        let target_count = targets.len();
        let mut name = String::new();
        for target in targets {
            target.living_entity.kill().await;
            name.clone_from(&target.gameprofile.name);
        }

        let msg = if target_count == 1 {
            TextComponent::translate(
                "commands.kill.success.single",
                [TextComponent::text(name)].into(),
            )
        } else {
            TextComponent::translate(
                "commands.kill.success.multiple",
                [TextComponent::text(target_count.to_string())].into(),
            )
        };

        sender.send_message(msg.color_named(NamedColor::Blue)).await;

        Ok(())
    }
}

struct KillSelfExecutor;

#[async_trait]
impl CommandExecutor for KillSelfExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &crate::server::Server,
        _args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let target = sender.as_player().ok_or(CommandError::InvalidRequirement)?;

        target.living_entity.kill().await;

        Ok(())
    }
}

#[allow(clippy::redundant_closure_for_method_calls)] // causes lifetime issues
pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(argument(ARG_TARGET, EntitiesArgumentConsumer).execute(KillExecutor))
        .then(require(|sender| sender.is_player()).execute(KillSelfExecutor))
}
