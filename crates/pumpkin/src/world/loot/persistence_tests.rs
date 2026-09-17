use std::{
    path::Path,
    sync::{Arc, Weak},
};

use arc_swap::ArcSwap;
use pumpkin_config::world::LevelConfig;
use pumpkin_data::{
    Block,
    data_component_impl::{ContainerLootImpl, CustomNameImpl, LockImpl, LoreImpl},
    dimension::Dimension,
    item::Item,
    item_stack::ItemStack,
};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::{math::position::BlockPos, text::TextComponent, world_seed::Seed};
use pumpkin_world::{
    chunk::{
        ChunkData,
        io::{FileIO, LoadedData},
    },
    level::Level,
    world_info::LevelData,
};

use crate::{
    block::{
        entities::{BlockEntity, shulker_box::ShulkerBoxBlockEntity},
        registry::default_registry,
    },
    world::World,
};

const POSITION: BlockPos = BlockPos::new(7, 80, 0);

struct SavedEntity {
    pending: NbtCompound,
    restored: NbtCompound,
    visible: Option<NbtCompound>,
}

/// Creates an isolated world whose chunk IO uses the supplied temporary directory.
fn world_at(directory: &Path) -> Arc<World> {
    Arc::new(World::load(
        Level::from_root_folder(
            &LevelConfig::default(),
            directory.to_path_buf(),
            0,
            Dimension::OVERWORLD,
        ),
        Arc::new(ArcSwap::from_pointee(LevelData::default(Seed(0)))),
        Dimension::OVERWORLD,
        default_registry(),
        Weak::new(),
    ))
}

/// Builds a named shulker whose lock and pending loot must remain absent from render NBT.
fn deferred_shulker_item() -> ItemStack {
    let mut stack = ItemStack::new(1, &Item::SHULKER_BOX);
    stack.set_data_component(CustomNameImpl {
        name: TextComponent::text("Deferred archive").bold(),
    });
    let mut predicate = NbtCompound::new();
    predicate.put_string("items", "minecraft:tripwire_hook".to_owned());
    stack.set_data_component(LockImpl { predicate });
    stack.set_data_component(ContainerLootImpl {
        loot_table: "minecraft:chests/simple_dungeon".to_owned(),
        seed: 42,
    });
    stack.set_data_component(LoreImpl {
        lines: vec![TextComponent::text("Retained archive metadata")],
    });
    stack
}

/// Reloads a saved region through a fresh world and returns the restored entity's persistent NBT.
async fn reload_saved_entity(directory: &Path) -> Result<NbtCompound, Box<dyn std::error::Error>> {
    let world = world_at(directory);
    let (send, mut receive) = tokio::sync::mpsc::channel(1);
    world
        .level
        .chunk_saver
        .fetch_chunks(
            &world.level.level_folder,
            &[POSITION.chunk_position()],
            send,
        )
        .await;
    let restored = match receive.recv().await {
        Some(LoadedData::Loaded(chunk)) => {
            world
                .level
                .loaded_chunks
                .insert(POSITION.chunk_position(), chunk);
            world
                .get_block_entity(&POSITION)
                .map(|entity| {
                    let mut nbt = NbtCompound::new();
                    entity.write_internal(&mut nbt);
                    if let Some(custom) = world.custom_block_entity_data.get(&POSITION) {
                        nbt.put_compound("PumpkinCustomData", custom.clone());
                    }
                    nbt
                })
                .ok_or("Saved region lost its shulker block entity".into())
        }
        Some(LoadedData::Error((_, error))) => Err(error.into()),
        Some(LoadedData::Missing(_)) | None => Err("Saved region could not be reloaded".into()),
    };
    world.level.shutdown().await;
    restored
}

/// Saves the same pending snapshot used by autosave without invoking World's full-save repair path.
async fn save_registered_entity(update: bool) -> Result<SavedEntity, Box<dyn std::error::Error>> {
    let directory = tempfile::TempDir::new()?;
    let world = world_at(directory.path());
    let chunk = ChunkData::empty_sync(0, 0);
    chunk.set_block_absolute_y(7, POSITION.0.y, 0, Block::SHULKER_BOX.default_state.id);
    world
        .level
        .loaded_chunks
        .insert(POSITION.chunk_position(), chunk.clone());
    let entity: Arc<dyn BlockEntity> = Arc::new(ShulkerBoxBlockEntity::new(POSITION));
    let mut custom = NbtCompound::new();
    custom.put_bool("preserved", true);
    world.custom_block_entity_data.insert(POSITION, custom);
    if update {
        world.add_block_entity(entity.clone());
    }
    entity.apply_item_components(&deferred_shulker_item());
    let visible = entity.chunk_data_nbt();
    if update {
        world.update_block_entity(&entity);
    } else {
        world.add_block_entity(entity);
    }
    let pending = chunk
        .pending_block_entities
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get(&POSITION)
        .cloned();
    let result = world
        .level
        .chunk_saver
        .save_chunks(
            &world.level.level_folder,
            vec![(POSITION.chunk_position(), chunk)],
        )
        .await;
    world.level.shutdown().await;
    result?;
    let pending = pending.ok_or("Registering the shulker omitted persistent pending NBT")?;
    let restored = reload_saved_entity(directory.path()).await?;
    Ok(SavedEntity {
        pending,
        restored,
        visible,
    })
}

/// Verifies persistent fields against independent fixture values, including the deferred seed.
fn assert_deferred_fields(nbt: &NbtCompound) {
    assert_eq!(
        nbt.get_compound("lock")
            .and_then(|lock| lock.get_string("items")),
        Some("minecraft:tripwire_hook")
    );
    assert_eq!(
        nbt.get_string("LootTable"),
        Some("minecraft:chests/simple_dungeon")
    );
    assert_eq!(nbt.get_long("LootTableSeed"), Some(42));
    assert_eq!(
        nbt.get_compound("CustomName")
            .and_then(|name| name.get_string("text")),
        Some("Deferred archive")
    );
    assert_eq!(
        nbt.get_compound("CustomName")
            .and_then(|name| name.get_bool("bold")),
        Some(true)
    );
    assert_eq!(
        nbt.get_compound("components")
            .and_then(|components| components.get_list("minecraft:lore"))
            .and_then(|lines| lines.first())
            .and_then(pumpkin_nbt::tag::NbtTag::extract_string),
        Some("Retained archive metadata")
    );
    assert_eq!(
        nbt.get_compound("PumpkinCustomData")
            .and_then(|custom| custom.get_bool("preserved")),
        Some(true)
    );
}

/// Container render NBT keeps persistent-only fields out of block update packets.
fn assert_render_fields(nbt: &NbtCompound) {
    assert!(nbt.get("lock").is_none());
    assert!(nbt.get("LootTable").is_none());
    assert!(nbt.get("LootTableSeed").is_none());
    assert!(nbt.get("components").is_none());
    assert!(nbt.get("PumpkinCustomData").is_none());
    assert!(nbt.get_compound("CustomName").is_some());
}

/// Adding a component-bearing entity retains full state both in the autosave snapshot and on disk.
#[tokio::test]
async fn adding_entity_preserves_deferred_fields_in_region_save()
-> Result<(), Box<dyn std::error::Error>> {
    let saved = save_registered_entity(false).await?;
    assert_deferred_fields(&saved.pending);
    assert_deferred_fields(&saved.restored);
    assert_render_fields(
        saved
            .visible
            .as_ref()
            .ok_or("Shulker omitted its render NBT")?,
    );
    Ok(())
}

/// Updating a placed entity refreshes full persistent state independently of its render packet.
#[tokio::test]
async fn updating_entity_preserves_deferred_fields_in_region_save()
-> Result<(), Box<dyn std::error::Error>> {
    let saved = save_registered_entity(true).await?;
    assert_deferred_fields(&saved.pending);
    assert_deferred_fields(&saved.restored);
    assert_render_fields(
        saved
            .visible
            .as_ref()
            .ok_or("Shulker omitted its render NBT")?,
    );
    Ok(())
}
