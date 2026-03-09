use pumpkin_data::translation;
use pumpkin_util::text::TextComponent;
use pumpkin_world::inventory::Inventory;
use pumpkin_world::item::ItemStack;

use crate::command::args::players::PlayersArgumentConsumer;
use crate::command::args::simple::SimpleArgConsumer;
use crate::command::args::{Arg, ConsumedArgs, FindArg};
use crate::command::dispatcher::CommandError;
use crate::command::tree::CommandTree;
use crate::command::tree::builder::{argument, literal};
use crate::command::{CommandExecutor, CommandResult, CommandSender};

const NAMES: [&str; 1] = ["item"];

const DESCRIPTION: &str = "Manipulates items in inventories.";

const ARG_TARGETS: &str = "targets";
const ARG_SLOT: &str = "slot";
const ARG_ITEM: &str = "item";
const ARG_COUNT: &str = "count";

fn parse_slot(slot_str: &str) -> Result<usize, CommandError> {
    // Parse slot identifiers like "container.0", "hotbar.0", "armor.head", etc.
    if let Some(rest) = slot_str.strip_prefix("container.") {
        rest.parse::<usize>()
            .map_err(|_| CommandError::CommandFailed(TextComponent::translate(
                translation::COMMANDS_ITEM_SOURCE_NO_SUCH_SLOT,
                [],
            )))
    } else if let Some(rest) = slot_str.strip_prefix("hotbar.") {
        let idx: usize = rest
            .parse()
            .map_err(|_| CommandError::CommandFailed(TextComponent::translate(
                translation::COMMANDS_ITEM_SOURCE_NO_SUCH_SLOT,
                [],
            )))?;
        if idx > 8 {
            return Err(CommandError::CommandFailed(TextComponent::translate(
                translation::COMMANDS_ITEM_SOURCE_NO_SUCH_SLOT,
                [],
            )));
        }
        Ok(idx)
    } else if let Some(rest) = slot_str.strip_prefix("inventory.") {
        let idx: usize = rest
            .parse()
            .map_err(|_| CommandError::CommandFailed(TextComponent::translate(
                translation::COMMANDS_ITEM_SOURCE_NO_SUCH_SLOT,
                [],
            )))?;
        Ok(idx + 9) // inventory slots start at 9
    } else {
        match slot_str {
            "armor.head" => Ok(36),
            "armor.chest" => Ok(37),
            "armor.legs" => Ok(38),
            "armor.feet" => Ok(39),
            "weapon.offhand" => Ok(40),
            "weapon.mainhand" => Ok(0), // Resolves to selected slot at runtime
            _ => Err(CommandError::CommandFailed(TextComponent::translate(
                translation::COMMANDS_ITEM_TARGET_NO_SUCH_SLOT,
                [],
            ))),
        }
    }
}

struct ReplaceEntityExecutor;

impl CommandExecutor for ReplaceEntityExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let targets = PlayersArgumentConsumer::find_arg(args, ARG_TARGETS)?;

            let Some(Arg::Simple(slot_str)) = args.get(ARG_SLOT) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_SLOT.into())));
            };
            let slot = parse_slot(slot_str)?;

            let Some(Arg::Simple(item_name)) = args.get(ARG_ITEM) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_ITEM.into())));
            };

            let item = pumpkin_data::item::Item::from_registry_key(item_name).ok_or(
                CommandError::CommandFailed(TextComponent::text(format!(
                    "Unknown item: {item_name}"
                ))),
            )?;

            let count: u8 = args
                .get(ARG_COUNT)
                .and_then(|a| {
                    if let Arg::Simple(s) = a {
                        s.parse::<u8>().ok()
                    } else {
                        None
                    }
                })
                .unwrap_or(1);

            let stack = ItemStack::new(count, item);
            let mut changed = 0i32;

            for target in targets {
                target.inventory().set_stack(slot, stack.clone()).await;
                changed += 1;
            }

            if targets.len() == 1 {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_ITEM_ENTITY_SET_SUCCESS_SINGLE,
                        [TextComponent::text(slot.to_string())],
                    ))
                    .await;
            } else {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_ITEM_ENTITY_SET_SUCCESS_MULTIPLE,
                        [
                            TextComponent::text(slot.to_string()),
                            TextComponent::text(changed.to_string()),
                        ],
                    ))
                    .await;
            }
            Ok(changed)
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).then(
        literal("replace").then(
            literal("entity").then(
                argument(ARG_TARGETS, PlayersArgumentConsumer).then(
                    argument(ARG_SLOT, SimpleArgConsumer).then(
                        literal("with").then(
                            argument(ARG_ITEM, SimpleArgConsumer)
                                .then(
                                    argument(ARG_COUNT, SimpleArgConsumer)
                                        .execute(ReplaceEntityExecutor),
                                )
                                .execute(ReplaceEntityExecutor),
                        ),
                    ),
                ),
            ),
        ),
    )
}
