use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::translation;
use pumpkin_inventory::screen_handler::InventoryPlayer;
use pumpkin_protocol::codec::item_stack_seralizer::ItemStackSerializer;
use pumpkin_protocol::java::client::play::CSetContainerSlot;
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::hover::HoverEvent;
use pumpkin_world::inventory::Inventory;

use crate::command::args::bounded_num::BoundedNumArgumentConsumer;
use crate::command::args::entities::EntitiesArgumentConsumer;
use crate::command::args::position_block::BlockPosArgumentConsumer;
use crate::command::args::resource::item::ItemArgumentConsumer;
use crate::command::args::slot::SlotArgumentConsumer;
use crate::command::args::{ConsumedArgs, FindArg, FindArgDefaultName};
use crate::command::dispatcher::CommandError;
use crate::command::tree::CommandTree;
use crate::command::tree::builder::{argument, argument_default_name, literal};
use crate::command::{CommandExecutor, CommandResult, CommandSender};

const NAMES: [&str; 1] = ["item"];
const DESCRIPTION: &str = "Modifies items in block or entity inventories.";

const ARG_POS: &str = "pos";
const ARG_SLOT: &str = "slot";
const ARG_ITEM: &str = "item";
const ARG_TARGETS: &str = "targets";

const fn count_consumer() -> BoundedNumArgumentConsumer<i32> {
    BoundedNumArgumentConsumer::new()
        .name("count")
        .min(1)
        .max(99)
}

struct BlockReplaceExecutor;

impl CommandExecutor for BlockReplaceExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let world = match sender {
                CommandSender::Console | CommandSender::Rcon(_) | CommandSender::Dummy => {
                    let guard = server.worlds.load();
                    guard
                        .first()
                        .cloned()
                        .ok_or(CommandError::InvalidRequirement)?
                }
                CommandSender::Player(player) => player.world().clone(),
                CommandSender::CommandBlock(_, w) => w.clone(),
            };

            let pos = BlockPosArgumentConsumer::find_loaded_arg(args, ARG_POS, &world)?;
            let (slot, slot_name) = SlotArgumentConsumer::find_arg(args, ARG_SLOT)?;
            let (item_name, parsed_stack) = ItemArgumentConsumer::find_arg(args, ARG_ITEM)?;
            let item = parsed_stack.item;
            let count = match count_consumer().find_arg_default_name(args) {
                Ok(Ok(c)) => c,
                Err(_) | Ok(Err(_)) => 1,
            };

            let block_entity = world.get_block_entity(&pos).ok_or_else(|| {
                CommandError::CommandFailed(TextComponent::translate_cross(
                    translation::java::COMMANDS_ITEM_TARGET_NOT_A_CONTAINER,
                    translation::java::COMMANDS_ITEM_TARGET_NOT_A_CONTAINER,
                    [
                        TextComponent::text(pos.0.x.to_string()),
                        TextComponent::text(pos.0.y.to_string()),
                        TextComponent::text(pos.0.z.to_string()),
                    ],
                ))
            })?;

            let inventory = block_entity.get_inventory().ok_or_else(|| {
                CommandError::CommandFailed(TextComponent::translate_cross(
                    translation::java::COMMANDS_ITEM_TARGET_NOT_A_CONTAINER,
                    translation::java::COMMANDS_ITEM_TARGET_NOT_A_CONTAINER,
                    [
                        TextComponent::text(pos.0.x.to_string()),
                        TextComponent::text(pos.0.y.to_string()),
                        TextComponent::text(pos.0.z.to_string()),
                    ],
                ))
            })?;

            if slot >= inventory.size() {
                return Err(CommandError::CommandFailed(TextComponent::translate_cross(
                    translation::java::COMMANDS_ITEM_TARGET_NO_SUCH_SLOT,
                    translation::java::COMMANDS_ITEM_TARGET_NO_SUCH_SLOT,
                    [TextComponent::text(slot_name)],
                )));
            }

            let mut item_stack = parsed_stack.clone();
            item_stack.item_count = count as u8;
            inventory.set_stack(slot, item_stack.clone()).await;

            let msg = TextComponent::translate_cross(
                translation::java::COMMANDS_ITEM_BLOCK_SET_SUCCESS,
                translation::java::COMMANDS_ITEM_BLOCK_SET_SUCCESS,
                [
                    TextComponent::text(pos.0.x.to_string()),
                    TextComponent::text(pos.0.y.to_string()),
                    TextComponent::text(pos.0.z.to_string()),
                    TextComponent::text("[")
                        .add_child(item.translated_name())
                        .add_child(TextComponent::text("]"))
                        .hover_event(HoverEvent::ShowItem {
                            id: item_name.to_string().into(),
                            count: Some(count),
                        }),
                ],
            );
            sender.send_message(msg).await;

            Ok(1)
        })
    }
}

struct EntityReplaceExecutor;

impl CommandExecutor for EntityReplaceExecutor {
    #[expect(clippy::too_many_lines)]
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let targets = EntitiesArgumentConsumer.find_arg_default_name(args)?;
            let (mojang_slot, slot_name) = SlotArgumentConsumer::find_arg(args, ARG_SLOT)?;
            let (item_name, parsed_stack) = ItemArgumentConsumer::find_arg(args, ARG_ITEM)?;
            let item = parsed_stack.item;
            let count = match count_consumer().find_arg_default_name(args) {
                Ok(Ok(c)) => c,
                Err(_) | Ok(Err(_)) => 1,
            };

            let mut modified_count = 0;
            let mut item_stack = parsed_stack.clone();
            item_stack.item_count = count as u8;

            for target in targets {
                if let Some(player) = target.get_player() {
                    if (200..=226).contains(&mojang_slot) {
                        let ender_slot = mojang_slot - 200;
                        if ender_slot < player.ender_chest_inventory.size() {
                            player
                                .ender_chest_inventory
                                .set_stack(ender_slot, item_stack.clone())
                                .await;
                            modified_count += 1;
                        }
                    } else {
                        let inventory = player.inventory();
                        let mapped_slot = if mojang_slot == 98 {
                            Some(inventory.get_selected_slot() as usize)
                        } else if mojang_slot == 99 {
                            Some(40)
                        } else if mojang_slot == 100 {
                            Some(36)
                        } else if mojang_slot == 101 {
                            Some(37)
                        } else if mojang_slot == 102 {
                            Some(38)
                        } else if mojang_slot == 103 {
                            Some(39)
                        } else if mojang_slot <= 35 {
                            Some(mojang_slot)
                        } else {
                            None
                        };

                        if let Some(slot) = mapped_slot
                            && slot < inventory.size()
                        {
                            inventory.set_stack(slot, item_stack.clone()).await;

                            let stack_serializer = ItemStackSerializer::from(item_stack.clone());
                            let packet =
                                CSetContainerSlot::new(0, 0, slot as i16, &stack_serializer);
                            player.enqueue_slot_packet(&packet).await;

                            let eq_slot = if slot == 36 {
                                Some(EquipmentSlot::FEET)
                            } else if slot == 37 {
                                Some(EquipmentSlot::LEGS)
                            } else if slot == 38 {
                                Some(EquipmentSlot::CHEST)
                            } else if slot == 39 {
                                Some(EquipmentSlot::HEAD)
                            } else if slot == 40 {
                                Some(EquipmentSlot::OFF_HAND)
                            } else if slot == inventory.get_selected_slot() as usize {
                                Some(EquipmentSlot::MAIN_HAND)
                            } else {
                                None
                            };

                            if let Some(eq) = eq_slot {
                                player
                                    .living_entity
                                    .send_equipment_changes(&[(eq, item_stack.clone())]);
                            }

                            modified_count += 1;
                        }
                    }
                } else if let Some(living) = target.get_living_entity() {
                    let mapped_eq = if mojang_slot == 98 {
                        Some(EquipmentSlot::MAIN_HAND)
                    } else if mojang_slot == 99 {
                        Some(EquipmentSlot::OFF_HAND)
                    } else if mojang_slot == 100 {
                        Some(EquipmentSlot::FEET)
                    } else if mojang_slot == 101 {
                        Some(EquipmentSlot::LEGS)
                    } else if mojang_slot == 102 {
                        Some(EquipmentSlot::CHEST)
                    } else if mojang_slot == 103 {
                        Some(EquipmentSlot::HEAD)
                    } else if mojang_slot == 105 {
                        Some(EquipmentSlot::BODY)
                    } else if mojang_slot == 106 {
                        Some(EquipmentSlot::SADDLE)
                    } else {
                        None
                    };

                    if let Some(eq) = mapped_eq {
                        living
                            .entity_equipment
                            .lock()
                            .await
                            .put(&eq, item_stack.clone())
                            .await;
                        living.send_equipment_changes(&[(eq, item_stack.clone())]);
                        modified_count += 1;
                    }
                }
            }

            if modified_count == 0 {
                return Err(CommandError::CommandFailed(TextComponent::translate_cross(
                    translation::java::COMMANDS_ITEM_TARGET_NO_SUCH_SLOT,
                    translation::java::COMMANDS_ITEM_TARGET_NO_SUCH_SLOT,
                    [TextComponent::text(slot_name)],
                )));
            }

            let msg = if targets.len() == 1 {
                TextComponent::translate_cross(
                    translation::java::COMMANDS_ITEM_ENTITY_SET_SUCCESS_SINGLE,
                    translation::java::COMMANDS_ITEM_ENTITY_SET_SUCCESS_SINGLE,
                    [
                        targets[0].get_display_name().await,
                        TextComponent::text("[")
                            .add_child(item.translated_name())
                            .add_child(TextComponent::text("]"))
                            .hover_event(HoverEvent::ShowItem {
                                id: item_name.to_string().into(),
                                count: Some(count),
                            }),
                    ],
                )
            } else {
                TextComponent::translate_cross(
                    translation::java::COMMANDS_ITEM_ENTITY_SET_SUCCESS_MULTIPLE,
                    translation::java::COMMANDS_ITEM_ENTITY_SET_SUCCESS_MULTIPLE,
                    [
                        TextComponent::text(modified_count.to_string()),
                        TextComponent::text("[")
                            .add_child(item.translated_name())
                            .add_child(TextComponent::text("]"))
                            .hover_event(HoverEvent::ShowItem {
                                id: item_name.to_string().into(),
                                count: Some(count),
                            }),
                    ],
                )
            };
            sender.send_message(msg).await;

            Ok(modified_count)
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).then(
        literal("replace")
            .then(
                literal("block").then(
                    argument(ARG_POS, BlockPosArgumentConsumer).then(
                        argument(ARG_SLOT, SlotArgumentConsumer)
                            .then(
                                argument(ARG_ITEM, ItemArgumentConsumer)
                                    .execute(BlockReplaceExecutor)
                                    .then(
                                        argument_default_name(count_consumer())
                                            .execute(BlockReplaceExecutor),
                                    ),
                            )
                            .then(
                                literal("with").then(
                                    argument(ARG_ITEM, ItemArgumentConsumer)
                                        .execute(BlockReplaceExecutor)
                                        .then(
                                            argument_default_name(count_consumer())
                                                .execute(BlockReplaceExecutor),
                                        ),
                                ),
                            ),
                    ),
                ),
            )
            .then(
                literal("entity").then(
                    argument(ARG_TARGETS, EntitiesArgumentConsumer).then(
                        argument(ARG_SLOT, SlotArgumentConsumer)
                            .then(
                                argument(ARG_ITEM, ItemArgumentConsumer)
                                    .execute(EntityReplaceExecutor)
                                    .then(
                                        argument_default_name(count_consumer())
                                            .execute(EntityReplaceExecutor),
                                    ),
                            )
                            .then(
                                literal("with").then(
                                    argument(ARG_ITEM, ItemArgumentConsumer)
                                        .execute(EntityReplaceExecutor)
                                        .then(
                                            argument_default_name(count_consumer())
                                                .execute(EntityReplaceExecutor),
                                        ),
                                ),
                            ),
                    ),
                ),
            ),
    )
}
