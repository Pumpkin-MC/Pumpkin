use std::sync::Arc;

use pumpkin_util::text::TextComponent;
use pumpkin_world::item::ItemStack;
use tokio::sync::Mutex;

use crate::command::args::bounded_num::BoundedNumArgumentConsumer;
use crate::command::args::players::PlayersArgumentConsumer;
use crate::command::args::resource::item::{ItemPredicate, ItemPredicateArgumentConsumer};
use crate::command::args::{Arg, ConsumedArgs, FindArg, FindArgDefaultName};
use crate::command::tree::CommandTree;
use crate::command::tree::builder::{argument, require};
use crate::command::{CommandError, CommandExecutor, CommandResult, CommandSender};
use crate::entity::EntityBase;
use crate::entity::player::Player;
use CommandError::InvalidConsumption;

const NAMES: [&str; 1] = ["clear"];
const DESCRIPTION: &str = "Clear your inventory or that of target(s)";

const ARG_TARGETS: &str = "targets";
const ARG_ITEM: &str = "item";
const ARG_COUNT: &str = "count";

const MAX_NO_UPPER_LIMIT: i32 = -1;
const MAX_NO_CLEAR_BUT_SIMULATE: i32 = 0;

/// Returns the number of items actually cleared.
/// 
/// If `max` provided is equal to [`MAX_NO_CLEAR_BUT_SIMULATE`] (`0`), then no items are cleared,
/// but instead returns the items that *could* be cleared.
/// 
/// If `max` provided is [`MAX_NO_UPPER_LIMIT`] (`-1`), then there is no limit in clearing.
/// 
/// Otherwise, at most `max` items are cleared.
async fn clear_player(target: &Player, item: &ItemPredicate, max: i32) -> i32 {
    let inventory = target.inventory();
    let mut count: i32 = 0;
    let mut max: i32 = max;
    let mut is_done: bool = false;

    iter_test_and_clear(&inventory.main_inventory, &mut count, &mut max, item, &mut is_done).await;

    let entity_equipment_lock = inventory.entity_equipment.lock().await;
    iter_test_and_clear(entity_equipment_lock.equipment.values(), &mut count, &mut max, item, &mut is_done).await;
    drop(entity_equipment_lock);

    async fn test_and_clear(count: &mut i32, max: &mut i32, item: &ItemPredicate, slot: &Arc<Mutex<ItemStack>>, is_done: &mut bool) {
        let mut slot_lock = slot.lock().await;
        if item.test_item_stack(&slot_lock) {
            let item_count = slot_lock.item_count as i32;
            if *max == MAX_NO_CLEAR_BUT_SIMULATE {
                *count += item_count;
            } else if *max == MAX_NO_UPPER_LIMIT {
                *count += item_count;
                *slot_lock = ItemStack::EMPTY.clone();
            } else {
                // We need more complex logic for this one.
                let taken = i32::min(*max, item_count);
                if taken == item_count {
                    // Take all items (empty it)
                    *count += taken;
                    *slot_lock = ItemStack::EMPTY.clone();
                    *max -= taken;
                    
                    // Set `is_done` flag if required.
                    *is_done = *max == 0;
                } else {
                    // Take some items (effectively finishing the operation)
                    *count += taken;
                    // As `slot_lock.item_count` is limited to `u8`, this should be fine.
                    slot_lock.decrement(taken as u8);
                    *max -= taken;

                    // Set `is_done` flag if required.
                    *is_done = true;
                }
            }
        }
    }

    async fn iter_test_and_clear<'i, I>(iter: I, count: &mut i32, max: &mut i32, item: &ItemPredicate, is_done: &mut bool)
    where
        I: IntoIterator<Item = &'i Arc<Mutex<ItemStack>>>,
    {
        // Don't need to enter the loop if we are already done.
        if !*is_done {
            for slot in iter {
                test_and_clear(count, max, item, slot, is_done).await;
                if *is_done {
                    break;
                }
            }
        }
    }

    count
}

async fn command_result(sender: &CommandSender, item_count: i32, max_count: i32, targets: &[Arc<Player>]) -> Result<i32, CommandError> {
    match clear_command_text_output(item_count, max_count, targets).await {
        Ok(success) => {
            sender.send_message(success).await;
            Ok(item_count)
        },
        Err(failure) => Err(CommandError::CommandFailed(failure))
    }
}

async fn clear_command_text_output(item_count: i32, max_count: i32, targets: &[Arc<Player>]) -> Result<TextComponent, TextComponent> {
    match (targets, item_count == 0, max_count == 0) {
        ([target], true, _) => {
            Err(
                TextComponent::translate("clear.failed.single", [target.get_display_name().await])
            )
        },
        (targets, true, _) => {
            Err(
                TextComponent::translate(
                "clear.failed.multiple",
                [TextComponent::text(targets.len().to_string())],
                )
            )
        },
        ([target], false, false) => {
            Ok(
                TextComponent::translate(
                    "commands.clear.success.single",
                    [
                        TextComponent::text(item_count.to_string()),
                        target.get_display_name().await,
                    ],
                )
            )
        },
        (targets, false, false) => {
            Ok(
                TextComponent::translate(
                    "commands.clear.success.multiple",
                    [
                        TextComponent::text(item_count.to_string()),
                        TextComponent::text(targets.len().to_string()),
                    ],
                )
            )
        },
        ([target], false, true) => {
            Ok(
                TextComponent::translate(
                    "commands.clear.test.single",
                    [
                        TextComponent::text(item_count.to_string()),
                        target.get_display_name().await,
                    ],
                )
            )
        },
        (targets, false, true) => {
            Ok(
                TextComponent::translate(
                    "commands.clear.test.multiple",
                    [
                        TextComponent::text(item_count.to_string()),
                        TextComponent::text(targets.len().to_string()),
                    ],
                )
            )
        },
    }
}

fn count_consumer() -> BoundedNumArgumentConsumer<i32> {
    BoundedNumArgumentConsumer::new().min(0)
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

            let items_cleared = clear_player(&target, &ItemPredicate::Any, MAX_NO_UPPER_LIMIT).await;

            command_result(sender, items_cleared, MAX_NO_UPPER_LIMIT, &[target]).await
        })
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
            let Some(Arg::Players(targets)) = args.get(&ARG_TARGETS) else {
                return Err(InvalidConsumption(Some(ARG_TARGETS.into())));
            };

            let mut total_items_cleared = 0;
            for target in targets {
                total_items_cleared += clear_player(target, &ItemPredicate::Any, MAX_NO_UPPER_LIMIT).await;
            }

            command_result(sender, total_items_cleared, MAX_NO_UPPER_LIMIT, targets).await
        })
    }
}

struct ItemExecutor;

impl CommandExecutor for ItemExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Players(targets)) = args.get(&ARG_TARGETS) else {
                return Err(InvalidConsumption(Some(ARG_TARGETS.into())));
            };

            let item = ItemPredicateArgumentConsumer::find_arg(args, ARG_ITEM)?;

            let mut total_items_cleared = 0;
            for target in targets {
                total_items_cleared += clear_player(target, &item, MAX_NO_UPPER_LIMIT).await;
            }

            command_result(sender, total_items_cleared, MAX_NO_UPPER_LIMIT, targets).await
        })
    }
}

struct ItemCountExecutor;

impl CommandExecutor for ItemCountExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Players(targets)) = args.get(&ARG_TARGETS) else {
                return Err(InvalidConsumption(Some(ARG_TARGETS.into())));
            };

            let item = ItemPredicateArgumentConsumer::find_arg(args, ARG_ITEM)?;
            let Ok(max) = count_consumer().find_arg_default_name(args)? else {
                return Err(
                    CommandError::CommandFailed(
                        TextComponent::translate(
                            "parsing.int.invalid",
                            [TextComponent::text(i32::MAX.to_string())],
                        )
                    )
                );
            };

            let mut total_items_cleared = 0;
            for target in targets {
                total_items_cleared += clear_player(target, &item, max).await;
            }

            command_result(sender, total_items_cleared, max, targets).await
        })
    }
}

// #[expect(clippy::redundant_closure_for_method_calls)] // causes lifetime issues
pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(
            argument(ARG_TARGETS, PlayersArgumentConsumer).execute(Executor)
            .then(
                argument(ARG_ITEM, ItemPredicateArgumentConsumer).execute(ItemExecutor)
                .then(
                    argument(ARG_COUNT, count_consumer().name(ARG_COUNT)).execute(ItemCountExecutor)
                )
            )
        )
        .then(require(super::super::CommandSender::is_player).execute(SelfExecutor))
}
