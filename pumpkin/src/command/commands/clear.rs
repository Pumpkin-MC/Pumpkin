use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::click::ClickEvent;
use pumpkin_util::text::color::NamedColor;
use pumpkin_util::text::hover::HoverEvent;
use pumpkin_world::item::ItemStack;

use crate::command::args::entities::EntitiesArgumentConsumer;
use crate::command::args::{Arg, ConsumedArgs};
use crate::command::tree::CommandTree;
use crate::command::tree::builder::{argument, require};
use crate::command::{CommandError, CommandExecutor, CommandSender};
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
        *slot_lock = ItemStack::EMPTY;
    }

    let entity_equipment_lock = inventory.entity_equipment.lock().await;
    for slot in entity_equipment_lock.equipment.values() {
        let mut slot_lock = slot.lock().await;
        if slot_lock.is_empty() {
            continue;
        }
        count += 1u64;
        *slot_lock = ItemStack::EMPTY;
    }

    count
}

fn clear_command_text_output(item_count: u64, targets: &[Arc<Player>]) -> TextComponent {
    match targets {
        [target] if item_count == 0 => TextComponent::translate(
            "clear.failed.single",
            [TextComponent::text(target.gameprofile.name.clone())],
        )
        .color_named(NamedColor::Red),
        [target] => TextComponent::translate(
            "commands.clear.success.single",
            [
                TextComponent::text(item_count.to_string()),
                TextComponent::text(target.gameprofile.name.clone())
                    .click_event(ClickEvent::SuggestCommand {
                        command: format!("/tell {} ", target.gameprofile.name.clone()).into(),
                    })
                    .hover_event(HoverEvent::show_entity(
                        target.living_entity.entity.entity_uuid.to_string(),
                        target.living_entity.entity.entity_type.resource_name.into(),
                        Some(TextComponent::text(target.gameprofile.name.clone())),
                    )),
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

        let mut item_count = 0;
        for target in targets {
            item_count += clear_player(target).await;
        }

        let msg = clear_command_text_output(item_count, targets);

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

        let item_count = clear_player(&target).await;

        let hold_target = [target];
        let msg = clear_command_text_output(item_count, &hold_target);

        sender.send_message(msg).await;

        Ok(())
    }
}

#[allow(clippy::redundant_closure_for_method_calls)] // causes lifetime issues
pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(argument(ARG_TARGET, EntitiesArgumentConsumer).execute(Executor))
        .then(require(|sender| sender.is_player()).execute(SelfExecutor))
}
