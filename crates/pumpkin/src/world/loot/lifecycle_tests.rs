use std::sync::{Arc, Mutex, RwLock, Weak};

use arc_swap::ArcSwap;
use pumpkin_config::{
    AdvancedConfiguration, BasicConfiguration, TelemetryConfig, world::LevelConfig,
};
use pumpkin_data::{
    Block,
    data_component::DataComponent,
    data_component_impl::{ContainerImpl, CustomNameImpl, LoreImpl},
    dimension::Dimension,
    item::Item,
    item_stack::ItemStack,
};
use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};
use pumpkin_util::{GameMode, math::position::BlockPos, text::TextComponent, world_seed::Seed};
use pumpkin_world::{
    chunk::ChunkData,
    level::Level,
    world::BlockFlags,
    world_info::{LevelData, WorldInfoWriter, anvil::AnvilLevelInfo},
};

use crate::{
    block::{entities::block_entity_from_nbt, registry::default_registry},
    data::{
        VanillaData, banned_ip::BannedIpList, banned_player::BannedPlayerList, op::OperatorConfig,
        usercache::UserCache, whitelist::WhitelistConfig,
    },
    entity::player::Player,
    net::{
        ClientPlatform, GameProfile, PacketRateLimiter, PlayerConfig,
        java::{JavaClient, pending::PendingConnection},
    },
    plugin::{BoxFuture, EventHandler, EventPriority, block::block_drop_item::BlockDropItemEvent},
    server::Server,
    world::{
        World,
        explosion::{BlockInteraction, Explosion},
    },
};

const POSITION: BlockPos = BlockPos::new(8, 64, 8);

#[derive(Clone, Copy)]
enum DropMethod<'a> {
    Break,
    Explosion,
    SkipDrops,
    Creative {
        player: &'a Arc<Player>,
        filled: bool,
    },
}

struct DropOutcome {
    items: Vec<ItemStack>,
    block_removed: bool,
    emptied: bool,
}

/// Builds version-matched container NBT with a styled name and independently specified sparse slots.
fn container_fixture(entity_id: &str) -> NbtCompound {
    let mut nbt = NbtCompound::new();
    nbt.put_string("id", entity_id.to_owned());
    nbt.put_int("x", POSITION.0.x);
    nbt.put_int("y", POSITION.0.y);
    nbt.put_int("z", POSITION.0.z);
    let mut name = NbtCompound::new();
    name.put_string("text", "Stored container".to_owned());
    name.put_bool("bold", true);
    nbt.put_compound("CustomName", name);
    let mut components = NbtCompound::new();
    components.put_list(
        "minecraft:lore",
        vec![NbtTag::String("Retained lore".into())],
    );
    nbt.put_compound("components", components);
    nbt.put_list(
        "Items",
        [(0, "diamond", 2), (10, "emerald", 3), (26, "gold_ingot", 4)]
            .into_iter()
            .map(|(slot, item, count)| {
                let mut stack = NbtCompound::new();
                stack.put_byte("Slot", slot);
                stack.put_string("id", format!("minecraft:{item}"));
                stack.put_int("count", count);
                NbtTag::Compound(stack)
            })
            .collect(),
    );
    nbt
}

/// Runs the real block-removal path in an isolated loaded chunk and shuts down its level before returning observations.
async fn collect_drops(
    block: &'static Block,
    entity_id: &str,
    method: DropMethod<'_>,
) -> Result<DropOutcome, Box<dyn std::error::Error>> {
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
    let result = remove_container(&world, block, entity_id, method);
    clear_dropped_entities(&world);
    level.shutdown().await;
    result
}

/// Inserts a fixture into a loaded chunk and records the real removal path's spawned items and source cleanup.
fn remove_container(
    world: &Arc<World>,
    block: &'static Block,
    entity_id: &str,
    method: DropMethod<'_>,
) -> Result<DropOutcome, Box<dyn std::error::Error>> {
    let chunk = ChunkData::empty_sync(0, 0);
    chunk.set_block_absolute_y(8, POSITION.0.y, 8, block.default_state.id);
    world
        .level
        .loaded_chunks
        .insert(POSITION.chunk_position(), chunk);
    let mut nbt = container_fixture(entity_id);
    if matches!(method, DropMethod::Creative { filled: false, .. }) {
        nbt.put_list("Items", Vec::new());
    }
    let Some(entity) = block_entity_from_nbt(&nbt) else {
        return Err("fixture block entity was not recognized".into());
    };
    world.add_block_entity(entity.clone());
    match method {
        DropMethod::Break => {
            world.break_block(&POSITION, None, BlockFlags::NOTIFY_ALL);
        }
        DropMethod::Explosion => {
            Explosion::new(4.0, POSITION.to_centered_f64(), BlockInteraction::Destroy)
                .explode(world);
        }
        DropMethod::SkipDrops => {
            world.break_block(
                &POSITION,
                None,
                BlockFlags::NOTIFY_ALL | BlockFlags::SKIP_DROPS,
            );
        }
        DropMethod::Creative { player, .. } => {
            world.break_block(
                &POSITION,
                Some(player),
                BlockFlags::NOTIFY_ALL | BlockFlags::SKIP_DROPS,
            );
        }
    }
    let items = world
        .entities
        .load()
        .iter()
        .filter_map(|entity| entity.get_item_entity())
        .map(|entity| {
            entity
                .get_item_stack()
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone()
        })
        .collect();
    let outcome = DropOutcome {
        items,
        block_removed: world.get_block(&POSITION).is_air(),
        emptied: entity
            .get_inventory()
            .is_some_and(|inventory| inventory.is_empty()),
    };
    Ok(outcome)
}

/// Releases item entities through normal removal so fixture worlds do not retain entity/world reference cycles.
fn clear_dropped_entities(world: &World) {
    let entities = world.entities.load_full();
    for entity in entities.iter() {
        world.remove_entity(entity.as_ref());
    }
}

/// Normal chest breaking copies its styled name while scattering each inventory stack exactly once.
#[tokio::test]
async fn breaking_chest_keeps_name_and_scatters_contents() -> Result<(), Box<dyn std::error::Error>>
{
    let outcome = collect_drops(&Block::CHEST, "minecraft:chest", DropMethod::Break).await?;
    assert!(outcome.block_removed);
    assert!(outcome.emptied);
    let chest = outcome
        .items
        .iter()
        .find(|stack| stack.item == &Item::CHEST)
        .ok_or("missing chest drop")?;
    assert_eq!(
        chest
            .get_data_component::<CustomNameImpl>()
            .map(|name| name.name.clone()),
        Some(TextComponent::text("Stored container").bold())
    );
    assert!(
        chest
            .get_data_component::<ContainerImpl>()
            .is_some_and(|container| container.items.is_empty())
    );
    assert!(
        !chest
            .patch
            .iter()
            .any(|(kind, _)| *kind == DataComponent::Container)
    );
    for (item, expected) in [
        (&Item::DIAMOND, 2),
        (&Item::EMERALD, 3),
        (&Item::GOLD_INGOT, 4),
    ] {
        assert_eq!(
            outcome
                .items
                .iter()
                .filter(|stack| stack.item == item)
                .map(|stack| u32::from(stack.item_count))
                .sum::<u32>(),
            expected
        );
    }
    assert_eq!(outcome.items.len(), 4);
    Ok(())
}

/// Shulker breaking packages slots 0, 10, and 26 into one named item before removing its block entity.
#[tokio::test]
async fn breaking_shulker_keeps_sparse_inventory() -> Result<(), Box<dyn std::error::Error>> {
    let outcome = collect_drops(
        &Block::SHULKER_BOX,
        "minecraft:shulker_box",
        DropMethod::Break,
    )
    .await?;
    assert_shulker_drop(&outcome)
}

/// Explosion loot reads the original shulker entity before replacement and keeps its sparse inventory.
#[tokio::test]
async fn exploding_shulker_keeps_original_components() -> Result<(), Box<dyn std::error::Error>> {
    let outcome = collect_drops(
        &Block::SHULKER_BOX,
        "minecraft:shulker_box",
        DropMethod::Explosion,
    )
    .await?;
    assert_shulker_drop(&outcome)
}

/// Explicitly suppressed shulker loot remains suppressed and does not scatter retained contents.
#[tokio::test]
async fn skipped_shulker_drops_remain_suppressed() -> Result<(), Box<dyn std::error::Error>> {
    let outcome = collect_drops(
        &Block::SHULKER_BOX,
        "minecraft:shulker_box",
        DropMethod::SkipDrops,
    )
    .await?;
    assert!(outcome.block_removed);
    assert!(outcome.items.is_empty());
    assert!(!outcome.emptied);
    Ok(())
}

/// Checks the independent expected name, slot indices, item types, and counts for a retained shulker drop.
fn assert_shulker_drop(outcome: &DropOutcome) -> Result<(), Box<dyn std::error::Error>> {
    assert!(outcome.block_removed);
    assert!(!outcome.emptied);
    assert_shulker_components(&outcome.items)
}

/// Checks that one shulker stack has the fixture's styled name and all three original sparse slots.
fn assert_shulker_components(items: &[ItemStack]) -> Result<(), Box<dyn std::error::Error>> {
    let [stack] = items else {
        return Err("expected exactly one shulker drop without scattered contents".into());
    };
    assert_eq!(stack.item, &Item::SHULKER_BOX);
    assert_eq!(
        stack
            .get_data_component::<CustomNameImpl>()
            .map(|name| name.name.clone()),
        Some(TextComponent::text("Stored container").bold())
    );
    let container = stack
        .get_data_component::<ContainerImpl>()
        .ok_or("missing retained inventory")?;
    assert_eq!(container.items.len(), 3);
    for (slot, item, count) in [
        (0, &Item::DIAMOND, 2),
        (10, &Item::EMERALD, 3),
        (26, &Item::GOLD_INGOT, 4),
    ] {
        let (_, stack) = container
            .items
            .iter()
            .find(|(index, _)| *index == slot)
            .ok_or("missing original slot")?;
        assert_eq!(stack.item, item);
        assert_eq!(stack.item_count, count);
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum DropOverride {
    Cancel,
    Replace,
    Edit,
}

struct DropObserver {
    action: Mutex<DropOverride>,
    observed: Mutex<Vec<ItemStack>>,
}

impl EventHandler<BlockDropItemEvent> for DropObserver {
    /// Records components visible to plugins, then cancels, replaces, or edits the generated item.
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut BlockDropItemEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            *self
                .observed
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = event.items.clone();
            match *self
                .action
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
            {
                DropOverride::Cancel => event.cancelled = true,
                DropOverride::Replace => event.items = vec![ItemStack::new(1, &Item::STONE)],
                DropOverride::Edit => {
                    for stack in &mut event.items {
                        stack.set_data_component(CustomNameImpl {
                            name: TextComponent::text("Plugin name"),
                        });
                        stack.set_data_component(ContainerImpl { items: Vec::new() });
                    }
                }
            }
        })
    }
}

/// Starts an in-process plugin fixture without listeners, authentication requests, persistent account data, or telemetry.
async fn plugin_test_server(
    directory: &std::path::Path,
) -> Result<Arc<Server>, Box<dyn std::error::Error>> {
    AnvilLevelInfo.write_world_info(&LevelData::default(Seed(0)), directory)?;
    let basic = BasicConfiguration {
        seed: Seed(0),
        default_level_name: directory.to_string_lossy().into_owned(),
        allow_nether: false,
        allow_end: false,
        use_favicon: false,
        allow_chat_reports: false,
        spawn_protection: 0,
        ..BasicConfiguration::default()
    };
    let mut advanced = AdvancedConfiguration::default();
    advanced.networking.java.enabled = false;
    advanced.networking.bedrock.enabled = false;
    advanced.networking.bedrock.authentication.enabled = false;
    let data = VanillaData {
        banned_ip_list: RwLock::new(BannedIpList::default()),
        banned_player_list: RwLock::new(BannedPlayerList::default()),
        operator_config: RwLock::new(OperatorConfig::default()),
        user_cache: RwLock::new(UserCache::default()),
        whitelist_config: RwLock::new(WhitelistConfig::default()),
    };
    Ok(Server::new(
        basic,
        advanced,
        TelemetryConfig {
            enabled: false,
            ..TelemetryConfig::default()
        },
        data,
    )
    .await)
}

/// Constructs a creative player from an immediately closed loopback listener without starting networking tasks.
async fn creative_player(
    world: &Arc<World>,
) -> Result<(Arc<Player>, tokio::net::TcpStream), Box<dyn std::error::Error>> {
    let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).await?;
    let peer = tokio::net::TcpStream::connect(listener.local_addr()?).await?;
    let (socket, address) = listener.accept().await?;
    drop(listener);
    let profile = GameProfile {
        id: uuid::Uuid::from_u128(1),
        name: "ComponentFixture".to_owned(),
        properties: ArcSwap::from_pointee(Vec::new()),
        profile_actions: None,
    };
    let config = PlayerConfig::default();
    let pending =
        PendingConnection::new(socket, address, 1, PacketRateLimiter::new(false, 1.0, 1.0));
    let client = JavaClient::from_pending(pending, profile.clone(), config.clone());
    Ok((
        Arc::new(Player::new(
            Arc::new(ClientPlatform::Java(client)),
            profile,
            config,
            world,
            GameMode::Creative,
        )),
        peer,
    ))
}

/// Creative filled shulkers copy retained lore, empty shulkers stay suppressed, and plugin replacement remains authoritative.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn creative_shulker_drops_keep_all_components() -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::TempDir::new()?;
    let server = plugin_test_server(directory.path()).await?;
    let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
    let (player, peer) = match creative_player(&world).await {
        Ok(player) => player,
        Err(error) => {
            world.level.world_portal.store(Arc::new(None));
            server.shutdown().await;
            return Err(error);
        }
    };
    let filled = remove_container(
        &world,
        &Block::SHULKER_BOX,
        "minecraft:shulker_box",
        DropMethod::Creative {
            player: &player,
            filled: true,
        },
    );
    clear_dropped_entities(&world);
    let empty = remove_container(
        &world,
        &Block::SHULKER_BOX,
        "minecraft:shulker_box",
        DropMethod::Creative {
            player: &player,
            filled: false,
        },
    );
    clear_dropped_entities(&world);
    let survival = remove_container(
        &world,
        &Block::SHULKER_BOX,
        "minecraft:shulker_box",
        DropMethod::Break,
    );
    clear_dropped_entities(&world);
    let observer = Arc::new(DropObserver {
        action: Mutex::new(DropOverride::Replace),
        observed: Mutex::new(Vec::new()),
    });
    server.plugin_manager.register::<BlockDropItemEvent, _>(
        observer.clone(),
        EventPriority::Normal,
        true,
    );
    let replaced = remove_container(
        &world,
        &Block::SHULKER_BOX,
        "minecraft:shulker_box",
        DropMethod::Creative {
            player: &player,
            filled: true,
        },
    );
    clear_dropped_entities(&world);
    let observed = observer
        .observed
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    drop(player);
    drop(peer);
    world.level.world_portal.store(Arc::new(None));
    server.shutdown().await;
    let filled = filled?;
    assert_shulker_drop(&filled)?;
    assert_retained_lore(&filled.items);
    let empty = empty?;
    assert!(empty.block_removed);
    assert!(empty.items.is_empty());
    let survival = survival?;
    assert_shulker_drop(&survival)?;
    assert!(
        survival
            .items
            .first()
            .and_then(|stack| stack.get_data_component::<LoreImpl>())
            .is_some_and(|lore| lore.lines.is_empty())
    );
    assert_retained_lore(&observed);
    let replaced = replaced?;
    assert!(replaced.block_removed);
    let [replacement] = replaced.items.as_slice() else {
        return Err("expected one creative plugin replacement".into());
    };
    assert_eq!(replacement.item, &Item::STONE);
    assert!(replacement.get_data_component::<ContainerImpl>().is_none());
    Ok(())
}

/// Requires the exact retained lore value on the first generated or plugin-observed item.
fn assert_retained_lore(items: &[ItemStack]) {
    assert_eq!(
        items
            .first()
            .and_then(|stack| stack.get_data_component::<LoreImpl>())
            .map(|lore| lore.lines.clone()),
        Some(vec![TextComponent::text("Retained lore")]),
    );
}

/// Plugins see generated components, and their cancellation, replacement items, and component edits survive spawning.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn plugin_drop_edits_remain_authoritative() -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::TempDir::new()?;
    let server = plugin_test_server(directory.path()).await?;
    let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
    let observer = Arc::new(DropObserver {
        action: Mutex::new(DropOverride::Cancel),
        observed: Mutex::new(Vec::new()),
    });
    server.plugin_manager.register::<BlockDropItemEvent, _>(
        observer.clone(),
        EventPriority::Normal,
        true,
    );
    let mut outcomes = Vec::new();
    for action in [
        DropOverride::Cancel,
        DropOverride::Replace,
        DropOverride::Edit,
    ] {
        *observer
            .action
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = action;
        let outcome = remove_container(
            &world,
            &Block::SHULKER_BOX,
            "minecraft:shulker_box",
            DropMethod::Break,
        );
        let observed = observer
            .observed
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        clear_dropped_entities(&world);
        outcomes.push((action, outcome, observed));
    }
    world.level.world_portal.store(Arc::new(None));
    server.shutdown().await;
    for (action, outcome, observed) in outcomes {
        let outcome = outcome?;
        assert!(outcome.block_removed);
        assert_shulker_components(&observed)?;
        match action {
            DropOverride::Cancel => assert!(outcome.items.is_empty()),
            DropOverride::Replace => {
                let [stack] = outcome.items.as_slice() else {
                    return Err("expected exactly one plugin replacement item".into());
                };
                assert_eq!(stack.item, &Item::STONE);
                assert!(stack.get_data_component::<ContainerImpl>().is_none());
                assert!(stack.get_data_component::<CustomNameImpl>().is_none());
            }
            DropOverride::Edit => {
                let [stack] = outcome.items.as_slice() else {
                    return Err("expected exactly one edited shulker".into());
                };
                assert_eq!(
                    stack
                        .get_data_component::<CustomNameImpl>()
                        .map(|name| name.name.clone()),
                    Some(TextComponent::text("Plugin name"))
                );
                assert!(
                    stack
                        .get_data_component::<ContainerImpl>()
                        .is_some_and(|container| container.items.is_empty())
                );
            }
        }
    }
    Ok(())
}
