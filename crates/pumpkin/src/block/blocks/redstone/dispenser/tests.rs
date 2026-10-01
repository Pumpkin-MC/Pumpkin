use std::sync::{Arc, Weak};

use arc_swap::ArcSwap;
use pumpkin_config::world::LevelConfig;
use pumpkin_data::{
    data_component_impl::{ContainerImpl, CustomNameImpl},
    dimension::Dimension,
};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::world_seed::Seed;
use pumpkin_world::{chunk::ChunkData, level::Level, world_info::LevelData};

use super::*;
use crate::block::registry::default_registry;

const SOURCE: BlockPos = BlockPos::new(8, 64, 8);
const TARGET: BlockPos = BlockPos::new(9, 64, 8);

struct DispenseOutcome {
    succeeded: bool,
    remaining: ItemStack,
    block: &'static Block,
    live_name: Option<TextComponent>,
    live_items: Vec<(usize, u16, u8)>,
    pending: Option<NbtCompound>,
}

/// Builds a styled shulker with three independently specified sparse inventory slots.
fn shulker_item() -> ItemStack {
    let mut stack = ItemStack::new(2, &Item::SHULKER_BOX);
    stack.set_data_component(CustomNameImpl {
        name: TextComponent::text("Dispensed archive").bold(),
    });
    stack.set_data_component(ContainerImpl {
        items: vec![
            (0, ItemStack::new(2, &Item::DIAMOND)),
            (10, ItemStack::new(3, &Item::EMERALD)),
            (26, ItemStack::new(4, &Item::GOLD_INGOT)),
        ],
    });
    stack
}

/// Runs actual dispenser placement in an isolated loaded chunk and shuts down its level before assertions.
async fn dispense(replaceable: bool) -> Result<DispenseOutcome, Box<dyn std::error::Error>> {
    let directory = tempfile::TempDir::new()?;
    let level = Level::from_root_folder(
        &LevelConfig::default(),
        directory.path().to_path_buf(),
        0,
        Dimension::OVERWORLD,
    );
    let world = Arc::new(World::load(
        level.clone(),
        Arc::new(ArcSwap::from_pointee(LevelData::default(Seed(0)))),
        Dimension::OVERWORLD,
        default_registry(),
        Weak::new(),
    ));
    let chunk = ChunkData::empty_sync(0, 0);
    chunk.set_block_absolute_y(8, SOURCE.0.y, 8, Block::DISPENSER.default_state.id);
    if !replaceable {
        chunk.set_block_absolute_y(9, TARGET.0.y, 8, Block::STONE.default_state.id);
    }
    world
        .level
        .loaded_chunks
        .insert(SOURCE.chunk_position(), chunk);
    let mut remaining = shulker_item();
    let context = DispenseContext {
        world: &world,
        position: &SOURCE,
        facing: Facing::East,
    };
    let succeeded = DispenserBlock::dispense_shulker_box(&context, &mut remaining);
    let entity = world.get_block_entity(&TARGET);
    let live_name = entity
        .as_ref()
        .and_then(|entity| entity.component_state())
        .and_then(|components| {
            components.with_component::<CustomNameImpl, _>(|name| name.name.clone())
        });
    let live_items = entity
        .and_then(crate::block::entities::BlockEntity::get_inventory)
        .map_or_else(Vec::new, |inventory| {
            (0..inventory.size())
                .filter_map(|slot| {
                    let stack = inventory.get_stack(slot);
                    (!stack.is_empty()).then_some((slot, stack.item.id, stack.item_count))
                })
                .collect()
        });
    let pending = world
        .level
        .read_chunk_sync(&TARGET.chunk_position(), |chunk| {
            chunk
                .pending_block_entities
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .get(&TARGET)
                .cloned()
        })
        .flatten();
    let outcome = DispenseOutcome {
        succeeded,
        remaining,
        block: world.get_block(&TARGET),
        live_name,
        live_items,
        pending,
    };
    level.shutdown().await;
    Ok(outcome)
}

/// Successful dispenser placement consumes exactly one item and refreshes saved update data after restoring metadata and slots.
#[tokio::test]
async fn placed_shulker_preserves_components_and_refreshes_pending_nbt()
-> Result<(), Box<dyn std::error::Error>> {
    let outcome = dispense(true).await?;
    assert!(outcome.succeeded);
    assert_eq!(outcome.remaining.item_count, 1);
    assert_eq!(outcome.block, &Block::SHULKER_BOX);
    assert_eq!(
        outcome.live_name,
        Some(TextComponent::text("Dispensed archive").bold())
    );
    assert_eq!(
        outcome.live_items,
        vec![
            (0, Item::DIAMOND.id, 2),
            (10, Item::EMERALD.id, 3),
            (26, Item::GOLD_INGOT.id, 4)
        ]
    );
    let pending = outcome.pending.ok_or("Missing placed block update data")?;
    let name = pending
        .get_compound("CustomName")
        .ok_or("Block update omitted the styled name")?;
    assert_eq!(name.get_string("text"), Some("Dispensed archive"));
    assert_eq!(name.get_bool("bold"), Some(true));
    let items = pending
        .get_list("Items")
        .ok_or("Block update omitted the restored items")?;
    let slots = items
        .iter()
        .map(|tag| {
            let item = tag
                .extract_compound()
                .ok_or("Invalid saved inventory entry")?;
            Ok((
                item.get_byte("Slot").ok_or("Missing saved slot")?,
                item.get_string("id").ok_or("Missing saved item")?,
                item.get_int("count").ok_or("Missing saved count")?,
            ))
        })
        .collect::<Result<Vec<_>, &'static str>>()?;
    assert_eq!(
        slots,
        vec![
            (0, "minecraft:diamond", 2),
            (10, "minecraft:emerald", 3),
            (26, "minecraft:gold_ingot", 4)
        ]
    );
    Ok(())
}

/// A blocked dispenser target leaves both the item count and its stored components unchanged.
#[tokio::test]
async fn blocked_shulker_placement_preserves_the_source_stack()
-> Result<(), Box<dyn std::error::Error>> {
    let outcome = dispense(false).await?;
    assert!(!outcome.succeeded);
    assert_eq!(outcome.block, &Block::STONE);
    assert_eq!(outcome.remaining.item_count, 2);
    assert_eq!(
        outcome
            .remaining
            .get_data_component::<CustomNameImpl>()
            .map(|name| name.name.clone()),
        Some(TextComponent::text("Dispensed archive").bold())
    );
    let container = outcome
        .remaining
        .get_data_component::<ContainerImpl>()
        .ok_or("Missing retained container")?;
    let items: Vec<_> = container
        .items
        .iter()
        .map(|(slot, stack)| (*slot, stack.item.id, stack.item_count))
        .collect();
    assert_eq!(
        items,
        vec![
            (0, Item::DIAMOND.id, 2),
            (10, Item::EMERALD.id, 3),
            (26, Item::GOLD_INGOT.id, 4)
        ]
    );
    assert!(outcome.live_items.is_empty());
    assert!(outcome.pending.is_none());
    Ok(())
}
