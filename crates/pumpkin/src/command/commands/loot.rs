use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::translation;
use pumpkin_inventory::screen_handler::InventoryPlayer;
use pumpkin_protocol::codec::item_stack_seralizer::ItemStackSerializer;
use pumpkin_protocol::java::client::play::CSetContainerSlot;
use pumpkin_util::PermissionLvl;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::hover::HoverEvent;
use pumpkin_world::inventory::Inventory;

use crate::command::argument_builder::{
    ArgumentBuilder, LiteralArgumentBuilder, argument, command, literal,
};
use crate::command::argument_types::coordinates::block_pos::BlockPosArgumentType;
use crate::command::argument_types::coordinates::vec3::Vec3ArgumentType;
use crate::command::argument_types::core::integer::IntegerArgumentType;
use crate::command::argument_types::core::string::StringArgumentType;
use crate::command::argument_types::entity::EntityArgumentType;
use crate::command::argument_types::item::ItemStackArgumentType;
use crate::command::argument_types::slot::SlotArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::entity::EntityBase;
use crate::world::loot::LootContextParameters;

const DESCRIPTION: &str =
    "Drops the given loot table into the specified inventory or into the world.";
const PERMISSION: &str = "minecraft:command.loot";

static ERROR_INVALID_LOOT_TABLE: CommandErrorType<1> = CommandErrorType::new(
    translation::bedrock::COMMANDS_LOOT_FAILURE_INVALIDLOOTTABLE,
    "Loot table '%s' not found",
);

static ERROR_NO_HELD_ITEMS: CommandErrorType<1> = CommandErrorType::new(
    translation::java::COMMANDS_DROP_NO_HELD_ITEMS,
    translation::java::COMMANDS_DROP_NO_HELD_ITEMS,
);

static ERROR_NO_ENTITY_LOOT_TABLE: CommandErrorType<1> = CommandErrorType::new(
    translation::java::COMMANDS_DROP_NO_LOOT_TABLE,
    translation::java::COMMANDS_DROP_NO_LOOT_TABLE,
);

static ERROR_NO_BLOCK_LOOT_TABLE: CommandErrorType<1> = CommandErrorType::new(
    translation::java::COMMANDS_DROP_NO_LOOT_TABLE_BLOCK,
    translation::java::COMMANDS_DROP_NO_LOOT_TABLE_BLOCK,
);

static ERROR_NOT_CONTAINER: CommandErrorType<3> = CommandErrorType::new(
    translation::java::COMMANDS_ITEM_TARGET_NOT_A_CONTAINER,
    translation::java::COMMANDS_ITEM_TARGET_NOT_A_CONTAINER,
);

static ERROR_NO_SUCH_SLOT: CommandErrorType<1> = CommandErrorType::new(
    translation::java::COMMANDS_ITEM_TARGET_NO_SUCH_SLOT,
    translation::java::COMMANDS_ITEM_TARGET_NO_SUCH_SLOT,
);

#[derive(Clone, Copy)]
enum Target {
    Give,
    Spawn,
    Insert,
    ReplaceEntity { has_count: bool },
    ReplaceBlock { has_count: bool },
}

#[derive(Clone, Copy)]
enum ToolSource {
    None,
    Item,
    MainHand,
    OffHand,
}

#[derive(Clone, Copy)]
enum Source {
    Loot,
    Kill,
    Mine { tool: ToolSource },
    Fish { tool: ToolSource },
}

struct LootExecutor {
    target: Target,
    source: Source,
}

fn distribute_to_container(inventory: &dyn Inventory, mut stack: ItemStack) -> bool {
    let mut changed = false;
    for i in 0..inventory.size() {
        if stack.is_empty() {
            break;
        }
        let mut slot_stack = inventory.get_stack(i);
        if slot_stack.is_empty() {
            inventory.set_stack(i, stack);
            changed = true;
            break;
        }
        if slot_stack.get_item().id == stack.get_item().id {
            let max_stack_size = stack.get_max_stack_size();
            let space = max_stack_size.saturating_sub(slot_stack.item_count);
            if space > 0 {
                let to_add = stack.item_count.min(space);
                slot_stack.item_count += to_add;
                stack.item_count -= to_add;
                inventory.set_stack(i, slot_stack);
                changed = true;
            }
        }
    }
    changed
}

fn get_hand_item(
    context: &CommandContext,
    is_mainhand: bool,
) -> Result<Option<ItemStack>, CommandSyntaxError> {
    context.source.as_player().map_or_else(
        || {
            let display_name = TextComponent::text("Server");
            Err(ERROR_NO_HELD_ITEMS.create_without_context(display_name))
        },
        |player| {
            let stack = if is_mainhand {
                let slot = player.inventory().get_selected_slot() as usize;
                player.inventory().get_stack(slot)
            } else {
                player.inventory().get_stack(40)
            };
            Ok(Some(stack))
        },
    )
}

#[allow(clippy::too_many_lines)]
fn replace_entity_slots(
    entity: &dyn EntityBase,
    start_slot: usize,
    count: usize,
    drops: &[ItemStack],
    used_items: &mut Vec<ItemStack>,
) {
    if let Some(player) = entity.get_player() {
        if let Some(player_arc) = player.world().get_player_by_uuid(player.gameprofile.id) {
            for i in 0..count {
                let mojang_slot = start_slot + i;
                let item_stack = if i < drops.len() {
                    drops[i].clone()
                } else {
                    ItemStack::EMPTY.clone()
                };
                if (200..=226).contains(&mojang_slot) {
                    let ender_slot = mojang_slot - 200;
                    if ender_slot < player.ender_chest_inventory.size() {
                        player_arc
                            .ender_chest_inventory
                            .set_stack(ender_slot, item_stack.clone());
                        used_items.push(item_stack);
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
                        used_items.push(item_stack);
                    }
                }
            }
        }
    } else if let Some(living) = entity.get_living_entity() {
        for i in 0..count {
            let mojang_slot = start_slot + i;
            let item_stack = if i < drops.len() {
                drops[i].clone()
            } else {
                ItemStack::EMPTY.clone()
            };
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
                used_items.push(item_stack);
            }
        }
    }
}

fn send_callback(context: &CommandContext, drops: &[ItemStack], table_id: Option<&str>) {
    let msg = match (drops.len(), table_id) {
        (1, Some(table)) => {
            let drop = &drops[0];
            let item = drop.item;
            let item_name = item.registry_key;
            let display_comp = TextComponent::text("[")
                .add_child(item.translated_name())
                .add_child(TextComponent::text("]"))
                .hover_event(HoverEvent::ShowItem {
                    id: item_name.to_string().into(),
                    count: Some(drop.item_count as i32),
                });
            TextComponent::translate_cross(
                translation::java::COMMANDS_DROP_SUCCESS_SINGLE_WITH_TABLE,
                translation::bedrock::COMMANDS_LOOT_SUCCESS,
                [
                    TextComponent::text(drop.item_count.to_string()),
                    display_comp,
                    TextComponent::text(table.to_string()),
                ],
            )
        }
        (1, None) => {
            let drop = &drops[0];
            let item = drop.item;
            let item_name = item.registry_key;
            let display_comp = TextComponent::text("[")
                .add_child(item.translated_name())
                .add_child(TextComponent::text("]"))
                .hover_event(HoverEvent::ShowItem {
                    id: item_name.to_string().into(),
                    count: Some(drop.item_count as i32),
                });
            TextComponent::translate_cross(
                translation::java::COMMANDS_DROP_SUCCESS_SINGLE,
                translation::bedrock::COMMANDS_LOOT_SUCCESS,
                [
                    TextComponent::text(drop.item_count.to_string()),
                    display_comp,
                ],
            )
        }
        (count, Some(table)) => TextComponent::translate_cross(
            translation::java::COMMANDS_DROP_SUCCESS_MULTIPLE_WITH_TABLE,
            translation::bedrock::COMMANDS_LOOT_SUCCESS,
            [
                TextComponent::text(count.to_string()),
                TextComponent::text(table.to_string()),
            ],
        ),
        (count, None) => TextComponent::translate_cross(
            translation::java::COMMANDS_DROP_SUCCESS_MULTIPLE,
            translation::bedrock::COMMANDS_LOOT_SUCCESS,
            [TextComponent::text(count.to_string())],
        ),
    };
    context.source.send_feedback(msg, true);
}

impl CommandExecutor for LootExecutor {
    #[expect(clippy::too_many_lines)]
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let mut drops = Vec::new();
        let mut table_id_for_callback: Option<String> = None;

        match self.source {
            Source::Fish { tool } => {
                let loot_table_str = StringArgumentType::get(context, "loot_table")?;
                let key = if loot_table_str.contains(':') {
                    loot_table_str.to_string()
                } else {
                    format!("minecraft:{loot_table_str}")
                };

                let pos = BlockPosArgumentType::get_block_pos(context, "fish_pos")?;
                let tool_stack = match tool {
                    ToolSource::None => None,
                    ToolSource::Item => Some(ItemStackArgumentType::get(context, "tool")?),
                    ToolSource::MainHand => get_hand_item(context, true)?,
                    ToolSource::OffHand => get_hand_item(context, false)?,
                };

                let loot_table =
                    pumpkin_data::loot_table::get_loot_table(&key).ok_or_else(|| {
                        ERROR_INVALID_LOOT_TABLE
                            .create_without_context(TextComponent::text(loot_table_str.to_string()))
                    })?;

                let params = LootContextParameters {
                    position: Some(Vector3::new(
                        f64::from(pos.0.x) + 0.5,
                        f64::from(pos.0.y) + 0.5,
                        f64::from(pos.0.z) + 0.5,
                    )),
                    tool: tool_stack,
                    ..Default::default()
                };
                let seed: i64 = rand::random();
                drops = crate::world::loot::generate_loot_with_context(loot_table, seed, &params);
            }
            Source::Loot => {
                let loot_table_str = StringArgumentType::get(context, "loot_table")?;
                let key = if loot_table_str.contains(':') {
                    loot_table_str.to_string()
                } else {
                    format!("minecraft:{loot_table_str}")
                };

                let loot_table =
                    pumpkin_data::loot_table::get_loot_table(&key).ok_or_else(|| {
                        ERROR_INVALID_LOOT_TABLE
                            .create_without_context(TextComponent::text(loot_table_str.to_string()))
                    })?;

                let params = LootContextParameters {
                    position: context.source.as_player().map(|p| p.position()),
                    ..Default::default()
                };
                let seed: i64 = rand::random();
                drops = crate::world::loot::generate_loot_with_context(loot_table, seed, &params);
            }
            Source::Kill => {
                let target_entities = EntityArgumentType::get_entities(context, "target_entity")?;
                let killer = context.source.as_player();
                let killer_tool = killer.as_ref().map(|p| {
                    p.inventory()
                        .get_stack(p.inventory().get_selected_slot() as usize)
                });
                let params = LootContextParameters {
                    killed_by_player: Some(killer.is_some()),
                    tool: killer_tool,
                    position: killer.as_ref().map(|p| p.position()),
                    ..Default::default()
                };

                let mut last_key = None;
                for entity in &target_entities {
                    let resource_name = entity.get_entity().entity_type.resource_name;
                    let key = format!("minecraft:entities/{resource_name}");
                    if let Some(loot_table) = pumpkin_data::loot_table::get_loot_table(&key) {
                        let seed: i64 = rand::random();
                        drops.extend(crate::world::loot::generate_loot_with_context(
                            loot_table, seed, &params,
                        ));
                        last_key = Some(key);
                    }
                }

                if drops.is_empty() && last_key.is_none() {
                    let display_name = target_entities.first().map_or_else(
                        || TextComponent::text("selected entity"),
                        |target| target.as_ref().get_display_name(),
                    );
                    return Err(ERROR_NO_ENTITY_LOOT_TABLE.create_without_context(display_name));
                }
                table_id_for_callback = last_key;
            }
            Source::Mine { tool } => {
                let pos = BlockPosArgumentType::get_block_pos(context, "mine_pos")?;
                let world = context.world();
                let block_state = world.get_block_state(&pos);
                let block = world.get_block(&pos);
                let key = format!("minecraft:blocks/{}", block.name);

                let loot_table =
                    pumpkin_data::loot_table::get_loot_table(&key).ok_or_else(|| {
                        ERROR_NO_BLOCK_LOOT_TABLE
                            .create_without_context(TextComponent::text(block.name.to_string()))
                    })?;

                let tool_stack = match tool {
                    ToolSource::None => None,
                    ToolSource::Item => Some(ItemStackArgumentType::get(context, "tool")?),
                    ToolSource::MainHand => get_hand_item(context, true)?,
                    ToolSource::OffHand => get_hand_item(context, false)?,
                };

                let params = LootContextParameters {
                    block_state: Some(block_state),
                    tool: tool_stack,
                    position: Some(Vector3::new(
                        f64::from(pos.0.x) + 0.5,
                        f64::from(pos.0.y) + 0.5,
                        f64::from(pos.0.z) + 0.5,
                    )),
                    ..Default::default()
                };
                let seed: i64 = rand::random();
                drops = crate::world::loot::generate_loot_with_context(loot_table, seed, &params);
                table_id_for_callback = Some(key);
            }
        }

        match self.target {
            Target::Give => {
                let targets = EntityArgumentType::get_players(context, "players")?;
                let mut used_items = Vec::new();
                for stack in &drops {
                    for player in &targets {
                        let mut remaining = stack.clone();
                        player.inventory().insert_stack_anywhere(&mut remaining);
                        if !remaining.is_empty() {
                            player.drop_item(remaining);
                        }
                        used_items.push(stack.clone());
                    }
                }
                send_callback(context, &used_items, table_id_for_callback.as_deref());
                Ok(used_items.len() as i32)
            }
            Target::Spawn => {
                let pos = Vec3ArgumentType::get_vector3(context, "target_pos")?;
                let world = context.world();
                let block_pos = BlockPos::new(
                    pos.x.floor() as i32,
                    pos.y.floor() as i32,
                    pos.z.floor() as i32,
                );
                for stack in &drops {
                    world.drop_stack(&block_pos, stack.clone());
                }
                send_callback(context, &drops, table_id_for_callback.as_deref());
                Ok(drops.len() as i32)
            }
            Target::Insert => {
                let pos = BlockPosArgumentType::get_block_pos(context, "target_pos")?;
                let world = context.world().clone();
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

                let mut used_items = Vec::new();
                for stack in &drops {
                    if distribute_to_container(inventory.as_ref(), stack.clone()) {
                        used_items.push(stack.clone());
                    }
                }
                send_callback(context, &used_items, table_id_for_callback.as_deref());
                Ok(used_items.len() as i32)
            }
            Target::ReplaceBlock { has_count } => {
                let pos = BlockPosArgumentType::get_block_pos(context, "target_pos")?;
                let slot = SlotArgumentType::get(context, "slot")?;
                let count = if has_count {
                    IntegerArgumentType::get(context, "count")? as usize
                } else {
                    drops.len()
                };

                let world = context.world().clone();
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
                    return Err(ERROR_NO_SUCH_SLOT
                        .create_without_context(TextComponent::text(slot.to_string())));
                }

                let mut used_items = Vec::new();
                for i in 0..count {
                    let s = slot + i;
                    if s < inventory.size() {
                        let to_add = if i < drops.len() {
                            drops[i].clone()
                        } else {
                            ItemStack::EMPTY.clone()
                        };
                        inventory.set_stack(s, to_add.clone());
                        used_items.push(to_add);
                    }
                }
                send_callback(context, &used_items, table_id_for_callback.as_deref());
                Ok(used_items.len() as i32)
            }
            Target::ReplaceEntity { has_count } => {
                let entities = EntityArgumentType::get_entities(context, "entities")?;
                let slot = SlotArgumentType::get(context, "slot")?;
                let count = if has_count {
                    IntegerArgumentType::get(context, "count")? as usize
                } else {
                    drops.len()
                };

                let mut used_items = Vec::new();
                for target in &entities {
                    replace_entity_slots(target.as_ref(), slot, count, &drops, &mut used_items);
                }

                if used_items.is_empty() && !entities.is_empty() {
                    return Err(ERROR_NO_SUCH_SLOT
                        .create_without_context(TextComponent::text(slot.to_string())));
                }

                send_callback(context, &used_items, table_id_for_callback.as_deref());
                Ok(used_items.len() as i32)
            }
        }
    }
}

fn add_sources(target: Target) -> Vec<LiteralArgumentBuilder> {
    vec![
        literal("fish").then(
            argument("loot_table", StringArgumentType::SingleWord).then(
                argument("fish_pos", BlockPosArgumentType)
                    .executes(LootExecutor {
                        target,
                        source: Source::Fish {
                            tool: ToolSource::None,
                        },
                    })
                    .then(
                        argument("tool", ItemStackArgumentType).executes(LootExecutor {
                            target,
                            source: Source::Fish {
                                tool: ToolSource::Item,
                            },
                        }),
                    )
                    .then(literal("mainhand").executes(LootExecutor {
                        target,
                        source: Source::Fish {
                            tool: ToolSource::MainHand,
                        },
                    }))
                    .then(literal("offhand").executes(LootExecutor {
                        target,
                        source: Source::Fish {
                            tool: ToolSource::OffHand,
                        },
                    })),
            ),
        ),
        literal("loot").then(
            argument("loot_table", StringArgumentType::SingleWord).executes(LootExecutor {
                target,
                source: Source::Loot,
            }),
        ),
        literal("kill").then(
            argument("target_entity", EntityArgumentType::Entities).executes(LootExecutor {
                target,
                source: Source::Kill,
            }),
        ),
        literal("mine").then(
            argument("mine_pos", BlockPosArgumentType)
                .executes(LootExecutor {
                    target,
                    source: Source::Mine {
                        tool: ToolSource::None,
                    },
                })
                .then(
                    argument("tool", ItemStackArgumentType).executes(LootExecutor {
                        target,
                        source: Source::Mine {
                            tool: ToolSource::Item,
                        },
                    }),
                )
                .then(literal("mainhand").executes(LootExecutor {
                    target,
                    source: Source::Mine {
                        tool: ToolSource::MainHand,
                    },
                }))
                .then(literal("offhand").executes(LootExecutor {
                    target,
                    source: Source::Mine {
                        tool: ToolSource::OffHand,
                    },
                })),
        ),
    ]
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    let mut give_arg = argument("players", EntityArgumentType::Players);
    for src in add_sources(Target::Give) {
        give_arg = give_arg.then(src);
    }

    let mut spawn_arg = argument("target_pos", Vec3ArgumentType::Default);
    for src in add_sources(Target::Spawn) {
        spawn_arg = spawn_arg.then(src);
    }

    let mut insert_arg = argument("target_pos", BlockPosArgumentType);
    for src in add_sources(Target::Insert) {
        insert_arg = insert_arg.then(src);
    }

    let mut replace_block_slot = argument("slot", SlotArgumentType);
    for src in add_sources(Target::ReplaceBlock { has_count: false }) {
        replace_block_slot = replace_block_slot.then(src);
    }
    let mut replace_block_count = argument("count", IntegerArgumentType::with_min(0));
    for src in add_sources(Target::ReplaceBlock { has_count: true }) {
        replace_block_count = replace_block_count.then(src);
    }
    let replace_block_slot = replace_block_slot.then(replace_block_count);

    let mut replace_entity_slot = argument("slot", SlotArgumentType);
    for src in add_sources(Target::ReplaceEntity { has_count: false }) {
        replace_entity_slot = replace_entity_slot.then(src);
    }
    let mut replace_entity_count = argument("count", IntegerArgumentType::with_min(0));
    for src in add_sources(Target::ReplaceEntity { has_count: true }) {
        replace_entity_count = replace_entity_count.then(src);
    }
    let replace_entity_slot = replace_entity_slot.then(replace_entity_count);

    let builder = command("loot", DESCRIPTION)
        .requires(PERMISSION)
        .then(literal("give").then(give_arg))
        .then(literal("spawn").then(spawn_arg))
        .then(literal("insert").then(insert_arg))
        .then(
            literal("replace")
                .then(
                    literal("block").then(
                        argument("target_pos", BlockPosArgumentType).then(replace_block_slot),
                    ),
                )
                .then(literal("entity").then(
                    argument("entities", EntityArgumentType::Entities).then(replace_entity_slot),
                )),
        );

    dispatcher.register(builder);
}
