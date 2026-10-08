use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};

use arc_swap::ArcSwap;
use pumpkin_config::{AdvancedConfiguration, BasicConfiguration, TelemetryConfig};
use pumpkin_data::data_component_impl::{CustomNameImpl, EquipmentSlot};
use pumpkin_data::game_rules::{GameRule, GameRuleValue};
use pumpkin_data::{Block, damage::DamageType, dimension::Dimension, entity::EntityType};
use pumpkin_data::{item::Item, item_stack::ItemStack};
use pumpkin_protocol::{ConnectionState, java::server::play::SAttack};
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
use pumpkin_util::{GameMode, text::TextComponent};
use pumpkin_world::chunk::ChunkData;
use tokio::net::{TcpListener, TcpStream};

use crate::data::VanillaData;
use crate::entity::vehicle::{boat::BoatEntity, minecart::MinecartEntity};
use crate::entity::{Entity, EntityBase, player::Player};
use crate::item::{ItemMetadata, items::boat::BoatItem};
use crate::net::java::{JavaClient, pending::PendingConnection};
use crate::net::{ClientPlatform, GameProfile, PacketRateLimiter, PlayerConfig};
use crate::plugin::EventHandler;
use crate::plugin::api::events::EventPriority;
use crate::plugin::api::events::entity::{
    entity_spawn::EntitySpawnEvent, item_spawn::ItemSpawnEvent,
};
use crate::plugin::api::events::vehicle::{
    vehicle_damage::VehicleDamageEvent, vehicle_destroy::VehicleDestroyEvent,
};
use crate::server::Server;
use crate::world::World;

const BOAT_POS: Vector3<f64> = Vector3::new(8.0, 100.0, 8.0);

async fn test_world() -> (tempfile::TempDir, Arc<Server>, Arc<World>) {
    let temp = tempfile::tempdir().unwrap();
    let basic = BasicConfiguration {
        default_level_name: temp.path().to_str().unwrap().to_owned(),
        allow_nether: false,
        allow_end: false,
        allow_chat_reports: false,
        use_favicon: false,
        ..Default::default()
    };
    let mut advanced = AdvancedConfiguration::default();
    advanced.networking.bedrock.online_mode = false;
    let server = Server::new(
        basic,
        advanced,
        TelemetryConfig::default(),
        VanillaData {
            banned_ip_list: RwLock::default(),
            banned_player_list: RwLock::default(),
            operator_config: RwLock::default(),
            user_cache: RwLock::default(),
            whitelist_config: RwLock::default(),
        },
        Vec::new(),
    )
    .await
    .unwrap();
    let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
    let chunk = ChunkData::empty_sync(0, 0);
    chunk.set_block_absolute_y(8, 99, 5, Block::STONE.default_state.id);
    world
        .level
        .loaded_chunks
        .insert(BlockPos::new(8, 100, 5).chunk_position(), chunk);
    (temp, server, world)
}

async fn equipped_player(server: &Arc<Server>, world: &Arc<World>) -> (Arc<Player>, TcpStream) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let peer = TcpStream::connect(listener.local_addr().unwrap())
        .await
        .unwrap();
    let (socket, address) = listener.accept().await.unwrap();
    let pending = PendingConnection::new(
        socket,
        address,
        1,
        PacketRateLimiter::new(false, 0.0, 0.0),
        Arc::downgrade(server),
    );
    pending.connection_state.store(ConnectionState::Play);
    let profile = GameProfile {
        id: uuid::Uuid::new_v4(),
        name: "BoatBreaker".to_owned(),
        properties: ArcSwap::from_pointee(Vec::new()),
        profile_actions: None,
    };
    let client = JavaClient::from_pending(pending, profile.clone(), PlayerConfig::default());
    let player = Arc::new(Player::new(
        Arc::new(ClientPlatform::Java(client)),
        profile,
        PlayerConfig::default(),
        world,
        GameMode::Survival,
    ));
    player.set_client_loaded(true);
    // Within attack range, outside pickup range. No ticks run after an attack.
    player.get_entity().set_pos(Vector3::new(8.0, 100.0, 5.0));
    player.get_entity().on_ground.store(true, Ordering::Relaxed);
    world.add_player(&player).unwrap();
    let sword = ItemStack::new(1, &Item::IRON_SWORD);
    player.inventory().set_held_item(sword.clone());
    player
        .living_entity
        .send_equipment_changes(&[(EquipmentSlot::MAIN_HAND, sword)]);
    for _ in 0..20 {
        player.tick(server);
    }
    (player, peer)
}

fn spawn_boat(world: &Arc<World>, item: &Item) -> Arc<BoatEntity> {
    let boat = Arc::new(BoatEntity::new(Entity::new(
        world.clone(),
        BOAT_POS,
        BoatItem::item_to_entity(item),
    )));
    assert!(world.spawn_entity(boat.clone()));
    boat
}

fn damage(entity: &dyn EntityBase, amount: f32) -> bool {
    entity.damage_with_context(entity, amount, DamageType::PLAYER_ATTACK, None, None, None)
}

fn attack(player: &Arc<Player>, boat: &BoatEntity, server: &Arc<Server>) {
    assert!(player.get_entity().on_ground.load(Ordering::Relaxed));
    player.client.java().unwrap().handle_attack(
        player,
        &SAttack {
            entity_id: boat.get_entity().entity_id.into(),
        },
        server,
    );
}

fn drops(world: &World) -> Vec<ItemStack> {
    world
        .entities
        .load()
        .iter()
        .filter_map(|entity| entity.get_item_entity())
        .map(|item| item.get_item_stack().lock().unwrap().clone())
        .collect()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn boats_drop_items_after_damage_and_equipped_player_attack() {
    let (_temp, server, world) = test_world().await;
    let (player, _peer) = equipped_player(&server, &world).await;
    let direct = spawn_boat(&world, &Item::OAK_BOAT);
    assert!(damage(direct.as_ref(), 5.0));
    assert!(direct.get_entity().is_removed());
    let direct_drops = drops(&world);

    let attacked = spawn_boat(&world, &Item::BIRCH_BOAT);
    attack(&player, &attacked, &server);
    assert!(attacked.get_entity().is_removed());
    let all_drops = drops(&world);
    assert_eq!(
        (direct_drops.len(), all_drops.len()),
        (1, 2),
        "valid damage and a charged equipped-player attack must each drop one boat"
    );
    assert_eq!(direct_drops[0].item, &Item::OAK_BOAT);
    assert_eq!(all_drops[1].item, &Item::BIRCH_BOAT);
    assert!(all_drops.iter().all(|stack| stack.item_count == 1));
    assert_eq!(player.inventory().get_slot_with_stack(&all_drops[1]), -1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn boat_variants_drop_named_items_once_and_leave_minecart_drops_unchanged() {
    let (_temp, _server, world) = test_world().await;
    for (index, id) in BoatItem::ids().iter().enumerate() {
        let item = Item::from_id(*id).unwrap();
        let boat = spawn_boat(&world, item);
        let name = TextComponent::text(format!("Named {}", item.registry_key));
        boat.get_entity().set_custom_name(name.clone());
        assert!(damage(boat.as_ref(), 4.0));
        assert!(boat.get_entity().is_alive(), "exactly 40 must not destroy");
        assert_eq!(drops(&world).len(), index);
        assert!(damage(boat.as_ref(), 1.0));
        assert!(boat.get_entity().is_removed());
        let spawned = drops(&world);
        assert_eq!(spawned.len(), index + 1, "{}", item.registry_key);
        let stack = &spawned[index];
        assert_eq!(stack.item, item);
        assert_eq!(stack.item_count, 1);
        assert_eq!(
            stack.get_data_component::<CustomNameImpl>().unwrap().name,
            name
        );
        assert!(damage(boat.as_ref(), 5.0));
        assert_eq!(
            drops(&world).len(),
            index + 1,
            "no second drop after removal"
        );
    }

    let before = drops(&world).len();
    let minecart = Arc::new(MinecartEntity::new(Entity::new(
        world.clone(),
        BOAT_POS,
        &EntityType::CHEST_MINECART,
    )));
    assert!(world.spawn_entity(minecart.clone()));
    assert!(damage(minecart.as_ref(), 5.0));
    assert!(minecart.get_entity().is_removed());
    let spawned = drops(&world);
    assert_eq!(spawned.len(), before + 1);
    assert_eq!(spawned[before].item, &Item::CHEST_MINECART);
    assert_eq!(spawned[before].item_count, 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn boat_drops_respect_entity_drops_and_creative() {
    let (_temp, server, world) = test_world().await;
    let (player, _peer) = equipped_player(&server, &world).await;
    world.set_game_rule(&GameRule::EntityDrops, GameRuleValue::Bool(false));
    let disabled = spawn_boat(&world, &Item::BAMBOO_CHEST_RAFT);
    assert!(damage(disabled.as_ref(), 5.0));
    assert!(disabled.get_entity().is_removed());
    assert!(drops(&world).is_empty());

    world.set_game_rule(&GameRule::EntityDrops, GameRuleValue::Bool(true));
    player.gamemode.store(GameMode::Creative);
    let creative = spawn_boat(&world, &Item::POPLAR_CHEST_BOAT);
    attack(&player, &creative, &server);
    assert!(creative.get_entity().is_removed());
    assert!(drops(&world).is_empty());
}

struct DropEvents {
    boat: Arc<BoatEntity>,
    cancel_at: &'static str,
    order: Mutex<Vec<&'static str>>,
    item: Mutex<Option<(i32, Vector3<f64>)>>,
}

impl DropEvents {
    fn record(&self, stage: &'static str) -> bool {
        let boat = self.boat.get_entity();
        if matches!(stage, "item" | "entity") {
            assert!(boat.is_removed());
            assert!(boat.world.load().get_entity_by_id(boat.entity_id).is_none());
        } else {
            assert!(boat.is_alive());
        }
        self.order.lock().unwrap().push(stage);
        self.cancel_at == stage
    }
}

impl EventHandler<VehicleDamageEvent> for DropEvents {
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut VehicleDamageEvent,
    ) -> futures::future::BoxFuture<'a, ()> {
        Box::pin(async move {
            assert_eq!(event.vehicle_id, self.boat.get_entity().entity_id);
            assert_eq!(event.attacker_id, None);
            assert_eq!(event.damage, 5.0);
            assert_eq!(self.boat.vehicle.get_damage(), 0.0);
            event.cancelled = self.record("damage");
        })
    }
}

impl EventHandler<VehicleDestroyEvent> for DropEvents {
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut VehicleDestroyEvent,
    ) -> futures::future::BoxFuture<'a, ()> {
        Box::pin(async move {
            assert_eq!(event.vehicle_id, self.boat.get_entity().entity_id);
            assert_eq!(event.attacker_id, None);
            assert_eq!(self.boat.vehicle.get_damage(), 50.0);
            event.cancelled = self.record("destroy");
        })
    }
}

impl EventHandler<ItemSpawnEvent> for DropEvents {
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut ItemSpawnEvent,
    ) -> futures::future::BoxFuture<'a, ()> {
        Box::pin(async move {
            assert_eq!(event.item_name, Item::OAK_BOAT.registry_key);
            *self.item.lock().unwrap() = Some((event.entity_id, event.position));
            event.cancelled = self.record("item");
        })
    }
}

impl EventHandler<EntitySpawnEvent> for DropEvents {
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut EntitySpawnEvent,
    ) -> futures::future::BoxFuture<'a, ()> {
        Box::pin(async move {
            assert_eq!(event.entity_type, EntityType::ITEM.id.to_string());
            assert_eq!(
                *self.item.lock().unwrap(),
                Some((event.entity_id, event.position))
            );
            assert!(event.world.get_entity_by_id(event.entity_id).is_none());
            event.cancelled = self.record("entity");
        })
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn boat_drop_events_preserve_cancellation_payloads_and_order() {
    for (cancel_at, expected, survives) in [
        ("damage", &["damage"][..], true),
        ("destroy", &["damage", "destroy"][..], true),
        ("item", &["damage", "destroy", "item"][..], false),
        (
            "entity",
            &["damage", "destroy", "item", "entity"][..],
            false,
        ),
        ("none", &["damage", "destroy", "item", "entity"][..], false),
    ] {
        let (_temp, server, world) = test_world().await;
        let boat = spawn_boat(&world, &Item::OAK_BOAT);
        let events = Arc::new(DropEvents {
            boat: boat.clone(),
            cancel_at,
            order: Mutex::default(),
            item: Mutex::default(),
        });
        server.plugin_manager.register::<VehicleDamageEvent, _>(
            events.clone(),
            EventPriority::Normal,
            true,
        );
        server.plugin_manager.register::<VehicleDestroyEvent, _>(
            events.clone(),
            EventPriority::Normal,
            true,
        );
        server.plugin_manager.register::<ItemSpawnEvent, _>(
            events.clone(),
            EventPriority::Normal,
            true,
        );
        server.plugin_manager.register::<EntitySpawnEvent, _>(
            events.clone(),
            EventPriority::Normal,
            true,
        );
        assert_eq!(damage(boat.as_ref(), 5.0), !survives);
        assert_eq!(boat.get_entity().is_alive(), survives);
        assert_eq!(drops(&world).len(), usize::from(cancel_at == "none"));
        assert_eq!(*events.order.lock().unwrap(), expected);
        if cancel_at == "damage" {
            assert_eq!(boat.vehicle.get_damage(), 0.0);
            assert_eq!(boat.vehicle.get_hurt_time(), 0);
        }
        if !survives {
            assert!(damage(boat.as_ref(), 5.0));
            assert_eq!(drops(&world).len(), usize::from(cancel_at == "none"));
            assert_eq!(*events.order.lock().unwrap(), expected);
        }
    }
}

struct ReenterDrop {
    boat: Arc<BoatEntity>,
    on_item_spawn: bool,
    entered: AtomicBool,
}

impl ReenterDrop {
    fn damage_once(&self) {
        // Set the guard before ONE nested valid hit, including on the unfixed path.
        if !self.entered.swap(true, Ordering::Relaxed) {
            assert!(damage(self.boat.as_ref(), 5.0));
        }
    }
}

impl EventHandler<ItemSpawnEvent> for ReenterDrop {
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        _event: &'a mut ItemSpawnEvent,
    ) -> futures::future::BoxFuture<'a, ()> {
        Box::pin(async move {
            if self.on_item_spawn {
                self.damage_once();
            }
        })
    }
}

impl EventHandler<EntitySpawnEvent> for ReenterDrop {
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        _event: &'a mut EntitySpawnEvent,
    ) -> futures::future::BoxFuture<'a, ()> {
        Box::pin(async move {
            if !self.on_item_spawn {
                self.damage_once();
            }
        })
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn boat_spawn_callbacks_cannot_drop_the_same_boat_twice() {
    for on_item_spawn in [true, false] {
        let (_temp, server, world) = test_world().await;
        let boat = spawn_boat(&world, &Item::OAK_BOAT);
        let handler = Arc::new(ReenterDrop {
            boat: boat.clone(),
            on_item_spawn,
            entered: AtomicBool::new(false),
        });
        server.plugin_manager.register::<ItemSpawnEvent, _>(
            handler.clone(),
            EventPriority::Normal,
            true,
        );
        server.plugin_manager.register::<EntitySpawnEvent, _>(
            handler.clone(),
            EventPriority::Normal,
            true,
        );
        assert!(damage(boat.as_ref(), 5.0));
        assert!(handler.entered.load(Ordering::Relaxed));
        assert!(boat.get_entity().is_removed());
        assert!(
            world
                .get_entity_by_id(boat.get_entity().entity_id)
                .is_none()
        );
        assert_eq!(drops(&world).len(), 1);
    }
}
