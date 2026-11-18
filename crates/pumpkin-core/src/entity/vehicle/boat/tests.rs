use std::sync::Arc;
use std::sync::mpsc::{Sender, channel};

use futures::future::BoxFuture;
use pumpkin_config::world::EntityIslandsConfig;
use pumpkin_config::{AdvancedConfiguration, BasicConfiguration, TelemetryConfig};
use pumpkin_data::block_properties::BubbleColumnLikeProperties;
use pumpkin_data::packet::{CURRENT_MC_VERSION, clientbound::play::MOVE_VEHICLE};
use pumpkin_data::{Block, dimension::Dimension};
use pumpkin_protocol::java::{client::play::CMoveVehicle, server::play::SMoveVehicle};
use pumpkin_protocol::{ConnectionState, ServerPacket};
use pumpkin_util::math::{position::BlockPos, vector2::Vector2};
use pumpkin_world::chunk::ChunkData;

use super::*;
use crate::data::VanillaData;
use crate::net::{
    ClientPlatform, GameProfile, PacketRateLimiter, PlayerConfig,
    java::{JavaClient, pending::PendingConnection},
};
use crate::plugin::api::events::block::block_physics::BlockPhysicsEvent;
use crate::plugin::api::events::block::cauldron_level_change::CauldronLevelChangeEvent;
use crate::plugin::api::events::entity::entity_combust::EntityCombustEvent;
use crate::plugin::api::events::entity::entity_spawn::EntitySpawnEvent;
use crate::plugin::api::events::vehicle::vehicle_move::VehicleMoveEvent;
use crate::plugin::api::events::vehicle::vehicle_update::VehicleUpdateEvent;
use crate::plugin::server::packet::PacketSentEvent;
use crate::plugin::{EventHandler, EventPriority};
use crate::world::entity_islands::{EntityReach, resolve};

type TestResult = Result<(), Box<dyn std::error::Error>>;

struct NoopListener;

impl EventHandler<BlockPhysicsEvent> for NoopListener {}
impl EventHandler<EntityCombustEvent> for NoopListener {}
impl EventHandler<CauldronLevelChangeEvent> for NoopListener {}

async fn server(folder: &tempfile::TempDir) -> Result<Arc<Server>, Box<dyn std::error::Error>> {
    let basic = BasicConfiguration {
        default_level_name: folder.path().to_string_lossy().into_owned(),
        allow_nether: false,
        allow_end: false,
        ..Default::default()
    };
    let mut advanced = AdvancedConfiguration::default();
    advanced.networking.bedrock.online_mode = false;
    Ok(Server::new(
        basic,
        advanced,
        TelemetryConfig::default(),
        VanillaData::default(),
        Vec::new(),
    )
    .await?)
}

struct TeleportBoat {
    boat: Arc<BoatEntity>,
    destination: Vector3<f64>,
}

impl TeleportBoat {
    fn teleport(&self) {
        self.boat.teleport(
            self.destination,
            None,
            None,
            self.boat.get_entity().world.load_full(),
        );
    }
}

impl EventHandler<VehicleMoveEvent> for TeleportBoat {
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        _event: &'a mut VehicleMoveEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async { self.boat.get_entity().set_pos(self.destination) })
    }
}

impl EventHandler<EntitySpawnEvent> for TeleportBoat {
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        _event: &'a mut EntitySpawnEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async { self.teleport() })
    }
}

impl EventHandler<VehicleUpdateEvent> for TeleportBoat {
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        _event: &'a mut VehicleUpdateEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async { self.teleport() })
    }
}

struct RecordCorrection(Sender<CMoveVehicle>);

impl EventHandler<PacketSentEvent> for RecordCorrection {
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut PacketSentEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async {
            if event.packet_id == MOVE_VEHICLE.0
                && let Ok(packet) = CMoveVehicle::read(&mut &event.payload[..], &CURRENT_MC_VERSION)
            {
                let _ = self.0.send(packet);
            }
        })
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn vehicle_move_listener_keeps_the_teleport_and_clears_the_sweep() -> TestResult {
    let folder = tempfile::tempdir()?;
    let server = server(&folder).await?;
    let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
    let from = Vector3::new(8.0, 64.0, 8.0);
    let destination = Vector3::new(24.0, 68.0, 24.0);
    let boat = Arc::new(BoatEntity::new(Entity::new(
        world.clone(),
        from,
        &EntityType::OAK_BOAT,
    )));
    let entity = boat.get_entity();
    let motion = Vector3::new(0.25, -0.1, 0.0);
    entity.velocity.store(motion);
    boat.lock_movements().record(from - motion, from, motion);
    server.plugin_manager.register::<VehicleMoveEvent, _>(
        Arc::new(TeleportBoat {
            boat: boat.clone(),
            destination,
        }),
        EventPriority::Normal,
        true,
    );

    let snapshot = boat.movement_snapshot(&boat.lock_movements());
    boat.move_boat(boat.as_ref(), Vector3::new(0.0, 0.5, 0.0), &snapshot);

    assert_eq!(entity.pos.load(), destination);
    assert_eq!(entity.last_pos.load(), destination);
    assert_eq!(entity.velocity.load(), motion);
    assert_eq!(boat.pending_block_movement_reach(), 0.0);

    let profile = GameProfile {
        id: uuid::Uuid::new_v4(),
        name: "BoatDriver".to_owned(),
        properties: arc_swap::ArcSwap::from_pointee(Vec::new()),
        profile_actions: None,
    };
    let pending = PendingConnection::from_transport(
        Box::new(tokio::io::empty()),
        Box::new(tokio::io::sink()),
        ([127, 0, 0, 1], 25565).into(),
        1,
        PacketRateLimiter::from_config(&server.advanced_config.networking.java.packet_limiter),
        Arc::downgrade(&server),
    );
    pending.connection_state.store(ConnectionState::Play);
    let client = Arc::new(ClientPlatform::Java(JavaClient::from_pending(
        pending,
        profile.clone(),
        PlayerConfig::default(),
    )));
    let player = Arc::new(Player::new(
        client.clone(),
        profile,
        PlayerConfig::default(),
        &world,
        GameMode::Creative,
    ));
    let ClientPlatform::Java(java) = client.as_ref() else {
        return Err("Expected a Java test client".into());
    };
    player.set_client_loaded(true);
    java.player.store(Arc::new(Some(player.clone())));
    entity.set_pos(from);
    entity.add_passenger(boat.clone(), player.clone());
    assert!(boat.is_client_controlled());
    let (sender, receiver) = channel();
    server.plugin_manager.register::<PacketSentEvent, _>(
        Arc::new(RecordCorrection(sender)),
        EventPriority::Normal,
        true,
    );

    let packet = SMoveVehicle {
        x: from.x + 0.5,
        y: from.y,
        z: from.z,
        yaw: 90.0,
        pitch: 0.0,
        on_ground: false,
    };
    let other_folder = tempfile::tempdir()?;
    let other_world = crate::entity::ai::test_support::world(&other_folder);
    player.get_entity().set_world(other_world.clone());
    java.handle_move_vehicle(&player, &packet);
    assert_eq!(entity.pos.load(), from);
    player.get_entity().set_world(world);
    other_world.level.shutdown().await;

    java.handle_move_vehicle(&player, &packet);
    java.player.store(Arc::new(None));

    assert_eq!(entity.pos.load(), destination);
    assert_eq!(boat.pending_block_movement_reach(), 0.0);
    let correction = receiver.try_recv()?;
    assert_eq!(
        Vector3::new(correction.x, correction.y, correction.z),
        destination
    );
    server.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn lily_pad_spawn_listener_serializes_the_boat_and_can_end_the_sweep() -> TestResult {
    let folder = tempfile::tempdir()?;
    let server = server(&folder).await?;
    let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
    let chunk = ChunkData::empty_sync(0, 0);
    world
        .level
        .loaded_chunks
        .insert(Vector2::new(0, 0), chunk.clone());
    let from = Vector3::new(6.5, 64.0, 8.5);
    let to = Vector3::new(9.5, 64.0, 8.5);
    let destination = Vector3::new(24.0, 68.0, 24.0);
    let boat = Arc::new(BoatEntity::new(Entity::new(
        world.clone(),
        to,
        &EntityType::OAK_BOAT,
    )));
    boat.lock_movements().record(from, to, to - from);
    let config = EntityIslandsConfig {
        unaudited_reach: 0.0,
        ..Default::default()
    };
    let plan = || resolve(vec![EntityReach::of(boat.as_ref(), &config)], &config);
    server.plugin_manager.register::<BlockPhysicsEvent, _>(
        Arc::new(NoopListener),
        EventPriority::Normal,
        true,
    );
    assert!(plan().is_some());
    for x in [6, 9] {
        chunk.set_block_absolute_y(x, 63, 8, Block::WATER.default_state.id);
        chunk.set_block_absolute_y(x, 64, 8, Block::LILY_PAD.default_state.id);
    }
    assert!(plan().is_none());
    server.plugin_manager.register::<EntitySpawnEvent, _>(
        Arc::new(TeleportBoat {
            boat: boat.clone(),
            destination,
        }),
        EventPriority::Normal,
        true,
    );
    assert!(plan().is_none());

    boat.apply_effects_from_blocks();

    assert_eq!(boat.get_entity().pos.load(), destination);
    assert_eq!(world.get_block(&BlockPos::new(6, 64, 8)), &Block::AIR);
    assert_eq!(world.get_block(&BlockPos::new(9, 64, 8)), &Block::LILY_PAD);
    server.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bubble_surface_contact_launches_after_sixty_ticks_and_ejects_on_sink() -> TestResult {
    let folder = tempfile::tempdir()?;
    let server = server(&folder).await?;
    let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
    let chunk = ChunkData::empty_sync(0, 0);
    let column = BlockPos::new(8, 64, 8);
    let upward = BubbleColumnLikeProperties { r#drag: false }.to_state_id(&Block::BUBBLE_COLUMN);
    let downward = BubbleColumnLikeProperties { r#drag: true }.to_state_id(&Block::BUBBLE_COLUMN);
    chunk.set_block_absolute_y(8, 64, 8, upward);
    world
        .level
        .loaded_chunks
        .insert(Vector2::new(0, 0), chunk.clone());
    let from = Vector3::new(6.5, 64.5, 8.5);
    let to = Vector3::new(10.5, 64.5, 8.5);
    let boat = Arc::new(BoatEntity::new(Entity::new(
        world.clone(),
        to,
        &EntityType::OAK_BOAT,
    )));
    let tick_contacts = || {
        boat.apply_effects_from_blocks();
        boat.tick_bubble_column();
    };

    boat.lock_movements().record(from, to, to - from);
    tick_contacts();
    assert_eq!(boat.bubble_time.load(), 59);
    assert_eq!(boat.get_entity().velocity.load(), Vector3::default());
    tick_contacts();
    assert_eq!(
        boat.bubble_time.load(),
        0,
        "leaving the column clears the timer"
    );

    let center = column.0.to_f64() + Vector3::new(0.5, 0.5, 0.5);
    boat.teleport(center, None, None, world.clone());
    let rider =
        crate::entity::r#type::from_type(&EntityType::PIG, center, &world, uuid::Uuid::new_v4());
    boat.get_entity().add_passenger(boat.clone(), rider.clone());
    assert!(rider.get_entity().has_vehicle());
    for _ in 0..59 {
        tick_contacts();
    }
    assert_eq!(boat.get_entity().velocity.load(), Vector3::default());
    tick_contacts();
    assert_eq!(
        boat.get_entity().velocity.load(),
        Vector3::new(0.0, 0.6, 0.0)
    );
    assert!(
        rider.get_entity().has_vehicle(),
        "an upward column keeps its rider"
    );

    chunk.set_block_absolute_y(8, 64, 8, downward);
    for _ in 0..59 {
        tick_contacts();
    }
    assert!(rider.get_entity().has_vehicle());
    tick_contacts();
    assert!(!rider.get_entity().has_vehicle());
    assert!(boat.get_entity().velocity.load().y < 0.0);
    server.shutdown().await;
    Ok(())
}

#[tokio::test]
async fn teleport_rejects_a_previously_validated_client_move() -> TestResult {
    let folder = tempfile::tempdir()?;
    let world = crate::entity::ai::test_support::world(&folder);
    let from = Vector3::new(8.0, 64.0, 8.0);
    let boat = BoatEntity::new(Entity::new(world.clone(), from, &EntityType::OAK_BOAT));

    let snapshot = boat.movement_snapshot(&boat.lock_movements());
    let target = from + Vector3::new(0.5, 0.0, 0.0);
    assert!(boat.commit_client_movement(&snapshot, target, 0.0, 0.0, true));
    assert!(boat.pending_block_movement_reach() > 0.0);

    let stale = boat.movement_snapshot(&boat.lock_movements());
    // A teleport can invalidate a packet even if its starting coordinates are unchanged.
    boat.teleport(target, Some(45.0), None, world.clone());
    assert!(!boat.commit_client_movement(&stale, from, 90.0, 0.0, false));
    assert_eq!(boat.get_entity().pos.load(), target);
    assert_eq!(boat.get_entity().last_pos.load(), target);
    assert_eq!(boat.get_entity().yaw.load(), 45.0);
    assert!(boat.get_entity().on_ground.load(Ordering::Relaxed));
    assert_eq!(boat.pending_block_movement_reach(), 0.0);
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cauldron_listener_serializes_a_boat_that_can_change_its_level() -> TestResult {
    let folder = tempfile::tempdir()?;
    let server = server(&folder).await?;
    let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
    let chunk = ChunkData::empty_sync(0, 0);
    world
        .level
        .loaded_chunks
        .insert(Vector2::new(0, 0), chunk.clone());
    let pos = BlockPos::new(8, 64, 8);
    let boat = BoatEntity::new(Entity::new(
        world.clone(),
        Vector3::new(8.5, 64.35, 8.5),
        &EntityType::OAK_BOAT,
    ));
    let config = EntityIslandsConfig {
        unaudited_reach: 0.0,
        ..Default::default()
    };
    let can_plan = || resolve(vec![EntityReach::of(&boat, &config)], &config).is_some();
    let old_state = Block::WATER_CAULDRON.default_state.id;
    chunk.set_block_absolute_y(8, 64, 8, old_state);
    assert!(can_plan());
    server
        .plugin_manager
        .register::<CauldronLevelChangeEvent, _>(
            Arc::new(NoopListener),
            EventPriority::Normal,
            true,
        );
    assert!(!can_plan());

    let entity = boat.get_entity();
    entity.set_has_no_gravity(true);
    entity.fire_ticks.store(100, Ordering::Relaxed);
    boat.tick(&boat, &server);
    assert_ne!(world.get_block_state_id(&pos), old_state);
    server.shutdown().await;
    Ok(())
}

#[tokio::test]
async fn combustion_listener_serializes_a_boat_in_lava() -> TestResult {
    let folder = tempfile::tempdir()?;
    let server = server(&folder).await?;
    let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
    let chunk = ChunkData::empty_sync(0, 0);
    chunk.set_block_absolute_y(8, 64, 8, Block::LAVA.default_state.id);
    world.level.loaded_chunks.insert(Vector2::new(0, 0), chunk);
    let boat = BoatEntity::new(Entity::new(
        world,
        Vector3::new(8.5, 64.0, 8.5),
        &EntityType::OAK_BOAT,
    ));
    let config = EntityIslandsConfig {
        unaudited_reach: 0.0,
        ..Default::default()
    };
    let plan = || resolve(vec![EntityReach::of(&boat, &config)], &config);
    assert!(plan().is_some());
    server.plugin_manager.register::<EntityCombustEvent, _>(
        Arc::new(NoopListener),
        EventPriority::Normal,
        true,
    );
    assert!(plan().is_none());
    server.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn teleport_during_vehicle_update_discards_the_old_water_transition() -> TestResult {
    let folder = tempfile::tempdir()?;
    let server = server(&folder).await?;
    let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
    let chunk = ChunkData::empty_sync(0, 0);
    chunk.set_block_absolute_y(8, 64, 8, Block::WATER.default_state.id);
    world.level.loaded_chunks.insert(Vector2::new(0, 0), chunk);
    let boat = Arc::new(BoatEntity::new(Entity::new(
        world,
        Vector3::new(8.5, 64.0, 8.5),
        &EntityType::OAK_BOAT,
    )));
    boat.status.store(Some(BoatStatus::InAir));
    assert!(boat.get_status().is_underwater());
    let destination = Vector3::new(8.5, 70.0, 8.5);
    server.plugin_manager.register::<VehicleUpdateEvent, _>(
        Arc::new(TeleportBoat {
            boat: boat.clone(),
            destination,
        }),
        EventPriority::Normal,
        true,
    );

    boat.tick(boat.as_ref(), &server);

    assert_eq!(boat.get_entity().pos.load(), destination);
    assert_eq!(boat.pending_block_movement_reach(), 0.0);
    server.shutdown().await;
    Ok(())
}
