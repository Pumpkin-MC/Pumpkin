use std::sync::Arc;

use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::translation;
use pumpkin_inventory::Inventory;
use pumpkin_inventory::screen_handler::InventoryPlayer;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_protocol::codec::item_stack_seralizer::ItemStackSerializer;
use pumpkin_protocol::java::client::play::CSetContainerSlot;
use pumpkin_util::PermissionLvl;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::hover::HoverEvent;

use crate::command::argument_builder::{
    ArgumentBuilder, LiteralArgumentBuilder, RequiredArgumentBuilder, argument, command, literal,
};
use crate::command::argument_types::coordinates::block_pos::BlockPosArgumentType;
use crate::command::argument_types::core::integer::IntegerArgumentType;
use crate::command::argument_types::entity::EntityArgumentType;
use crate::command::argument_types::item::ItemStackArgumentType;
use crate::command::argument_types::item_modifier::{ItemModifier, ItemModifierArgumentType};
use crate::command::argument_types::slot::SlotArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::entity::EntityBase;
use crate::world::World;

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
            if write_entity_slot(target, mojang_slot, &item_stack) {
                modified_count += 1;
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

/// Maps a vanilla armor/hand slot index to the equipment slot it names.
const fn equipment_slot(mojang_slot: usize) -> Option<EquipmentSlot> {
    match mojang_slot {
        98 => Some(EquipmentSlot::MAIN_HAND),
        99 => Some(EquipmentSlot::OFF_HAND),
        100 => Some(EquipmentSlot::FEET),
        101 => Some(EquipmentSlot::LEGS),
        102 => Some(EquipmentSlot::CHEST),
        103 => Some(EquipmentSlot::HEAD),
        105 => Some(EquipmentSlot::BODY),
        106 => Some(EquipmentSlot::SADDLE),
        _ => None,
    }
}

/// Reads the stack in a vanilla slot index of an entity, or `None` if it has no such slot.
pub fn read_entity_slot(target: &Arc<dyn EntityBase>, mojang_slot: usize) -> Option<ItemStack> {
    if let Some(player) = target.get_player() {
        if (200..=226).contains(&mojang_slot) {
            let ender_slot = mojang_slot - 200;
            return (ender_slot < player.ender_chest_inventory.size())
                .then(|| player.ender_chest_inventory.get_stack(ender_slot));
        }
        let inventory = player.inventory();
        let slot = match mojang_slot {
            98 => inventory.get_selected_slot() as usize,
            99 => 40,
            100 => 36,
            101 => 37,
            102 => 38,
            103 => 39,
            0..=35 => mojang_slot,
            _ => return None,
        };
        return (slot < inventory.size()).then(|| inventory.get_stack(slot));
    }
    let living = target.get_living_entity()?;
    let eq = equipment_slot(mojang_slot)?;
    Some(
        living
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&eq),
    )
}

/// Reads the stack in a slot of a container block, or `None` if it isn't a container slot.
pub fn read_block_slot(world: &World, pos: &BlockPos, slot: usize) -> Option<ItemStack> {
    let inventory = world.get_block_entity(pos)?.get_inventory()?;
    (slot < inventory.size()).then(|| inventory.get_stack(slot))
}

/// Applies an inline item modifier (a single function or a list of them). Only the
/// functions datapacks commonly use are implemented; others leave the stack as is.
pub fn apply_item_modifier(stack: &mut ItemStack, modifier: &NbtTag) {
    match modifier {
        NbtTag::List(list) => {
            for function in list {
                apply_item_modifier(stack, function);
            }
        }
        NbtTag::Compound(function) => {
            let name = function.get_string("function").unwrap_or_default();
            match name.strip_prefix("minecraft:").unwrap_or(name) {
                "set_damage" => {
                    let Some(max) = stack.get_max_damage() else {
                        return;
                    };
                    let amount = number_provider(function.get("damage")).unwrap_or(1.0);
                    let add = function.get_bool("add").unwrap_or(false);
                    let base = if add {
                        (max - stack.get_damage()) as f32 / max as f32
                    } else {
                        0.0
                    };
                    let durability = (base + amount).clamp(0.0, 1.0);
                    let damage = max - (durability * max as f32).floor() as i32;
                    stack.set_damage(damage.clamp(0, max));
                }
                "set_count" => {
                    let amount = number_provider(function.get("count")).unwrap_or(1.0) as i32;
                    let add = function.get_bool("add").unwrap_or(false);
                    let count = if add {
                        i32::from(stack.item_count) + amount
                    } else {
                        amount
                    };
                    stack.item_count = count.clamp(0, i32::from(stack.get_max_stack_size())) as u8;
                }
                other => tracing::debug!("Unsupported item modifier function {other}"),
            }
        }
        _ => {}
    }
}

/// Reads a constant number provider (`1.0` or `{type:"constant",value:1.0}`).
fn number_provider(tag: Option<&NbtTag>) -> Option<f32> {
    match tag? {
        NbtTag::Compound(c) => c.get("value").and_then(NbtTag::extract_float),
        other => other.extract_float(),
    }
}

/// Where `item replace ... from` copies its stack from.
#[derive(Clone, Copy)]
enum ItemSource {
    FromBlock { modified: bool },
    FromEntity { modified: bool },
}

impl ItemSource {
    fn stack(self, context: &CommandContext) -> Result<ItemStack, CommandSyntaxError> {
        let (mut stack, modified) = match self {
            Self::FromBlock { modified } => {
                let pos = BlockPosArgumentType::get_loaded_block_pos(context, "source")?;
                let slot = SlotArgumentType::get(context, "sourceSlot")?;
                let stack =
                    read_block_slot(context.source.world(), &pos, slot).ok_or_else(|| {
                        ERROR_NO_SUCH_SLOT
                            .create_without_context(TextComponent::text(slot.to_string()))
                    })?;
                (stack, modified)
            }
            Self::FromEntity { modified } => {
                let entity = EntityArgumentType::get_entity(context, "source")?;
                let slot = SlotArgumentType::get(context, "sourceSlot")?;
                let stack = read_entity_slot(&entity, slot).ok_or_else(|| {
                    ERROR_NO_SUCH_SLOT.create_without_context(TextComponent::text(slot.to_string()))
                })?;
                (stack, modified)
            }
        };
        if modified
            && let ItemModifier::Inline(tag) = ItemModifierArgumentType::get(context, "modifier")?
        {
            apply_item_modifier(&mut stack, tag);
        }
        Ok(stack)
    }
}

/// `item replace entity <targets> <slot> from ...`.
struct EntityCopyExecutor(ItemSource);

impl CommandExecutor for EntityCopyExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let targets = EntityArgumentType::get_entities(context, "targets")?;
        let slot = SlotArgumentType::get(context, "slot")?;
        let stack = self.0.stack(context)?;
        let count = targets
            .iter()
            .filter(|target| write_entity_slot(target, slot, &stack))
            .count() as i32;
        if count == 0 {
            return Err(
                ERROR_NO_SUCH_SLOT.create_without_context(TextComponent::text(slot.to_string()))
            );
        }
        Ok(count)
    }
}

/// `item replace block <pos> <slot> from ...`.
struct BlockCopyExecutor(ItemSource);

impl CommandExecutor for BlockCopyExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let pos = BlockPosArgumentType::get_loaded_block_pos(context, "pos")?;
        let slot = SlotArgumentType::get(context, "slot")?;
        let stack = self.0.stack(context)?;
        set_block_slot(context, &pos, slot, |_| stack)
    }
}

/// Replaces a container block's slot with `update(current)`.
fn set_block_slot(
    context: &CommandContext,
    pos: &BlockPos,
    slot: usize,
    update: impl FnOnce(ItemStack) -> ItemStack,
) -> CommandExecutorResult {
    let inventory = context
        .source
        .world()
        .get_block_entity(pos)
        .and_then(crate::block::entities::BlockEntity::get_inventory)
        .ok_or_else(|| {
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
    inventory.set_stack(slot, update(inventory.get_stack(slot)));
    Ok(1)
}

/// `item modify entity <targets> <slot> <modifier>`.
struct EntityModifyExecutor;

impl CommandExecutor for EntityModifyExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let targets = EntityArgumentType::get_entities(context, "targets")?;
        let slot = SlotArgumentType::get(context, "slot")?;
        let ItemModifier::Inline(modifier) = ItemModifierArgumentType::get(context, "modifier")?
        else {
            return Ok(0);
        };
        let mut count = 0;
        for target in &targets {
            if let Some(mut stack) = read_entity_slot(target, slot) {
                apply_item_modifier(&mut stack, modifier);
                if write_entity_slot(target, slot, &stack) {
                    count += 1;
                }
            }
        }
        if count == 0 {
            return Err(
                ERROR_NO_SUCH_SLOT.create_without_context(TextComponent::text(slot.to_string()))
            );
        }
        Ok(count)
    }
}

/// `item modify block <pos> <slot> <modifier>`.
struct BlockModifyExecutor;

impl CommandExecutor for BlockModifyExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let pos = BlockPosArgumentType::get_loaded_block_pos(context, "pos")?;
        let slot = SlotArgumentType::get(context, "slot")?;
        let ItemModifier::Inline(modifier) = ItemModifierArgumentType::get(context, "modifier")?
        else {
            return Ok(0);
        };
        set_block_slot(context, &pos, slot, |mut stack| {
            apply_item_modifier(&mut stack, modifier);
            stack
        })
    }
}

/// `from block <source> <sourceSlot> [<modifier>]` / `from entity ...`.
fn from_node<E: CommandExecutor + 'static>(make: fn(ItemSource) -> E) -> LiteralArgumentBuilder {
    literal("from")
        .then(
            literal("block").then(
                argument("source", BlockPosArgumentType).then(
                    argument("sourceSlot", SlotArgumentType)
                        .executes(make(ItemSource::FromBlock { modified: false }))
                        .then(
                            argument("modifier", ItemModifierArgumentType)
                                .executes(make(ItemSource::FromBlock { modified: true })),
                        ),
                ),
            ),
        )
        .then(
            literal("entity").then(
                argument("source", EntityArgumentType::Entity).then(
                    argument("sourceSlot", SlotArgumentType)
                        .executes(make(ItemSource::FromEntity { modified: false }))
                        .then(
                            argument("modifier", ItemModifierArgumentType)
                                .executes(make(ItemSource::FromEntity { modified: true })),
                        ),
                ),
            ),
        )
}

/// Writes `item_stack` into the vanilla slot index of an entity, returning whether the
/// entity has that slot.
pub fn write_entity_slot(
    target: &Arc<dyn EntityBase>,
    mojang_slot: usize,
    item_stack: &ItemStack,
) -> bool {
    if let Some(player) = target.get_player() {
        if let Some(player_arc) = player.world().get_player_by_uuid(player.gameprofile.id) {
            if (200..=226).contains(&mojang_slot) {
                let ender_slot = mojang_slot - 200;
                if ender_slot < player.ender_chest_inventory.size() {
                    player_arc
                        .ender_chest_inventory
                        .set_stack(ender_slot, item_stack.clone());
                    return true;
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
                    let packet = CSetContainerSlot::new(0, 0, slot as i16, &stack_serializer);
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

                    return true;
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
            return true;
        }
    }
    false
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
                .then(literal("with").then(block_item_node()))
                .then(from_node(BlockCopyExecutor)),
        ),
    );

    let entity_node = literal("entity").then(
        argument("targets", EntityArgumentType::Entities).then(
            argument("slot", SlotArgumentType)
                .then(literal("with").then(entity_item_node()))
                .then(from_node(EntityCopyExecutor)),
        ),
    );

    let modify_node = literal("modify")
        .then(literal("block").then(
            argument("pos", BlockPosArgumentType).then(argument("slot", SlotArgumentType).then(
                argument("modifier", ItemModifierArgumentType).executes(BlockModifyExecutor),
            )),
        ))
        .then(
            literal("entity").then(argument("targets", EntityArgumentType::Entities).then(
                argument("slot", SlotArgumentType).then(
                    argument("modifier", ItemModifierArgumentType).executes(EntityModifyExecutor),
                ),
            )),
        );

    dispatcher.register(
        command("item", DESCRIPTION)
            .requires(PERMISSION)
            .then(literal("replace").then(block_node).then(entity_node))
            .then(modify_node),
    );
}
