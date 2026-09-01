use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::coordinates::block_pos::BlockPosArgumentType;
use crate::command::argument_types::core::string::StringArgumentType;
use crate::command::argument_types::entity::EntityArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

const DESCRIPTION: &str =
    "Drops the given loot table into the specified inventory or into the world.";
const PERMISSION: &str = "minecraft:command.loot";

static ERROR_INVALID_LOOT_TABLE: CommandErrorType<1> = CommandErrorType::new(
    translation::bedrock::COMMANDS_LOOT_FAILURE_INVALIDLOOTTABLE,
    "Loot table '%s' not found",
);

static ERROR_ENTITY_NO_LOOT_TABLE: CommandErrorType<1> = CommandErrorType::new(
    translation::bedrock::COMMANDS_LOOT_FAILURE_ENTITYNOLOOTTABLE,
    "Entity %s has no loot table",
);

static ERROR_NO_CONTAINER: CommandErrorType<1> = CommandErrorType::new(
    translation::bedrock::COMMANDS_LOOT_FAILURE_NOCONTAINER,
    "Target position %s is not a container",
);

#[derive(Clone, Copy)]
enum Target {
    Give,
    Spawn,
    Insert,
}

#[derive(Clone, Copy)]
enum Source {
    Loot,
    Kill,
    Mine {
        #[allow(dead_code)]
        has_tool: bool,
    },
}

struct LootExecutor {
    target: Target,
    source: Source,
}

fn insert_into_inventory(
    inventory: &dyn pumpkin_world::inventory::Inventory,
    mut stack: pumpkin_data::item_stack::ItemStack,
) -> pumpkin_data::item_stack::ItemStack {
    for i in 0..inventory.size() {
        if stack.is_empty() {
            break;
        }
        let mut slot_stack = inventory.get_stack(i);
        if !slot_stack.is_empty() && slot_stack.get_item().id == stack.get_item().id {
            let max_stack_size = 64;
            let space = max_stack_size - slot_stack.item_count;
            if space > 0 {
                let to_add = stack.item_count.min(space);
                slot_stack.item_count += to_add;
                stack.item_count -= to_add;
                inventory.set_stack(i, slot_stack);
            }
        }
    }

    for i in 0..inventory.size() {
        if stack.is_empty() {
            break;
        }
        let slot_stack = inventory.get_stack(i);
        if slot_stack.is_empty() {
            inventory.set_stack(i, stack.clone());
            stack.item_count = 0;
            break;
        }
    }

    stack
}

impl CommandExecutor for LootExecutor {
    #[allow(clippy::too_many_lines)]
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let mut stacks = Vec::new();

        match self.source {
            Source::Loot => {
                let loot_table_str = StringArgumentType::get(context, "loot_table")?;
                let formatted_key = if loot_table_str.contains(':') {
                    loot_table_str.to_string()
                } else {
                    format!("minecraft:{loot_table_str}")
                };

                let loot_table = pumpkin_data::loot_table::get_loot_table(&formatted_key);
                if let Some(table) = loot_table {
                    let seed: i64 = rand::random();
                    stacks = crate::world::loot::generate_loot(table, seed);
                } else {
                    return Err(ERROR_INVALID_LOOT_TABLE
                        .create_without_context(TextComponent::text(loot_table_str.to_string())));
                }
            }
            Source::Kill => {
                let target_entities = EntityArgumentType::get_entities(context, "target_entity")?;
                let mut has_loot = false;
                for entity in target_entities {
                    let resource_name = entity.get_entity().entity_type.resource_name;
                    let key = format!("minecraft:entities/{resource_name}");
                    if let Some(loot_table) = pumpkin_data::loot_table::get_loot_table(&key) {
                        has_loot = true;
                        let seed: i64 = rand::random();
                        stacks.extend(crate::world::loot::generate_loot(loot_table, seed));
                    }
                }
                if !has_loot {
                    let entity_name = "selected entity".to_string();
                    return Err(ERROR_ENTITY_NO_LOOT_TABLE
                        .create_without_context(TextComponent::text(entity_name)));
                }
            }
            Source::Mine { has_tool: _ } => {
                let pos = BlockPosArgumentType::get_block_pos(context, "mine_pos")
                    .or_else(|_| BlockPosArgumentType::get_block_pos(context, "pos"))?;
                let world = context.world();
                let block = world.get_block(&pos);
                let key = format!("minecraft:blocks/{}", block.name);

                if let Some(loot_table) = pumpkin_data::loot_table::get_loot_table(&key) {
                    let seed: i64 = rand::random();
                    stacks.extend(crate::world::loot::generate_loot(loot_table, seed));
                }
            }
        }

        let total_items: i32 = stacks.iter().map(|s| s.item_count as i32).sum();

        match self.target {
            Target::Give => {
                let targets = EntityArgumentType::get_players(context, "targets")?;
                for player in &targets {
                    for stack in &stacks {
                        let mut remaining = stack.clone();
                        player.inventory.insert_stack_anywhere(&mut remaining);
                        if !remaining.is_empty() {
                            player.drop_item(remaining);
                        }
                    }
                }
            }
            Target::Spawn => {
                let pos = BlockPosArgumentType::get_block_pos(context, "pos")?;
                let world = context.world();
                for stack in stacks {
                    world.drop_stack(&pos, stack);
                }
            }
            Target::Insert => {
                let pos = BlockPosArgumentType::get_block_pos(context, "pos")?;
                let world = context.world().clone();
                let Some(block_entity) = world.get_block_entity(&pos) else {
                    return Err(ERROR_NO_CONTAINER
                        .create_without_context(TextComponent::text(pos.to_string())));
                };
                let Some(inventory) = block_entity.get_inventory() else {
                    return Err(ERROR_NO_CONTAINER
                        .create_without_context(TextComponent::text(pos.to_string())));
                };

                for stack in stacks {
                    let remaining = insert_into_inventory(inventory.as_ref(), stack);
                    if !remaining.is_empty() {
                        world.drop_stack(&pos, remaining);
                    }
                }
            }
        }

        let msg = TextComponent::translate_cross(
            translation::bedrock::COMMANDS_LOOT_SUCCESS,
            translation::bedrock::COMMANDS_LOOT_SUCCESS,
            [TextComponent::text(total_items.to_string())],
        );
        context.source.send_feedback(msg, true);

        Ok(total_items)
    }
}

#[expect(clippy::too_many_lines)]
pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    let builder = command("loot", DESCRIPTION)
        .requires(PERMISSION)
        // give <targets>
        .then(
            literal("give").then(
                argument("targets", EntityArgumentType::Players)
                    .then(literal("loot").then(
                        argument("loot_table", StringArgumentType::SingleWord).executes(
                            LootExecutor {
                                target: Target::Give,
                                source: Source::Loot,
                            },
                        ),
                    ))
                    .then(literal("kill").then(
                        argument("target_entity", EntityArgumentType::Entities).executes(
                            LootExecutor {
                                target: Target::Give,
                                source: Source::Kill,
                            },
                        ),
                    ))
                    .then(
                        literal("mine").then(
                            argument("pos", BlockPosArgumentType)
                                .executes(LootExecutor {
                                    target: Target::Give,
                                    source: Source::Mine { has_tool: false },
                                })
                                .then(argument("tool", StringArgumentType::SingleWord).executes(
                                    LootExecutor {
                                        target: Target::Give,
                                        source: Source::Mine { has_tool: true },
                                    },
                                )),
                        ),
                    ),
            ),
        )
        // spawn <pos>
        .then(
            literal("spawn").then(
                argument("pos", BlockPosArgumentType)
                    .then(literal("loot").then(
                        argument("loot_table", StringArgumentType::SingleWord).executes(
                            LootExecutor {
                                target: Target::Spawn,
                                source: Source::Loot,
                            },
                        ),
                    ))
                    .then(literal("kill").then(
                        argument("target_entity", EntityArgumentType::Entities).executes(
                            LootExecutor {
                                target: Target::Spawn,
                                source: Source::Kill,
                            },
                        ),
                    ))
                    .then(
                        literal("mine").then(
                            argument("mine_pos", BlockPosArgumentType)
                                .executes(LootExecutor {
                                    target: Target::Spawn,
                                    source: Source::Mine { has_tool: false },
                                })
                                .then(argument("tool", StringArgumentType::SingleWord).executes(
                                    LootExecutor {
                                        target: Target::Spawn,
                                        source: Source::Mine { has_tool: true },
                                    },
                                )),
                        ),
                    ),
            ),
        )
        // insert <pos>
        .then(
            literal("insert").then(
                argument("pos", BlockPosArgumentType)
                    .then(literal("loot").then(
                        argument("loot_table", StringArgumentType::SingleWord).executes(
                            LootExecutor {
                                target: Target::Insert,
                                source: Source::Loot,
                            },
                        ),
                    ))
                    .then(literal("kill").then(
                        argument("target_entity", EntityArgumentType::Entities).executes(
                            LootExecutor {
                                target: Target::Insert,
                                source: Source::Kill,
                            },
                        ),
                    ))
                    .then(
                        literal("mine").then(
                            argument("mine_pos", BlockPosArgumentType)
                                .executes(LootExecutor {
                                    target: Target::Insert,
                                    source: Source::Mine { has_tool: false },
                                })
                                .then(argument("tool", StringArgumentType::SingleWord).executes(
                                    LootExecutor {
                                        target: Target::Insert,
                                        source: Source::Mine { has_tool: true },
                                    },
                                )),
                        ),
                    ),
            ),
        );

    dispatcher.register(builder);
}
