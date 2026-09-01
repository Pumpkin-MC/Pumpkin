use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::translation;
use pumpkin_inventory::screen_handler::InventoryPlayer;
use pumpkin_protocol::codec::item_stack_seralizer::ItemStackSerializer;
use pumpkin_protocol::java::client::play::CSetContainerSlot;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::hover::HoverEvent;
use pumpkin_world::inventory::Inventory;

use crate::command::argument_builder::{
    ArgumentBuilder, RequiredArgumentBuilder, argument, command, literal,
};
use crate::command::argument_types::coordinates::block_pos::BlockPosArgumentType;
use crate::command::argument_types::core::integer::IntegerArgumentType;
use crate::command::argument_types::entity::EntityArgumentType;
use crate::command::argument_types::item::ItemStackArgumentType;
use crate::command::argument_types::slot::SlotArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};

const DESCRIPTION: &str = "Modifies items in block or entity inventories.";
const PERMISSION: &str = "minecraft:command.item";

const ERROR_NOT_CONTAINER: CommandErrorType<3> = CommandErrorType::new(
    translation::java::COMMANDS_ITEM_TARGET_NOT_A_CONTAINER,
    translation::java::COMMANDS_ITEM_TARGET_NOT_A_CONTAINER,
);

const ERROR_NO_SUCH_SLOT: CommandErrorType<1> = CommandErrorType::new(
    translation::java::COMMANDS_ITEM_TARGET_NO_SUCH_SLOT,
    translation::java::COMMANDS_ITEM_TARGET_NO_SUCH_SLOT,
);

struct BlockReplaceExecutor {
    has_count: bool,
}

impl CommandExecutor for BlockReplaceExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let world = context.source.world();
        let pos = BlockPosArgumentType::get_loaded_block_pos(context, "pos")?;
        let slot = SlotArgumentType::get(context, "slot")?;
        let parsed_stack = ItemStackArgumentType::get(context, "item")?;
        let item = parsed_stack.item;
        let count = if self.has_count {
            IntegerArgumentType::get(context, "count")?
        } else {
            1
        };

        let block_entity = world.get_block_entity(&pos).ok_or_else(|| {
            ERROR_NOT_CONTAINER.create_without_context(
                TextComponent::text(pos.0.x.to_string()),
                TextComponent::text(pos.0.y.to_string()),
                TextComponent::text(pos.0.z.to_string()),
            )
        })?;

        let inventory = block_entity.get_inventory().ok_or_else(|| {
            ERROR_NOT_CONTAINER.create_without_context(
                TextComponent::text(pos.0.x.to_string()),
                TextComponent::text(pos.0.y.to_string()),
                TextComponent::text(pos.0.z.to_string()),
            )
        })?;

        if slot >= inventory.size() {
            return Err(
                ERROR_NO_SUCH_SLOT.create_without_context(TextComponent::text(slot.to_string()))
            );
        }

        let mut item_stack = parsed_stack.clone();
        item_stack.item_count = count as u8;
        inventory.set_stack(slot, item_stack);

        let item_name = item.registry_key;
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
        context.source.send_feedback(msg, true);

        Ok(1)
    }
}

struct EntityReplaceExecutor {
    has_count: bool,
}

impl CommandExecutor for EntityReplaceExecutor {
    #[expect(clippy::too_many_lines)]
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let targets = EntityArgumentType::get_entities(context, "targets")?;
        let mojang_slot = SlotArgumentType::get(context, "slot")?;
        let parsed_stack = ItemStackArgumentType::get(context, "item")?;
        let item = parsed_stack.item;
        let count = if self.has_count {
            IntegerArgumentType::get(context, "count")?
        } else {
            1
        };

        let mut modified_count = 0;
        let mut item_stack = parsed_stack.clone();
        item_stack.item_count = count as u8;

        for target in &targets {
            if let Some(player) = target.get_player() {
                if let Some(player_arc) = player.world().get_player_by_uuid(player.gameprofile.id) {
                    if (200..=226).contains(&mojang_slot) {
                        let ender_slot = mojang_slot - 200;
                        if ender_slot < player.ender_chest_inventory.size() {
                            player_arc
                                .ender_chest_inventory
                                .set_stack(ender_slot, item_stack.clone());
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
                            player_arc.inventory().set_stack(slot, item_stack.clone());

                            let stack_serializer = ItemStackSerializer::from(item_stack.clone());
                            let packet =
                                CSetContainerSlot::new(0, 0, slot as i16, &stack_serializer);
                            player_arc.enqueue_slot_packet(&packet, None, 0);

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
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .put(&eq, item_stack.clone());
                    living.send_equipment_changes(&[(eq, item_stack.clone())]);
                    modified_count += 1;
                }
            }
        }

        if modified_count == 0 {
            return Err(ERROR_NO_SUCH_SLOT
                .create_without_context(TextComponent::text(mojang_slot.to_string())));
        }

        let item_name = item.registry_key;
        let msg = if targets.len() == 1 {
            TextComponent::translate_cross(
                translation::java::COMMANDS_ITEM_ENTITY_SET_SUCCESS_SINGLE,
                translation::java::COMMANDS_ITEM_ENTITY_SET_SUCCESS_SINGLE,
                [
                    targets[0].as_ref().get_display_name(),
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
        context.source.send_feedback(msg, true);

        Ok(modified_count)
    }
}

fn block_item_node() -> RequiredArgumentBuilder {
    argument("item", ItemStackArgumentType)
        .executes(BlockReplaceExecutor { has_count: false })
        .then(
            argument("count", IntegerArgumentType::new(1, 99))
                .executes(BlockReplaceExecutor { has_count: true }),
        )
}

fn entity_item_node() -> RequiredArgumentBuilder {
    argument("item", ItemStackArgumentType)
        .executes(EntityReplaceExecutor { has_count: false })
        .then(
            argument("count", IntegerArgumentType::new(1, 99))
                .executes(EntityReplaceExecutor { has_count: true }),
        )
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    let block_node = literal("block").then(
        argument("pos", BlockPosArgumentType).then(
            argument("slot", SlotArgumentType)
                .then(block_item_node())
                .then(literal("with").then(block_item_node())),
        ),
    );

    let entity_node = literal("entity").then(
        argument("targets", EntityArgumentType::Entities).then(
            argument("slot", SlotArgumentType)
                .then(entity_item_node())
                .then(literal("with").then(entity_item_node())),
        ),
    );

    dispatcher.register(
        command("item", DESCRIPTION)
            .requires(PERMISSION)
            .then(literal("replace").then(block_node).then(entity_node)),
    );
}
