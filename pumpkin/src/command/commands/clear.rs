use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_inventory::Container;
use pumpkin_util::text::click::ClickEvent;
use pumpkin_util::text::color::NamedColor;
use pumpkin_util::text::hover::HoverEvent;
use pumpkin_util::text::TextComponent;

use crate::command::args::entities::EntitiesArgumentConsumer;
use crate::command::args::{Arg, ConsumedArgs};
use crate::command::tree::builder::{argument, require};
use crate::command::tree::CommandTree;
use crate::command::{CommandError, CommandExecutor, CommandSender};
use crate::entity::player::Player;
use CommandError::InvalidConsumption;

const NAMES: [&str; 1] = ["clear"];
const DESCRIPTION: &str = "Clear yours or targets inventory.";

const ARG_TARGET: &str = "target";

async fn clear_player(target: &Player) -> usize {
    let mut inventory = target.inventory().lock().await;

    let slots = inventory.all_slots();
    let items_count = slots
        .iter()
        .filter_map(|slot| slot.as_ref().map(|slot| slot.item_count as usize))
        .sum();
    for slot in slots {
        *slot = None;
    }
    drop(inventory);
    target.set_container_content(None).await;
    items_count
}

fn clear_command_text_output(item_count: usize, targets: &[Arc<Player>]) -> TextComponent {
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
                    .click_event(ClickEvent::SuggestCommand(
                        format!("/tell {} ", target.gameprofile.name.clone()).into(),
                    ))
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

struct ClearExecutor;

#[async_trait]
impl CommandExecutor for ClearExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
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

struct ClearSelfExecutor;

#[async_trait]
impl CommandExecutor for ClearSelfExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
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
        .then(argument(ARG_TARGET, EntitiesArgumentConsumer).execute(ClearExecutor))
        .then(require(|sender| sender.is_player()).execute(ClearSelfExecutor))
}
