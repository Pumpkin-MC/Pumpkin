use std::sync::Arc;

use pumpkin_util::text::TextComponent;
use pumpkin_util::text::color::NamedColor;
use pumpkin_world::item::ItemStack;

use crate::command::args::players::PlayersArgumentConsumer;
use crate::command::args::{Arg, ConsumedArgs};
use crate::command::tree::CommandTree;
use crate::command::tree::builder::{argument, require};
use crate::command::{CommandError, CommandExecutor, CommandResult, CommandSender};
use crate::entity::EntityBase;
use crate::entity::player::Player;
use CommandError::InvalidConsumption;

const NAMES: [&str; 1] = ["clear"];
const DESCRIPTION: &str = "Clear yours or targets inventory.";

const ARG_TARGET: &str = "target";

async fn clear_player(target: &Player) -> u64 {
    let inventory = target.inventory();
    let mut count: u64 = 0;
    for slot in &inventory.main_inventory {
        let mut slot_lock = slot.lock().await;
        count += u64::from(slot_lock.item_count);
        *slot_lock = ItemStack::EMPTY.clone();
    }

    let entity_equipment_lock = inventory.entity_equipment.lock().await;
    for slot in entity_equipment_lock.equipment.values() {
        let mut slot_lock = slot.lock().await;
        if slot_lock.is_empty() {
            continue;
        }
        count += 1u64;
        *slot_lock = ItemStack::EMPTY.clone();
    }

    count
}

async fn clear_command_text_output(item_count: u64, targets: &[Arc<Player>]) -> TextComponent {
    match targets {
        [target] if item_count == 0 => {
            TextComponent::translate("clear.failed.single", [target.get_display_name().await])
                .color_named(NamedColor::Red)
        }
        [target] => TextComponent::translate(
            "commands.clear.success.single",
            [
                TextComponent::text(item_count.to_string()),
                target.get_display_name().await,
            ],
        ),
        targets if item_count == 0 => TextComponent::translate(
            "clear.failed.multiple",
            [TextComponent::text(targets.len().to_string())],
        )
        .color_named(NamedColor::Red),
        targets => TextComponent::translate(
            "commands.clear.success.multiple",
            [
                TextComponent::text(item_count.to_string()),
                TextComponent::text(targets.len().to_string()),
            ],
        ),
    }
}

struct Executor;

impl CommandExecutor for Executor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Players(targets)) = args.get(&ARG_TARGET) else {
                return Err(InvalidConsumption(Some(ARG_TARGET.into())));
            };

            let mut item_count = 0;
            for target in targets {
                item_count += clear_player(target).await;
            }

            let msg = clear_command_text_output(item_count, targets).await;

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

            let item_count = clear_player(&target).await;

            let hold_target = [target];
            let msg = clear_command_text_output(item_count, &hold_target).await;

            sender.send_message(msg).await;

            Ok(())
        })
    }
}

// #[allow(clippy::redundant_closure_for_method_calls)] // causes lifetime issues
pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(argument(ARG_TARGET, PlayersArgumentConsumer).execute(Executor))
        .then(require(super::super::CommandSender::is_player).execute(SelfExecutor))
}
