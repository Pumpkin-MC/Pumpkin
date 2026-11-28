use std::sync::{Arc, Mutex, PoisonError, atomic::AtomicBool, atomic::Ordering};

use pumpkin_data::{
    Block,
    entity::{EntityPose, EntityType},
    game_event::GameEvent,
    packet::CURRENT_MC_VERSION,
    sound::Sound,
};
use pumpkin_protocol::{
    java::client::play::CSoundEffect, packet::MultiVersionJavaPacket, ser::NetworkReadExt,
};
use pumpkin_util::math::{vector2::Vector2, vector3::Vector3};
use pumpkin_world::chunk::ChunkData;

use crate::{
    entity::{Entity, EntityBase, passive::pig::PigEntity, vehicle::boat::BoatEntity},
    plugin::{
        BoxFuture, EventPriority, EventRwHandler,
        api::events::{
            player::player_interact_event::PlayerInteractEvent,
            vehicle::{vehicle_exit::VehicleExitEvent, vehicle_move::VehicleMoveEvent},
            world::generic_game::GenericGameEvent,
        },
        server::packet::PacketSentEvent,
    },
    server::Server,
};

use super::{
    World,
    vehicle_tests::{
        CENTER, TestResult, assert_mounted, bedrock_links, bedrock_rider, java_rider,
        mounted_java_boat, queue_vehicle_position, vehicle_world,
    },
};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn duplicate_vehicle_uuid_does_not_commit_placement() -> TestResult {
    let folder = tempfile::tempdir()?;
    let (_, world) = vehicle_world(&folder).await?;
    let boat: Arc<dyn EntityBase> = Arc::new(BoatEntity::new(Entity::new(
        world.clone(),
        CENTER,
        &EntityType::OAK_BOAT,
    )));
    let uuid = boat.get_entity().entity_uuid;
    let duplicate: Arc<dyn EntityBase> = Arc::new(BoatEntity::new(Entity::from_uuid(
        uuid,
        world.clone(),
        CENTER,
        &EntityType::OAK_BOAT,
    )));
    assert_ne!(
        boat.get_entity().entity_id,
        duplicate.get_entity().entity_id
    );
    assert!(
        world
            .entity_registrations
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(uuid, boat.get_entity().entity_id)
            .is_none()
    );
    assert!(
        world
            .get_entity_by_id(boat.get_entity().entity_id)
            .is_none()
    );
    let mut debited = false;
    let mut debit = || {
        debited = true;
        Some(())
    };
    assert!(matches!(
        world.spawn_placed_vehicle(&duplicate, &mut debit),
        Err(super::VehiclePlacementError::UuidUnavailable)
    ));
    assert!(
        world
            .entity_registrations
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .remove(&uuid)
            .is_some()
    );
    world.add_entity_silent(boat.clone());
    let id = boat.get_entity().entity_id;
    assert!(world.get_entity_by_id(id).is_some());

    let placed = world.spawn_placed_vehicle(&duplicate, debit);
    assert!(matches!(
        placed,
        Err(super::VehiclePlacementError::UuidUnavailable)
    ));
    assert!(!debited);
    assert!(
        world
            .get_entity_by_id(id)
            .is_some_and(|live| Arc::ptr_eq(&live, &boat))
    );
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn autonomous_boat_rebounds_from_slime_after_a_partial_clip() -> TestResult {
    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let chunk = ChunkData::empty_sync(0, 0);
    for x in 5..=10 {
        for z in 5..=10 {
            let block = if x <= 7 {
                &Block::SLIME_BLOCK
            } else {
                &Block::STONE
            };
            chunk.set_block_absolute_y(x, 63, z, block.default_state.id);
        }
    }
    world.level.loaded_chunks.insert(Vector2::new(0, 0), chunk);
    for (x, y, initial_y, rebound) in [
        (6.5, 64.0, 0.0, 0.0),
        (6.5, 64.125, -0.2, 0.208_666_669_776_042_3),
        (9.5, 64.125, -0.2, 0.0),
    ] {
        let position = Vector3::new(x, y, 8.5);
        let boat = BoatEntity::new(Entity::new(world.clone(), position, &EntityType::OAK_BOAT));
        let entity = boat.get_entity();
        entity.velocity.store(Vector3::new(0.0, initial_y, 0.0));

        boat.tick(&boat, &server);

        assert_eq!(entity.pos.load(), Vector3::new(x, 64.0, 8.5));
        assert!((entity.velocity.load().y - rebound).abs() < 1.0e-12);
        assert!(entity.on_ground.load(Ordering::Relaxed));
    }
    // Apply gravity before the shared movement call and use its legacy restitution path.
    let control = Entity::new(
        world.clone(),
        Vector3::new(6.5, 64.125, 8.5),
        &EntityType::OAK_BOAT,
    );
    let impact = Vector3::new(0.0, -0.2 - 0.04, 0.0);
    control.velocity.store(impact);
    assert!(control.move_entity(&control, impact));
    assert_eq!(control.pos.load(), Vector3::new(6.5, 64.0, 8.5));
    assert!((control.velocity.load().y - 0.192).abs() < 1.0e-12);
    world.level.shutdown().await;
    Ok(())
}

#[derive(Debug, PartialEq)]
enum BounceObservation {
    Sound(Vector3<i32>, Vector3<f64>),
    Event(Vector3<f64>),
}

struct BounceObservations {
    boat: Arc<BoatEntity>,
    trace: Mutex<Vec<BounceObservation>>,
    teleport: AtomicBool,
}

impl EventRwHandler<GenericGameEvent> for BounceObservations {
    fn handle<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut GenericGameEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async {
            if event.event_key == GameEvent::Bounce.name() {
                let boat = self.boat.get_entity();
                self.trace
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .push(BounceObservation::Event(boat.velocity.load()));
                if self.teleport.load(Ordering::Relaxed) {
                    boat.velocity.store(Vector3::new(0.0, 0.125, 0.0));
                    self.boat
                        .teleport(boat.pos.load(), None, None, boat.world.load_full());
                }
            }
        })
    }
}

impl EventRwHandler<PacketSentEvent> for BounceObservations {
    fn handle<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut PacketSentEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async {
            let mut payload = event.payload.as_ref();
            if event.packet_id == CSoundEffect::to_id(CURRENT_MC_VERSION)
                && payload
                    .get_var_int()
                    .is_ok_and(|id| id.0 == Sound::BlockShelfMushroomBounce as i32 + 1)
                && payload.get_var_int().is_ok()
                && let (Ok(x), Ok(y), Ok(z)) = (
                    payload.get_i32_be(),
                    payload.get_i32_be(),
                    payload.get_i32_be(),
                )
            {
                let velocity = self.boat.get_entity().velocity.load();
                self.trace
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .push(BounceObservation::Sound(Vector3::new(x, y, z), velocity));
            }
        })
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn boat_bounce_fires_the_shelf_mushroom_sound_and_game_event_before_velocity() -> TestResult {
    use pumpkin_util::math::position::BlockPos;

    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let chunk = ChunkData::empty_sync(0, 0);
    let shelf = Block::SHELF_MUSHROOM.default_state;
    chunk.set_block_absolute_y(8, 63, 8, shelf.id);
    chunk.set_block_absolute_y(9, 64, 8, Block::STONE.default_state.id);
    world.level.loaded_chunks.insert(Vector2::new(0, 0), chunk);
    let surface = 63.0
        + shelf
            .get_block_collision_shapes_at(&BlockPos::new(8, 63, 8))
            .map(|shape| shape.max.y)
            .reduce(f64::max)
            .ok_or("Shelf mushroom has no top collision surface")?;
    let (watcher, _peer) = java_rider(&server, &world, "Notch")?;
    watcher.get_entity().set_pos(CENTER.add_raw(3.0, 0.0, 0.0));
    let client = watcher.client.java().ok_or("Missing Java watcher")?;
    client.player.store(Arc::new(Some(watcher.clone())));
    let start = Vector3::new(8.25, surface + 0.125, 8.5);
    let boat = Arc::new(BoatEntity::new(Entity::new(
        world.clone(),
        start,
        &EntityType::OAK_BOAT,
    )));
    let observed = Arc::new(BounceObservations {
        boat: boat.clone(),
        trace: Mutex::default(),
        teleport: AtomicBool::new(false),
    });
    server
        .plugin_manager
        .register_blocking::<GenericGameEvent, _>(observed.clone(), EventPriority::Normal, None);
    server
        .plugin_manager
        .register_blocking::<PacketSentEvent, _>(observed.clone(), EventPriority::Normal, None);
    let entity = boat.get_entity();
    let air_friction = f64::from(0.9f32);
    let impact = Vector3::new(
        0.2 * air_friction,
        -0.2 - boat.get_gravity(),
        0.1 * air_friction,
    );
    for (teleport, expected_rebound) in [(false, 0.156_500_006_218_751_3), (true, 0.125)] {
        observed.teleport.store(teleport, Ordering::Relaxed);
        entity.set_pos(start);
        entity.velocity.store(Vector3::new(0.2, -0.2, 0.1));
        entity.sync_position.store(false, Ordering::Relaxed);

        boat.tick(boat.as_ref(), &server);

        assert_eq!(entity.pos.load().y, surface);
        assert!(entity.horizontal_collision.load(Ordering::Relaxed));
        assert_eq!(entity.velocity.load().x, 0.0);
        assert_eq!(entity.sync_position.load(Ordering::Relaxed), !teleport);
        let rebound = entity.velocity.load().y;
        assert!((rebound - expected_rebound).abs() < 1.0e-9, "{rebound}");
        let trace = std::mem::take(
            &mut *observed
                .trace
                .lock()
                .unwrap_or_else(PoisonError::into_inner),
        );
        assert_eq!(
            trace,
            [
                BounceObservation::Sound(Vector3::new(68, 508, 68), impact),
                BounceObservation::Event(impact),
            ],
        );
    }
    client.player.store(Arc::new(None));
    client.close();
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn boat_surface_column_launches_and_ejects_on_tick_sixty() -> TestResult {
    use crate::world::entity_islands::{EntityReach, resolve};
    use pumpkin_data::block_properties::BubbleColumnLikeProperties;

    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let chunk = ChunkData::empty_sync(0, 0);
    for x in 7..=9 {
        for z in 7..=9 {
            chunk.set_block_absolute_y(x, 63, z, Block::STONE.default_state.id);
            chunk.set_block_absolute_y(x, 64, z, Block::WATER.default_state.id);
        }
    }
    world
        .level
        .loaded_chunks
        .insert(Vector2::new(0, 0), chunk.clone());
    let start = Vector3::new(8.5, 64.45, 8.5);
    for drag_down in [false, true] {
        let column =
            BubbleColumnLikeProperties { r#drag: drag_down }.to_state_id(&Block::BUBBLE_COLUMN);
        chunk.set_block_absolute_y(8, 64, 8, column);
        let boat = Arc::new(BoatEntity::new(Entity::new(
            world.clone(),
            start,
            &EntityType::OAK_BOAT,
        )));
        let rider = PigEntity::new(Entity::new(world.clone(), start, &EntityType::PIG));
        let entity = boat.get_entity();
        entity.add_passenger(boat.clone(), rider.clone());
        assert!(rider.get_entity().has_vehicle());
        for _ in 0..59 {
            boat.tick(boat.as_ref(), &server);
            assert!(entity.velocity.load().y.abs() < 0.2);
            assert!(rider.get_entity().has_vehicle());
        }
        let config = pumpkin_config::world::EntityIslandsConfig {
            unaudited_reach: 0.0,
            knockback_reach: 0.0,
        };
        assert!(resolve(vec![EntityReach::of(boat.as_ref(), &config)], &config).is_none());
        boat.tick(boat.as_ref(), &server);
        if drag_down {
            assert!(entity.velocity.load().y < -0.65);
            assert!(!rider.get_entity().has_vehicle());
            assert!(!entity.has_passengers());
        } else {
            assert_eq!(entity.velocity.load(), Vector3::new(0.0, 0.6, 0.0));
            assert!(rider.get_entity().has_vehicle());
        }
    }
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[expect(
    clippy::too_many_lines,
    reason = "Exercise both launch outcomes with the real tracking and packet observer in one fixture"
)]
async fn boat_launch_status_precedes_the_impulse_and_a_newer_teleport_wins() -> TestResult {
    use crate::plugin::server::packet::PacketSentEvent;
    use pumpkin_data::{
        block_properties::BubbleColumnLikeProperties, entity::EntityStatus,
        packet::CURRENT_MC_VERSION,
    };
    use pumpkin_protocol::{java::client::play::CEntityStatus, packet::MultiVersionJavaPacket};
    use pumpkin_world::cylindrical_chunk_iterator::Cylindrical;
    use std::sync::Mutex;

    struct ObserveLaunch {
        boat: Arc<BoatEntity>,
        teleport: bool,
        observed: Mutex<Vec<f64>>,
    }
    impl EventRwHandler<PacketSentEvent> for ObserveLaunch {
        fn handle<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut PacketSentEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async {
                let entity = self.boat.get_entity();
                if event.packet_id != CEntityStatus::to_id(CURRENT_MC_VERSION)
                    || event
                        .payload
                        .get(..4)
                        .is_none_or(|id| id != entity.entity_id.to_be_bytes())
                    || event.payload.get(4) != Some(&(EntityStatus::BoatLaunch as u8))
                {
                    return;
                }
                self.observed
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .push(entity.velocity.load().y);
                if self.teleport {
                    self.boat.teleport(
                        entity.pos.load(),
                        Some(90.0),
                        None,
                        entity.world.load_full(),
                    );
                }
            })
        }
    }

    for teleport in [false, true] {
        let folder = tempfile::tempdir()?;
        let (server, world) = vehicle_world(&folder).await?;
        let chunk_pos = Vector2::new(0, 0);
        let chunk = ChunkData::empty_sync(0, 0);
        for x in 7..=9 {
            for z in 7..=9 {
                chunk.set_block_absolute_y(x, 63, z, Block::STONE.default_state.id);
                chunk.set_block_absolute_y(x, 64, z, Block::WATER.default_state.id);
            }
        }
        let column =
            BubbleColumnLikeProperties { r#drag: false }.to_state_id(&Block::BUBBLE_COLUMN);
        chunk.set_block_absolute_y(8, 64, 8, column);
        world.level.loaded_chunks.insert(chunk_pos, chunk);
        let (watcher, _peer) = java_rider(&server, &world, "Notch")?;
        let client = watcher.client.java().ok_or("Missing Java watcher")?;
        client.player.store(Arc::new(Some(watcher.clone())));
        watcher.get_entity().set_pos(CENTER.add_raw(3.0, 0.0, 0.0));
        watcher.watched_section.store(Cylindrical::new(
            chunk_pos,
            std::num::NonZeroU8::new(4).ok_or("Zero view distance")?,
        ));
        watcher
            .chunk_sender
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .mark_sent_out_of_band(chunk_pos);
        let boat = Arc::new(BoatEntity::new(Entity::new(
            world.clone(),
            CENTER.add_raw(0.0, 0.45, 0.0),
            &EntityType::OAK_BOAT,
        )));
        world.spawn_entity(boat.clone());
        let observed = Arc::new(ObserveLaunch {
            boat: boat.clone(),
            teleport,
            observed: Mutex::default(),
        });
        server
            .plugin_manager
            .register_blocking::<PacketSentEvent, _>(observed.clone(), EventPriority::Normal, None);
        for _ in 0..60 {
            boat.tick(boat.as_ref(), &server);
        }
        let observed_velocity = {
            let velocities = observed
                .observed
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            assert_eq!(velocities.len(), 1);
            velocities[0]
        };
        assert!(observed_velocity.abs() < 0.2);
        if teleport {
            assert_eq!(boat.get_entity().yaw.load(), 90.0);
            assert!(boat.get_entity().velocity.load().y.abs() < 0.2);
        } else {
            assert_eq!(
                boat.get_entity().velocity.load().y.to_bits(),
                0.6f64.to_bits()
            );
        }
        client.player.store(Arc::new(None));
        client.close();
        world.level.shutdown().await;
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn java_rider_receives_the_surface_column_event_on_tick_sixty() -> TestResult {
    use pumpkin_data::block_properties::BubbleColumnLikeProperties;
    use pumpkin_protocol::java::client::play::CKeepAlive;
    use pumpkin_world::cylindrical_chunk_iterator::Cylindrical;

    for drag_down in [false, true] {
        let folder = tempfile::tempdir()?;
        let (server, world) = vehicle_world(&folder).await?;
        let chunk_pos = Vector2::new(0, 0);
        let chunk = ChunkData::empty_sync(0, 0);
        for x in 7..=9 {
            for z in 7..=9 {
                chunk.set_block_absolute_y(x, 63, z, Block::STONE.default_state.id);
                chunk.set_block_absolute_y(x, 64, z, Block::WATER.default_state.id);
            }
        }
        let column =
            BubbleColumnLikeProperties { r#drag: drag_down }.to_state_id(&Block::BUBBLE_COLUMN);
        chunk.set_block_absolute_y(8, 64, 8, column);
        world.level.loaded_chunks.insert(chunk_pos, chunk);

        let (rider, peer) = java_rider(&server, &world, "Notch")?;
        let client = rider.client.java().ok_or("Missing Java client")?;
        let start = CENTER.add_raw(0.0, 0.45, 0.0);
        rider.get_entity().set_pos(start);
        rider.watched_section.store(Cylindrical::new(
            chunk_pos,
            std::num::NonZeroU8::new(4).ok_or("Zero view distance")?,
        ));
        rider
            .chunk_sender
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .mark_sent_out_of_band(chunk_pos);
        let boat = mounted_java_boat(&world, &rider, start);
        let entity = boat.get_entity();
        let tracked = world
            .entity_tracker
            .get_tracked_entity(entity.entity_id)
            .ok_or("Boat is not tracked")?;
        assert!(tracked.is_seen_by(&rider.gameprofile.id));
        assert_eq!(
            boat.get_controlling_passenger()
                .map(|passenger| passenger.get_entity().entity_id),
            Some(rider.entity_id())
        );

        for _ in 0..59 {
            boat.tick(boat.as_ref(), &server);
            assert!(rider.get_entity().has_vehicle());
            assert_eq!(entity.pos.load(), start);
            assert_eq!(entity.velocity.load(), Vector3::default());
        }
        assert!(client.try_send_packet(&CKeepAlive::new(59)));
        boat.tick(boat.as_ref(), &server);
        if drag_down {
            assert!(!rider.get_entity().has_vehicle());
            assert!(!entity.has_passengers());
            assert_eq!(entity.velocity.load(), Vector3::new(0.0, -0.7, 0.0));
        } else {
            assert_mounted(entity, rider.get_entity());
            assert_eq!(entity.velocity.load(), Vector3::default());
        }
        assert!(client.try_send_packet(&CKeepAlive::new(60)));
        boat.tick(boat.as_ref(), &server);
        assert!(client.try_send_packet(&CKeepAlive::new(61)));

        assert_surface_column_packets(peer, entity.entity_id, drag_down).await?;
        client.close();
        world.level.shutdown().await;
    }
    Ok(())
}

async fn assert_surface_column_packets(
    peer: tokio::io::DuplexStream,
    entity_id: i32,
    drag_down: bool,
) -> TestResult {
    use pumpkin_data::{entity::EntityStatus, packet::CURRENT_MC_VERSION};
    use pumpkin_protocol::{
        java::{
            client::play::{CEntityStatus, CKeepAlive, CSetPassengers},
            packet_decoder::TCPNetworkDecoder,
        },
        packet::MultiVersionJavaPacket,
        ser::NetworkReadExt,
    };

    #[derive(Debug, PartialEq, Eq)]
    enum Observation {
        Detached,
        Status(i8),
    }

    let mut decoder = TCPNetworkDecoder::new(tokio::io::BufReader::new(peer));
    for tick in [59, 60, 61] {
        let observed = tokio::time::timeout(std::time::Duration::from_secs(5), async {
            let mut observed = Vec::new();
            loop {
                let packet = decoder.get_raw_packet().await?;
                let mut payload = packet.payload.as_ref();
                if packet.id == CKeepAlive::to_id(CURRENT_MC_VERSION) {
                    assert_eq!(payload.get_i64_be()?, tick);
                    return Ok::<_, Box<dyn std::error::Error>>(observed);
                }
                if packet.id == CSetPassengers::to_id(CURRENT_MC_VERSION)
                    && payload.get_var_int()?.0 == entity_id
                    && payload.get_var_int()?.0 == 0
                {
                    observed.push(Observation::Detached);
                }
                if packet.id == CEntityStatus::to_id(CURRENT_MC_VERSION)
                    && payload.get_i32_be()? == entity_id
                {
                    observed.push(Observation::Status(payload.get_i8()?));
                }
            }
        })
        .await??;
        if tick != 60 {
            assert!(observed.is_empty(), "unexpected boat event at tick {tick}");
        } else if drag_down {
            assert_eq!(
                observed,
                [
                    Observation::Detached,
                    Observation::Status(EntityStatus::BoatSink as i8),
                ]
            );
        } else {
            assert_eq!(
                observed,
                [Observation::Status(EntityStatus::BoatLaunch as i8)]
            );
        }
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn boat_ejects_after_sixty_consecutive_submerged_ticks() -> TestResult {
    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let chunk = ChunkData::empty_sync(0, 0);
    for x in 7..=9 {
        for z in 7..=9 {
            chunk.set_block_absolute_y(x, 63, z, Block::STONE.default_state.id);
            for y in 64..=65 {
                chunk.set_block_absolute_y(x, y, z, Block::WATER.default_state.id);
            }
        }
    }
    world.level.loaded_chunks.insert(Vector2::new(0, 0), chunk);
    let start = Vector3::new(8.5, 64.1, 8.5);
    let boat = Arc::new(BoatEntity::new(Entity::new(
        world.clone(),
        start,
        &EntityType::OAK_BOAT,
    )));
    let rider = PigEntity::new(Entity::new(world.clone(), start, &EntityType::PIG));
    let entity = boat.get_entity();
    entity.add_passenger(boat.clone(), rider.clone());
    for _ in 0..59 {
        boat.tick(boat.as_ref(), &server);
        assert!(rider.get_entity().has_vehicle());
    }
    assert!(entity.touching_water.load(Ordering::Relaxed));
    boat.tick(boat.as_ref(), &server);
    assert!(!rider.get_entity().has_vehicle());
    assert!(!entity.has_passengers());
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn vehicle_move_listener_can_replace_boat_movement_with_a_same_position_teleport()
-> TestResult {
    use crate::plugin::api::events::vehicle::vehicle_move::VehicleMoveEvent;

    struct TeleportBoat(Arc<BoatEntity>, AtomicBool);

    impl EventRwHandler<VehicleMoveEvent> for TeleportBoat {
        fn handle<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut VehicleMoveEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async {
                let entity = self.0.get_entity();
                if event.vehicle_id == entity.entity_id && !self.1.swap(true, Ordering::Relaxed) {
                    self.0.teleport(
                        entity.pos.load(),
                        Some(90.0),
                        None,
                        entity.world.load_full(),
                    );
                }
            })
        }
    }

    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let boat = Arc::new(BoatEntity::new(Entity::new(
        world.clone(),
        CENTER,
        &EntityType::OAK_BOAT,
    )));
    let entity = boat.get_entity();
    let movement = Vector3::new(0.25, 0.0, 0.0);
    entity.record_vehicle_movement(CENTER - movement, movement, movement);
    assert!(entity.vehicle_block_effect_reach() > 0.0);
    let listener = Arc::new(TeleportBoat(boat.clone(), AtomicBool::new(false)));
    server
        .plugin_manager
        .register_blocking::<VehicleMoveEvent, _>(listener.clone(), EventPriority::Normal, None);

    boat.tick(boat.as_ref(), &server);

    assert!(listener.1.load(Ordering::Relaxed));
    assert_eq!(entity.pos.load(), CENTER);
    assert_eq!(entity.yaw.load(), 90.0);
    assert_eq!(entity.vehicle_block_effect_reach(), 0.0);
    boat.tick(boat.as_ref(), &server);
    assert!(entity.pos.load().y < CENTER.y);
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn vehicle_enter_relocation_discards_the_old_boat_pickup_contacts() -> TestResult {
    use crate::plugin::api::events::vehicle::vehicle_enter::VehicleEnterEvent;
    use std::sync::atomic::AtomicUsize;

    struct RelocateOnEnter(Arc<BoatEntity>, AtomicUsize);
    impl EventRwHandler<VehicleEnterEvent> for RelocateOnEnter {
        fn handle<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut VehicleEnterEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async {
                let entity = self.0.get_entity();
                if event.vehicle_id == entity.entity_id
                    && self.1.fetch_add(1, Ordering::Relaxed) == 0
                {
                    self.0.teleport(
                        CENTER.add_raw(4.0, 0.0, 0.0),
                        None,
                        None,
                        entity.world.load_full(),
                    );
                }
            })
        }
    }

    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let chunk = Vector2::new(0, 0);
    world
        .level
        .loaded_chunks
        .insert(chunk, ChunkData::empty_sync(0, 0));
    *world
        .active_chunks
        .write()
        .unwrap_or_else(PoisonError::into_inner) = Arc::new(std::iter::once(chunk).collect());
    let boat = Arc::new(BoatEntity::new(Entity::new(
        world.clone(),
        CENTER,
        &EntityType::OAK_BOAT,
    )));
    boat.get_entity().set_has_no_gravity(true);
    world.spawn_entity(boat.clone());
    let pigs = [-0.5, 0.5].map(|x| {
        let pig = PigEntity::new(Entity::new(
            world.clone(),
            CENTER.add_raw(x, 0.0, 0.0),
            &EntityType::PIG,
        ));
        world.spawn_entity(pig.clone());
        pig
    });
    let listener = Arc::new(RelocateOnEnter(boat.clone(), AtomicUsize::new(0)));
    server
        .plugin_manager
        .register_blocking::<VehicleEnterEvent, _>(listener.clone(), EventPriority::Normal, None);
    for _ in 0..2 {
        boat.tick(boat.as_ref(), &server);
        assert_eq!(listener.1.load(Ordering::Relaxed), 1);
        assert_eq!(boat.get_entity().pos.load(), CENTER.add_raw(4.0, 0.0, 0.0));
        assert!(!boat.get_entity().has_passengers());
        assert!(pigs.iter().all(|pig| !pig.is_passenger()));
    }
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn vehicle_move_listener_can_block_a_previously_clear_boat_surface_snap() -> TestResult {
    assert_surface_snap_rechecks_listener_changes(SurfaceChange::Obstruct).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn boat_discards_a_stale_surface_snap_after_water_is_drained() -> TestResult {
    assert_surface_snap_rechecks_listener_changes(SurfaceChange::Drain).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn boat_retries_entry_after_a_listener_lowers_the_water_surface() -> TestResult {
    assert_surface_snap_rechecks_listener_changes(SurfaceChange::Lower).await
}

#[derive(Clone, Copy)]
enum SurfaceChange {
    Obstruct,
    Drain,
    Lower,
}

struct ChangeSurface {
    world: Arc<World>,
    fired: AtomicBool,
    change: SurfaceChange,
}

impl EventRwHandler<VehicleMoveEvent> for ChangeSurface {
    fn handle<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut VehicleMoveEvent,
    ) -> BoxFuture<'a, ()> {
        use pumpkin_util::math::position::BlockPos;
        use pumpkin_world::world::BlockFlags;

        Box::pin(async {
            if event.to.y - event.from.y > 1.0 {
                match self.change {
                    SurfaceChange::Obstruct => {
                        self.world.set_block_state(
                            &BlockPos::new(8, 65, 8),
                            Block::STONE.default_state.id,
                            BlockFlags::NOTIFY_LISTENERS,
                        );
                    }
                    SurfaceChange::Drain | SurfaceChange::Lower => {
                        let bottom = if matches!(self.change, SurfaceChange::Drain) {
                            64
                        } else {
                            65
                        };
                        for x in 7..=9 {
                            for z in 7..=9 {
                                for y in bottom..=65 {
                                    self.world.set_block_state(
                                        &BlockPos::new(x, y, z),
                                        Block::AIR.default_state.id,
                                        BlockFlags::NOTIFY_LISTENERS,
                                    );
                                }
                            }
                        }
                    }
                }
                self.fired.store(true, Ordering::Relaxed);
            }
        })
    }
}

async fn assert_surface_snap_rechecks_listener_changes(change: SurfaceChange) -> TestResult {
    use pumpkin_util::math::position::BlockPos;

    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let chunk = ChunkData::empty_sync(0, 0);
    world
        .level
        .loaded_chunks
        .insert(Vector2::new(0, 0), chunk.clone());
    let boat = BoatEntity::new(Entity::new(world.clone(), CENTER, &EntityType::OAK_BOAT));
    let entity = boat.get_entity();
    entity.set_has_no_gravity(true);
    // Establish the previous airborne state through the normal tick with an empty chunk.
    boat.tick(&boat, &server);
    assert_eq!(entity.pos.load(), CENTER);
    for x in 7..=9 {
        for z in 7..=9 {
            for y in 64..=65 {
                chunk.set_block_absolute_y(x, y, z, Block::WATER.default_state.id);
            }
        }
    }
    let falling = Vector3::new(0.0, -0.25, 0.0);
    entity.velocity.store(falling);
    let listener = Arc::new(ChangeSurface {
        world: world.clone(),
        fired: AtomicBool::new(false),
        change,
    });
    server
        .plugin_manager
        .register_blocking::<VehicleMoveEvent, _>(listener.clone(), EventPriority::Normal, None);

    boat.tick(&boat, &server);

    assert!(listener.fired.load(Ordering::Relaxed));
    assert_eq!(entity.pos.load(), CENTER);
    assert!(world.no_vehicle_collision(&boat, entity.bounding_box.load()));
    match change {
        SurfaceChange::Obstruct => {}
        SurfaceChange::Drain => {
            assert_eq!(entity.velocity.load(), falling);
            boat.tick(&boat, &server);
            assert_eq!(entity.pos.load(), CENTER + falling);
        }
        SurfaceChange::Lower => {
            assert_eq!(entity.velocity.load(), falling);
            let lower_water = BlockPos::new(8, 64, 8);
            let upper_air = BlockPos::new(8, 65, 8);
            assert_eq!(world.get_block(&lower_water).id, Block::WATER.id);
            assert_eq!(world.get_block(&upper_air).id, Block::AIR.id);
            let (fluid, state) = world.get_fluid_and_fluid_state(&lower_water);
            let source_height = 8.0f32 / 9.0;
            assert!(state.is_source);
            assert_eq!(
                world.get_fluid_height(&lower_water, fluid, &state),
                source_height
            );
            let surface = 64.0f32 + source_height;
            let bounds = entity.bounding_box.load();
            assert!(bounds.max.y + 0.001 < f64::from(surface));
            let position = Vector3::new(CENTER.x, f64::from(surface - 0.5625) + 0.101, CENTER.z);
            assert!(world.no_vehicle_collision(&boat, bounds.shift(position - CENTER)));

            boat.tick(&boat, &server);

            assert_eq!(entity.pos.load(), position);
            assert_eq!(entity.velocity.load(), Vector3::default());
        }
    }
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test]
async fn boat_dismount_uses_the_rider_heading() -> TestResult {
    use crate::entity::{ai::test_support::world, passive::snow_golem::SnowGolemEntity};

    let folder = tempfile::tempdir()?;
    let world = world(&folder);
    let chunk = pumpkin_world::chunk::ChunkData::empty_sync(0, 0);
    world
        .level
        .loaded_chunks
        .insert(Vector2::new(0, 0), chunk.clone());
    for x in 6..=10 {
        for z in 6..=10 {
            chunk.set_block_absolute_y(x, 63, z, Block::STONE.default_state.id);
        }
    }

    let center = Vector3::new(8.5, 64.0, 8.5);
    let boat = Arc::new(BoatEntity::new(Entity::new(
        world.clone(),
        center,
        &EntityType::OAK_BOAT,
    )));
    let rider = SnowGolemEntity::new(Entity::new(world.clone(), center, &EntityType::SNOW_GOLEM));
    boat.get_entity().add_passenger(boat.clone(), rider.clone());
    assert!(rider.get_entity().has_vehicle());
    boat.get_entity()
        .remove_passenger(rider.get_entity().entity_id);
    let boat_exit = rider.get_entity().pos.load();
    assert!(!rider.get_entity().has_vehicle());
    assert_eq!(boat_exit.x, center.x);
    assert_eq!(boat_exit.y, center.y);
    assert!(boat_exit.z > 9.5, "boat exit was {boat_exit:?}");

    world.level.shutdown().await;
    Ok(())
}

struct ReplaceBoat(AtomicBool);
impl EventRwHandler<PlayerInteractEvent> for ReplaceBoat {
    fn handle<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut PlayerInteractEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            if !self.0.swap(true, Ordering::Relaxed) {
                event
                    .player
                    .inventory()
                    .set_held_item(pumpkin_data::item_stack::ItemStack::new(
                        1,
                        &pumpkin_data::item::Item::STICK,
                    ));
            }
        })
    }
}

const BOAT_PLACEMENT_OFFSET: Vector3<f64> = Vector3::new(8.9, 64.0, 7.0);

fn boat_placement_platform(world: &World) -> Arc<ChunkData> {
    let chunk = ChunkData::empty_sync(0, 0);
    chunk.set_block_absolute_y(8, 63, 8, Block::STONE.default_state.id);
    world
        .level
        .loaded_chunks
        .insert(Vector2::new(0, 0), chunk.clone());
    chunk
}

fn boat_placement_actors(world: &World) -> Vec<Arc<dyn EntityBase>> {
    world
        .entities
        .load()
        .iter()
        .filter(|entity| entity.get_entity().entity_type == &EntityType::OAK_BOAT)
        .cloned()
        .collect()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn boat_placement_checks_fences_rooted_below_the_hull() -> TestResult {
    use pumpkin_data::{
        data_component_impl::BlocksAttacksImpl, item::Item, item_stack::ItemStack,
        statistic::StatisticCategory,
    };
    use pumpkin_protocol::java::server::play::SUseItem;
    use pumpkin_util::Hand;

    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let chunk = boat_placement_platform(&world);
    chunk.set_block_absolute_y(9, 63, 8, Block::OAK_FENCE.default_state.id);
    let (player, _peer) = java_rider(&server, &world, "Notch")?;
    player.get_entity().set_pos(BOAT_PLACEMENT_OFFSET);
    let mut stack = ItemStack::new(1, &Item::OAK_BOAT);
    stack.set_data_component(BlocksAttacksImpl::DEFAULT);
    player.inventory.set_stack_in_hand(Hand::Right, stack);
    let uses = || player.get_stat(StatisticCategory::Used, i32::from(Item::OAK_BOAT.id));

    let client = player.client.java().ok_or("Java client")?;
    let air_use = |sequence: i32, pitch| {
        client.handle_use_item(
            &player,
            &SUseItem {
                hand: 0.into(),
                sequence: sequence.into(),
                yaw: 0.0,
                pitch,
            },
            &server,
        );
    };
    let rejects_placement = || {
        assert!(boat_placement_actors(&world).is_empty());
        assert_eq!(uses(), 0);
        assert_eq!(player.inventory().held_item().item, &Item::OAK_BOAT);
    };
    air_use(0, 45.0);
    rejects_placement();
    chunk.set_block_absolute_y(9, 63, 8, Block::AIR.default_state.id);

    player.get_entity().set_pos(Vector3::new(8.9, 64.0, 8.5));
    air_use(1, 90.0);
    rejects_placement();
    player.get_entity().set_pos(BOAT_PLACEMENT_OFFSET);
    let (blocker, _blocker_peer) = java_rider(&server, &world, "jeb_")?;
    blocker.get_entity().set_pos(BOAT_PLACEMENT_OFFSET);
    air_use(2, 45.0);
    rejects_placement();
    blocker.get_entity().set_pos(CENTER.add_raw(8.0, 0.0, 0.0));

    let listener = Arc::new(ReplaceBoat(AtomicBool::new(false)));
    server
        .plugin_manager
        .register_blocking::<PlayerInteractEvent, _>(listener.clone(), EventPriority::Normal, None);
    air_use(3, 45.0);
    assert!(listener.0.load(Ordering::Relaxed));
    assert!(boat_placement_actors(&world).is_empty());
    assert_eq!(uses(), 0);
    assert_eq!(player.inventory().held_item().item, &Item::STICK);
    player
        .inventory()
        .set_held_item(ItemStack::new(1, &Item::OAK_BOAT));
    air_use(4, 45.0);
    assert_eq!(boat_placement_actors(&world).len(), 1);
    assert_eq!(uses(), 1);
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn boat_placement_applies_placer_yaw_and_materializes_fresh_remainders() -> TestResult {
    use pumpkin_data::{
        data_component_impl::{EntityDataImpl, UseRemainderImpl},
        item::Item,
        item_stack::ItemStack,
        statistic::StatisticCategory,
    };
    use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};
    use pumpkin_protocol::java::server::play::{SUseItem, SUseItemOn};

    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    boat_placement_platform(&world);
    let (player, _peer) = java_rider(&server, &world, "Notch")?;
    player.get_entity().set_pos(BOAT_PLACEMENT_OFFSET);
    let client = player.client.java().ok_or("Java client")?;
    let uses = || player.get_stat(StatisticCategory::Used, i32::from(Item::OAK_BOAT.id));

    let mut placing = ItemStack::new(1, &Item::OAK_BOAT);
    let template = ItemStack::new(1, &Item::STICK);
    placing.set_data_component(UseRemainderImpl {
        remainder: Some(template.clone()),
    });
    let mut nbt = NbtCompound::new();
    nbt.put_list("Rotation", vec![NbtTag::Float(70.0), NbtTag::Float(27.0)]);
    placing.set_data_component(EntityDataImpl {
        entity_type: Some(&EntityType::OAK_BOAT),
        nbt: Some(nbt),
    });
    player.inventory().set_held_item(placing.clone());
    client.handle_use_item_on(
        &player,
        &SUseItemOn {
            hand: 0.into(),
            position: pumpkin_util::math::position::BlockPos::new(8, 63, 8),
            face: 1.into(),
            cursor_pos: Vector3::new(0.9, 1.0, 0.5),
            inside_block: false,
            is_against_world_border: false,
            sequence: 1.into(),
        },
        &server,
    )?;
    assert_eq!(uses(), 0);
    let placer_yaw = 12.0;
    let mut prior_remainder = template.uid;
    for (sequence, expected_uses) in [(2i32, 1), (3, 2)] {
        player.inventory().set_held_item(placing.clone());
        client.handle_use_item(
            &player,
            &SUseItem {
                hand: 0.into(),
                sequence: sequence.into(),
                yaw: placer_yaw,
                pitch: 45.0,
            },
            &server,
        );
        let boats = boat_placement_actors(&world);
        assert_eq!(boats.len(), 1);
        let boat = boats.first().ok_or("Placed boat")?;
        assert_eq!(boat.get_entity().yaw.load(), placer_yaw);
        assert_eq!(boat.get_entity().pitch.load(), 27.0);
        assert_eq!(uses(), expected_uses);
        let remainder = player.inventory().held_item();
        assert_eq!(remainder.item, &Item::STICK);
        assert_ne!(remainder.uid, prior_remainder);
        prior_remainder = remainder.uid;
        world.remove_entity(boat.as_ref());
    }
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_driven_boat_vehicle_pushes_nearby_mobs_and_bedrock_players() -> TestResult {
    use crate::entity::passive::cow::CowEntity;

    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let chunk = Vector2::new(0, 0);
    world
        .level
        .loaded_chunks
        .insert(chunk, ChunkData::empty_sync(0, 0));
    *world
        .active_chunks
        .write()
        .unwrap_or_else(PoisonError::into_inner) = Arc::new(std::iter::once(chunk).collect());
    let (rider, _peer) = java_rider(&server, &world, "Notch")?;
    let boat = mounted_java_boat(&world, &rider, CENTER);
    let cow = CowEntity::new(Entity::new(
        world.clone(),
        CENTER.add_raw(1.25, 0.0, 0.0),
        &EntityType::COW,
    ));
    world.spawn_entity(cow.clone());
    let (neighbor, _packets) = bedrock_rider(&server, &world, "jeb_").await?;
    neighbor
        .get_entity()
        .set_pos(CENTER.add_raw(-1.1, 0.0, 0.0));
    let hull = boat.get_entity().bounding_box.load();
    for target in [cow.get_entity(), neighbor.get_entity()] {
        assert!(!hull.intersects(&target.bounding_box.load()));
        assert_eq!(target.velocity.load(), Vector3::default());
    }

    boat.tick(boat.as_ref(), &server);
    assert!(cow.get_entity().velocity.load().x > 0.0);
    assert!(!cow.get_entity().has_vehicle());
    assert!(neighbor.get_entity().push_impulse.load().x < 0.0);
    assert_eq!(boat.get_entity().velocity.load(), Vector3::default());
    let client = neighbor.client.bedrock().ok_or("Missing Bedrock client")?;
    client.clear_player();
    client.close().await;
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn boat_vehicle_visits_an_accepted_bent_path_in_the_first_effect_pass() -> TestResult {
    use crate::block::{
        BlockBehaviour, BlockMetadata, OnEntityCollisionArgs, registry::default_registry,
    };
    use pumpkin_data::BlockId;
    use pumpkin_util::math::position::BlockPos;
    use std::sync::Mutex;

    struct ObserveEndpoint(Arc<Mutex<Vec<i32>>>);
    impl BlockMetadata for ObserveEndpoint {
        fn ids() -> Box<[BlockId]> {
            Box::new([Block::COBWEB.id])
        }
    }
    impl BlockBehaviour for ObserveEndpoint {
        fn on_entity_collision(&self, args: OnEntityCollisionArgs<'_>) {
            self.0
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push(args.entity.get_entity().get_frozen_ticks());
        }
    }

    let directory = tempfile::tempdir()?;
    let (server, base_world) = vehicle_world(&directory).await?;
    let samples = Arc::new(Mutex::new(Vec::new()));
    let mut registry = default_registry();
    Arc::get_mut(&mut registry)
        .ok_or("New registry is already shared")?
        .register(ObserveEndpoint(samples.clone()));
    let world = Arc::new(World::load(
        base_world.level.clone(),
        base_world.level_info.clone(),
        base_world.dimension.clone(),
        registry,
        Arc::downgrade(&server),
    ));
    let chunk = ChunkData::empty_sync(0, 0);
    let pad = BlockPos::new(7, 64, 8);
    chunk.set_block_absolute_y(7, 63, 8, Block::WATER.default_state.id);
    chunk.set_block_absolute_y(7, 64, 8, Block::LILY_PAD.default_state.id);
    chunk.set_block_absolute_y(8, 64, 8, Block::POWDER_SNOW.default_state.id);
    chunk.set_block_absolute_y(10, 64, 14, Block::COBWEB.default_state.id);
    world.level.loaded_chunks.insert(Vector2::new(0, 0), chunk);
    let (rider, _peer) = java_rider(&server, &world, "Notch")?;
    let start = Vector3::new(4.5, 64.125, 8.5);
    let corner = Vector3::new(10.5, 64.125, 8.5);
    let destination = Vector3::new(10.5, 64.125, 14.5);
    let boat = mounted_java_boat(&world, &rider, start);
    let entity = boat.get_entity();
    assert_eq!(world.get_block(&pad), &Block::LILY_PAD);

    queue_vehicle_position(&rider, corner, 0.0)?;
    queue_vehicle_position(&rider, destination, 0.0)?;
    World::process_player_packets(&world.players.load());
    assert_eq!(entity.pos.load(), destination);
    assert_eq!(world.get_block(&pad), &Block::LILY_PAD);
    assert_eq!(entity.get_frozen_ticks(), 0);
    boat.tick(boat.as_ref(), &server);
    assert_eq!(world.get_block(&pad), &Block::AIR);
    assert_eq!(entity.get_frozen_ticks(), 1);
    assert_eq!(
        samples
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .as_slice(),
        [0, 1],
    );
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn boat_vehicle_extinguishes_after_lava_and_water_contact_in_the_same_step() -> TestResult {
    use pumpkin_protocol::{bedrock::client::CPlaySound, packet::Packet};

    let directory = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&directory).await?;
    let (watcher, mut packets) = bedrock_rider(&server, &world, "jeb_").await?;
    // Keep the boat alive for both inside-block passes so the assertion tests
    // the order of effects independently of the damage threshold.
    server
        .plugin_manager
        .register_blocking::<crate::plugin::api::events::vehicle::vehicle_damage::VehicleDamageEvent, _>(
            Arc::new(PreventVehicleDamage),
            EventPriority::Normal,
            None,
        );
    let chunk = ChunkData::empty_sync(0, 0);
    world
        .level
        .loaded_chunks
        .insert(Vector2::new(0, 0), chunk.clone());
    for (first, second) in [(&Block::WATER, &Block::LAVA), (&Block::LAVA, &Block::WATER)] {
        chunk.set_block_absolute_y(8, 64, 8, first.default_state.id);
        chunk.set_block_absolute_y(9, 64, 8, second.default_state.id);
        let boat = BoatEntity::new(Entity::new(
            world.clone(),
            CENTER.add_raw(0.5, 0.0, 0.0),
            &EntityType::OAK_BOAT,
        ));
        boat.get_entity().set_silent(true);
        boat.get_entity().fire_ticks.store(10, Ordering::Relaxed);
        boat.get_entity().set_frozen_ticks(40);
        boat.tick(&boat, &server);
        assert!(!boat.get_entity().is_removed());
        assert!(!boat.get_entity().is_on_fire());
        assert_eq!(boat.get_entity().get_frozen_ticks(), 0);

        let mut fizz = 0;
        while let Ok(packet) = packets.try_recv() {
            if let Some((id, payload)) = crate::net::bedrock::split_raw_packet(&packet.data)
                && i32::from(id) == CPlaySound::PACKET_ID
                && payload
                    .as_ref()
                    .get_str()
                    .is_ok_and(|name| name.as_ref() == "random.fizz")
            {
                fizz += 1;
            }
        }
        assert_eq!(fizz, 1, "A silent hull still emits its extinguish sound");
    }
    let client = watcher.client.bedrock().ok_or("Bedrock watcher")?;
    client.clear_player();
    client.close().await;
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ride_command_uses_the_boat_exit_when_the_seat_is_below_the_floor() -> TestResult {
    use crate::command::CommandSender;

    struct VetoExit(AtomicBool);

    impl EventRwHandler<VehicleExitEvent> for VetoExit {
        fn handle<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut VehicleExitEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                event.cancelled = self.0.load(Ordering::Relaxed);
            })
        }
    }

    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let chunk = ChunkData::empty_sync(0, 0);
    for z in 8..=10 {
        chunk.set_block_absolute_y(8, 63, z, Block::STONE.default_state.id);
    }
    world.level.loaded_chunks.insert(Vector2::new(0, 0), chunk);
    let (rider, _peer) = java_rider(&server, &world, "Notch")?;
    let boat = mounted_java_boat(&world, &rider, CENTER);
    let player = rider.get_entity();
    crate::entity::position_rider(boat.as_ref(), rider.as_ref());
    let seated_position = player.pos.load();
    assert!(seated_position.y < CENTER.y);
    let listener = Arc::new(VetoExit(AtomicBool::new(true)));
    server
        .plugin_manager
        .register_blocking::<VehicleExitEvent, _>(listener.clone(), EventPriority::Normal, None);
    let source = CommandSender::Console.into_source(&server).with_silent();
    let dismount = || {
        server
            .command_dispatcher
            .load()
            .execute_input("ride Notch dismount", &source)
    };
    assert!(dismount().is_err());
    assert_mounted(boat.get_entity(), player);
    assert_eq!(player.pos.load(), seated_position);
    listener.0.store(false, Ordering::Relaxed);
    let result = dismount().map_err(|error| error.message.get_text())?;

    assert_eq!(result, 1);
    assert!(!player.has_vehicle());
    assert!(!boat.get_entity().has_passenger(rider.entity_id()));
    assert_eq!(player.pos.load().y, CENTER.y);
    assert_eq!(player.pos.load().x, CENTER.x);
    assert!(player.pos.load().z > CENTER.z);
    assert!(
        rider
            .awaiting_teleport
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .is_some_and(|(_, position)| position == player.pos.load())
    );
    world.level.shutdown().await;
    Ok(())
}

struct ObserveBoatMount {
    boat: Arc<BoatEntity>,
    passenger: Arc<dyn EntityBase>,
    detaches: bool,
    entry_position: Vector3<f64>,
    mount_event_seen: AtomicBool,
    mount_packet_seen: AtomicBool,
}

impl EventRwHandler<GenericGameEvent> for ObserveBoatMount {
    fn handle<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut GenericGameEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            if event.event_key == GameEvent::EntityMount.name() {
                let hull = self.boat.get_entity();
                let passenger = self.passenger.get_entity();
                assert_eq!(event.position, hull.pos.load());
                assert_mounted(hull, passenger);
                assert_eq!(passenger.pos.load(), self.entry_position);
                assert_eq!(passenger.pose.load(), EntityPose::Standing);
                assert_eq!(passenger.is_leashed(), self.detaches);
                assert!(!self.mount_event_seen.swap(true, Ordering::Relaxed));
            }
        })
    }
}

impl EventRwHandler<PacketSentEvent> for ObserveBoatMount {
    fn handle<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut PacketSentEvent,
    ) -> BoxFuture<'a, ()> {
        use pumpkin_protocol::java::client::play::{CPlayerPosition, CSetPassengers};

        Box::pin(async move {
            let mut data = event.payload.as_ref();
            let hull = self.boat.get_entity();
            let passenger = self.passenger.get_entity();
            if !self.detaches
                && event.packet_id == CPlayerPosition::to_id(CURRENT_MC_VERSION)
                && passenger.has_vehicle()
            {
                assert!(self.mount_event_seen.load(Ordering::Relaxed));
                assert_eq!(passenger.yaw.load().to_bits(), 105.0f32.to_bits());
                assert_eq!(passenger.body_yaw.load().to_bits(), 0.0f32.to_bits());
                assert_eq!(passenger.velocity.load(), Vector3::default());
            }
            if event.packet_id == CSetPassengers::to_id(CURRENT_MC_VERSION)
                && data.get_var_int().is_ok_and(|id| id.0 == hull.entity_id)
                && data.get_var_int().is_ok_and(|count| count.0 == 1)
                && data
                    .get_var_int()
                    .is_ok_and(|id| id.0 == passenger.entity_id)
            {
                assert!(self.mount_event_seen.load(Ordering::Relaxed));
                assert_mounted(hull, passenger);
                assert_eq!(passenger.pose.load(), EntityPose::Standing);
                assert!(!passenger.is_leashed());
                assert!(!self.mount_packet_seen.swap(true, Ordering::Relaxed));
                if self.detaches {
                    hull.remove_passenger_on_disconnect(passenger.entity_id);
                }
            }
        })
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn boat_pickup_finishes_mount_effects_before_passenger_packet() -> TestResult {
    assert_boat_mount_packet_order(true).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn boat_interaction_emits_committed_mount_before_passenger_packet() -> TestResult {
    assert_boat_mount_packet_order(false).await
}

async fn assert_boat_mount_packet_order(pickup: bool) -> TestResult {
    use pumpkin_data::item_stack::ItemStack;
    use pumpkin_protocol::java::client::play::CKeepAlive;

    let directory = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&directory).await?;
    let chunk = Vector2::new(0, 0);
    world
        .level
        .loaded_chunks
        .insert(chunk, ChunkData::empty_sync(0, 0));
    *world
        .active_chunks
        .write()
        .unwrap_or_else(PoisonError::into_inner) = Arc::new(std::iter::once(chunk).collect());
    let (watcher, mut peer) = java_rider(&server, &world, "Notch")?;
    let client = watcher.client.java().ok_or("Missing Java watcher")?;
    client.player.store(Arc::new(Some(watcher.clone())));
    watcher.get_entity().set_pos(CENTER.add_raw(3.0, 0.0, 0.0));
    watcher
        .watched_section
        .store(pumpkin_world::cylindrical_chunk_iterator::Cylindrical::new(
            chunk,
            std::num::NonZeroU8::new(4).ok_or("Zero view distance")?,
        ));
    let boat = Arc::new(BoatEntity::new(Entity::new(
        world.clone(),
        CENTER,
        &EntityType::OAK_BOAT,
    )));
    let hull = boat.get_entity();
    hull.set_has_no_gravity(true);
    world.spawn_entity(boat.clone());
    let passenger: Arc<dyn EntityBase> = if pickup {
        let pig = PigEntity::new(Entity::new(world.clone(), CENTER, &EntityType::PIG));
        pig.get_entity()
            .attach_leash(&(watcher.clone() as Arc<dyn EntityBase>));
        world.spawn_entity(pig.clone());
        pig
    } else {
        watcher.clone()
    };
    let rider = passenger.get_entity();
    rider.set_pose(EntityPose::Crouching);
    rider.set_rotation(150.0, -14.0);
    if !pickup {
        rider.velocity.store(Vector3::new(0.25, 0.125, 0.0));
    }
    assert!(!rider.has_vehicle());
    assert!(!hull.has_passengers());
    assert_eq!(rider.pose.load(), EntityPose::Crouching);
    assert_eq!(rider.is_leashed(), pickup);

    let listener = Arc::new(ObserveBoatMount {
        boat: boat.clone(),
        passenger: passenger.clone(),
        detaches: pickup,
        entry_position: rider.pos.load(),
        mount_event_seen: AtomicBool::new(false),
        mount_packet_seen: AtomicBool::new(false),
    });
    server
        .plugin_manager
        .register_blocking::<GenericGameEvent, _>(listener.clone(), EventPriority::Normal, None);
    server
        .plugin_manager
        .register_blocking::<PacketSentEvent, _>(listener.clone(), EventPriority::Normal, None);

    if pickup {
        boat.tick(boat.as_ref(), &server);
        assert!(!rider.has_vehicle());
        assert!(!hull.has_passengers());
        assert!(!rider.is_leashed());
    } else {
        assert!(boat.interact(&watcher, &mut ItemStack::EMPTY.clone()));
        assert_mounted(hull, rider);
    }
    assert!(listener.mount_event_seen.load(Ordering::Relaxed));
    assert!(listener.mount_packet_seen.load(Ordering::Relaxed));
    assert_eq!(rider.pose.load(), EntityPose::Standing);
    assert!(client.try_send_packet(&CKeepAlive::new(73)));
    let (published, placement) = read_vehicle_passenger_updates(&mut peer, hull.entity_id).await?;
    let expected = Vec::from_iter((!pickup).then_some(rider.entity_id));
    assert_eq!(published, [expected]);
    assert_eq!(placement.is_some(), !pickup);
    if let Some(placement) = placement {
        assert_java_seat_and_exit_confirmation(
            hull,
            &watcher,
            &placement,
            listener.entry_position,
        )?;
    }
    client.player.store(Arc::new(None));
    client.close();
    world.level.shutdown().await;
    Ok(())
}

fn assert_java_seat_and_exit_confirmation(
    hull: &Entity,
    watcher: &crate::entity::player::Player,
    placement: &pumpkin_protocol::java::client::play::CPlayerPosition,
    entry_position: Vector3<f64>,
) -> TestResult {
    use pumpkin_protocol::{PositionFlag, java::server::play::SConfirmTeleport};

    let rider = watcher.get_entity();
    let client = watcher.client.java().ok_or("Missing Java watcher")?;
    assert_ne!(placement.position, entry_position);
    assert_eq!(placement.position, rider.pos.load());
    assert_eq!(placement.yaw.to_bits(), 0.0f32.to_bits());
    assert_eq!(placement.pitch.to_bits(), 0.0f32.to_bits());
    assert_eq!(
        placement.relatives,
        [PositionFlag::YRot, PositionFlag::XRot]
    );
    assert_eq!(rider.yaw.load().to_bits(), 105.0f32.to_bits());
    assert_eq!(rider.pitch.load().to_bits(), (-14.0f32).to_bits());

    let pending = || {
        *watcher
            .awaiting_teleport
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    };
    let confirm = |teleport_id, position, yaw| {
        client.handle_confirm_teleport(
            watcher,
            &SConfirmTeleport {
                teleport_id,
                position,
                yaw,
                pitch: -14.0,
            },
        );
    };
    confirm(placement.teleport_id, placement.position, 150.0);
    assert_eq!(pending(), None);
    assert_eq!(rider.yaw.load().to_bits(), 150.0f32.to_bits());
    let carrier = rider.get_vehicle().ok_or("Missing boat attachment")?;
    crate::entity::position_rider(carrier.as_ref(), watcher);
    assert_eq!(rider.yaw.load().to_bits(), 105.0f32.to_bits());

    hull.remove_passenger(rider.entity_id);
    let (exit_id, exit) = pending().ok_or("Missing exit confirmation")?;
    assert_ne!(exit_id, placement.teleport_id);
    confirm(placement.teleport_id, placement.position, -150.0);
    assert_eq!(pending(), Some((exit_id, exit)));
    assert_eq!(rider.pos.load(), exit);
    assert_eq!(rider.yaw.load().to_bits(), 105.0f32.to_bits());
    confirm(exit_id, exit.add_raw(0.5, 0.0, 0.0), 150.0);
    assert_eq!(pending(), None);
    assert_eq!(rider.pos.load(), exit);
    assert_eq!(rider.yaw.load().to_bits(), 150.0f32.to_bits());
    assert!(!client.is_closed());
    Ok(())
}

pub(super) async fn read_vehicle_passenger_updates(
    peer: &mut tokio::io::DuplexStream,
    vehicle_id: i32,
) -> Result<
    (
        Vec<Vec<i32>>,
        Option<pumpkin_protocol::java::client::play::CPlayerPosition>,
    ),
    Box<dyn std::error::Error>,
> {
    use pumpkin_protocol::{
        ServerPacket,
        java::{
            client::play::{CKeepAlive, CPlayerPosition, CSetPassengers},
            packet_decoder::TCPNetworkDecoder,
        },
    };

    let mut decoder = TCPNetworkDecoder::new(tokio::io::BufReader::new(peer));
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        let mut lists = Vec::new();
        let mut placement = None;
        loop {
            let packet = decoder.get_raw_packet().await?;
            let mut payload = packet.payload.as_ref();
            if packet.id == CKeepAlive::to_id(CURRENT_MC_VERSION) {
                assert_eq!(payload.get_i64_be()?, 73);
                return Ok((lists, placement));
            }
            if packet.id == CPlayerPosition::to_id(CURRENT_MC_VERSION) {
                assert!(lists.is_empty());
                assert!(placement.is_none());
                placement = Some(CPlayerPosition::read(&mut payload, &CURRENT_MC_VERSION)?);
            }
            if packet.id == CSetPassengers::to_id(CURRENT_MC_VERSION)
                && payload.get_var_int()?.0 == vehicle_id
            {
                let count = payload.get_var_int()?.0;
                let passengers = (0..count)
                    .map(|_| payload.get_var_int().map(|id| id.0))
                    .collect::<Result<Vec<_>, _>>()?;
                lists.push(passengers);
            }
        }
    })
    .await?
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_player_boarding_an_occupied_boat_takes_the_controls() -> TestResult {
    use pumpkin_data::item_stack::ItemStack;
    use pumpkin_protocol::java::{client::play::CKeepAlive, server::play::SConfirmTeleport};

    let directory = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&directory).await?;
    world
        .level
        .loaded_chunks
        .insert(Vector2::new(0, 0), ChunkData::empty_sync(0, 0));
    let (viewer, mut bedrock_packets) = bedrock_rider(&server, &world, "jeb_").await?;
    let boat = Arc::new(BoatEntity::new(Entity::new(
        world.clone(),
        CENTER,
        &EntityType::BAMBOO_RAFT,
    )));
    world.spawn_entity(boat.clone());
    let hull = boat.get_entity();
    let pig = PigEntity::new(Entity::new(world.clone(), CENTER, &EntityType::PIG));
    world.spawn_entity(pig.clone());
    let pig_id = pig.get_entity().entity_id;
    hull.add_passenger(boat.clone(), pig);
    assert_eq!(
        bedrock_links(&mut bedrock_packets, hull.entity_id)?,
        [(i64::from(pig_id), 1)]
    );

    let (driver, mut peer) = java_rider(&server, &world, "Notch")?;
    driver.get_entity().set_pos(CENTER);
    assert!(boat.interact(&driver, &mut ItemStack::EMPTY.clone()));
    let client = driver.client.java().ok_or("Missing Java client")?;
    assert!(client.try_send_packet(&CKeepAlive::new(73)));
    let (passengers, seat) = read_vehicle_passenger_updates(&mut peer, hull.entity_id).await?;
    assert_eq!(passengers, [vec![driver.entity_id(), pig_id]]);
    assert_eq!(
        bedrock_links(&mut bedrock_packets, hull.entity_id)?,
        [(i64::from(driver.entity_id()), 1), (i64::from(pig_id), 2)],
    );
    let seat = seat.ok_or("Mount did not send a seat correction")?;
    client.handle_confirm_teleport(
        &driver,
        &SConfirmTeleport {
            teleport_id: seat.teleport_id,
            position: seat.position,
            yaw: 0.0,
            pitch: 0.0,
        },
    );
    let destination = CENTER.add_raw(0.1, 0.0, 0.0);
    queue_vehicle_position(&driver, destination, 0.0)?;
    World::process_player_packets(&world.players.load());
    assert_eq!(hull.pos.load(), destination);

    hull.remove_passenger(pig_id);
    let (second_player, _peer) = java_rider(&server, &world, "deadmau5")?;
    second_player.get_entity().set_pos(destination);
    assert!(boat.interact(&second_player, &mut ItemStack::EMPTY.clone()));
    assert_eq!(
        bedrock_links(&mut bedrock_packets, hull.entity_id)?,
        [
            (i64::from(pig_id), 0),
            (i64::from(second_player.entity_id()), 2),
        ],
    );
    assert_eq!(
        boat.get_controlling_passenger()
            .map(|rider| rider.get_entity().entity_id),
        Some(driver.entity_id())
    );
    let client = viewer.client.bedrock().ok_or("Missing Bedrock viewer")?;
    client.clear_player();
    client.close().await;
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn boat_seating_sets_animal_facing_and_preserves_the_breeze_turn_exception() -> TestResult {
    use crate::entity::{mob::breeze::BreezeEntity, position_rider};

    let directory = tempfile::tempdir()?;
    let (_server, world) = vehicle_world(&directory).await?;
    let boat = Arc::new(BoatEntity::new(Entity::new(
        world.clone(),
        CENTER,
        &EntityType::OAK_BOAT,
    )));
    world.spawn_entity(boat.clone());
    let pig = PigEntity::new(Entity::new(world.clone(), CENTER, &EntityType::PIG));
    let breeze = BreezeEntity::new(Entity::new(world.clone(), CENTER, &EntityType::BREEZE));
    for passenger in [pig.get_entity(), breeze.get_entity()] {
        passenger.set_rotation(140.0, 0.0);
    }
    let carrier = boat.get_entity();
    carrier.add_passenger_silently(boat.clone(), pig.clone());
    position_rider(boat.as_ref(), pig.as_ref());
    let pig_entity = pig.get_entity();
    assert_eq!(pig_entity.yaw.load().to_bits(), 0.0f32.to_bits());
    assert_eq!(pig_entity.head_yaw.load().to_bits(), 75.0f32.to_bits());

    carrier.add_passenger_silently(boat.clone(), breeze.clone());
    position_rider(boat.as_ref(), pig.as_ref());
    position_rider(boat.as_ref(), breeze.as_ref());
    let expected_yaw: f32 = if pig_entity.entity_id % 2 == 0 {
        90.0
    } else {
        270.0
    };
    assert_eq!(pig_entity.yaw.load().to_bits(), expected_yaw.to_bits());
    assert_eq!(pig_entity.body_yaw.load().to_bits(), expected_yaw.to_bits());
    assert_eq!(breeze.get_entity().yaw.load().to_bits(), 140.0f32.to_bits());
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_baby_hoglin_uses_the_animal_seat_in_a_full_boat() -> TestResult {
    use crate::entity::{ageable::AgeableMob, mob::hoglin::HoglinEntity, position_rider};

    let directory = tempfile::tempdir()?;
    let (_server, world) = vehicle_world(&directory).await?;
    let boat = Arc::new(BoatEntity::new(Entity::new(
        world.clone(),
        CENTER,
        &EntityType::OAK_BOAT,
    )));
    world.spawn_entity(boat.clone());
    let hoglin = HoglinEntity::new(Entity::new(world.clone(), CENTER, &EntityType::HOGLIN));
    hoglin.set_baby(true);
    let rider = hoglin.get_entity();
    rider.set_rotation(140.0, 0.0);
    let hull = boat.get_entity();
    hull.add_passenger_silently(boat.clone(), hoglin.clone());
    position_rider(boat.as_ref(), hoglin.as_ref());
    assert_eq!(rider.yaw.load().to_bits(), 0.0f32.to_bits());

    let pig = PigEntity::new(Entity::new(world.clone(), CENTER, &EntityType::PIG));
    hull.add_passenger_silently(boat.clone(), pig);
    position_rider(boat.as_ref(), hoglin.as_ref());
    let expected_yaw = if rider.entity_id % 2 == 0 {
        90.0f32
    } else {
        270.0
    };
    assert_eq!(rider.yaw.load().to_bits(), expected_yaw.to_bits());
    assert_eq!(rider.body_yaw.load().to_bits(), expected_yaw.to_bits());
    let seat = boat
        .passenger_attachment_point(hoglin.as_ref(), 0)
        .ok_or("Missing hoglin seat")?;
    assert_eq!(seat.z.to_bits(), f64::from(0.4f32).to_bits());
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn saved_vehicle_reattachment_sends_seat_without_mount_effects() -> TestResult {
    use crate::entity::vehicle::minecart::MinecartEntity;
    use pumpkin_data::advancement::Advancement;
    use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};
    use pumpkin_protocol::java::client::play::CKeepAlive;

    struct MountObserver(AtomicBool);

    impl EventRwHandler<GenericGameEvent> for MountObserver {
        fn handle<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut GenericGameEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                if event.event_key == GameEvent::EntityMount.name() {
                    self.0.store(true, Ordering::Relaxed);
                }
            })
        }
    }

    let directory = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&directory).await?;
    let observer = Arc::new(MountObserver(AtomicBool::new(false)));
    server
        .plugin_manager
        .register_blocking::<GenericGameEvent, _>(observer.clone(), EventPriority::Normal, None);
    let boat = Arc::new(BoatEntity::new(Entity::new(
        world.clone(),
        CENTER,
        &EntityType::OAK_BOAT,
    )));
    let cart = Arc::new(MinecartEntity::new(Entity::new(
        world.clone(),
        CENTER,
        &EntityType::MINECART,
    )));
    let cases: [(&str, Arc<dyn EntityBase>); 2] = [("Notch", boat), ("jeb_", cart)];
    let advancement = Advancement::HUSBANDRY_RIDE_A_BOAT_WITH_A_GOAT;

    for (name, vehicle) in cases {
        world.spawn_entity(vehicle.clone());
        let (player, mut peer) = java_rider(&server, &world, name)?;
        let client = player.client.java().ok_or("Missing Java rider")?;
        let carrier = vehicle.get_entity();
        let uuid = carrier.entity_uuid.as_u128();
        let mut attachment = NbtCompound::new();
        attachment.put(
            "Attach",
            NbtTag::IntArray([96, 64, 32, 0].map(|shift| (uuid >> shift) as i32).to_vec()),
        );
        let mut saved = NbtCompound::new();
        saved.put_compound("RootVehicle", attachment);
        player.read_custom_nbt(&saved);
        let rider = player.get_entity();
        let entry = CENTER.add_raw(2.0, 0.0, 0.0);
        rider.set_pos(entry);
        rider.set_pose(EntityPose::Crouching);
        assert!(!rider.has_vehicle());
        assert!(!carrier.has_passenger(rider.entity_id));
        listen_for_saved_riding_criterion(&player, advancement);

        player.try_restore_vehicle(&vehicle);

        assert_mounted(carrier, rider);
        assert_eq!(rider.pose.load(), EntityPose::Standing);
        assert!(
            !observer.0.load(Ordering::Relaxed),
            "save restoration emitted ENTITY_MOUNT"
        );
        assert!(
            !player
                .advancements
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .progress
                .map
                .contains_key(advancement),
            "save restoration triggered a riding criterion"
        );
        assert!(client.try_send_packet(&CKeepAlive::new(73)));
        let (passengers, placement) =
            read_vehicle_passenger_updates(&mut peer, carrier.entity_id).await?;
        assert_eq!(passengers, [vec![rider.entity_id]]);
        let placement = placement.ok_or("Saved mount did not send a seat correction")?;
        assert_eq!(placement.position, rider.pos.load());
        assert_ne!(placement.position, entry);
    }
    world.level.shutdown().await;
    Ok(())
}

fn listen_for_saved_riding_criterion(
    player: &Arc<crate::entity::player::Player>,
    advancement: &'static pumpkin_data::advancement::Advancement,
) {
    use crate::entity::player::advancement::{
        ActiveCriterionListener, predicate::CriterionConditions,
    };

    let mut advancements = player
        .advancements
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    advancements.set_player(player);
    assert!(!advancements.progress.map.contains_key(advancement));
    advancements.active_listeners.insert(
        "minecraft:started_riding",
        vec![ActiveCriterionListener {
            advancement,
            criterion: advancement.criteria[0],
            conditions: Arc::new(CriterionConditions::StartedRiding(None)),
        }],
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bedrock_viewer_receives_committed_pose_after_mount_callback_dismounts() -> TestResult {
    assert_bedrock_pose_after_mount_callback(false).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bedrock_viewer_keeps_newer_pose_from_packet_observer() -> TestResult {
    assert_bedrock_pose_after_mount_callback(true).await
}

struct ReplaceStandingPose {
    rider: Arc<crate::entity::player::Player>,
    fired: AtomicBool,
}

impl EventRwHandler<PacketSentEvent> for ReplaceStandingPose {
    fn handle<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut PacketSentEvent,
    ) -> BoxFuture<'a, ()> {
        use pumpkin_protocol::{
            bedrock::client::set_actor_data::{CSetActorData, entity_data_key},
            codec::var_ulong::VarULong,
            packet::Packet,
        };

        Box::pin(async move {
            if event.packet_id == CSetActorData::PACKET_ID
                && CSetActorData::updates_key(
                    &event.payload,
                    VarULong(self.rider.entity_id() as u64),
                    pumpkin_world::CURRENT_BEDROCK_MC_PROTOCOL,
                    entity_data_key::POSE_INDEX,
                )
                .unwrap_or(false)
                && !self.fired.swap(true, Ordering::Relaxed)
            {
                assert_eq!(self.rider.get_entity().pose.load(), EntityPose::Standing);
                self.rider.get_entity().set_pose(EntityPose::Crouching);
            }
        })
    }
}

async fn assert_bedrock_pose_after_mount_callback(replace_pose: bool) -> TestResult {
    struct DismountOnMount {
        boat: Arc<BoatEntity>,
        rider_id: i32,
    }

    impl EventRwHandler<GenericGameEvent> for DismountOnMount {
        fn handle<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut GenericGameEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                if event.event_key == GameEvent::EntityMount.name() {
                    self.boat.get_entity().remove_passenger(self.rider_id);
                }
            })
        }
    }

    let directory = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&directory).await?;
    let chunk = ChunkData::empty_sync(0, 0);
    for z in 8..=10 {
        chunk.set_block_absolute_y(8, 63, z, Block::STONE.default_state.id);
    }
    world.level.loaded_chunks.insert(Vector2::new(0, 0), chunk);
    let (player, _peer) = java_rider(&server, &world, "Notch")?;
    let (viewer, mut packets) = bedrock_rider(&server, &world, "jeb_").await?;
    viewer
        .chunk_sender
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .mark_sent_out_of_band(Vector2::new(0, 0));
    let boat = Arc::new(BoatEntity::new(Entity::new(
        world.clone(),
        CENTER,
        &EntityType::OAK_BOAT,
    )));
    world.spawn_entity(boat.clone());
    let rider = player.get_entity();
    rider.set_pos(CENTER);
    rider.set_pose(EntityPose::Crouching);
    world
        .entity_tracker
        .add_entity(&(player.clone() as Arc<dyn EntityBase>), &world);
    assert!(
        world
            .entity_tracker
            .get_tracked_entity(rider.entity_id)
            .is_some_and(|tracked| tracked.is_seen_by(&viewer.gameprofile.id))
    );
    while packets.try_recv().is_ok() {}
    server
        .plugin_manager
        .register_blocking::<GenericGameEvent, _>(
            Arc::new(DismountOnMount {
                boat: boat.clone(),
                rider_id: rider.entity_id,
            }),
            EventPriority::Normal,
            None,
        );
    let observer = Arc::new(ReplaceStandingPose {
        rider: player.clone(),
        fired: AtomicBool::new(false),
    });
    if replace_pose {
        server
            .plugin_manager
            .register_blocking::<PacketSentEvent, _>(observer.clone(), EventPriority::Normal, None);
    }

    boat.get_entity()
        .add_passenger(boat.clone(), player.clone());

    assert!(!rider.has_vehicle());
    assert!(!boat.get_entity().has_passenger(rider.entity_id));
    assert_eq!(observer.fired.load(Ordering::Relaxed), replace_pose);
    let expected = if replace_pose {
        EntityPose::Crouching
    } else {
        EntityPose::Standing
    };
    assert_eq!(rider.pose.load(), expected);
    assert_eq!(
        read_bedrock_seat_metadata(&mut packets, rider.entity_id)?.pose,
        Some(expected as i32),
        "Bedrock viewer received a superseded pose or missed the committed pose"
    );
    world.level.shutdown().await;
    Ok(())
}

#[derive(Default)]
struct BedrockSeatMetadata {
    spawned: bool,
    pairing_marker: bool,
    pose: Option<i32>,
    width: Option<f32>,
    height: Option<f32>,
    scale: Option<f32>,
    flags: Option<i64>,
    flags_two: Option<i64>,
    name: Option<String>,
    last_equipment_id: Option<i16>,
    last_equipment_uid: Option<std::num::NonZeroI32>,
    empty_equipment_updates: usize,
    armor_updates: usize,
    last_head_equipment: Option<(i16, Option<std::num::NonZeroI32>)>,
}

fn read_bedrock_seat_metadata(
    packets: &mut super::vehicle_tests::BedrockQueue,
    actor_id: i32,
) -> Result<BedrockSeatMetadata, Box<dyn std::error::Error>> {
    use pumpkin_protocol::bedrock::client::mob_armor_equipment::CMobArmorEquipment;
    use pumpkin_protocol::{
        bedrock::{
            client::{
                CAddActor, CMobEquipment, CSetActorMotion,
                set_actor_data::{CSetActorData, entity_data_key},
            },
            network_item::NetworkItemStackDescriptor,
        },
        codec::{var_int::VarInt, var_long::VarLong, var_uint::VarUInt, var_ulong::VarULong},
        packet::Packet,
        serial::PacketRead,
    };

    let mut metadata = BedrockSeatMetadata::default();
    while let Ok(packet) = packets.try_recv() {
        let Some((id, payload)) = crate::net::bedrock::split_raw_packet(&packet.data) else {
            continue;
        };
        if i32::from(id) == CAddActor::PACKET_ID {
            metadata.spawned |= VarLong::read(&mut payload.as_ref())?.0 == i64::from(actor_id);
            continue;
        }
        if i32::from(id) == CSetActorMotion::PACKET_ID {
            metadata.pairing_marker |= VarULong::read(&mut payload.as_ref())?.0 == u64::MAX;
            continue;
        }
        if ![
            CSetActorData::PACKET_ID,
            CMobEquipment::PACKET_ID,
            CMobArmorEquipment::PACKET_ID,
        ]
        .contains(&i32::from(id))
        {
            continue;
        }
        let mut data = payload.as_ref();
        if VarULong::read(&mut data)?.0 != actor_id as u64 {
            continue;
        }
        if i32::from(id) == CMobEquipment::PACKET_ID {
            let stack = NetworkItemStackDescriptor::read(&mut data)?;
            metadata.last_equipment_id = Some(stack.id);
            metadata.last_equipment_uid = stack.net_id;
            metadata.empty_equipment_updates += usize::from(stack.id == 0);
            continue;
        }
        if i32::from(id) == CMobArmorEquipment::PACKET_ID {
            metadata.armor_updates += 1;
            let head = NetworkItemStackDescriptor::read(&mut data)?;
            metadata.last_head_equipment = Some((head.id, head.net_id));
            continue;
        }
        let count = VarUInt::read(&mut data)?.0;
        for _ in 0..count {
            let key = VarUInt::read(&mut data)?.0;
            let kind = VarUInt::read(&mut data)?.0;
            assert_eq!(u32::from(u8::read(&mut data)?), kind);
            match (key, kind) {
                (entity_data_key::POSE_INDEX, 2) => {
                    metadata.pose = Some(VarInt::read(&mut data)?.0);
                }
                (entity_data_key::WIDTH, 3) => metadata.width = Some(f32::read(&mut data)?),
                (entity_data_key::HEIGHT, 3) => metadata.height = Some(f32::read(&mut data)?),
                (entity_data_key::SCALE, 3) => metadata.scale = Some(f32::read(&mut data)?),
                (entity_data_key::NAME, 4) => metadata.name = Some(String::read(&mut data)?),
                (entity_data_key::FLAGS, 7) => metadata.flags = Some(VarLong::read(&mut data)?.0),
                (entity_data_key::FLAGS_TWO, 7) => {
                    metadata.flags_two = Some(VarLong::read(&mut data)?.0);
                }
                (_, 0) => {
                    u8::read(&mut data)?;
                }
                (_, 2) => {
                    VarInt::read(&mut data)?;
                }
                (_, 3) => {
                    f32::read(&mut data)?;
                }
                (_, 7) => {
                    VarLong::read(&mut data)?;
                }
                _ => {
                    return Err(
                        format!("Unexpected Bedrock metadata type {kind} for key {key}").into(),
                    );
                }
            }
        }
    }
    Ok(metadata)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bedrock_viewer_keeps_newer_dimensions_when_mount_pose_observer_resizes_rider() -> TestResult
{
    assert_bedrock_metadata_after_mount_resize(MountResize::Baby).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bedrock_viewer_keeps_latest_age_when_an_age_packet_observer_resizes_rider() -> TestResult {
    assert_bedrock_metadata_after_mount_resize(MountResize::RestoreAdult).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bedrock_viewer_keeps_age_and_detachment_when_an_age_observer_dismounts_rider() -> TestResult
{
    assert_bedrock_metadata_after_mount_resize(MountResize::Dismount).await
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum MountResize {
    Baby,
    RestoreAdult,
    Dismount,
}

struct ResizeOnPose {
    boat: Arc<BoatEntity>,
    pig: Arc<PigEntity>,
    made_baby: AtomicBool,
    resize: MountResize,
    acted_on_age: AtomicBool,
}

impl EventRwHandler<PacketSentEvent> for ResizeOnPose {
    fn handle<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut PacketSentEvent,
    ) -> BoxFuture<'a, ()> {
        use crate::entity::ageable::AgeableMob;
        use pumpkin_protocol::{
            bedrock::client::set_actor_data::{CSetActorData, entity_data_key},
            codec::var_ulong::VarULong,
            packet::Packet,
        };

        Box::pin(async move {
            if event.packet_id != CSetActorData::PACKET_ID {
                return;
            }
            let rider = self.pig.get_entity();
            let updates = |key| {
                CSetActorData::updates_key(
                    &event.payload,
                    VarULong(rider.entity_id as u64),
                    pumpkin_world::CURRENT_BEDROCK_MC_PROTOCOL,
                    key,
                )
                .unwrap_or(false)
            };
            if updates(entity_data_key::POSE_INDEX) && !self.made_baby.swap(true, Ordering::Relaxed)
            {
                assert_eq!(rider.pose.load(), EntityPose::Standing);
                self.pig.set_baby(true);
            } else if self.resize != MountResize::Baby
                && updates(entity_data_key::SCALE)
                && !self.acted_on_age.swap(true, Ordering::Relaxed)
            {
                assert!(self.pig.is_baby());
                assert!(rider.has_vehicle());
                if self.resize == MountResize::RestoreAdult {
                    self.pig.set_baby(false);
                } else {
                    self.boat.get_entity().remove_passenger(rider.entity_id);
                    rider.clear_custom_name();
                }
            }
        })
    }
}

async fn assert_bedrock_metadata_after_mount_resize(resize: MountResize) -> TestResult {
    use crate::entity::ageable::AgeableMob;
    use pumpkin_protocol::bedrock::client::set_actor_data::entity_data_flag;

    let directory = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&directory).await?;
    world
        .level
        .loaded_chunks
        .insert(Vector2::new(0, 0), ChunkData::empty_sync(0, 0));
    let (viewer, mut packets) = bedrock_rider(&server, &world, "jeb_").await?;
    viewer
        .chunk_sender
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .mark_sent_out_of_band(Vector2::new(0, 0));
    let boat = Arc::new(BoatEntity::new(Entity::new(
        world.clone(),
        CENTER,
        &EntityType::OAK_BOAT,
    )));
    let pig = PigEntity::new(Entity::new(world.clone(), CENTER, &EntityType::PIG));
    world.spawn_entity(boat.clone());
    world.spawn_entity(pig.clone());
    let rider = pig.get_entity();
    rider.set_pose(EntityPose::Crouching);
    if resize == MountResize::Dismount {
        rider.set_custom_name(pumpkin_util::text::TextComponent::text("Pip"));
    }
    let adult_width = rider.width();
    assert!(!pig.is_baby());
    assert!(
        world
            .entity_tracker
            .get_tracked_entity(rider.entity_id)
            .is_some_and(|tracked| tracked.is_seen_by(&viewer.gameprofile.id))
    );
    while packets.try_recv().is_ok() {}
    let observer = Arc::new(ResizeOnPose {
        boat: boat.clone(),
        pig: pig.clone(),
        made_baby: AtomicBool::new(false),
        resize,
        acted_on_age: AtomicBool::new(false),
    });
    server
        .plugin_manager
        .register_blocking::<PacketSentEvent, _>(observer.clone(), EventPriority::Normal, None);

    boat.get_entity().add_passenger(boat.clone(), pig.clone());

    assert!(observer.made_baby.load(Ordering::Relaxed));
    assert_eq!(
        observer.acted_on_age.load(Ordering::Relaxed),
        resize != MountResize::Baby
    );
    let baby = resize != MountResize::RestoreAdult;
    let mounted = resize != MountResize::Dismount;
    assert_eq!(pig.is_baby(), baby);
    assert_eq!(rider.width() < adult_width, baby);
    assert_eq!(rider.has_vehicle(), mounted);
    assert_eq!(boat.get_entity().has_passenger(rider.entity_id), mounted);
    let metadata = read_bedrock_seat_metadata(&mut packets, rider.entity_id)?;
    assert_eq!(metadata.pose, Some(EntityPose::Standing as i32));
    assert_eq!(
        metadata.width.map(f32::to_bits),
        Some(rider.width().to_bits())
    );
    assert_eq!(
        metadata.height.map(f32::to_bits),
        Some(rider.height().to_bits())
    );
    let expected_scale: f32 = if baby { 0.5 } else { 1.0 };
    assert_eq!(
        metadata.scale.map(f32::to_bits),
        Some(expected_scale.to_bits())
    );
    let flags = metadata.flags.ok_or("Missing rider flags")?;
    assert_eq!(
        flags & (1i64 << entity_data_flag::RIDING) != 0,
        mounted,
        "Last RIDING flag must match the rider's attachment"
    );
    assert_eq!(flags & (1i64 << entity_data_flag::BABY) != 0, baby);
    if !mounted {
        assert!(rider.custom_name.load().is_none());
        assert_eq!(
            metadata.name.as_deref(),
            Some(""),
            "Age update must preserve the cleared name"
        );
    }
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bedrock_viewer_keeps_species_flags_across_mount_growth_and_dismount() -> TestResult {
    use crate::entity::ageable::AgeableMob;
    use crate::entity::mob::Mob;
    use crate::entity::passive::{tamable::TamableAnimal, wolf::WolfEntity};
    use pumpkin_protocol::bedrock::client::set_actor_data::entity_data_flag::{
        BABY, BLOCKING, RIDING, TAMED,
    };

    let directory = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&directory).await?;
    world
        .level
        .loaded_chunks
        .insert(Vector2::new(0, 0), ChunkData::empty_sync(0, 0));
    let (viewer, mut packets) = bedrock_rider(&server, &world, "jeb_").await?;
    viewer
        .chunk_sender
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .mark_sent_out_of_band(Vector2::new(0, 0));
    let boat = Arc::new(BoatEntity::new(Entity::new(
        world.clone(),
        CENTER,
        &EntityType::OAK_BOAT,
    )));
    let wolf = WolfEntity::new(Entity::new(world.clone(), CENTER, &EntityType::WOLF));
    world.spawn_entity(boat.clone());
    world.spawn_entity(wolf.clone());
    let rider = wolf.get_entity();
    assert!(
        world
            .entity_tracker
            .get_tracked_entity(rider.entity_id)
            .is_some_and(|tracked| tracked.is_seen_by(&viewer.gameprofile.id))
    );
    while packets.try_recv().is_ok() {}

    wolf.set_baby(true);
    let baby_scale = read_bedrock_seat_metadata(&mut packets, rider.entity_id)?.scale;
    assert_eq!(baby_scale.map(f32::to_bits), Some(0.55f32.to_bits()));
    wolf.tame(viewer.gameprofile.id);
    let blocking = 1i64 << (BLOCKING - 64);
    wolf.get_mob_entity()
        .living_entity
        .set_bedrock_blocking(true);
    let before = read_bedrock_seat_metadata(&mut packets, rider.entity_id)?;
    let flags = before.flags.ok_or("Missing tame flags")?;
    assert_ne!(flags & (1i64 << TAMED), 0);
    assert_eq!(flags & (1i64 << RIDING), 0);
    assert_eq!(before.flags_two.map(|word| word & blocking), Some(blocking));

    boat.get_entity().add_passenger(boat.clone(), wolf.clone());
    assert_mounted(boat.get_entity(), rider);
    let mounted = read_bedrock_seat_metadata(&mut packets, rider.entity_id)?;
    let flags = mounted.flags.ok_or("Missing mounted flags")?;
    assert_ne!(flags & (1i64 << RIDING), 0);
    assert_ne!(flags & (1i64 << BABY), 0);
    assert_ne!(flags & (1i64 << TAMED), 0, "Mount must preserve TAMED");
    assert_eq!(
        mounted.flags_two.map(|word| word & blocking),
        Some(blocking)
    );

    wolf.set_baby(false);
    assert!(!wolf.is_baby());
    assert_mounted(boat.get_entity(), rider);
    let grown = read_bedrock_seat_metadata(&mut packets, rider.entity_id)?;
    let flags = grown.flags.ok_or("Missing grown-up flags")?;
    assert_eq!(flags & (1i64 << BABY), 0, "Growing up must clear BABY");
    assert_ne!(flags & (1i64 << TAMED), 0, "Growing up must keep TAMED");
    assert_ne!(flags & (1i64 << RIDING), 0, "Growing up must keep RIDING");
    assert_eq!(grown.flags_two.map(|word| word & blocking), Some(blocking));

    boat.get_entity().remove_passenger(rider.entity_id);
    assert!(wolf.is_tame());
    assert!(!rider.has_vehicle());
    assert!(!boat.get_entity().has_passenger(rider.entity_id));
    let detached = read_bedrock_seat_metadata(&mut packets, rider.entity_id)?;
    let flags = detached.flags.ok_or("Missing detached flags")?;
    assert_eq!(flags & (1i64 << RIDING), 0);
    assert_ne!(flags & (1i64 << TAMED), 0, "Dismount must preserve TAMED");
    assert_eq!(
        detached.flags_two.map(|word| word & blocking),
        Some(blocking)
    );
    world.level.shutdown().await;
    Ok(())
}

struct ExpireAngerOnMountFlags {
    world: Arc<World>,
    rider_id: i32,
    expired: AtomicBool,
}

impl EventRwHandler<PacketSentEvent> for ExpireAngerOnMountFlags {
    fn handle<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut PacketSentEvent,
    ) -> BoxFuture<'a, ()> {
        use pumpkin_protocol::{
            bedrock::client::set_actor_data::{CSetActorData, entity_data_key},
            codec::var_ulong::VarULong,
            packet::Packet,
        };
        Box::pin(async move {
            if event.packet_id == CSetActorData::PACKET_ID
                && CSetActorData::updates_key(
                    &event.payload,
                    VarULong(self.rider_id as u64),
                    pumpkin_world::CURRENT_BEDROCK_MC_PROTOCOL,
                    entity_data_key::FLAGS,
                )
                .unwrap_or(false)
                && !self.expired.swap(true, Ordering::Relaxed)
            {
                self.world
                    .level_time
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .world_age += 1;
            }
        })
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bedrock_viewer_gets_current_mount_flags_when_wolf_anger_expires() -> TestResult {
    use crate::entity::{mob::neutral::NeutralMob, passive::wolf::WolfEntity};
    use pumpkin_protocol::bedrock::client::set_actor_data::entity_data_flag::{ANGRY, RIDING};

    let directory = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&directory).await?;
    let chunk = Vector2::new(0, 0);
    world
        .level
        .loaded_chunks
        .insert(chunk, ChunkData::empty_sync(0, 0));
    let (viewer, mut packets) = bedrock_rider(&server, &world, "jeb_").await?;
    viewer
        .chunk_sender
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .mark_sent_out_of_band(chunk);
    let boat = Arc::new(BoatEntity::new(Entity::new(
        world.clone(),
        CENTER,
        &EntityType::OAK_BOAT,
    )));
    let wolf = WolfEntity::new(Entity::new(world.clone(), CENTER, &EntityType::WOLF));
    let rider = wolf.get_entity();
    let now = world
        .level_time
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .world_age;
    wolf.set_anger_end_time(now + 1);
    assert!(wolf.is_angry());
    world.spawn_entity(boat.clone());
    world.spawn_entity(wolf.clone());
    assert!(!rider.has_vehicle());
    assert!(
        world
            .entity_tracker
            .get_tracked_entity(rider.entity_id)
            .is_some_and(|tracked| tracked.is_seen_by(&viewer.gameprofile.id))
    );
    while packets.try_recv().is_ok() {}

    let observer = Arc::new(ExpireAngerOnMountFlags {
        world: world.clone(),
        rider_id: rider.entity_id,
        expired: AtomicBool::new(false),
    });
    server
        .plugin_manager
        .register_blocking::<PacketSentEvent, _>(observer.clone(), EventPriority::Normal, None);
    boat.get_entity().add_passenger(boat.clone(), wolf.clone());
    assert_mounted(boat.get_entity(), rider);
    assert!(observer.expired.load(Ordering::Relaxed));
    assert!(!wolf.is_angry());
    let flags = read_bedrock_seat_metadata(&mut packets, rider.entity_id)?
        .flags
        .ok_or("Missing mounted flags after anger expired")?;
    assert_ne!(flags & (1i64 << RIDING), 0);
    assert_eq!(flags & (1i64 << ANGRY), 0);
    world.level.shutdown().await;
    Ok(())
}

fn java_equipment_watcher(
    server: &Arc<Server>,
    world: &Arc<World>,
) -> Result<(Arc<crate::entity::player::Player>, tokio::io::DuplexStream), Box<dyn std::error::Error>>
{
    use pumpkin_world::cylindrical_chunk_iterator::Cylindrical;
    let chunk = Vector2::new(0, 0);
    world
        .level
        .loaded_chunks
        .insert(chunk, ChunkData::empty_sync(0, 0));
    let (watcher, peer) = java_rider(server, world, "Notch")?;
    let client = watcher.client.java().ok_or("Java watcher")?;
    client.player.store(Arc::new(Some(watcher.clone())));
    watcher.get_entity().set_pos(CENTER.add_raw(3.0, 0.0, 0.0));
    watcher.watched_section.store(Cylindrical::new(
        chunk,
        std::num::NonZeroU8::new(4).ok_or("distance")?,
    ));
    Ok((watcher, peer))
}

fn mob_offhand(
    living: &crate::entity::living::LivingEntity,
) -> pumpkin_data::item_stack::ItemStack {
    living
        .entity_equipment
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .get(&pumpkin_data::data_component_impl::EquipmentSlot::OFF_HAND)
}

async fn read_rider_equipment_updates(
    peer: tokio::io::DuplexStream,
    rider_id: i32,
) -> Result<(Vec<(i8, pumpkin_data::item_stack::ItemStack)>, usize), Box<dyn std::error::Error>> {
    use pumpkin_protocol::{
        ServerPacket,
        java::{
            client::play::{CKeepAlive, CSetEquipment, CSpawnEntity},
            packet_decoder::TCPNetworkDecoder,
        },
    };

    let mut decoder = TCPNetworkDecoder::new(tokio::io::BufReader::new(peer));
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        let mut updates = Vec::new();
        let mut spawns = 0;
        loop {
            let packet = decoder.get_raw_packet().await?;
            let mut payload = packet.payload.as_ref();
            if packet.id == CKeepAlive::to_id(CURRENT_MC_VERSION) {
                assert_eq!(payload.get_i64_be()?, 73);
                return Ok((updates, spawns));
            }
            if packet.id == CSpawnEntity::to_id(CURRENT_MC_VERSION)
                && payload.get_var_int()?.0 == rider_id
            {
                spawns += 1;
                updates.clear();
            }
            if packet.id == CSetEquipment::to_id(CURRENT_MC_VERSION) {
                let packet = CSetEquipment::read(&mut payload, &CURRENT_MC_VERSION)?;
                if packet.entity_id.0 == rider_id {
                    updates.extend(
                        packet
                            .equipment
                            .into_iter()
                            .map(|(slot, stack)| (slot, stack.0.into_owned())),
                    );
                }
            }
        }
    })
    .await?
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PairingEquipmentChange {
    Replace,
    NestedReplace,
    ViewerOverride,
    Wear,
    Break,
    Retry,
    RePair,
    Armor,
    Remove,
    RemoveOnSpawn,
}

struct DetachOnPairingEquipment {
    change: PairingEquipmentChange,
    pig: Arc<PigEntity>,
    boat: Arc<BoatEntity>,
    called: AtomicBool,
    observations: std::sync::atomic::AtomicU32,
}

impl DetachOnPairingEquipment {
    fn change_equipment(&self, event: &mut PacketSentEvent) {
        use crate::entity::mob::Mob;
        use pumpkin_data::{data_component_impl::EquipmentSlot, item::Item, item_stack::ItemStack};
        let rider = self.pig.get_entity();
        let mob = self.pig.get_mob_entity();
        match self.change {
            PairingEquipmentChange::Wear | PairingEquipmentChange::Break => {
                assert!(mob.living_entity.is_blocking());
                mob.living_entity.damage_with_context(
                    self.pig.as_ref(),
                    10.0,
                    pumpkin_data::damage::DamageType::MOB_ATTACK,
                    Some(CENTER.add_raw(0.0, 0.0, 1.0)),
                    None,
                    None,
                );
            }
            PairingEquipmentChange::Armor => {
                mob.set_item_slot(
                    &EquipmentSlot::HEAD,
                    ItemStack::new(1, &Item::DIAMOND_HELMET),
                );
            }
            PairingEquipmentChange::Retry => {}
            PairingEquipmentChange::Remove | PairingEquipmentChange::RemoveOnSpawn => {
                if event.player.client.bedrock().is_some() {
                    Self::append_bedrock_removal_extras(event, rider.entity_id);
                } else {
                    event
                        .extra_packets
                        .push((event.packet_id, event.payload.clone()));
                }
                rider.remove();
            }
            PairingEquipmentChange::RePair => {
                mob.set_item_slot(
                    &EquipmentSlot::MAIN_HAND,
                    ItemStack::new(1, &Item::DIAMOND_SWORD),
                );
                let world = rider.world.load_full();
                if let Some(tracked) = world.entity_tracker.get_tracked_entity(rider.entity_id) {
                    self.observations.store(1, Ordering::Relaxed);
                    tracked.remove_player(&event.player);
                    tracked.update_player(&event.player, &world);
                }
            }
            PairingEquipmentChange::Replace
            | PairingEquipmentChange::NestedReplace
            | PairingEquipmentChange::ViewerOverride => {
                mob.set_item_slot(
                    &EquipmentSlot::MAIN_HAND,
                    ItemStack::new(1, &Item::DIAMOND_SWORD),
                );
                if self.change == PairingEquipmentChange::Replace
                    && event.player.client.java().is_some()
                {
                    Self::append_viewer_equipment(
                        event,
                        rider.entity_id,
                        &[
                            (EquipmentSlot::MAIN_HAND, &Item::STICK),
                            (EquipmentSlot::OFF_HAND, &Item::CARROT),
                            (EquipmentSlot::HEAD, &Item::GOLDEN_HELMET),
                        ],
                    );
                }
            }
        }
    }

    fn append_bedrock_removal_extras(event: &mut PacketSentEvent, actor_id: i32) {
        use pumpkin_protocol::{
            bedrock::client::{CSetActorMotion, mob_armor_equipment::CMobArmorEquipment},
            bedrock::network_item::NetworkItemStackDescriptor,
            packet::Packet,
            serial::PacketWrite,
        };
        let armor = CMobArmorEquipment {
            target_runtime_id: (actor_id as u64).into(),
            head: NetworkItemStackDescriptor::default(),
            chest: NetworkItemStackDescriptor::default(),
            legs: NetworkItemStackDescriptor::default(),
            feet: NetworkItemStackDescriptor::default(),
            body: NetworkItemStackDescriptor::default(),
        };
        let marker = CSetActorMotion {
            target_runtime_id: u64::MAX.into(),
            motion: Vector3::default(),
            tick: 0u64.into(),
        };
        let mut stale = Vec::new();
        let mut unrelated = Vec::new();
        assert!(armor.write(&mut stale).is_ok());
        assert!(marker.write(&mut unrelated).is_ok());
        event
            .extra_packets
            .push((CMobArmorEquipment::PACKET_ID, stale.into()));
        event
            .extra_packets
            .push((CSetActorMotion::PACKET_ID, unrelated.into()));
    }

    fn is_target(&self, event: &PacketSentEvent, rider_id: i32) -> bool {
        use pumpkin_protocol::{
            bedrock::client::{CAddActor, CMobEquipment, mob_armor_equipment::CMobArmorEquipment},
            codec::{var_long::VarLong, var_ulong::VarULong},
            java::client::play::{CHeadRot, CSetEquipment, CSpawnEntity},
            packet::Packet,
            serial::PacketRead,
        };
        if event.player.client.bedrock().is_some() {
            if self.change == PairingEquipmentChange::RemoveOnSpawn {
                event.packet_id == CAddActor::PACKET_ID
                    && VarLong::read(&mut event.payload.as_ref())
                        .is_ok_and(|id| id.0 == i64::from(rider_id))
            } else {
                event.packet_id
                    == if self.change == PairingEquipmentChange::Armor {
                        CMobArmorEquipment::PACKET_ID
                    } else {
                        CMobEquipment::PACKET_ID
                    }
                    && VarULong::read(&mut event.payload.as_ref())
                        .is_ok_and(|id| id.0 == rider_id as u64)
            }
        } else {
            let packet = match self.change {
                PairingEquipmentChange::RePair => CHeadRot::to_id(CURRENT_MC_VERSION),
                PairingEquipmentChange::RemoveOnSpawn => CSpawnEntity::to_id(CURRENT_MC_VERSION),
                _ => CSetEquipment::to_id(CURRENT_MC_VERSION),
            };
            event.packet_id == packet
                && event
                    .payload
                    .as_ref()
                    .get_var_int()
                    .is_ok_and(|id| id.0 == rider_id)
        }
    }

    fn append_viewer_equipment(
        event: &mut PacketSentEvent,
        rider_id: i32,
        slots: &[(
            pumpkin_data::data_component_impl::EquipmentSlot,
            &'static pumpkin_data::item::Item,
        )],
    ) {
        use pumpkin_data::item_stack::ItemStack;
        use pumpkin_protocol::java::client::play::CSetEquipment;
        use pumpkin_protocol::{ClientPacket, codec::item_stack_seralizer::ItemStackSerializer};
        let extra = CSetEquipment::new(
            rider_id.into(),
            slots
                .iter()
                .map(|(slot, item)| {
                    (
                        slot.discriminant(),
                        ItemStackSerializer::from(ItemStack::new(1, item)),
                    )
                })
                .collect(),
        );
        let mut payload = Vec::new();
        assert!(
            extra
                .write_packet_data(&mut payload, &CURRENT_MC_VERSION)
                .is_ok()
        );
        event.extra_packets.push((event.packet_id, payload.into()));
    }
}

impl EventRwHandler<PacketSentEvent> for DetachOnPairingEquipment {
    fn handle<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut PacketSentEvent,
    ) -> BoxFuture<'a, ()> {
        use crate::entity::mob::Mob;
        use pumpkin_data::{data_component_impl::EquipmentSlot, item::Item, item_stack::ItemStack};
        use pumpkin_protocol::java::client::play::CSetEquipment;

        Box::pin(async move {
            let rider = self.pig.get_entity();
            if self.change == PairingEquipmentChange::RePair
                && event.packet_id == CSetEquipment::to_id(CURRENT_MC_VERSION)
                && event
                    .payload
                    .as_ref()
                    .get_var_int()
                    .is_ok_and(|id| id.0 == rider.entity_id)
                && self
                    .observations
                    .compare_exchange(1, 2, Ordering::Relaxed, Ordering::Relaxed)
                    .is_ok()
            {
                Self::append_viewer_equipment(
                    event,
                    rider.entity_id,
                    &[(EquipmentSlot::MAIN_HAND, &Item::CARROT)],
                );
                return;
            }
            if !self.is_target(event, rider.entity_id) {
                return;
            }
            if self.change == PairingEquipmentChange::Retry {
                let attempt = self.observations.fetch_add(1, Ordering::Relaxed);
                if let Some(item) = [&Item::DIAMOND_SWORD, &Item::CARROT].get(attempt as usize) {
                    self.pig
                        .get_mob_entity()
                        .living_entity
                        .entity_equipment
                        .lock()
                        .unwrap_or_else(PoisonError::into_inner)
                        .put(&EquipmentSlot::MAIN_HAND, ItemStack::new(1, item));
                }
            }
            if !self.called.swap(true, Ordering::Relaxed) {
                assert!(rider.has_vehicle());
                assert!(
                    self.pig
                        .get_mob_entity()
                        .living_entity
                        .entity_equipment
                        .try_lock()
                        .is_ok()
                );
                self.boat.get_entity().remove_passenger(rider.entity_id);
                self.change_equipment(event);
            } else if matches!(
                self.change,
                PairingEquipmentChange::NestedReplace | PairingEquipmentChange::ViewerOverride
            ) && self.observations.fetch_add(1, Ordering::Relaxed) == 0
            {
                if self.change == PairingEquipmentChange::NestedReplace {
                    self.pig
                        .get_mob_entity()
                        .set_item_slot(&EquipmentSlot::MAIN_HAND, ItemStack::new(1, &Item::CARROT));
                } else {
                    event.cancelled = true;
                    Self::append_viewer_equipment(
                        event,
                        rider.entity_id,
                        &[(EquipmentSlot::MAIN_HAND, &Item::CARROT)],
                    );
                }
            }
        })
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bedrock_pairing_equipment_observer_can_dismount_the_rider() -> TestResult {
    for change in [
        PairingEquipmentChange::Replace,
        PairingEquipmentChange::NestedReplace,
        PairingEquipmentChange::Armor,
        PairingEquipmentChange::Retry,
        PairingEquipmentChange::Break,
        PairingEquipmentChange::Remove,
        PairingEquipmentChange::RemoveOnSpawn,
    ] {
        assert_bedrock_pairing_equipment(change).await?;
    }
    Ok(())
}

async fn assert_bedrock_pairing_equipment(change: PairingEquipmentChange) -> TestResult {
    use crate::entity::mob::Mob;

    let directory = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&directory).await?;
    let chunk = Vector2::new(0, 0);
    world
        .level
        .loaded_chunks
        .insert(chunk, ChunkData::empty_sync(0, 0));
    let (viewer, mut packets) = bedrock_rider(&server, &world, "jeb_").await?;
    let boat = Arc::new(BoatEntity::new(Entity::new(
        world.clone(),
        CENTER,
        &EntityType::OAK_BOAT,
    )));
    let pig = PigEntity::new(Entity::new(world.clone(), CENTER, &EntityType::PIG));
    world.spawn_entity(boat.clone());
    world.spawn_entity(pig.clone());
    let rider = pig.get_entity();
    boat.get_entity().add_passenger(boat.clone(), pig.clone());
    let (slot, initial_uid) = equip_bedrock_pairing_rider(&pig, change)?;
    let tracked = world
        .entity_tracker
        .get_tracked_entity(rider.entity_id)
        .ok_or("tracker")?;
    assert!(!tracked.is_seen_by(&viewer.gameprofile.id));
    viewer
        .chunk_sender
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .mark_sent_out_of_band(chunk);
    let observer = Arc::new(DetachOnPairingEquipment {
        pig: pig.clone(),
        boat: boat.clone(),
        change,
        called: AtomicBool::new(false),
        observations: std::sync::atomic::AtomicU32::new(0),
    });
    server
        .plugin_manager
        .register_blocking::<PacketSentEvent, _>(observer.clone(), EventPriority::Normal, None);
    if change == PairingEquipmentChange::Retry {
        tracked.last_section_pos.store(Vector3::new(32, 0, 32));
        world.entity_tracker.update_all(&world);
        assert_eq!(observer.observations.load(Ordering::Relaxed), 2);
        world.entity_tracker.update_all(&world);
        assert_eq!(observer.observations.load(Ordering::Relaxed), 3);
    } else if matches!(
        change,
        PairingEquipmentChange::Remove | PairingEquipmentChange::RemoveOnSpawn
    ) {
        run_chunk_pairing(world.clone(), viewer, chunk).await?;
        assert!(rider.is_removed() && observer.called.load(Ordering::Relaxed));
        assert!(
            world
                .entity_tracker
                .get_tracked_entity(rider.entity_id)
                .is_none()
        );
        let wire = read_bedrock_seat_metadata(&mut packets, rider.entity_id)?;
        assert_eq!(wire.spawned, change == PairingEquipmentChange::Remove);
        assert_eq!(wire.last_equipment_id, None);
        assert_eq!(wire.armor_updates, 0);
        assert!(wire.pairing_marker);
        world.level.shutdown().await;
        return Ok(());
    } else {
        tracked.update_player(&viewer, &world);
    }
    assert!(observer.called.load(Ordering::Relaxed));
    assert!(!rider.has_vehicle());
    assert!(!boat.get_entity().has_passenger(rider.entity_id));
    let metadata = read_bedrock_seat_metadata(&mut packets, rider.entity_id)?;
    let equipped = pig
        .get_mob_entity()
        .living_entity
        .entity_equipment
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .get(&slot);
    if change == PairingEquipmentChange::NestedReplace {
        use pumpkin_data::attributes::Attributes;
        let living = &pig.get_mob_entity().living_entity;
        assert_eq!(
            living
                .get_attribute_value(&Attributes::ATTACK_DAMAGE)
                .to_bits(),
            living
                .get_attribute_base(&Attributes::ATTACK_DAMAGE)
                .to_bits()
        );
    }
    assert_bedrock_pairing_equipment_wire(&metadata, &equipped, initial_uid, change)?;
    world.level.shutdown().await;
    Ok(())
}

fn equip_bedrock_pairing_rider(
    pig: &PigEntity,
    change: PairingEquipmentChange,
) -> Result<
    (
        pumpkin_data::data_component_impl::EquipmentSlot,
        std::num::NonZeroI32,
    ),
    Box<dyn std::error::Error>,
> {
    use crate::entity::mob::Mob;
    use pumpkin_data::{data_component_impl::EquipmentSlot, item::Item, item_stack::ItemStack};

    let (slot, item) = match change {
        PairingEquipmentChange::Armor => (EquipmentSlot::HEAD, &Item::GOLDEN_HELMET),
        PairingEquipmentChange::Break => (EquipmentSlot::MAIN_HAND, &Item::SHIELD),
        _ => (EquipmentSlot::MAIN_HAND, &Item::DIAMOND_SWORD),
    };
    let mut initial = ItemStack::new(1, item);
    let mob = pig.get_mob_entity();
    if change == PairingEquipmentChange::Break {
        initial.set_damage(initial.get_max_damage().ok_or("shield durability")? - 1);
        mob.living_entity.set_active_hand(
            pumpkin_util::Hand::Right,
            initial.clone(),
            initial.get_max_use_time() - 20,
        );
    }
    let uid = initial.uid;
    mob.set_item_slot(&slot, initial);
    Ok((slot, uid))
}

async fn run_chunk_pairing(
    world: Arc<World>,
    viewer: Arc<crate::entity::player::Player>,
    chunk: Vector2<i32>,
) -> TestResult {
    let (finished, done) = tokio::sync::oneshot::channel();
    let pairing = std::thread::spawn(move || {
        world
            .entity_tracker
            .update_player_chunks(&viewer, &world, &[chunk]);
        let _ = finished.send(());
    });
    tokio::time::timeout(std::time::Duration::from_secs(5), done).await??;
    pairing.join().map_err(|_| "Chunk pairing panicked")?;
    Ok(())
}

fn assert_bedrock_pairing_equipment_wire(
    metadata: &BedrockSeatMetadata,
    equipped: &pumpkin_data::item_stack::ItemStack,
    initial_uid: std::num::NonZeroI32,
    change: PairingEquipmentChange,
) -> TestResult {
    use pumpkin_data::item::Item;
    use pumpkin_protocol::bedrock::{
        client::set_actor_data::entity_data_flag::RIDING, network_item::NetworkItemStackDescriptor,
    };

    let flags = metadata.flags.ok_or("detached flags")?;
    assert_eq!(flags & (1i64 << RIDING), 0);
    if change == PairingEquipmentChange::Break {
        assert!(equipped.is_empty());
        assert_eq!(metadata.last_equipment_id, Some(0));
        assert_eq!(metadata.empty_equipment_updates, 1);
    } else if change == PairingEquipmentChange::Armor {
        assert_eq!(equipped.item, &Item::DIAMOND_HELMET);
        assert_eq!(metadata.armor_updates, 1);
        let expected = NetworkItemStackDescriptor::from(equipped);
        assert_eq!(
            metadata.last_head_equipment,
            Some((expected.id, expected.net_id))
        );
    } else {
        let expected_item = if matches!(
            change,
            PairingEquipmentChange::Retry | PairingEquipmentChange::NestedReplace
        ) {
            &Item::CARROT
        } else {
            &Item::DIAMOND_SWORD
        };
        assert_eq!(equipped.item, expected_item);
        assert_ne!(initial_uid, equipped.uid);
        let expected = NetworkItemStackDescriptor::from(equipped);
        assert_eq!(metadata.last_equipment_id, Some(expected.id));
        assert_eq!(metadata.last_equipment_uid, expected.net_id);
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn java_pairing_keeps_the_replacement_and_the_unchanged_equipment() -> TestResult {
    for change in [
        PairingEquipmentChange::Replace,
        PairingEquipmentChange::NestedReplace,
        PairingEquipmentChange::ViewerOverride,
        PairingEquipmentChange::Wear,
        PairingEquipmentChange::Break,
        PairingEquipmentChange::Retry,
        PairingEquipmentChange::RePair,
        PairingEquipmentChange::RemoveOnSpawn,
    ] {
        assert_java_pairing_equipment(change).await?;
    }
    Ok(())
}

async fn assert_java_pairing_equipment(change: PairingEquipmentChange) -> TestResult {
    use crate::entity::mob::Mob;
    use pumpkin_data::{data_component_impl::EquipmentSlot, item::Item, item_stack::ItemStack};
    use pumpkin_protocol::java::client::play::CKeepAlive;

    let directory = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&directory).await?;
    let chunk = Vector2::new(0, 0);
    let (viewer, peer) = java_equipment_watcher(&server, &world)?;
    let client = viewer.client.java().ok_or("Java viewer")?;
    let boat = Arc::new(BoatEntity::new(Entity::new(
        world.clone(),
        CENTER,
        &EntityType::OAK_BOAT,
    )));
    let pig = PigEntity::new(Entity::new(world.clone(), CENTER, &EntityType::PIG));
    let rider = pig.get_entity();
    world.spawn_entity(boat.clone());
    world.spawn_entity(pig.clone());
    boat.get_entity().add_passenger(boat.clone(), pig.clone());
    let mob = pig.get_mob_entity();
    if matches!(
        change,
        PairingEquipmentChange::Wear | PairingEquipmentChange::Break
    ) {
        let mut shield = ItemStack::new(1, &Item::SHIELD);
        if change == PairingEquipmentChange::Break {
            shield.set_damage(shield.get_max_damage().ok_or("shield durability")? - 1);
        }
        let duration = shield.get_max_use_time() - 20;
        mob.set_item_slot(&EquipmentSlot::MAIN_HAND, shield.clone());
        mob.living_entity
            .set_active_hand(pumpkin_util::Hand::Right, shield, duration);
    } else {
        mob.set_item_slot(&EquipmentSlot::MAIN_HAND, ItemStack::new(1, &Item::STICK));
    }
    mob.set_item_slot(&EquipmentSlot::OFF_HAND, ItemStack::new(1, &Item::APPLE));
    if change == PairingEquipmentChange::Retry {
        mob.set_item_slot(
            &EquipmentSlot::HEAD,
            ItemStack::new(1, &Item::GOLDEN_HELMET),
        );
    }
    let tracked = world
        .entity_tracker
        .get_tracked_entity(rider.entity_id)
        .ok_or("tracker")?;
    assert!(!tracked.is_seen_by(&viewer.gameprofile.id));
    viewer
        .chunk_sender
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .mark_sent_out_of_band(chunk);
    let observer = Arc::new(DetachOnPairingEquipment {
        pig: pig.clone(),
        boat: boat.clone(),
        change,
        called: AtomicBool::new(false),
        observations: std::sync::atomic::AtomicU32::new(0),
    });
    server
        .plugin_manager
        .register_blocking::<PacketSentEvent, _>(observer.clone(), EventPriority::Normal, None);
    if change == PairingEquipmentChange::Retry {
        tracked.last_section_pos.store(Vector3::new(32, 0, 32));
        world.entity_tracker.update_all(&world);
        assert_eq!(observer.observations.load(Ordering::Relaxed), 2);
        world.entity_tracker.update_all(&world);
        assert_eq!(observer.observations.load(Ordering::Relaxed), 3);
    } else {
        tracked.update_player(&viewer, &world);
    }
    assert!(observer.called.load(Ordering::Relaxed));
    assert!(!rider.has_vehicle());
    assert!(!boat.get_entity().has_passenger(rider.entity_id));
    assert!(client.try_send_packet(&CKeepAlive::new(73)));
    let (wire, spawns) = read_rider_equipment_updates(peer, rider.entity_id).await?;
    if change == PairingEquipmentChange::RemoveOnSpawn {
        assert!(rider.is_removed());
        assert_eq!(spawns, 0);
        assert!(wire.is_empty());
        world.level.shutdown().await;
        return Ok(());
    }
    assert_eq!(
        spawns,
        if change == PairingEquipmentChange::RePair {
            2
        } else {
            1
        }
    );
    assert_java_equipment_wire(&wire, &mob.living_entity, change)?;
    world.level.shutdown().await;
    Ok(())
}

fn assert_java_equipment_wire(
    wire: &[(i8, pumpkin_data::item_stack::ItemStack)],
    living: &crate::entity::living::LivingEntity,
    change: PairingEquipmentChange,
) -> TestResult {
    use pumpkin_data::{attributes::Attributes, data_component_impl::EquipmentSlot, item::Item};
    let last_stack = |slot: &EquipmentSlot| {
        wire.iter()
            .rev()
            .find(|(id, _)| *id == slot.discriminant())
            .map(|(_, stack)| stack)
    };
    let last_item = |slot: &EquipmentSlot| last_stack(slot).map(|stack| stack.item);
    let off_hand = if change == PairingEquipmentChange::Replace {
        &Item::CARROT
    } else {
        &Item::APPLE
    };
    assert_eq!(last_item(&EquipmentSlot::OFF_HAND), Some(off_hand));
    assert_eq!(mob_offhand(living).item, &Item::APPLE);
    if change == PairingEquipmentChange::Wear {
        let main = living
            .entity_equipment
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(&EquipmentSlot::MAIN_HAND);
        assert_eq!(main.get_damage(), 11);
        let delivered = last_stack(&EquipmentSlot::MAIN_HAND).ok_or("shield update")?;
        assert!(delivered.are_equal(&main));
    } else if change == PairingEquipmentChange::Break {
        let clears = wire
            .iter()
            .filter(|(slot, stack)| {
                *slot == EquipmentSlot::MAIN_HAND.discriminant() && stack.is_empty()
            })
            .count();
        assert_eq!(clears, 1);
    } else {
        let main = if matches!(
            change,
            PairingEquipmentChange::Retry
                | PairingEquipmentChange::NestedReplace
                | PairingEquipmentChange::ViewerOverride
                | PairingEquipmentChange::RePair
        ) {
            &Item::CARROT
        } else {
            &Item::DIAMOND_SWORD
        };
        assert_eq!(last_item(&EquipmentSlot::MAIN_HAND), Some(main));
        if change == PairingEquipmentChange::NestedReplace
            || change == PairingEquipmentChange::ViewerOverride
        {
            let equipped = living
                .entity_equipment
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .get(&EquipmentSlot::MAIN_HAND);
            let damage = living.get_attribute_value(&Attributes::ATTACK_DAMAGE);
            let base = living.get_attribute_base(&Attributes::ATTACK_DAMAGE);
            if change == PairingEquipmentChange::NestedReplace {
                assert_eq!(equipped.item, &Item::CARROT);
                assert_eq!(damage.to_bits(), base.to_bits());
            } else {
                assert_eq!(equipped.item, &Item::DIAMOND_SWORD);
                assert!(damage > base);
            }
        }
        if matches!(
            change,
            PairingEquipmentChange::Retry | PairingEquipmentChange::Replace
        ) {
            assert_eq!(last_item(&EquipmentSlot::HEAD), Some(&Item::GOLDEN_HELMET));
        }
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bedrock_viewer_keeps_baby_scale_after_nested_age_callbacks() -> TestResult {
    use crate::entity::ageable::AgeableMob;
    use pumpkin_protocol::{
        bedrock::client::set_actor_data::{CSetActorData, entity_data_flag, entity_data_key},
        codec::var_ulong::VarULong,
        packet::Packet,
    };
    use std::sync::atomic::AtomicU8;
    struct ChangeAge(Arc<PigEntity>, AtomicU8);
    impl EventRwHandler<PacketSentEvent> for ChangeAge {
        fn handle<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut PacketSentEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                let rider = self.0.get_entity();
                if event.packet_id != CSetActorData::PACKET_ID {
                    return;
                }
                let has = |key| {
                    CSetActorData::updates_key(
                        &event.payload,
                        VarULong(rider.entity_id as u64),
                        pumpkin_world::CURRENT_BEDROCK_MC_PROTOCOL,
                        key,
                    )
                    .unwrap_or(false)
                };
                if has(entity_data_key::WIDTH)
                    && self.0.is_baby()
                    && self
                        .1
                        .compare_exchange(0, 1, Ordering::Relaxed, Ordering::Relaxed)
                        .is_ok()
                {
                    self.0.set_baby(false);
                    self.1.store(2, Ordering::Relaxed);
                } else if has(entity_data_key::SCALE)
                    && self
                        .1
                        .compare_exchange(2, 3, Ordering::Relaxed, Ordering::Relaxed)
                        .is_ok()
                {
                    assert!(!self.0.is_baby());
                    self.0.set_baby(true);
                }
            })
        }
    }
    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let chunk = Vector2::new(0, 0);
    world
        .level
        .loaded_chunks
        .insert(chunk, ChunkData::empty_sync(0, 0));
    let (viewer, mut packets) = bedrock_rider(&server, &world, "jeb_").await?;
    viewer
        .chunk_sender
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .mark_sent_out_of_band(chunk);
    let boat = Arc::new(BoatEntity::new(Entity::new(
        world.clone(),
        CENTER,
        &EntityType::OAK_BOAT,
    )));
    let pig = PigEntity::new(Entity::new(world.clone(), CENTER, &EntityType::PIG));
    world.spawn_entity(boat.clone());
    world.spawn_entity(pig.clone());
    boat.get_entity().add_passenger(boat.clone(), pig.clone());
    while packets.try_recv().is_ok() {}
    let observer = Arc::new(ChangeAge(pig.clone(), AtomicU8::new(0)));
    server
        .plugin_manager
        .register_blocking::<PacketSentEvent, _>(observer.clone(), EventPriority::Normal, None);
    pig.set_baby(true);
    let rider = pig.get_entity();
    assert_mounted(boat.get_entity(), rider);
    assert!(pig.is_baby());
    assert_eq!(observer.1.load(Ordering::Relaxed), 3);
    let meta = read_bedrock_seat_metadata(&mut packets, rider.entity_id)?;
    let flags = meta.flags.ok_or("flags")?;
    assert_ne!(flags & (1i64 << entity_data_flag::BABY), 0);
    assert_ne!(flags & (1i64 << entity_data_flag::RIDING), 0);
    assert_eq!(meta.width.map(f32::to_bits), Some(rider.width().to_bits()));
    assert_eq!(
        meta.height.map(f32::to_bits),
        Some(rider.height().to_bits())
    );
    assert_eq!(
        meta.scale.map(f32::to_bits),
        Some(0.5f32.to_bits()),
        "Baby scale after nested adult and baby callbacks"
    );
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bedrock_viewer_gets_current_dismount_flags_after_observer_changes_them() -> TestResult {
    for changes in [1, 2] {
        assert_bedrock_dismount_flags_after_observer_changes(changes).await?;
    }
    Ok(())
}

struct ChangeAiOnDismountFlags {
    pig: Arc<PigEntity>,
    changes: u8,
    late_viewer: Option<Arc<crate::entity::player::Player>>,
    observed: std::sync::atomic::AtomicU8,
}

impl EventRwHandler<PacketSentEvent> for ChangeAiOnDismountFlags {
    fn handle<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut PacketSentEvent,
    ) -> BoxFuture<'a, ()> {
        use crate::entity::mob::Mob;
        use pumpkin_protocol::{
            bedrock::client::set_actor_data::{CSetActorData, entity_data_key},
            codec::var_ulong::VarULong,
            packet::Packet,
        };

        Box::pin(async move {
            let rider = self.pig.get_entity();
            if event.packet_id == CSetActorData::PACKET_ID
                && CSetActorData::updates_key(
                    &event.payload,
                    VarULong(rider.entity_id as u64),
                    pumpkin_world::CURRENT_BEDROCK_MC_PROTOCOL,
                    entity_data_key::FLAGS,
                )
                .unwrap_or(false)
            {
                let count = self.observed.fetch_add(1, Ordering::Relaxed);
                if count < self.changes {
                    assert!(!rider.has_vehicle());
                    if count == 0
                        && let Some(viewer) = &self.late_viewer
                    {
                        let world = rider.world.load_full();
                        let Some(tracked) =
                            world.entity_tracker.get_tracked_entity(rider.entity_id)
                        else {
                            panic!("Rider must remain tracked during dismount");
                        };
                        assert!(!tracked.is_seen_by(&viewer.gameprofile.id));
                        viewer
                            .chunk_sender
                            .lock()
                            .unwrap_or_else(PoisonError::into_inner)
                            .mark_sent_out_of_band(Vector2::new(0, 0));
                        tracked.update_player(viewer, &world);
                        assert!(tracked.is_seen_by(&viewer.gameprofile.id));
                    }
                    self.pig.get_mob_entity().set_no_ai(count.is_multiple_of(2));
                }
            }
        })
    }
}

async fn assert_bedrock_dismount_flags_after_observer_changes(changes: u8) -> TestResult {
    use pumpkin_protocol::bedrock::client::set_actor_data::entity_data_flag;
    use std::sync::atomic::AtomicU8;

    let directory = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&directory).await?;
    let chunk = Vector2::new(0, 0);
    world
        .level
        .loaded_chunks
        .insert(chunk, ChunkData::empty_sync(0, 0));
    let (viewer, mut packets) = bedrock_rider(&server, &world, "jeb_").await?;
    let late = if changes == 1 {
        Some(bedrock_rider(&server, &world, "Notch").await?)
    } else {
        None
    };
    viewer
        .chunk_sender
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .mark_sent_out_of_band(chunk);
    let boat = Arc::new(BoatEntity::new(Entity::new(
        world.clone(),
        CENTER,
        &EntityType::OAK_BOAT,
    )));
    let pig = PigEntity::new(Entity::new(world.clone(), CENTER, &EntityType::PIG));
    world.spawn_entity(boat.clone());
    world.spawn_entity(pig.clone());
    let rider = pig.get_entity();
    boat.get_entity().add_passenger(boat.clone(), pig.clone());
    let before = read_bedrock_seat_metadata(&mut packets, rider.entity_id)?
        .flags
        .ok_or("Missing mounted flags")?;
    let riding = 1i64 << entity_data_flag::RIDING;
    assert_ne!(before & riding, 0);
    let observer = Arc::new(ChangeAiOnDismountFlags {
        pig: pig.clone(),
        changes,
        late_viewer: late.as_ref().map(|(viewer, _)| viewer.clone()),
        observed: AtomicU8::new(0),
    });
    server
        .plugin_manager
        .register_blocking::<PacketSentEvent, _>(observer.clone(), EventPriority::Normal, None);
    boat.get_entity().remove_passenger(rider.entity_id);
    assert!(!rider.has_vehicle());
    assert!(!boat.get_entity().has_passenger(rider.entity_id));
    let mut latest = read_bedrock_seat_metadata(&mut packets, rider.entity_id)?.flags;
    if changes == 2 {
        assert_eq!(observer.observed.load(Ordering::Relaxed), 2);
        assert!(latest.is_none());
        world.entity_tracker.update_all(&world);
        latest = read_bedrock_seat_metadata(&mut packets, rider.entity_id)?.flags;
    }
    let effective = latest.unwrap_or(before);
    assert_eq!(
        effective & riding,
        0,
        "Detached viewer must see RIDING clear"
    );
    assert_eq!(
        effective & (1i64 << entity_data_flag::NO_AI) != 0,
        changes == 1
    );
    if let Some((_, mut late_packets)) = late {
        let late_flags = read_bedrock_seat_metadata(&mut late_packets, rider.entity_id)?.flags;
        assert_eq!(late_flags, Some(effective));
    }
    assert_eq!(observer.observed.load(Ordering::Relaxed), 3);
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn second_boat_mount_evaluates_riding_criteria_for_both_players() -> TestResult {
    use crate::entity::player::advancement::{
        ActiveCriterionListener, AdvancementProgress,
        predicate::{CriterionConditions, EntityPredicate, TagOrIdMatcher},
    };
    use pumpkin_data::{advancement::Advancement, item_stack::ItemStack};

    let directory = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&directory).await?;
    let (first, _first_peer) = java_rider(&server, &world, "Notch")?;
    let (second, _second_peer) = java_rider(&server, &world, "jeb_")?;
    first.get_entity().set_pos(CENTER);
    second.get_entity().set_pos(CENTER);
    let boat = BoatEntity::new(Entity::new(world.clone(), CENTER, &EntityType::OAK_BOAT));
    let boat = Arc::new(boat);
    world.spawn_entity(boat.clone());
    assert!(boat.interact(&first, &mut ItemStack::EMPTY.clone()));
    assert_mounted(boat.get_entity(), first.get_entity());

    let advancement = Advancement::HUSBANDRY_RIDE_A_BOAT_WITH_A_GOAT;
    let listener = ActiveCriterionListener {
        advancement,
        criterion: advancement.criteria[0],
        conditions: Arc::new(CriterionConditions::StartedRiding(Some(EntityPredicate {
            vehicle_type: Some(TagOrIdMatcher {
                patterns: vec!["minecraft:oak_boat".into()],
            }),
            passenger_type: Some(TagOrIdMatcher {
                patterns: vec!["minecraft:player".into()],
            }),
            ..EntityPredicate::default()
        }))),
    };
    for player in [&first, &second] {
        let mut advancements = player
            .advancements
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        advancements.set_player(player);
        assert!(!advancements.progress.map.contains_key(advancement));
        advancements
            .active_listeners
            .insert("minecraft:started_riding", vec![listener.clone()]);
    }
    assert!(boat.interact(&second, &mut ItemStack::EMPTY.clone()));
    assert_mounted(boat.get_entity(), second.get_entity());
    for player in [&first, &second] {
        let advancements = player
            .advancements
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        assert!(
            advancements
                .progress
                .map
                .get(advancement)
                .is_some_and(AdvancementProgress::is_done)
        );
    }
    world.level.shutdown().await;
    Ok(())
}

struct PreventVehicleDamage;

impl EventRwHandler<crate::plugin::api::events::vehicle::vehicle_damage::VehicleDamageEvent>
    for PreventVehicleDamage
{
    fn handle<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut crate::plugin::api::events::vehicle::vehicle_damage::VehicleDamageEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            event.cancelled = true;
        })
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn boat_hands_authority_to_java_rider_mounted_by_paddle_observer() -> TestResult {
    use pumpkin_data::tracked_data::boat::{ID_PADDLE_LEFT, ID_PADDLE_RIGHT};
    use pumpkin_protocol::java::client::play::{CSetEntityMetadata, Metadata};
    use pumpkin_util::math::position::BlockPos;

    struct MountWhenPaddlesClear {
        boat: Arc<BoatEntity>,
        rider: i32,
        cleared_metadata: Vec<u8>,
        ran: AtomicBool,
    }

    impl EventRwHandler<PacketSentEvent> for MountWhenPaddlesClear {
        fn handle<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut PacketSentEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                let hull = self.boat.get_entity();
                let mut payload = event.payload.as_ref();
                if event.packet_id == CSetEntityMetadata::to_id(CURRENT_MC_VERSION)
                    && event.player.entity_id() == self.rider
                    && payload.get_var_int().is_ok_and(|id| id.0 == hull.entity_id)
                    && payload == self.cleared_metadata.as_slice()
                    && !self.ran.swap(true, Ordering::Relaxed)
                {
                    hull.add_passenger(self.boat.clone(), event.player.clone());
                }
            })
        }
    }

    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let chunk_pos = Vector2::new(0, 0);
    let chunk = ChunkData::empty_sync(0, 0);
    let pad = BlockPos::new(7, 64, 8);
    chunk.set_block_absolute_y(7, 63, 8, Block::WATER.default_state.id);
    chunk.set_block_absolute_y(7, 64, 8, Block::LILY_PAD.default_state.id);
    world.level.loaded_chunks.insert(chunk_pos, chunk);
    let (rider, _peer) = java_rider(&server, &world, "Notch")?;
    let client = rider.client.java().ok_or("Missing Java rider")?;
    client.player.store(Arc::new(Some(rider.clone())));
    rider.get_entity().set_pos(CENTER);
    rider
        .watched_section
        .store(pumpkin_world::cylindrical_chunk_iterator::Cylindrical::new(
            chunk_pos,
            std::num::NonZeroU8::new(4).ok_or("Zero view distance")?,
        ));
    rider
        .chunk_sender
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .mark_sent_out_of_band(chunk_pos);

    let start = Vector3::new(10.5, 64.125, 8.5);
    let boat = Arc::new(BoatEntity::new(Entity::new(
        world.clone(),
        start,
        &EntityType::OAK_BOAT,
    )));
    world.spawn_entity(boat.clone());
    let hull = boat.get_entity();
    boat.set_paddles(true, true);
    hull.velocity.store(Vector3::new(0.25, -0.2, 0.0));
    let crossing = Vector3::new(6.0, 0.0, 0.0);
    hull.record_vehicle_movement(start - crossing, crossing, crossing);
    let epoch = hull.vehicle_motion_epoch();
    assert!(!rider.get_entity().has_vehicle());
    assert!(boat.get_controlling_passenger().is_none());
    assert_eq!(world.get_block(&pad), &Block::LILY_PAD);
    assert!(!server.plugin_manager.has_handlers::<VehicleMoveEvent>());

    let mut cleared_metadata = Vec::new();
    for tracked in [ID_PADDLE_LEFT, ID_PADDLE_RIGHT] {
        Metadata::new(tracked, false).write(&mut cleared_metadata, &CURRENT_MC_VERSION)?;
    }
    cleared_metadata.push(255);
    let listener = Arc::new(MountWhenPaddlesClear {
        boat: boat.clone(),
        rider: rider.entity_id(),
        cleared_metadata,
        ran: AtomicBool::new(false),
    });
    server
        .plugin_manager
        .register_blocking::<PacketSentEvent, _>(listener.clone(), EventPriority::Normal, None);

    boat.tick(boat.as_ref(), &server);

    assert!(listener.ran.load(Ordering::Relaxed));
    assert_mounted(hull, rider.get_entity());
    assert_eq!(hull.pos.load(), start);
    assert_eq!(hull.vehicle_motion_epoch(), epoch);
    assert!(Arc::ptr_eq(&world, &hull.world.load()));
    assert_eq!(hull.velocity.load(), Vector3::default());
    assert_eq!(world.get_block(&pad), &Block::AIR);
    client.player.store(Arc::new(None));
    client.close();
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn boat_interaction_rechecks_submersion_after_mount_listener() -> TestResult {
    use crate::plugin::api::events::entity::entity_mount::EntityMountEvent;
    use pumpkin_data::item_stack::ItemStack;

    struct FloodOnMount {
        chunk: Arc<ChunkData>,
        boat_id: i32,
        flooded: AtomicBool,
    }

    impl EventRwHandler<EntityMountEvent> for FloodOnMount {
        fn handle<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut EntityMountEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                if event.mount_id == self.boat_id && !self.flooded.swap(true, Ordering::Relaxed) {
                    self.chunk
                        .set_block_absolute_y(8, 64, 8, Block::WATER.default_state.id);
                    self.chunk
                        .set_block_absolute_y(8, 65, 8, Block::WATER.default_state.id);
                }
            })
        }
    }

    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let chunk = ChunkData::empty_sync(0, 0);
    world
        .level
        .loaded_chunks
        .insert(Vector2::new(0, 0), chunk.clone());
    let (rider, _peer) = java_rider(&server, &world, "Notch")?;
    rider.get_entity().set_pos(CENTER);
    let boat = Arc::new(BoatEntity::new(Entity::new(
        world.clone(),
        CENTER,
        &EntityType::OAK_BOAT,
    )));
    world.spawn_entity(boat.clone());
    let hull = boat.get_entity();
    let observer = Arc::new(FloodOnMount {
        chunk: chunk.clone(),
        boat_id: hull.entity_id,
        flooded: AtomicBool::new(false),
    });
    server
        .plugin_manager
        .register_blocking::<EntityMountEvent, _>(observer.clone(), EventPriority::Normal, None);
    assert!(!hull.is_submerged_in_water());
    let mut hand = ItemStack::EMPTY.clone();

    assert!(!boat.interact(&rider, &mut hand));
    assert!(observer.flooded.load(Ordering::Relaxed));
    assert!(hull.is_submerged_in_water());
    assert!(!rider.get_entity().has_vehicle());
    assert!(!hull.has_passengers());

    chunk.set_block_absolute_y(8, 64, 8, Block::AIR.default_state.id);
    chunk.set_block_absolute_y(8, 65, 8, Block::AIR.default_state.id);
    assert!(boat.interact(&rider, &mut hand));
    assert_mounted(hull, rider.get_entity());
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ride_mount_reports_failure_if_mount_listener_vetoes_attachment() -> TestResult {
    use crate::{
        command::CommandSender, plugin::api::events::entity::entity_mount::EntityMountEvent,
    };

    struct VetoMount(AtomicBool);

    impl EventRwHandler<EntityMountEvent> for VetoMount {
        fn handle<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut EntityMountEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                event.cancelled = self.0.load(Ordering::Relaxed);
            })
        }
    }

    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let (rider, _peer) = java_rider(&server, &world, "Notch")?;
    let boat = Arc::new(BoatEntity::new(Entity::new(
        world.clone(),
        CENTER,
        &EntityType::OAK_BOAT,
    )));
    world.spawn_entity(boat.clone());
    let observer = Arc::new(VetoMount(AtomicBool::new(true)));
    server
        .plugin_manager
        .register_blocking::<EntityMountEvent, _>(observer.clone(), EventPriority::Normal, None);
    let source = CommandSender::Console.into_source(&server).with_silent();
    let mount = || {
        server.command_dispatcher.load().execute_input(
            "ride Notch mount @e[type=minecraft:oak_boat,limit=1]",
            &source,
        )
    };

    assert!(mount().is_err());
    assert!(!rider.get_entity().has_vehicle());
    assert!(!boat.get_entity().has_passengers());

    observer.0.store(false, Ordering::Relaxed);
    assert_eq!(mount().map_err(|error| error.message.get_text())?, 1);
    assert_mounted(boat.get_entity(), rider.get_entity());
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn boat_placement_is_visible_to_a_nested_spawn_packet_observer() -> TestResult {
    assert_reentrant_boat_placement(false).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn boat_placement_commits_cooldown_before_pairing_observers() -> TestResult {
    assert_reentrant_boat_placement(true).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn boat_placement_publishes_remaining_cooldown_after_observers() -> TestResult {
    assert_placement_cooldown_publication(false, CURRENT_MC_VERSION).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn boat_placement_does_not_republish_cleared_cooldown() -> TestResult {
    assert_placement_cooldown_publication(true, CURRENT_MC_VERSION).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn boat_placement_projects_translated_cooldown_packets() -> TestResult {
    assert_placement_cooldown_publication(
        false,
        pumpkin_util::version::JavaMinecraftVersion::V_1_21_4,
    )
    .await
}

const BOAT_PLACEMENT_COOLDOWN_GROUP: &str = "minecraft:test_boat";
const BOAT_PLACEMENT_UNRELATED_COOLDOWN: &str = "minecraft:unrelated";
struct ObserveCooldown {
    clear: bool,
    called: AtomicBool,
    version: pumpkin_util::version::JavaMinecraftVersion,
}
impl EventRwHandler<PacketSentEvent> for ObserveCooldown {
    fn handle<'a>(
        &'a self,
        _: &'a Arc<Server>,
        event: &'a mut PacketSentEvent,
    ) -> BoxFuture<'a, ()> {
        use pumpkin_protocol::{ClientPacket, java::client::play::CItemCooldown};
        Box::pin(async move {
            let packet_id = if self.version == CURRENT_MC_VERSION {
                CItemCooldown::to_id(self.version)
            } else {
                23 // Java 1.21.4 packet registry.
            };
            if (event.packet_id == CItemCooldown::to_id(CURRENT_MC_VERSION)
                || event.packet_id == packet_id)
                && !self.called.swap(true, Ordering::Relaxed)
            {
                let encode = |group: &str, ticks: i32| {
                    let mut payload = Vec::new();
                    assert!(
                        CItemCooldown::new(group.to_owned(), ticks.into())
                            .write_packet_data(&mut payload, &self.version)
                            .is_ok()
                    );
                    payload.into()
                };
                if self.clear {
                    event.player.set_item_cooldown("test_boat", 0);
                } else {
                    event.player.tick_counter.fetch_add(5, Ordering::Relaxed);
                    if self.version != CURRENT_MC_VERSION {
                        event.packet_id = packet_id;
                        event.payload = encode(BOAT_PLACEMENT_COOLDOWN_GROUP, 19);
                    }
                }
                event
                    .extra_packets
                    .push((packet_id, encode("test_boat", 20)));
                event
                    .extra_packets
                    .push((packet_id, encode(BOAT_PLACEMENT_UNRELATED_COOLDOWN, 7)));
            }
        })
    }
}

async fn assert_placement_cooldown_publication(
    clear: bool,
    version: pumpkin_util::version::JavaMinecraftVersion,
) -> TestResult {
    use pumpkin_data::{data_component_impl::UseCooldownImpl, item::Item, item_stack::ItemStack};
    use pumpkin_protocol::java::{
        client::play::{CItemCooldown, CKeepAlive, CSpawnEntity},
        packet_decoder::TCPNetworkDecoder,
    };
    use pumpkin_util::Hand;

    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let (player, peer) = java_equipment_watcher(&server, &world)?;
    let client = player.client.java().ok_or("Java player")?;
    client.version.store(version);
    boat_placement_platform(&world);
    player.get_entity().set_pos(BOAT_PLACEMENT_OFFSET);
    player
        .chunk_sender
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .mark_sent_out_of_band(Vector2::new(0, 0));
    let mut stack = ItemStack::new(1, &Item::OAK_BOAT);
    stack.set_data_component(UseCooldownImpl {
        seconds: 1.0,
        cooldown_group: Some(BOAT_PLACEMENT_COOLDOWN_GROUP.to_owned()),
    });
    player.inventory().set_held_item(stack.clone());
    let observer = Arc::new(ObserveCooldown {
        clear,
        called: AtomicBool::new(false),
        version,
    });
    server
        .plugin_manager
        .register_blocking::<PacketSentEvent, _>(observer.clone(), EventPriority::Normal, None);
    server
        .item_registry
        .on_use_with_rotation(&stack, &player, 0.0, 45.0, Hand::Right);
    assert!(observer.called.load(Ordering::Relaxed));
    assert_eq!(boat_placement_actors(&world).len(), 1);
    let expected = if clear { 0 } else { 15 };
    assert_eq!(
        player
            .get_item_cooldown(BOAT_PLACEMENT_COOLDOWN_GROUP)
            .unwrap_or_default(),
        expected
    );
    assert!(client.try_send_packet(&CKeepAlive::new(73)));
    let mut decoder = TCPNetworkDecoder::new(tokio::io::BufReader::new(peer));
    let updates = tokio::time::timeout(std::time::Duration::from_secs(5), async {
        let mut updates = Vec::new();
        let mut unrelated = Vec::new();
        let mut spawned = false;
        loop {
            let packet = decoder.get_raw_packet().await?;
            if [
                CKeepAlive::to_id(CURRENT_MC_VERSION),
                CKeepAlive::to_id(version),
            ]
            .contains(&packet.id)
            {
                return Ok::<_, Box<dyn std::error::Error>>((updates, unrelated));
            }
            if [
                CSpawnEntity::to_id(CURRENT_MC_VERSION),
                CSpawnEntity::to_id(version),
            ]
            .contains(&packet.id)
            {
                spawned = true;
            }
            let cooldown_packet_id = if version == CURRENT_MC_VERSION {
                CItemCooldown::to_id(version)
            } else {
                23
            };
            if packet.id == cooldown_packet_id {
                assert!(spawned);
                let mut payload = packet.payload.as_ref();
                let length = usize::try_from(payload.get_var_int()?.0)?;
                let (group, mut duration) = payload.split_at(length);
                if matches!(group, b"test_boat" | b"minecraft:test_boat") {
                    updates.push(duration.get_var_int()?.0);
                } else {
                    assert_eq!(group, BOAT_PLACEMENT_UNRELATED_COOLDOWN.as_bytes());
                    unrelated.push(duration.get_var_int()?.0);
                }
            }
        }
    })
    .await??;
    if clear {
        assert_eq!(updates, (vec![0], vec![7]));
    } else {
        assert_eq!(updates, (vec![15, 15], vec![7]));
    }
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn boat_placement_uses_the_same_implicit_and_explicit_cooldown_group() -> TestResult {
    use pumpkin_data::{data_component_impl::UseCooldownImpl, item::Item, item_stack::ItemStack};
    use pumpkin_util::Hand;

    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let (player, _peer) = java_equipment_watcher(&server, &world)?;
    let chunk = boat_placement_platform(&world);
    chunk.set_block_absolute_y(12, 63, 8, Block::STONE.default_state.id);
    player.get_entity().set_pos(BOAT_PLACEMENT_OFFSET);
    let mut implicit = ItemStack::new(1, &Item::OAK_BOAT);
    implicit.set_data_component(UseCooldownImpl::new(1.0, None));
    let mut explicit = implicit.copy_with_count(1);
    explicit.set_data_component(UseCooldownImpl::new(1.0, Some("minecraft:oak_boat".into())));
    let place = |stack: &ItemStack| {
        server
            .item_registry
            .on_use_with_rotation(stack, &player, 0.0, 45.0, Hand::Right);
    };

    player.inventory().set_held_item(implicit.clone());
    place(&implicit);
    assert_eq!(boat_placement_actors(&world).len(), 1);
    assert!(player.inventory().held_item().is_empty());

    player
        .get_entity()
        .set_pos(BOAT_PLACEMENT_OFFSET.add_raw(4.0, 0.0, 0.0));
    player.inventory().set_held_item(explicit.clone());
    place(&explicit);
    assert_eq!(boat_placement_actors(&world).len(), 1);
    assert_eq!(player.inventory().held_item().uid, explicit.uid);

    player.tick_counter.fetch_add(20, Ordering::Relaxed);
    place(&explicit);
    assert_eq!(boat_placement_actors(&world).len(), 2);
    assert!(player.inventory().held_item().is_empty());
    world.level.shutdown().await;
    Ok(())
}

async fn assert_reentrant_boat_placement(second_platform: bool) -> TestResult {
    use pumpkin_data::{
        data_component_impl::UseCooldownImpl, item::Item, item_stack::ItemStack,
        statistic::StatisticCategory,
    };
    use pumpkin_protocol::java::client::play::CSpawnEntity;
    use pumpkin_util::{GameMode, Hand};

    struct PlaceAgain {
        called: AtomicBool,
        second_platform: bool,
    }
    impl EventRwHandler<PacketSentEvent> for PlaceAgain {
        fn handle<'a>(
            &'a self,
            server: &'a Arc<Server>,
            event: &'a mut PacketSentEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                if event.packet_id == CSpawnEntity::to_id(CURRENT_MC_VERSION)
                    && !self.called.swap(true, Ordering::Relaxed)
                {
                    if self.second_platform {
                        event
                            .player
                            .get_entity()
                            .set_pos(BOAT_PLACEMENT_OFFSET.add_raw(4.0, 0.0, 0.0));
                    }
                    let stack = event.player.inventory().held_item();
                    server.item_registry.on_use_with_rotation(
                        &stack,
                        &event.player,
                        0.0,
                        45.0,
                        Hand::Right,
                    );
                }
            })
        }
    }

    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let (player, _peer) = java_equipment_watcher(&server, &world)?;
    let chunk = boat_placement_platform(&world);
    if second_platform {
        chunk.set_block_absolute_y(12, 63, 8, Block::STONE.default_state.id);
    }
    player.get_entity().set_pos(BOAT_PLACEMENT_OFFSET);
    player
        .chunk_sender
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .mark_sent_out_of_band(Vector2::new(0, 0));
    player.gamemode.store(GameMode::Creative);
    let mut stack = ItemStack::new(1, &Item::OAK_BOAT);
    if second_platform {
        stack.set_data_component(UseCooldownImpl {
            seconds: 1.0,
            cooldown_group: Some("minecraft:test_boat".to_owned()),
        });
    }
    player.inventory().set_held_item(stack.clone());
    let observer = Arc::new(PlaceAgain {
        called: AtomicBool::new(false),
        second_platform,
    });
    server
        .plugin_manager
        .register_blocking::<PacketSentEvent, _>(observer.clone(), EventPriority::Normal, None);
    server
        .item_registry
        .on_use_with_rotation(&stack, &player, 0.0, 45.0, Hand::Right);
    assert!(observer.called.load(Ordering::Relaxed));
    assert_eq!(boat_placement_actors(&world).len(), 1);
    assert_eq!(
        player.get_stat(StatisticCategory::Used, i32::from(Item::OAK_BOAT.id)),
        1
    );
    if second_platform {
        assert!(player.is_on_cooldown("minecraft:test_boat"));
        player.tick_counter.fetch_add(20, Ordering::Relaxed);
        server
            .item_registry
            .on_use_with_rotation(&stack, &player, 0.0, 45.0, Hand::Right);
        assert_eq!(boat_placement_actors(&world).len(), 2);
    }
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn boat_placement_removed_during_initial_metadata_is_not_tracked() -> TestResult {
    use pumpkin_data::{item::Item, item_stack::ItemStack};
    use pumpkin_protocol::java::client::play::CSetEntityMetadata;
    use pumpkin_util::Hand;
    use std::sync::atomic::AtomicI32;

    struct RemoveDuringMetadata(AtomicI32);
    impl EventRwHandler<PacketSentEvent> for RemoveDuringMetadata {
        fn handle<'a>(
            &'a self,
            _: &'a Arc<Server>,
            event: &'a mut PacketSentEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                if event.packet_id != CSetEntityMetadata::to_id(CURRENT_MC_VERSION) {
                    return;
                }
                let world = event.player.world();
                if let Ok(id) = event.payload.as_ref().get_var_int()
                    && let Some(boat) = world.get_entity_by_id(id.0)
                    && boat.get_entity().entity_type == &EntityType::OAK_BOAT
                {
                    self.0.store(id.0, Ordering::Relaxed);
                    world.remove_entity(boat.as_ref());
                }
            })
        }
    }

    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let (player, _peer) = java_equipment_watcher(&server, &world)?;
    boat_placement_platform(&world);
    player.get_entity().set_pos(BOAT_PLACEMENT_OFFSET);
    let stack = ItemStack::new(1, &Item::OAK_BOAT);
    player.inventory().set_held_item(stack.clone());
    let observer = Arc::new(RemoveDuringMetadata(AtomicI32::new(-1)));
    server
        .plugin_manager
        .register_blocking::<PacketSentEvent, _>(observer.clone(), EventPriority::Normal, None);
    server
        .item_registry
        .on_use_with_rotation(&stack, &player, 0.0, 45.0, Hand::Right);
    let id = observer.0.load(Ordering::Relaxed);
    assert_ne!(id, -1);
    assert!(world.get_entity_by_id(id).is_none());
    assert!(!world.entity_tracker.has_entity_with_id(id));
    world.level.shutdown().await;
    Ok(())
}

struct ObserveDispense {
    remove: bool,
    id: std::sync::atomic::AtomicI32,
    dispenser: Arc<crate::block::entities::dispenser::DispenserBlockEntity>,
}
impl EventRwHandler<PacketSentEvent> for ObserveDispense {
    fn handle<'a>(
        &'a self,
        server: &'a Arc<Server>,
        event: &'a mut PacketSentEvent,
    ) -> BoxFuture<'a, ()> {
        use pumpkin_inventory::inventory::Inventory;
        use pumpkin_protocol::java::client::play::CSpawnEntity;
        use pumpkin_util::Hand;

        Box::pin(async move {
            if event.packet_id != CSpawnEntity::to_id(CURRENT_MC_VERSION)
                || self.id.load(Ordering::Relaxed) != -1
            {
                return;
            }
            let world = event.player.world();
            if let Ok(id) = event.payload.as_ref().get_var_int()
                && let Some(tracked) = world.entity_tracker.get_tracked_entity(id.0)
                && tracked.entity.get_entity().entity_type == &EntityType::OAK_BOAT
            {
                self.id.store(id.0, Ordering::Relaxed);
                assert!(self.dispenser.get_stack(0).is_empty());
                self.dispenser.set_stack(
                    0,
                    pumpkin_data::item_stack::ItemStack::new(1, &pumpkin_data::item::Item::STICK),
                );
                if self.remove {
                    world.remove_entity(tracked.entity.as_ref());
                } else {
                    let held = event.player.inventory().held_item();
                    server.item_registry.on_use_with_rotation(
                        &held,
                        &event.player,
                        0.0,
                        45.0,
                        Hand::Right,
                    );
                }
            }
        })
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn dispensed_boat_is_live_during_its_spawn_packet_observer() -> TestResult {
    assert_dispensed_boat_registration(false).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn dispensed_boat_removed_from_its_spawn_observer_is_not_reregistered() -> TestResult {
    assert_dispensed_boat_registration(true).await
}

async fn assert_dispensed_boat_registration(remove: bool) -> TestResult {
    use crate::block::{
        BlockBehaviour, OnScheduledTickArgs, blocks::redstone::dispenser::DispenserBlock,
        entities::dispenser::DispenserBlockEntity,
    };
    use pumpkin_data::{
        block_properties::{DispenserLikeProperties, Facing},
        item::Item,
        item_stack::ItemStack,
    };
    use pumpkin_inventory::inventory::Inventory;
    use pumpkin_util::{Hand, math::position::BlockPos};
    use std::sync::atomic::AtomicI32;

    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let (player, _peer) = java_equipment_watcher(&server, &world)?;
    let chunk = boat_placement_platform(&world);
    let position = BlockPos::new(8, 65, 9);
    let mut props = DispenserLikeProperties::default(&Block::DISPENSER);
    props.facing = Facing::North;
    chunk.set_block_absolute_y(8, 65, 9, props.to_state_id(&Block::DISPENSER));
    chunk.set_block_absolute_y(8, 64, 8, Block::WATER.default_state.id);
    chunk.set_block_absolute_y(9, 65, 8, Block::WATER.default_state.id);
    let dispenser = Arc::new(DispenserBlockEntity::new(position));
    world.add_block_entity(dispenser.clone());
    let item = ItemStack::new(1, &Item::OAK_BOAT);
    player.inventory().set_held_item(item.clone());
    player
        .get_entity()
        .set_pos(BOAT_PLACEMENT_OFFSET.add_raw(0.2, 1.0, 0.0));
    player
        .chunk_sender
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .mark_sent_out_of_band(Vector2::new(0, 0));
    let observer = Arc::new(ObserveDispense {
        remove,
        id: AtomicI32::new(-1),
        dispenser: dispenser.clone(),
    });
    server
        .plugin_manager
        .register_blocking::<PacketSentEvent, _>(observer.clone(), EventPriority::Normal, None);
    let dispense = || {
        dispenser.set_stack(0, item.copy_with_count(1));
        DispenserBlock.on_scheduled_tick(OnScheduledTickArgs {
            world: &world,
            block: &Block::DISPENSER,
            position: &position,
        });
    };
    let place = || {
        server
            .item_registry
            .on_use_with_rotation(&item, &player, 0.0, 45.0, Hand::Right);
    };
    dispense();
    let id = observer.id.load(Ordering::Relaxed);
    assert_ne!(id, -1);
    assert_eq!(dispenser.get_stack(0).item, &Item::STICK);
    if remove {
        assert!(world.get_entity_by_id(id).is_none());
        assert!(!world.entity_tracker.has_entity_with_id(id));
        dispense();
        assert_eq!(boat_placement_actors(&world).len(), 1);
    } else {
        let first = world.get_entity_by_id(id).ok_or("Missing dispensed hull")?;
        assert_eq!(first.get_entity().pos.load(), Vector3::new(8.5, 65.5, 8.25));
        let bounds = first.get_entity().bounding_box.load();
        assert_eq!(boat_placement_actors(&world).len(), 1);
        assert_eq!(player.inventory().held_item().uid, item.uid);
        place();
        assert_eq!(boat_placement_actors(&world).len(), 1);
        assert_eq!(player.inventory().held_item().uid, item.uid);
        world.remove_entity(first.as_ref());
        place();
        let boats = boat_placement_actors(&world);
        assert_eq!(boats.len(), 1);
        assert!(bounds.intersects(&boats[0].get_entity().bounding_box.load()));
        assert!(player.inventory().held_item().is_empty());
        dispense();
        assert_eq!(boat_placement_actors(&world).len(), 2);
        assert!(dispenser.get_stack(0).is_empty());
    }
    world.level.shutdown().await;
    Ok(())
}

struct ObserveDispenserEffects {
    position: pumpkin_util::math::position::BlockPos,
    effects: Mutex<Vec<(i32, i32)>>,
    excluded_player: i32,
    excluded_packets: std::sync::atomic::AtomicUsize,
}
impl EventRwHandler<PacketSentEvent> for ObserveDispenserEffects {
    fn handle<'a>(
        &'a self,
        _: &'a Arc<Server>,
        event: &'a mut PacketSentEvent,
    ) -> BoxFuture<'a, ()> {
        use pumpkin_protocol::java::client::play::CWorldEvent;

        Box::pin(async move {
            let mut payload = event.payload.as_ref();
            if event.packet_id == CWorldEvent::to_id(CURRENT_MC_VERSION)
                && let Ok(code) = payload.get_i32_be()
                && payload
                    .get_block_pos(&CURRENT_MC_VERSION)
                    .is_ok_and(|pos| pos == self.position)
                && let Ok(data) = payload.get_i32_be()
            {
                if event.player.entity_id() == self.excluded_player {
                    self.excluded_packets.fetch_add(1, Ordering::Relaxed);
                } else {
                    self.effects
                        .lock()
                        .unwrap_or_else(PoisonError::into_inner)
                        .push((code, data));
                }
            }
        })
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[expect(
    clippy::too_many_lines,
    reason = "One fixture compares dispense effects across both client protocols."
)]
async fn dispensed_boat_preserves_stack_pitch_and_plays_java_effects() -> TestResult {
    use crate::block::{
        BlockBehaviour, OnScheduledTickArgs, blocks::redstone::dispenser::DispenserBlock,
        entities::dispenser::DispenserBlockEntity,
    };
    use pumpkin_data::{
        block_properties::{DispenserLikeProperties, Facing},
        data_component_impl::EntityDataImpl,
        item::Item,
        item_stack::ItemStack,
    };
    use pumpkin_inventory::inventory::Inventory;
    use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};
    use pumpkin_protocol::{
        bedrock::client::level_event::CLevelEvent, codec::var_int::VarInt, packet::Packet,
        serial::PacketRead,
    };
    use pumpkin_util::math::position::BlockPos;

    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let (watcher, _peer) = java_equipment_watcher(&server, &world)?;
    watcher
        .chunk_sender
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .mark_sent_out_of_band(Vector2::new(0, 0));
    let chunk = boat_placement_platform(&world);
    let position = BlockPos::new(8, 65, 9);
    watcher
        .get_entity()
        .set_pos(Vector3::new(-55.75, 65.0, 9.0));
    let (far_watcher, _far_peer) = java_rider(&server, &world, "deadmau5")?;
    let far_client = far_watcher.client.java().ok_or("Java watcher")?;
    far_client.player.store(Arc::new(Some(far_watcher.clone())));
    far_watcher
        .get_entity()
        .set_pos(Vector3::new(72.0, 65.0, 9.0));
    let (bedrock, mut bedrock_packets) = bedrock_rider(&server, &world, "jeb_").await?;
    let dispenser = Arc::new(DispenserBlockEntity::new(position));
    world.add_block_entity(dispenser.clone());
    let observer = Arc::new(ObserveDispenserEffects {
        position,
        effects: Mutex::new(Vec::new()),
        excluded_player: far_watcher.entity_id(),
        excluded_packets: std::sync::atomic::AtomicUsize::new(0),
    });
    server
        .plugin_manager
        .register_blocking::<PacketSentEvent, _>(observer.clone(), EventPriority::Normal, None);
    let mut rotation = NbtCompound::new();
    rotation.put_list("Rotation", vec![NbtTag::Float(70.0), NbtTag::Float(27.0)]);
    let mut stack = ItemStack::new(1, &Item::OAK_BOAT);
    stack.set_data_component(EntityDataImpl {
        entity_type: Some(&EntityType::OAK_BOAT),
        nbt: Some(rotation),
    });

    let dispense = |facing| {
        observer
            .effects
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clear();
        let mut props = DispenserLikeProperties::default(&Block::DISPENSER);
        props.facing = facing;
        chunk.set_block_absolute_y(8, 65, 9, props.to_state_id(&Block::DISPENSER));
        dispenser.set_stack(0, stack.copy_with_count(1));
        DispenserBlock.on_scheduled_tick(OnScheduledTickArgs {
            world: &world,
            block: &Block::DISPENSER,
            position: &position,
        });
        assert!(dispenser.get_stack(0).is_empty());
        observer
            .effects
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    };

    chunk.set_block_absolute_y(9, 64, 9, Block::WATER.default_state.id);
    chunk.set_block_absolute_y(8, 64, 9, Block::WATER.default_state.id);
    for (facing, event_data) in [(Facing::East, 5), (Facing::Down, 0)] {
        let effects = dispense(facing);
        assert_eq!(observer.excluded_packets.load(Ordering::Relaxed), 0);
        assert_eq!(effects, [(1000, 0), (2000, event_data)]);
        let boats = boat_placement_actors(&world);
        assert_eq!(boats.len(), 1);
        let hull = boats[0].get_entity();
        assert_eq!((hull.yaw.load(), hull.pitch.load()), (270.0, 27.0));
        world.remove_entity(boats[0].as_ref());
    }
    assert_eq!(
        dispense(Facing::North),
        [(1000, 0), (2000, 2), (1000, 0), (2000, 2)]
    );
    assert!(boat_placement_actors(&world).is_empty());
    assert_eq!(
        world
            .entities
            .load()
            .iter()
            .filter_map(|entity| entity
                .cast_any()
                .downcast_ref::<crate::entity::item::ItemEntity>())
            .map(crate::entity::item::ItemEntity::get_pickup_delay)
            .collect::<Vec<_>>(),
        [0]
    );
    assert_eq!(observer.excluded_packets.load(Ordering::Relaxed), 0);
    let mut bedrock_effects = Vec::new();
    while let Ok(packet) = bedrock_packets.try_recv() {
        if let Some((id, payload)) = crate::net::bedrock::split_raw_packet(&packet.data)
            && i32::from(id) == CLevelEvent::PACKET_ID
        {
            let mut input = payload.as_ref();
            let code = VarInt::read(&mut input)?.0;
            let _position = Vector3::<f32>::read(&mut input)?;
            let data = VarInt::read(&mut input)?.0;
            bedrock_effects.push((code, data));
        }
    }
    assert_eq!(
        bedrock_effects,
        [
            (1000, 0),
            (2000, 5),
            (1000, 0),
            (2000, 4),
            (1000, 0),
            (2000, 1),
            (1000, 0),
            (2000, 1)
        ]
    );
    let bedrock_client = bedrock.client.bedrock().ok_or("Bedrock watcher")?;
    bedrock_client.clear_player();
    bedrock_client.close().await;
    far_client.close();
    far_client.player.store(Arc::new(None));
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn dispensed_boat_removed_during_dry_ejection_stays_removed() -> TestResult {
    use crate::block::{
        BlockBehaviour, OnScheduledTickArgs, blocks::redstone::dispenser::DispenserBlock,
        entities::dispenser::DispenserBlockEntity,
    };
    use pumpkin_data::{
        block_properties::{DispenserLikeProperties, Facing},
        item::Item,
        item_stack::ItemStack,
    };
    use pumpkin_inventory::inventory::Inventory;
    use pumpkin_protocol::java::client::play::CSpawnEntity;
    use pumpkin_util::math::position::BlockPos;
    use std::sync::atomic::AtomicI32;

    struct RemoveEjection(AtomicI32);
    impl EventRwHandler<PacketSentEvent> for RemoveEjection {
        fn handle<'a>(
            &'a self,
            _: &'a Arc<Server>,
            event: &'a mut PacketSentEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                let world = event.player.world();
                if event.packet_id == CSpawnEntity::to_id(CURRENT_MC_VERSION)
                    && let Ok(id) = event.payload.as_ref().get_var_int()
                    && let Some(tracked) = world.entity_tracker.get_tracked_entity(id.0)
                    && tracked.entity.get_entity().entity_type == &EntityType::ITEM
                {
                    world.remove_entity(tracked.entity.as_ref());
                    self.0.store(id.0, Ordering::Relaxed);
                }
            })
        }
    }

    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let (watcher, _peer) = java_equipment_watcher(&server, &world)?;
    watcher
        .chunk_sender
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .mark_sent_out_of_band(Vector2::new(0, 0));
    let chunk = boat_placement_platform(&world);
    let position = BlockPos::new(8, 65, 9);
    let mut properties = DispenserLikeProperties::default(&Block::DISPENSER);
    properties.facing = Facing::North;
    chunk.set_block_absolute_y(8, 65, 9, properties.to_state_id(&Block::DISPENSER));
    let dispenser = Arc::new(DispenserBlockEntity::new(position));
    dispenser.set_stack(0, ItemStack::new(1, &Item::OAK_BOAT));
    world.add_block_entity(dispenser.clone());

    let observer = Arc::new(RemoveEjection(AtomicI32::new(-1)));
    server
        .plugin_manager
        .register_blocking::<PacketSentEvent, _>(observer.clone(), EventPriority::Normal, None);
    DispenserBlock.on_scheduled_tick(OnScheduledTickArgs {
        world: &world,
        block: &Block::DISPENSER,
        position: &position,
    });

    let id = observer.0.load(Ordering::Relaxed);
    assert_ne!(id, -1);
    assert!(dispenser.get_stack(0).is_empty());
    assert!(world.get_entity_by_id(id).is_none());
    assert!(!world.entity_tracker.has_entity_with_id(id));
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn dispensed_boat_rejects_equal_stack_replaced_by_spawn_observer() -> TestResult {
    assert_dispensed_boat_rejects_replaced_source(false).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn dispensed_boat_rejects_dispenser_replaced_by_spawn_observer() -> TestResult {
    assert_dispensed_boat_rejects_replaced_source(true).await
}

async fn assert_dispensed_boat_rejects_replaced_source(replace_dispenser: bool) -> TestResult {
    use crate::{
        block::{
            BlockBehaviour, OnScheduledTickArgs, blocks::redstone::dispenser::DispenserBlock,
            entities::dispenser::DispenserBlockEntity,
        },
        plugin::api::events::entity::entity_spawn::EntitySpawnEvent,
    };
    use pumpkin_data::{
        block_properties::{DispenserLikeProperties, Facing},
        item::Item,
        item_stack::ItemStack,
    };
    use pumpkin_inventory::inventory::Inventory;
    use pumpkin_util::math::position::BlockPos;

    struct ReplaceSource {
        world: Arc<World>,
        original: Arc<DispenserBlockEntity>,
        replacement: Arc<DispenserBlockEntity>,
        stack: ItemStack,
    }
    impl EventRwHandler<EntitySpawnEvent> for ReplaceSource {
        fn handle<'a>(
            &'a self,
            _: &'a Arc<Server>,
            _: &'a mut EntitySpawnEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                self.replacement.set_stack(0, self.stack.clone());
                if !Arc::ptr_eq(&self.original, &self.replacement) {
                    self.world.add_block_entity(self.replacement.clone());
                }
            })
        }
    }

    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let chunk = boat_placement_platform(&world);
    let position = BlockPos::new(8, 65, 9);
    let mut props = DispenserLikeProperties::default(&Block::DISPENSER);
    props.facing = Facing::North;
    chunk.set_block_absolute_y(8, 65, 9, props.to_state_id(&Block::DISPENSER));
    chunk.set_block_absolute_y(8, 64, 8, Block::WATER.default_state.id);
    let dispenser = Arc::new(DispenserBlockEntity::new(position));
    world.add_block_entity(dispenser.clone());
    let stack = ItemStack::new(1, &Item::OAK_BOAT);
    let replacement = stack.copy_with_count(1);
    assert_ne!(replacement.uid, stack.uid);
    let original_uid = stack.uid;
    dispenser.set_stack(0, stack);
    let current = if replace_dispenser {
        Arc::new(DispenserBlockEntity::new(position))
    } else {
        dispenser.clone()
    };
    server
        .plugin_manager
        .register_blocking::<EntitySpawnEvent, _>(
            Arc::new(ReplaceSource {
                world: world.clone(),
                original: dispenser.clone(),
                replacement: current.clone(),
                stack: replacement.clone(),
            }),
            EventPriority::Normal,
            None,
        );
    DispenserBlock.on_scheduled_tick(OnScheduledTickArgs {
        world: &world,
        block: &Block::DISPENSER,
        position: &position,
    });
    assert!(boat_placement_actors(&world).is_empty());
    assert_eq!(current.get_stack(0).uid, replacement.uid);
    assert_eq!(current.get_stack(0).item_count, 1);
    if replace_dispenser {
        assert_eq!(dispenser.get_stack(0).uid, original_uid);
        assert_eq!(dispenser.get_stack(0).item_count, 1);
    }
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn concurrent_boat_placements_commit_only_one_overlapping_hull() -> TestResult {
    use crate::plugin::api::events::entity::entity_spawn::EntitySpawnEvent;
    use pumpkin_data::{item::Item, item_stack::ItemStack, statistic::StatisticCategory};
    use pumpkin_util::Hand;
    use std::sync::atomic::AtomicUsize;

    struct WaitForBothPlacements(tokio::sync::Barrier, AtomicUsize);
    impl EventRwHandler<EntitySpawnEvent> for WaitForBothPlacements {
        fn handle<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut EntitySpawnEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                if event.entity_type == EntityType::OAK_BOAT.id.to_string() {
                    self.1.fetch_add(1, Ordering::Relaxed);
                    assert!(
                        tokio::time::timeout(std::time::Duration::from_secs(5), self.0.wait())
                            .await
                            .is_ok(),
                        "Both placement attempts should reach the spawn event"
                    );
                }
            })
        }
    }

    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let (first, _first_peer) = java_rider(&server, &world, "Notch")?;
    let (second, _second_peer) = java_rider(&server, &world, "jeb_")?;
    boat_placement_platform(&world);
    first.get_entity().set_pos(BOAT_PLACEMENT_OFFSET);
    second
        .get_entity()
        .set_pos(BOAT_PLACEMENT_OFFSET.add_raw(-0.75, 0.0, 0.0));
    let barrier = Arc::new(WaitForBothPlacements(
        tokio::sync::Barrier::new(2),
        AtomicUsize::new(0),
    ));
    server
        .plugin_manager
        .register_blocking::<EntitySpawnEvent, _>(barrier.clone(), EventPriority::Normal, None);
    let place = |player: Arc<crate::entity::player::Player>| {
        let stack = ItemStack::new(1, &Item::OAK_BOAT);
        player.inventory().set_held_item(stack.clone());
        let server = server.clone();
        tokio::task::spawn_blocking(move || {
            server
                .item_registry
                .on_use_with_rotation(&stack, &player, 0.0, 45.0, Hand::Right);
        })
    };
    let first_request = place(first.clone());
    let second_request = place(second.clone());
    first_request.await?;
    second_request.await?;
    assert_eq!(barrier.1.load(Ordering::Relaxed), 2);

    assert_eq!(boat_placement_actors(&world).len(), 1);
    let mut outcomes = [first, second].map(|player| {
        (
            player.get_stat(StatisticCategory::Used, i32::from(Item::OAK_BOAT.id)),
            player.inventory().held_item().item_count,
        )
    });
    outcomes.sort_unstable();
    assert_eq!(outcomes, [(0, 1), (1, 0)]);
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bedrock_boat_placement_uses_the_selected_hotbar_slot() -> TestResult {
    use crate::plugin::api::events::player::item_held::PlayerItemHeldEvent;
    use pumpkin_data::{item::Item, item_stack::ItemStack, statistic::StatisticCategory};
    use std::sync::atomic::AtomicUsize;

    struct KeepSelected;
    impl EventRwHandler<PlayerItemHeldEvent> for KeepSelected {
        fn handle<'a>(
            &'a self,
            _: &'a Arc<Server>,
            event: &'a mut PlayerItemHeldEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move { event.cancelled = true })
        }
    }
    struct MoveOnUse {
        enabled: AtomicBool,
        calls: AtomicUsize,
    }
    impl EventRwHandler<PlayerInteractEvent> for MoveOnUse {
        fn handle<'a>(
            &'a self,
            _: &'a Arc<Server>,
            event: &'a mut PlayerInteractEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                self.calls.fetch_add(1, Ordering::Relaxed);
                if self.enabled.swap(false, Ordering::Relaxed) {
                    let inventory = event.player.inventory();
                    let same_boat = inventory.held_item();
                    inventory.set_slot(0, ItemStack::EMPTY.clone());
                    inventory.set_slot(2, same_boat);
                    inventory.set_selected_slot(2);
                }
            })
        }
    }

    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    boat_placement_platform(&world);
    let (player, mut packets) = bedrock_rider(&server, &world, "jeb_").await?;
    player.get_entity().set_pos(BOAT_PLACEMENT_OFFSET);
    player.get_entity().set_rotation(0.0, 45.0);
    let boat = ItemStack::new(1, &Item::OAK_BOAT);
    player.inventory().set_slot(0, boat.clone());
    let other = boat.copy_with_count(1);
    player.inventory().set_slot(1, other.clone());
    server
        .plugin_manager
        .register_blocking::<PlayerItemHeldEvent, _>(
            Arc::new(KeepSelected),
            EventPriority::Normal,
            None,
        );
    let client = player.client.bedrock().ok_or("Bedrock player")?;
    client.select_hotbar_slot(&server, &player, 1, 0);
    assert_eq!(player.inventory().get_selected_slot(), 0);
    let observer = Arc::new(MoveOnUse {
        enabled: AtomicBool::new(false),
        calls: AtomicUsize::new(0),
    });
    server
        .plugin_manager
        .register_blocking::<PlayerInteractEvent, _>(observer.clone(), EventPriority::Normal, None);

    bedrock_boat_click(&player, 1, &other)?;
    assert_eq!(observer.calls.load(Ordering::Relaxed), 0);
    assert!(boat_placement_actors(&world).is_empty());
    assert_eq!(player.inventory().get_slot(0).uid, boat.uid);
    assert_eq!(
        player.get_stat(StatisticCategory::Used, i32::from(Item::OAK_BOAT.id)),
        0
    );
    observer.enabled.store(true, Ordering::Relaxed);
    bedrock_boat_click(&player, 0, &boat)?;
    assert_eq!(observer.calls.load(Ordering::Relaxed), 1);
    assert_eq!(player.inventory().get_selected_slot(), 2);
    assert_eq!(player.inventory().held_item().uid, boat.uid);
    assert!(boat_placement_actors(&world).is_empty());

    let mut packet = bedrock_use_item_transaction(2, &boat, 1);
    packet.legacy_request_id = pumpkin_protocol::codec::var_int::VarInt(-5);
    client.handle_inventory_action(&player, packet);
    assert_eq!(bedrock_boat_legacy_responses(&mut packets)?, [(-5, 0)]);
    assert_eq!(boat_placement_actors(&world).len(), 1);
    assert!(player.inventory().held_item().is_empty());
    assert_eq!(player.inventory().get_slot(1).uid, other.uid);
    assert_eq!(
        player.get_stat(StatisticCategory::Used, i32::from(Item::OAK_BOAT.id)),
        1
    );
    client.clear_player();
    client.close().await;
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bedrock_boat_placement_rejects_a_different_boat_in_the_same_slot() -> TestResult {
    use pumpkin_data::{item::Item, item_stack::ItemStack, statistic::StatisticCategory};

    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    boat_placement_platform(&world);
    let (player, _packets) = bedrock_rider(&server, &world, "jeb_").await?;
    player.get_entity().set_pos(BOAT_PLACEMENT_OFFSET);
    player.get_entity().set_rotation(0.0, 45.0);
    let old = ItemStack::new(1, &Item::OAK_BOAT);
    let chest_boat = ItemStack::new(1, &Item::OAK_CHEST_BOAT);
    player.inventory().set_held_item(old.clone());
    player.inventory().set_held_item(chest_boat.clone());
    let placed = || {
        world
            .entities
            .load()
            .iter()
            .filter(|entity| {
                let kind = entity.get_entity().entity_type;
                kind == &EntityType::OAK_CHEST_BOAT || kind == &EntityType::OAK_BOAT
            })
            .count()
    };
    bedrock_boat_click(&player, 0, &old)?;
    assert_eq!(placed(), 0);
    assert_eq!(player.inventory().held_item().uid, chest_boat.uid);
    bedrock_boat_interaction(&player, 0, &old, 0)?;
    assert_eq!(placed(), 0);
    assert_eq!(player.inventory().held_item().item, &Item::OAK_CHEST_BOAT);
    assert_eq!(player.inventory().held_item().uid, chest_boat.uid);
    assert_eq!(
        player.get_stat(StatisticCategory::Used, i32::from(Item::OAK_BOAT.id)),
        0
    );
    bedrock_boat_click(&player, 0, &chest_boat)?;
    assert_eq!(placed(), 1);
    assert!(player.inventory().held_item().is_empty());
    assert_eq!(
        player.get_stat(StatisticCategory::Used, i32::from(Item::OAK_CHEST_BOAT.id)),
        1
    );
    let client = player.client.bedrock().ok_or("Bedrock player")?;
    client.clear_player();
    client.close().await;
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bedrock_boat_block_click_debits_the_original_slot() -> TestResult {
    use crate::{
        block::entities::decorated_pot::DecoratedPotBlockEntity,
        plugin::api::events::player::player_statistic_increment::PlayerStatisticIncrementEvent,
    };
    use pumpkin_data::{item::Item, item_stack::ItemStack, statistic::StatisticCategory};
    use pumpkin_inventory::player::player_inventory::PlayerInventory;
    use pumpkin_protocol::bedrock::{
        network_item::NetworkItemDescriptor, server::inventory_transaction::InventoryAction,
    };
    use pumpkin_util::math::position::BlockPos;

    struct MoveOriginalAndSelectOtherSlot;
    impl EventRwHandler<PlayerStatisticIncrementEvent> for MoveOriginalAndSelectOtherSlot {
        fn handle<'a>(
            &'a self,
            _: &'a Arc<Server>,
            event: &'a mut PlayerStatisticIncrementEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                let inventory = event.player.inventory();
                let original = inventory.get_slot(1);
                inventory.set_slot(1, ItemStack::EMPTY.clone());
                inventory.set_slot(PlayerInventory::OFF_HAND_SLOT, original);
                inventory.set_selected_slot(0);
            })
        }
    }

    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let chunk = boat_placement_platform(&world);
    chunk.set_block_absolute_y(8, 63, 8, Block::DECORATED_POT.default_state.id);
    let pot = Arc::new(DecoratedPotBlockEntity::new(BlockPos::new(8, 63, 8)));
    world.add_block_entity(pot.clone());
    let (player, mut packets) = bedrock_rider(&server, &world, "jeb_").await?;
    player.get_entity().set_pos(BOAT_PLACEMENT_OFFSET);
    let first = ItemStack::new(1, &Item::OAK_BOAT);
    let second = first.copy_with_count(1);
    let inventory = player.inventory();
    inventory.set_slot(0, first.clone());
    inventory.set_slot(1, second.clone());
    inventory.set_selected_slot(1);

    let client = player.client.bedrock().ok_or("Bedrock player")?;
    let predicted_deposit = |slot, stack: &ItemStack, request| {
        let mut packet = bedrock_use_item_transaction(slot, stack, 0);
        packet.legacy_request_id = pumpkin_protocol::codec::var_int::VarInt(request);
        packet.actions.push(InventoryAction {
            source_type: 0,
            window_id: Some(0),
            source_flags: None,
            inventory_slot: u32::try_from(slot)?,
            old_item: NetworkItemDescriptor::from(stack),
            new_item: NetworkItemDescriptor::from(ItemStack::EMPTY),
        });
        client.handle_inventory_action(&player, packet);
        Ok::<_, Box<dyn std::error::Error>>(())
    };
    predicted_deposit(0, &first, -1)?;
    assert!(pot.get_item().is_none_or(|stack| stack.is_empty()));
    assert_eq!(inventory.get_slot(0).uid, first.uid);
    assert_eq!(inventory.get_slot(1).uid, second.uid);
    assert!(boat_placement_actors(&world).is_empty());

    server
        .plugin_manager
        .register_blocking::<PlayerStatisticIncrementEvent, _>(
            Arc::new(MoveOriginalAndSelectOtherSlot),
            EventPriority::Normal,
            None,
        );
    predicted_deposit(1, &second, -3)?;
    assert_eq!(inventory.get_selected_slot(), 0);
    assert_eq!(inventory.get_slot(0).uid, first.uid);
    assert!(inventory.get_slot(1).is_empty());
    assert!(
        inventory
            .get_slot(PlayerInventory::OFF_HAND_SLOT)
            .is_empty()
    );
    let contained = pot.get_item().ok_or("The pot should contain the boat")?;
    assert_eq!(contained.item, &Item::OAK_BOAT);
    assert_eq!(contained.item_count, 1);
    assert_eq!(
        player.get_stat(StatisticCategory::Used, i32::from(Item::OAK_BOAT.id)),
        1,
    );
    assert_eq!(
        bedrock_boat_legacy_responses(&mut packets)?,
        [(-1, 1), (-3, 0)]
    );
    client.clear_player();
    client.close().await;
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[expect(
    clippy::too_many_lines,
    reason = "The lever, item and pot share one addressed offhand."
)]
async fn bedrock_boat_in_main_hand_preserves_offhand_block_and_item_actions() -> TestResult {
    use crate::{
        block::entities::decorated_pot::DecoratedPotBlockEntity,
        plugin::api::events::player::player_statistic_increment::PlayerStatisticIncrementEvent,
    };
    use pumpkin_data::{block_properties::LeverLikeProperties, item::Item, item_stack::ItemStack};
    use pumpkin_inventory::player::player_inventory::PlayerInventory;
    use pumpkin_protocol::bedrock::server::inventory_transaction::{HandSlot, TransactionData};

    struct MoveOffhandAtInsertion;
    impl EventRwHandler<PlayerStatisticIncrementEvent> for MoveOffhandAtInsertion {
        fn handle<'a>(
            &'a self,
            _: &'a Arc<Server>,
            event: &'a mut PlayerStatisticIncrementEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                let inventory = event.player.inventory();
                let offhand = inventory.get_slot(PlayerInventory::OFF_HAND_SLOT);
                inventory.set_slot(PlayerInventory::OFF_HAND_SLOT, ItemStack::EMPTY.clone());
                inventory.set_slot(2, offhand);
            })
        }
    }

    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let chunk = boat_placement_platform(&world);
    chunk.set_block_absolute_y(8, 63, 8, Block::LEVER.default_state.id);
    let (player, _packets) = bedrock_rider(&server, &world, "jeb_").await?;
    player.get_entity().set_pos(BOAT_PLACEMENT_OFFSET);
    let boat = ItemStack::new(1, &Item::OAK_BOAT);
    player.inventory().set_held_item(boat.clone());

    let mut packet = bedrock_use_item_transaction(0, ItemStack::EMPTY, 0);
    let TransactionData::UseItem(data) = &mut packet.transaction_data else {
        return Err("Expected an item-use transaction".into());
    };
    data.hand = HandSlot::Offhand;
    let position = data.block_position;
    let powered =
        || LeverLikeProperties::from_state_id(world.get_block_state_id(&position)).powered;
    let client = player.client.bedrock().ok_or("Bedrock player")?;
    assert!(!powered());
    client.handle_inventory_action(&player, packet);
    assert!(powered());
    assert_eq!(player.inventory().held_item().uid, boat.uid);

    chunk.set_block_absolute_y(8, 63, 8, Block::STONE.default_state.id);
    let rocket = ItemStack::new(1, &Item::FIREWORK_ROCKET);
    player
        .inventory()
        .set_slot(PlayerInventory::OFF_HAND_SLOT, rocket.clone());
    let mut packet = bedrock_use_item_transaction(0, &rocket, 0);
    if let TransactionData::UseItem(data) = &mut packet.transaction_data {
        data.hand = HandSlot::Offhand;
    }
    client.handle_inventory_action(&player, packet);
    assert_eq!(player.inventory().held_item().uid, boat.uid);
    assert!(
        player
            .inventory()
            .get_slot(PlayerInventory::OFF_HAND_SLOT)
            .is_empty()
    );
    assert_eq!(
        world
            .entities
            .load()
            .iter()
            .filter(|entity| entity.get_entity().entity_type == &EntityType::FIREWORK_ROCKET)
            .count(),
        1
    );
    chunk.set_block_absolute_y(8, 63, 8, Block::DECORATED_POT.default_state.id);
    let pot = Arc::new(DecoratedPotBlockEntity::new(position));
    world.add_block_entity(pot.clone());
    server
        .plugin_manager
        .register_blocking::<PlayerStatisticIncrementEvent, _>(
            Arc::new(MoveOffhandAtInsertion),
            EventPriority::Normal,
            None,
        );
    player
        .inventory()
        .set_slot(PlayerInventory::OFF_HAND_SLOT, rocket.clone());
    let mut deposit = bedrock_use_item_transaction(0, &rocket, 0);
    if let TransactionData::UseItem(data) = &mut deposit.transaction_data {
        data.hand = HandSlot::Offhand;
        data.item_in_hand_network_id = Some(rocket.uid);
    }
    client.handle_inventory_action(&player, deposit);
    assert_eq!(player.inventory().held_item().uid, boat.uid);
    assert!(player.inventory().get_slot(2).is_empty());
    assert!(
        player
            .inventory()
            .get_slot(PlayerInventory::OFF_HAND_SLOT)
            .is_empty()
    );
    assert!(
        pot.get_item().is_some_and(
            |stack| stack.get_item() == &Item::FIREWORK_ROCKET && stack.item_count == 1
        )
    );
    client.clear_player();
    client.close().await;
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn boat_pot_transfer_rejects_a_retiring_or_detached_pot() -> TestResult {
    use crate::{
        block::entities::{BlockEntity, decorated_pot::DecoratedPotBlockEntity},
        entity::player::Player,
        plugin::api::events::entity::entity_spawn::EntitySpawnEvent,
    };
    use pumpkin_data::{item::Item, item_stack::ItemStack};
    use pumpkin_inventory::Inventory;
    use pumpkin_util::math::position::BlockPos;
    use pumpkin_world::world::BlockFlags;

    fn transfer(world: &World, pot: &DecoratedPotBlockEntity, player: &Player) -> bool {
        let source = player.inventory().get_slot(0);
        let mut awarded = source.copy_with_count(1);
        pot.try_insert_item_from_slot(world, &mut awarded, 1, || {
            player
                .inventory()
                .replace_slot_if_same(0, &source, ItemStack::EMPTY.clone())
        })
        .is_some()
    }

    struct TransferDuringScatter {
        pot: Arc<DecoratedPotBlockEntity>,
        chunk: Arc<ChunkData>,
        player: Arc<Player>,
        called: AtomicBool,
    }

    impl EventRwHandler<EntitySpawnEvent> for TransferDuringScatter {
        fn handle<'a>(
            &'a self,
            _: &'a Arc<Server>,
            event: &'a mut EntitySpawnEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                if event.entity_type == EntityType::ITEM.id.to_string() {
                    self.called.store(true, Ordering::Relaxed);
                    let current: Arc<dyn BlockEntity> = self.pot.clone();
                    let mapped = event.world.get_block_entity(&self.pot.position);
                    assert!(mapped.is_some_and(|entity| Arc::ptr_eq(&current, &entity)));
                    assert!(self.pot.get_item().is_none());
                    // A second block write can restore the state before the old entity is detached.
                    let state = Block::DECORATED_POT.default_state.id;
                    self.chunk.set_block_absolute_y(8, 63, 8, state);
                    assert!(!transfer(&event.world, &self.pot, &self.player));
                }
            })
        }
    }

    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let chunk = boat_placement_platform(&world);
    let position = BlockPos::new(8, 63, 8);
    chunk.set_block_absolute_y(8, 63, 8, Block::DECORATED_POT.default_state.id);
    let pot = Arc::new(DecoratedPotBlockEntity::new(position));
    pot.set_stack(0, ItemStack::new(1, &Item::STICK));
    world.add_block_entity(pot.clone());
    let (player, _peer) = java_rider(&server, &world, "Notch")?;
    let boat = ItemStack::new(1, &Item::OAK_BOAT);
    player.inventory().set_held_item(boat.clone());
    let observer = Arc::new(TransferDuringScatter {
        pot: pot.clone(),
        chunk,
        player: player.clone(),
        called: AtomicBool::new(false),
    });
    server
        .plugin_manager
        .register_blocking::<EntitySpawnEvent, _>(observer.clone(), EventPriority::Normal, None);
    world.set_block_state(
        &position,
        Block::AIR.default_state.id,
        BlockFlags::UPDATE_KNOWN_SHAPE,
    );
    assert!(observer.called.load(Ordering::Relaxed));
    assert!(pot.get_item().is_none());
    assert_eq!(player.inventory().held_item().uid, boat.uid);

    let detached = Arc::new(DecoratedPotBlockEntity::new(position));
    world.add_block_entity(detached.clone());
    let replacement = Arc::new(DecoratedPotBlockEntity::new(position));
    world.add_block_entity(replacement.clone());
    assert!(!transfer(&world, &detached, &player));
    assert_eq!(player.inventory().held_item().uid, boat.uid);
    assert!(detached.get_item().is_none());
    assert!(replacement.get_item().is_none());
    assert!(transfer(&world, &replacement, &player));
    assert!(player.inventory().held_item().is_empty());
    assert!(
        replacement
            .get_item()
            .is_some_and(|item| item.item == &Item::OAK_BOAT && item.item_count == 1)
    );
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bedrock_boat_pot_uses_the_live_hand_and_leaves_canonical_air() -> TestResult {
    use crate::block::entities::decorated_pot::DecoratedPotBlockEntity;
    use pumpkin_data::{block_properties::LeverLikeProperties, item::Item, item_stack::ItemStack};
    use pumpkin_protocol::bedrock::server::inventory_transaction::TransactionData;
    use pumpkin_util::math::position::BlockPos;

    struct VetoPot(AtomicBool);
    impl EventRwHandler<PlayerInteractEvent> for VetoPot {
        fn handle<'a>(
            &'a self,
            _: &'a Arc<Server>,
            event: &'a mut PlayerInteractEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                event.cancelled = self.0.load(Ordering::Relaxed);
            })
        }
    }

    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let chunk = boat_placement_platform(&world);
    let pot_pos = BlockPos::new(8, 63, 8);
    let lever = BlockPos::new(9, 63, 8);
    chunk.set_block_absolute_y(8, 63, 8, Block::DECORATED_POT.default_state.id);
    chunk.set_block_absolute_y(9, 63, 8, Block::LEVER.default_state.id);
    let pot = Arc::new(DecoratedPotBlockEntity::new(pot_pos));
    world.add_block_entity(pot.clone());
    let (player, _packets) = bedrock_rider(&server, &world, "jeb_").await?;
    player.get_entity().set_pos(BOAT_PLACEMENT_OFFSET);
    let client = player.client.bedrock().ok_or("Bedrock client")?;
    let earlier = ItemStack::new(1, &Item::OAK_BOAT);
    let current = earlier.copy_with_count(1);
    player.inventory().set_held_item(current.clone());
    let veto = Arc::new(VetoPot(AtomicBool::new(false)));
    server
        .plugin_manager
        .register_blocking::<PlayerInteractEvent, _>(veto.clone(), EventPriority::Normal, None);

    let deposit = |id| {
        let mut packet = bedrock_use_item_transaction(0, &current, 0);
        if let TransactionData::UseItem(data) = &mut packet.transaction_data {
            data.item_in_hand_network_id = Some(id);
        }
        client.handle_inventory_action(&player, packet);
    };
    deposit(earlier.uid);
    assert!(pot.get_item().is_none_or(|item| item.is_empty()));
    veto.0.store(true, Ordering::Relaxed);
    deposit(current.uid);
    assert!(pot.get_item().is_none_or(|item| item.is_empty()));
    veto.0.store(false, Ordering::Relaxed);
    player
        .get_entity()
        .set_pos(BOAT_PLACEMENT_OFFSET.add_raw(20.0, 0.0, 0.0));
    deposit(current.uid);
    assert!(pot.get_item().is_none_or(|item| item.is_empty()));
    assert_eq!(player.inventory().held_item().uid, current.uid);

    player.get_entity().set_pos(BOAT_PLACEMENT_OFFSET);
    deposit(current.uid);
    assert!(
        pot.get_item()
            .is_some_and(|item| item.get_item() == &Item::OAK_BOAT)
    );
    assert_eq!(player.inventory().held_item().item, &Item::AIR);
    let mut packet = bedrock_use_item_transaction(0, ItemStack::EMPTY, 0);
    if let TransactionData::UseItem(data) = &mut packet.transaction_data {
        data.block_position = lever;
    }
    client.handle_inventory_action(&player, packet);
    assert!(LeverLikeProperties::from_state_id(world.get_block_state_id(&lever)).powered);
    client.clear_player();
    client.close().await;
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn java_boat_shelf_click_rechecks_the_addressed_hand_after_interaction() -> TestResult {
    use crate::block::entities::shelf::ShelfBlockEntity;
    use pumpkin_data::{
        block_properties::{AcaciaShelfLikeProperties, HorizontalFacing},
        item::Item,
        item_stack::ItemStack,
    };
    use pumpkin_inventory::inventory::Inventory;
    use pumpkin_protocol::java::server::play::SUseItemOn;
    use pumpkin_util::math::position::BlockPos;

    struct SelectOtherSlot(AtomicBool);
    impl EventRwHandler<PlayerInteractEvent> for SelectOtherSlot {
        fn handle<'a>(
            &'a self,
            _: &'a Arc<Server>,
            event: &'a mut PlayerInteractEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                if self.0.swap(false, Ordering::Relaxed) {
                    event.player.inventory().set_selected_slot(1);
                }
            })
        }
    }

    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let chunk = boat_placement_platform(&world);
    let position = BlockPos::new(8, 63, 8);
    let mut properties = AcaciaShelfLikeProperties::default(&Block::OAK_SHELF);
    properties.facing = HorizontalFacing::North;
    chunk.set_block_absolute_y(8, 63, 8, properties.to_state_id(&Block::OAK_SHELF));
    let shelf = Arc::new(ShelfBlockEntity::new(position));
    world.add_block_entity(shelf.clone());
    let (player, _peer) = java_rider(&server, &world, "Notch")?;
    player.get_entity().set_pos(BOAT_PLACEMENT_OFFSET);
    let client = player.client.java().ok_or("Java player")?;
    let boat = ItemStack::new(1, &Item::OAK_BOAT);
    let stick = ItemStack::new(1, &Item::STICK);
    player.inventory().set_slot(0, boat.clone());
    player.inventory().set_slot(1, stick.clone());
    server
        .plugin_manager
        .register_blocking::<PlayerInteractEvent, _>(
            Arc::new(SelectOtherSlot(AtomicBool::new(true))),
            EventPriority::Normal,
            None,
        );
    let click = |sequence| {
        client.handle_use_item_on(
            &player,
            &SUseItemOn {
                hand: 0.into(),
                position,
                face: 2.into(),
                cursor_pos: Vector3::new(0.5, 0.5, 0.0),
                inside_block: false,
                is_against_world_border: false,
                sequence,
            },
            &server,
        )
    };

    click(1.into())?;
    assert!(shelf.get_stack(1).is_empty());
    assert_eq!(player.inventory().get_slot(0).uid, boat.uid);
    assert_eq!(player.inventory().get_slot(1).uid, stick.uid);
    player.inventory().set_selected_slot(0);
    click(2.into())?;
    assert_eq!(shelf.get_stack(1).uid, boat.uid);
    assert!(player.inventory().get_slot(0).is_empty());
    assert_eq!(player.inventory().get_slot(1).uid, stick.uid);
    world.level.shutdown().await;
    Ok(())
}

fn bedrock_boat_click(
    player: &Arc<crate::entity::player::Player>,
    slot: i32,
    stack: &pumpkin_data::item_stack::ItemStack,
) -> TestResult {
    bedrock_boat_interaction(player, slot, stack, 1)
}

fn bedrock_boat_interaction(
    player: &Arc<crate::entity::player::Player>,
    slot: i32,
    stack: &pumpkin_data::item_stack::ItemStack,
    action: i32,
) -> TestResult {
    let client = player.client.bedrock().ok_or("Bedrock player")?;
    client.handle_inventory_action(player, bedrock_use_item_transaction(slot, stack, action));
    Ok(())
}

pub(super) fn bedrock_use_item_transaction(
    slot: i32,
    stack: &pumpkin_data::item_stack::ItemStack,
    action: i32,
) -> pumpkin_protocol::bedrock::server::inventory_transaction::SInventoryTransaction {
    use pumpkin_protocol::bedrock::{
        network_item::NetworkItemDescriptor,
        server::inventory_transaction::{
            HandSlot, SInventoryTransaction, TransactionData, UseItemTransactionData,
        },
    };
    use pumpkin_util::math::position::BlockPos;

    SInventoryTransaction {
        legacy_request_id: 0.into(),
        legacy_set_item_slots: Vec::new(),
        has_value: false,
        actions: Vec::new(),
        transaction_type: 2u32.into(),
        transaction_data: TransactionData::UseItem(UseItemTransactionData {
            action_type: action.into(),
            trigger_type: 0,
            block_position: BlockPos::new(8, 63, 8),
            block_face: 1,
            hot_bar_slot: slot.into(),
            hand: HandSlot::Mainhand,
            item_in_hand: NetworkItemDescriptor::from(stack),
            item_in_hand_network_id: None,
            player_position: BOAT_PLACEMENT_OFFSET.to_f32_lossy(),
            click_position: Vector3::default(),
            block_runtime_id: 0u32.into(),
            client_prediction: 0,
            client_cooldown_state: 0,
        }),
    }
}

fn bedrock_boat_legacy_responses(
    packets: &mut super::vehicle_tests::BedrockQueue,
) -> Result<Vec<(i32, u8)>, Box<dyn std::error::Error>> {
    use pumpkin_protocol::{
        Packet,
        bedrock::client::item_stack_response::CItemStackResponse,
        codec::{var_int::VarInt, var_uint::VarUInt},
        serial::PacketRead,
    };
    let mut responses = Vec::new();
    while let Ok(packet) = packets.try_recv() {
        if let Some((id, payload)) = crate::net::bedrock::split_raw_packet(&packet.data)
            && i32::from(id) == CItemStackResponse::PACKET_ID
        {
            let mut payload = payload.as_ref();
            assert_eq!(VarUInt::read(&mut payload)?.0, 1);
            let status = u8::read(&mut payload)?;
            responses.push((VarInt::read(&mut payload)?.0, status));
        }
    }
    Ok(responses)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn boat_placement_requires_the_survival_debit_after_the_spawn_observer() -> TestResult {
    use crate::plugin::api::events::entity::entity_spawn::EntitySpawnEvent;
    use pumpkin_data::{
        data_component_impl::{UseCooldownImpl, UseRemainderImpl},
        item::Item,
        item_stack::ItemStack,
        statistic::StatisticCategory,
    };
    use pumpkin_inventory::player::player_inventory::PlayerInventory;
    use pumpkin_util::Hand;

    struct InvalidatePlacement {
        player: Arc<crate::entity::player::Player>,
        obstruction: Option<Arc<ChunkData>>,
        called: AtomicBool,
    }
    impl EventRwHandler<EntitySpawnEvent> for InvalidatePlacement {
        fn handle<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut EntitySpawnEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                if event.entity_type == EntityType::OAK_BOAT.id.to_string() {
                    if let Some(chunk) = &self.obstruction {
                        chunk.set_block_absolute_y(8, 64, 8, Block::OAK_FENCE.default_state.id);
                    } else {
                        let stack = self.player.inventory().held_item();
                        self.player
                            .inventory()
                            .set_held_item(ItemStack::EMPTY.clone());
                        self.player.inventory().set_stack_in_hand(Hand::Left, stack);
                    }
                    self.called.store(true, Ordering::Relaxed);
                }
            })
        }
    }

    for obstructed in [false, true] {
        let folder = tempfile::tempdir()?;
        let (server, world) = vehicle_world(&folder).await?;
        let chunk = boat_placement_platform(&world);
        let (player, _peer) = java_rider(&server, &world, "Notch")?;
        player.get_entity().set_pos(BOAT_PLACEMENT_OFFSET);
        let mut stack = ItemStack::new(1, &Item::OAK_BOAT);
        stack.set_data_component(UseRemainderImpl {
            remainder: Some(ItemStack::new(1, &Item::STICK)),
        });
        stack.set_data_component(UseCooldownImpl {
            seconds: 5.0,
            cooldown_group: Some("minecraft:test_boat".to_owned()),
        });
        player.inventory().set_held_item(stack.clone());
        let listener = Arc::new(InvalidatePlacement {
            player: player.clone(),
            obstruction: obstructed.then_some(chunk),
            called: AtomicBool::new(false),
        });
        server
            .plugin_manager
            .register_blocking::<EntitySpawnEvent, _>(
                listener.clone(),
                EventPriority::Normal,
                None,
            );
        server
            .item_registry
            .on_use_with_rotation(&stack, &player, 0.0, 45.0, Hand::Right);
        assert!(listener.called.load(Ordering::Relaxed));
        let survivor = if obstructed {
            assert!(player.inventory().get_stack_in_hand(Hand::Left).is_empty());
            player.inventory().held_item()
        } else {
            assert!(player.inventory().held_item().is_empty());
            player.inventory().get_stack_in_hand(Hand::Left)
        };
        assert_eq!(survivor.uid, stack.uid);
        assert_eq!(survivor.item_count, 1);
        assert!(boat_placement_actors(&world).is_empty());
        assert_eq!(
            player.get_stat(StatisticCategory::Used, i32::from(Item::OAK_BOAT.id)),
            0
        );
        assert!(!player.is_on_cooldown("minecraft:test_boat"));
        assert!(
            (0..PlayerInventory::MAIN_SIZE)
                .all(|slot| player.inventory().get_slot(slot).item != &Item::STICK)
        );
        assert_eq!(dropped_boat_remainders(&world), 0);
        world.level.shutdown().await;
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn boat_placement_rejects_a_replaced_stack_of_the_same_item() -> TestResult {
    use pumpkin_data::{
        data_component_impl::UseCooldownImpl, item::Item, item_stack::ItemStack,
        statistic::StatisticCategory,
    };
    use pumpkin_util::Hand;

    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    boat_placement_platform(&world);
    let (player, _peer) = java_rider(&server, &world, "Notch")?;
    player.get_entity().set_pos(BOAT_PLACEMENT_OFFSET);
    let mut captured = ItemStack::new(1, &Item::OAK_BOAT);
    captured.set_data_component(UseCooldownImpl {
        seconds: 5.0,
        cooldown_group: Some("minecraft:test_boat".to_owned()),
    });
    player.inventory().set_held_item(captured.clone());
    let replacement = captured.copy_with_count(1);
    assert!(replacement.are_equal(&captured));
    assert_ne!(replacement.uid, captured.uid);
    player.inventory().set_held_item(replacement.clone());

    let place = |stack: &ItemStack| {
        server
            .item_registry
            .on_use_with_rotation(stack, &player, 0.0, 45.0, Hand::Right);
    };
    place(&captured);
    assert!(boat_placement_actors(&world).is_empty());
    assert_eq!(player.inventory().held_item().uid, replacement.uid);
    let uses = || player.get_stat(StatisticCategory::Used, i32::from(Item::OAK_BOAT.id));
    assert_eq!(uses(), 0);
    assert!(!player.is_on_cooldown("minecraft:test_boat"));

    place(&replacement);
    assert_eq!(boat_placement_actors(&world).len(), 1);
    assert!(player.inventory().held_item().is_empty());
    assert_eq!(uses(), 1);
    assert!(player.is_on_cooldown("minecraft:test_boat"));
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn boat_placement_delivers_earned_remainder_after_stat_observer_moves_the_stack() -> TestResult
{
    use crate::plugin::api::events::player::player_statistic_increment::PlayerStatisticIncrementEvent;
    use pumpkin_data::{
        data_component_impl::{MaxStackSizeImpl, UseRemainderImpl},
        item::Item,
        item_stack::ItemStack,
        statistic::StatisticCategory,
    };
    use pumpkin_inventory::player::player_inventory::PlayerInventory;
    use pumpkin_util::Hand;

    struct MoveDebitedStack {
        remaining_boats: u8,
        called: AtomicBool,
    }
    impl EventRwHandler<PlayerStatisticIncrementEvent> for MoveDebitedStack {
        fn handle<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut PlayerStatisticIncrementEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                let inventory = event.player.inventory();
                let reduced = inventory.held_item();
                if !self.called.swap(true, Ordering::Relaxed) {
                    assert_eq!(reduced.item_count, self.remaining_boats);
                    if self.remaining_boats == 0 {
                        assert!(reduced.is_empty());
                        inventory.set_slot(1, ItemStack::new(1, &Item::DIRT));
                    } else {
                        assert_eq!(reduced.item, &Item::OAK_BOAT);
                        inventory.set_stack_in_hand(Hand::Left, reduced);
                    }
                    inventory.set_held_item(ItemStack::new(1, &Item::DIRT));
                }
            })
        }
    }

    for count in [2, 1] {
        let folder = tempfile::tempdir()?;
        let (server, world) = vehicle_world(&folder).await?;
        boat_placement_platform(&world);
        let (player, _peer) = java_rider(&server, &world, "Notch")?;
        player.get_entity().set_pos(BOAT_PLACEMENT_OFFSET);
        let mut stack = ItemStack::new(count, &Item::OAK_BOAT);
        stack.set_data_component(MaxStackSizeImpl { size: 2 });
        stack.set_data_component(UseRemainderImpl {
            remainder: Some(ItemStack::new(1, &Item::STICK)),
        });
        player.inventory().set_held_item(stack.clone());
        let listener = Arc::new(MoveDebitedStack {
            remaining_boats: count - 1,
            called: AtomicBool::new(false),
        });
        server
            .plugin_manager
            .register_blocking::<PlayerStatisticIncrementEvent, _>(
                listener.clone(),
                EventPriority::Normal,
                None,
            );
        server
            .item_registry
            .on_use_with_rotation(&stack, &player, 0.0, 45.0, Hand::Right);
        assert!(listener.called.load(Ordering::Relaxed));
        assert_eq!(player.inventory().held_item().item, &Item::DIRT);
        let remaining = player.inventory().get_stack_in_hand(Hand::Left);
        if count == 1 {
            assert!(remaining.is_empty());
            assert_eq!(player.inventory().get_slot(1).item, &Item::DIRT);
        } else {
            assert_eq!(remaining.item, &Item::OAK_BOAT);
            assert_eq!(remaining.item_count, 1);
        }
        assert_eq!(boat_placement_actors(&world).len(), 1);
        assert_eq!(
            (0..PlayerInventory::MAIN_SIZE)
                .map(|slot| player.inventory().get_slot(slot))
                .filter(|stack| stack.item == &Item::STICK)
                .map(|stack| usize::from(stack.item_count))
                .sum::<usize>(),
            1,
        );
        assert_eq!(dropped_boat_remainders(&world), 0);
        assert_eq!(
            player.get_stat(StatisticCategory::Used, i32::from(Item::OAK_BOAT.id)),
            1
        );
        world.level.shutdown().await;
    }
    Ok(())
}

fn dropped_boat_remainders(world: &World) -> usize {
    world
        .entities
        .load()
        .iter()
        .filter_map(|entity| {
            let item = entity
                .cast_any()
                .downcast_ref::<crate::entity::item::ItemEntity>()?;
            let stack = item
                .get_item_stack()
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            (stack.item == &pumpkin_data::item::Item::STICK)
                .then_some(usize::from(stack.item_count))
        })
        .sum()
}

struct ReplaceDuringHandUpdate {
    player: Arc<crate::entity::player::Player>,
    called: AtomicBool,
}

impl EventRwHandler<PacketSentEvent> for ReplaceDuringHandUpdate {
    fn handle<'a>(
        &'a self,
        _: &'a Arc<Server>,
        event: &'a mut PacketSentEvent,
    ) -> BoxFuture<'a, ()> {
        use pumpkin_data::{item::Item, item_stack::ItemStack};
        use pumpkin_inventory::inventory::Inventory;
        use pumpkin_protocol::{
            ClientPacket,
            codec::item_stack_seralizer::ItemStackSerializer,
            java::client::play::{CSetContainerContent, CSetContainerSlot, CSetPlayerInventory},
        };
        Box::pin(async move {
            let targeted = event.packet_id == CSetPlayerInventory::to_id(CURRENT_MC_VERSION);
            if !targeted || self.called.swap(true, Ordering::Relaxed) {
                return;
            }
            let new = ItemStack::new(1, &Item::DIAMOND);
            self.player.inventory().set_held_item(new.clone());
            self.player.sync_hand_slot(0, new);
            let marker = ItemStackSerializer::from(ItemStack::new(1, &Item::DIRT));
            let mut physical = Vec::new();
            let mut personal = Vec::new();
            let mut active = Vec::new();
            let mut unrelated = Vec::new();
            let mut snapshot = Vec::new();
            let window = {
                let screen = self
                    .player
                    .current_screen_handler
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner);
                let screen = screen.lock().unwrap_or_else(PoisonError::into_inner);
                let inventory: Arc<dyn Inventory> = self.player.inventory().clone();
                let slot = screen.get_slot_index(&inventory, 0);
                assert_eq!(slot, Some(54));
                assert_ne!(screen.sync_id(), 0);
                screen.sync_id() as i8
            };
            assert!(
                CSetContainerSlot::new(window, 1, 54, &marker)
                    .write_packet_data(&mut active, &CURRENT_MC_VERSION)
                    .is_ok()
            );
            assert!(
                CSetPlayerInventory::new(0.into(), &marker)
                    .write_packet_data(&mut physical, &CURRENT_MC_VERSION)
                    .is_ok()
            );
            assert!(
                CSetContainerSlot::new(0, 1, 36, &marker)
                    .write_packet_data(&mut personal, &CURRENT_MC_VERSION)
                    .is_ok()
            );
            assert!(
                CSetPlayerInventory::new(
                    1.into(),
                    &ItemStackSerializer::from(ItemStack::new(1, &Item::STICK))
                )
                .write_packet_data(&mut unrelated, &CURRENT_MC_VERSION)
                .is_ok()
            );
            let empty = ItemStackSerializer::from(ItemStack::EMPTY.clone());
            let personal_slots: [_; 46] = std::array::from_fn(|slot| {
                ItemStackSerializer::from(if slot == 36 {
                    ItemStack::new(1, &Item::DIRT)
                } else {
                    ItemStack::EMPTY.clone()
                })
            });
            assert!(
                CSetContainerContent::new(0.into(), 1.into(), &personal_slots, &empty)
                    .write_packet_data(&mut snapshot, &CURRENT_MC_VERSION)
                    .is_ok()
            );
            event.extra_packets.extend([
                (
                    CSetPlayerInventory::to_id(CURRENT_MC_VERSION),
                    physical.into(),
                ),
                (
                    CSetContainerSlot::to_id(CURRENT_MC_VERSION),
                    personal.into(),
                ),
                (CSetContainerSlot::to_id(CURRENT_MC_VERSION), active.into()),
                (
                    CSetContainerContent::to_id(CURRENT_MC_VERSION),
                    snapshot.into(),
                ),
                (
                    CSetPlayerInventory::to_id(CURRENT_MC_VERSION),
                    unrelated.into(),
                ),
            ]);
        })
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn java_boat_hand_sync_filters_stale_physical_aliases() -> TestResult {
    use crate::plugin::api::events::player::player_statistic_increment::PlayerStatisticIncrementEvent;
    use pumpkin_data::{item::Item, item_stack::ItemStack};
    use pumpkin_protocol::java::client::play::CKeepAlive;
    use pumpkin_util::Hand;

    struct OpenPlacedChestBoat(AtomicBool);
    impl EventRwHandler<PlayerStatisticIncrementEvent> for OpenPlacedChestBoat {
        fn handle<'a>(
            &'a self,
            _: &'a Arc<Server>,
            event: &'a mut PlayerStatisticIncrementEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                if self.0.load(Ordering::Relaxed) {
                    return;
                }
                let world = event.player.world();
                let entities = world.entities.load();
                let boat = entities.iter().find_map(|entity| {
                    (entity.get_entity().entity_type == &EntityType::OAK_CHEST_BOAT)
                        .then(|| entity.cast_any().downcast_ref::<BoatEntity>())
                        .flatten()
                });
                if let Some(boat) = boat {
                    self.0.store(
                        boat.open_custom_inventory_screen(&event.player),
                        Ordering::Relaxed,
                    );
                }
            })
        }
    }

    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let (player, peer) = java_equipment_watcher(&server, &world)?;
    boat_placement_platform(&world);
    player.get_entity().set_pos(BOAT_PLACEMENT_OFFSET);
    let open = Arc::new(OpenPlacedChestBoat(AtomicBool::new(false)));
    server
        .plugin_manager
        .register_blocking::<PlayerStatisticIncrementEvent, _>(
            open.clone(),
            EventPriority::Normal,
            None,
        );
    let observer = Arc::new(ReplaceDuringHandUpdate {
        player: player.clone(),
        called: AtomicBool::new(false),
    });
    server
        .plugin_manager
        .register_blocking::<PacketSentEvent, _>(observer.clone(), EventPriority::Normal, None);

    let old = ItemStack::new(1, &Item::OAK_CHEST_BOAT);
    player.inventory().set_held_item(old.clone());
    server
        .item_registry
        .on_use_with_rotation(&old, &player, 0.0, 45.0, Hand::Right);
    assert!(open.0.load(Ordering::Relaxed));
    assert!(observer.called.load(Ordering::Relaxed));
    assert_eq!(player.inventory().held_item().item, &Item::DIAMOND);
    let client = player.client.java().ok_or("Java player")?;
    assert!(client.try_send_packet(&CKeepAlive::new(73)));
    let updates = read_boat_hand_updates(peer).await?;
    assert_eq!(updates.hand, Some(&Item::DIAMOND));
    assert!(updates.items.contains(&&Item::STICK));
    assert!(!updates.items.contains(&&Item::DIRT));
    assert!(!updates.items.contains(&&Item::OAK_CHEST_BOAT));
    world.level.shutdown().await;
    Ok(())
}

#[derive(Default)]
struct BoatHandUpdates {
    items: Vec<&'static pumpkin_data::item::Item>,
    hand: Option<&'static pumpkin_data::item::Item>,
}

async fn read_boat_hand_updates(
    peer: tokio::io::DuplexStream,
) -> Result<BoatHandUpdates, Box<dyn std::error::Error>> {
    use pumpkin_protocol::{
        codec::item_stack_seralizer::ItemStackSerializer,
        java::{
            client::play::{
                CKeepAlive, CSetContainerContent, CSetContainerSlot, CSetPlayerInventory,
            },
            packet_decoder::TCPNetworkDecoder,
        },
    };

    let mut decoder = TCPNetworkDecoder::new(tokio::io::BufReader::new(peer));
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        let mut updates = BoatHandUpdates::default();
        loop {
            let packet = decoder.get_raw_packet().await?;
            let mut payload = packet.payload.as_ref();
            if packet.id == CKeepAlive::to_id(CURRENT_MC_VERSION) {
                return Ok(updates);
            }
            let targets_hand = if packet.id == CSetPlayerInventory::to_id(CURRENT_MC_VERSION) {
                payload.get_var_int()?.0 == 0
            } else if packet.id == CSetContainerSlot::to_id(CURRENT_MC_VERSION) {
                let window = payload.get_container_id(&CURRENT_MC_VERSION)?.0;
                payload.get_var_int()?;
                let slot = payload.get_i16_be()?;
                (window == 0 && slot == 36) || (window > 0 && slot == 54)
            } else if packet.id == CSetContainerContent::to_id(CURRENT_MC_VERSION) {
                let window = payload.get_container_id(&CURRENT_MC_VERSION)?.0;
                if window != 0 {
                    continue;
                }
                payload.get_var_int()?;
                let count = payload.get_var_int()?.0;
                for slot in 0..count.min(37) {
                    let stack =
                        ItemStackSerializer::read_with_version(&mut payload, &CURRENT_MC_VERSION)?;
                    if slot == 36 {
                        updates.hand = Some(stack.0.item);
                        updates.items.push(stack.0.item);
                    }
                }
                continue;
            } else {
                continue;
            };
            let stack = ItemStackSerializer::read_with_version(&mut payload, &CURRENT_MC_VERSION)?;
            if targets_hand {
                updates.hand = Some(stack.0.item);
            }
            updates.items.push(stack.0.item);
        }
    })
    .await?
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn boat_placement_rechecks_ray_reach_after_a_same_world_move() -> TestResult {
    use crate::plugin::api::events::entity::entity_spawn::EntitySpawnEvent;
    use pumpkin_data::{item::Item, item_stack::ItemStack, statistic::StatisticCategory};
    use pumpkin_util::Hand;

    struct MoveDuringSpawn {
        player: Arc<crate::entity::player::Player>,
        enabled: AtomicBool,
    }
    impl EventRwHandler<EntitySpawnEvent> for MoveDuringSpawn {
        fn handle<'a>(
            &'a self,
            _: &'a Arc<Server>,
            _: &'a mut EntitySpawnEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                if self.enabled.load(Ordering::Relaxed) {
                    self.player
                        .get_entity()
                        .set_pos(BOAT_PLACEMENT_OFFSET.add_raw(32.0, 0.0, 0.0));
                }
            })
        }
    }

    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    boat_placement_platform(&world);
    let (player, _peer) = java_rider(&server, &world, "Notch")?;
    let boat = server
        .item_registry
        .boat_item(Item::BAMBOO_RAFT.id)
        .ok_or("bamboo raft")?;
    let stack = ItemStack::new(1, &Item::BAMBOO_RAFT);
    player.inventory().set_held_item(stack.clone());
    player.get_entity().set_pos(BOAT_PLACEMENT_OFFSET);
    let observer = Arc::new(MoveDuringSpawn {
        player: player.clone(),
        enabled: AtomicBool::new(true),
    });
    server
        .plugin_manager
        .register_blocking::<EntitySpawnEvent, _>(observer.clone(), EventPriority::Normal, None);
    assert!(!boat.place_from_hand(&stack, &player, 0.0, 45.0, Hand::Right));
    assert_eq!(player.inventory().held_item().uid, stack.uid);
    assert_eq!(
        player.get_stat(StatisticCategory::Used, i32::from(Item::BAMBOO_RAFT.id)),
        0
    );
    assert!(
        world
            .entities
            .load()
            .iter()
            .all(|entity| entity.get_entity().entity_type != &EntityType::BAMBOO_RAFT)
    );

    observer.enabled.store(false, Ordering::Relaxed);
    player.get_entity().set_pos(BOAT_PLACEMENT_OFFSET);
    assert!(boat.place_from_hand(&stack, &player, 0.0, 45.0, Hand::Right));
    assert!(player.inventory().held_item().is_empty());
    assert_eq!(
        player.get_stat(StatisticCategory::Used, i32::from(Item::BAMBOO_RAFT.id)),
        1
    );
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn boat_placement_rejects_a_water_target_removed_during_spawn() -> TestResult {
    check_boat_spawn_revalidation(false).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn boat_placement_rejects_a_new_eye_obstruction_during_spawn() -> TestResult {
    check_boat_spawn_revalidation(true).await
}

async fn check_boat_spawn_revalidation(obstructs_eye: bool) -> TestResult {
    use crate::{
        entity::player::Player, plugin::api::events::entity::entity_spawn::EntitySpawnEvent,
    };
    use pumpkin_data::{item::Item, item_stack::ItemStack, statistic::StatisticCategory};
    use pumpkin_util::Hand;

    const PLACER_POS: Vector3<f64> = Vector3::new(8.5, 66.0, 8.5);

    enum SpawnChange {
        DrainWater(Arc<ChunkData>),
        ObstructEye(Arc<Player>),
    }

    struct ChangeBoatTarget {
        change: SpawnChange,
        ran: AtomicBool,
    }
    impl EventRwHandler<EntitySpawnEvent> for ChangeBoatTarget {
        fn handle<'a>(
            &'a self,
            _: &'a Arc<Server>,
            event: &'a mut EntitySpawnEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                if event.entity_type != EntityType::OAK_BOAT.id.to_string()
                    || self.ran.swap(true, Ordering::Relaxed)
                {
                    return;
                }
                match &self.change {
                    SpawnChange::DrainWater(chunk) => {
                        chunk.set_block_absolute_y(8, 64, 8, Block::AIR.default_state.id);
                    }
                    SpawnChange::ObstructEye(blocker) => blocker.get_entity().set_pos(PLACER_POS),
                }
            })
        }
    }

    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let chunk = ChunkData::empty_sync(0, 0);
    chunk.set_block_absolute_y(8, 64, 8, Block::WATER.default_state.id);
    world
        .level
        .loaded_chunks
        .insert(Vector2::new(0, 0), chunk.clone());
    let (player, _peer) = java_rider(&server, &world, "Notch")?;
    player.get_entity().set_pos(PLACER_POS);
    let blocker = if obstructs_eye {
        Some(java_rider(&server, &world, "jeb_")?)
    } else {
        None
    };
    let parked = PLACER_POS.add_raw(7.0, 0.0, 0.0);
    let change = if let Some((blocker, _)) = &blocker {
        blocker.get_entity().set_pos(parked);
        SpawnChange::ObstructEye(blocker.clone())
    } else {
        SpawnChange::DrainWater(chunk.clone())
    };
    let observer = Arc::new(ChangeBoatTarget {
        change,
        ran: AtomicBool::new(false),
    });
    server
        .plugin_manager
        .register_blocking::<EntitySpawnEvent, _>(observer.clone(), EventPriority::Normal, None);

    let boat = server
        .item_registry
        .boat_item(Item::OAK_BOAT.id)
        .ok_or("Oak boat")?;
    let stack = ItemStack::new(1, &Item::OAK_BOAT);
    player.inventory().set_held_item(stack.clone());
    let uses = || player.get_stat(StatisticCategory::Used, i32::from(Item::OAK_BOAT.id));
    assert!(!boat.place_from_hand(&stack, &player, 0.0, 90.0, Hand::Right));
    assert!(observer.ran.load(Ordering::Relaxed));
    assert!(boat_placement_actors(&world).is_empty());
    assert_eq!(uses(), 0);
    assert_eq!(player.inventory().held_item().uid, stack.uid);

    if let Some((blocker, _)) = &blocker {
        blocker.get_entity().set_pos(parked);
    } else {
        chunk.set_block_absolute_y(8, 64, 8, Block::WATER.default_state.id);
    }
    assert!(boat.place_from_hand(&stack, &player, 0.0, 90.0, Hand::Right));
    assert_eq!(boat_placement_actors(&world).len(), 1);
    assert_eq!(uses(), 1);
    assert!(player.inventory().held_item().is_empty());
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn java_boat_air_swing_stays_in_the_placement_world() -> TestResult {
    use crate::plugin::api::events::player::player_statistic_increment::PlayerStatisticIncrementEvent;
    use pumpkin_data::{item::Item, item_stack::ItemStack, statistic::StatisticCategory};
    use pumpkin_protocol::java::server::play::SUseItem;

    struct DepartAfterPlacement(Arc<World>);
    impl EventRwHandler<PlayerStatisticIncrementEvent> for DepartAfterPlacement {
        fn handle<'a>(
            &'a self,
            _: &'a Arc<Server>,
            event: &'a mut PlayerStatisticIncrementEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                event.player.get_entity().world.store(self.0.clone());
            })
        }
    }

    let origin_folder = tempfile::tempdir()?;
    let destination_folder = tempfile::tempdir()?;
    let (server, origin) = vehicle_world(&origin_folder).await?;
    let (_destination_server, destination) = vehicle_world(&destination_folder).await?;
    boat_placement_platform(&origin).set_block_absolute_y(8, 64, 8, Block::WATER.default_state.id);
    let (placer, _peer) = java_rider(&server, &origin, "Notch")?;
    placer.get_entity().set_pos(BOAT_PLACEMENT_OFFSET);
    placer
        .inventory()
        .set_held_item(ItemStack::new(1, &Item::OAK_BOAT));
    placer.last_attacked_ticks.store(20, Ordering::Relaxed);
    let client = placer.client.java().ok_or("Java placer")?;
    client.handle_use_item(
        &placer,
        &SUseItem {
            hand: 0.into(),
            sequence: 1.into(),
            yaw: 0.0,
            pitch: 45.0,
        },
        &server,
    );
    let boats = boat_placement_actors(&origin);
    assert_eq!(boats.len(), 1);
    assert_eq!(placer.last_attacked_ticks.load(Ordering::Relaxed), 0);
    origin.remove_entity(boats.first().ok_or("Control boat")?.as_ref());
    placer
        .inventory()
        .set_held_item(ItemStack::new(1, &Item::OAK_BOAT));
    placer.last_attacked_ticks.store(20, Ordering::Relaxed);

    server
        .plugin_manager
        .register_blocking::<PlayerStatisticIncrementEvent, _>(
            Arc::new(DepartAfterPlacement(destination.clone())),
            EventPriority::Normal,
            None,
        );
    client.handle_use_item(
        &placer,
        &SUseItem {
            hand: 0.into(),
            sequence: 2.into(),
            yaw: 0.0,
            pitch: 45.0,
        },
        &server,
    );
    assert!(Arc::ptr_eq(&placer.world(), &destination));
    assert_eq!(boat_placement_actors(&origin).len(), 1);
    assert!(boat_placement_actors(&destination).is_empty());
    assert!(placer.inventory().held_item().is_empty());
    assert_eq!(
        placer.get_stat(StatisticCategory::Used, i32::from(Item::OAK_BOAT.id)),
        2
    );
    assert_eq!(placer.last_attacked_ticks.load(Ordering::Relaxed), 20);
    destination.level.shutdown().await;
    origin.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn boat_air_use_cannot_place_in_a_world_selected_by_the_interact_callback() -> TestResult {
    use pumpkin_data::{item::Item, item_stack::ItemStack, statistic::StatisticCategory};
    use pumpkin_protocol::java::server::play::SUseItem;

    struct MoveBoatPlacer(Arc<World>);
    impl EventRwHandler<PlayerInteractEvent> for MoveBoatPlacer {
        fn handle<'a>(
            &'a self,
            _: &'a Arc<Server>,
            event: &'a mut PlayerInteractEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                event.player.get_entity().world.store(self.0.clone());
            })
        }
    }

    let origin_folder = tempfile::tempdir()?;
    let destination_folder = tempfile::tempdir()?;
    let (origin_server, origin) = vehicle_world(&origin_folder).await?;
    let (destination_server, destination) = vehicle_world(&destination_folder).await?;
    boat_placement_platform(&origin).set_block_absolute_y(8, 64, 8, Block::WATER.default_state.id);
    boat_placement_platform(&destination).set_block_absolute_y(
        8,
        64,
        8,
        Block::WATER.default_state.id,
    );
    origin_server
        .plugin_manager
        .register_blocking::<PlayerInteractEvent, _>(
            Arc::new(MoveBoatPlacer(destination.clone())),
            EventPriority::Normal,
            None,
        );
    let (java, _peer) = java_rider(&origin_server, &origin, "Notch")?;
    let (bedrock, _packets) = bedrock_rider(&origin_server, &origin, "jeb_").await?;

    for player in [&java, &bedrock] {
        player.get_entity().set_pos(BOAT_PLACEMENT_OFFSET);
        player.get_entity().set_rotation(0.0, 45.0);
        let stack = ItemStack::new(1, &Item::OAK_BOAT);
        player.inventory().set_held_item(stack.clone());
        let use_boat = |server: &Arc<Server>, sequence: i32| {
            let Some(client) = player.client.java() else {
                return bedrock_boat_click(player, 0, &stack);
            };
            client.handle_use_item(
                player,
                &SUseItem {
                    hand: 0.into(),
                    sequence: sequence.into(),
                    yaw: 0.0,
                    pitch: 45.0,
                },
                server,
            );
            Ok(())
        };
        let uses = || player.get_stat(StatisticCategory::Used, i32::from(Item::OAK_BOAT.id));
        assert!(Arc::ptr_eq(&player.world(), &origin));
        use_boat(&origin_server, 1)?;
        assert!(Arc::ptr_eq(&player.world(), &destination));
        assert!(boat_placement_actors(&origin).is_empty());
        assert!(boat_placement_actors(&destination).is_empty());
        assert_eq!(uses(), 0);
        assert_eq!(player.inventory().held_item().uid, stack.uid);

        use_boat(&destination_server, 2)?;
        let boats = boat_placement_actors(&destination);
        assert_eq!(boats.len(), 1);
        assert_eq!(uses(), 1);
        assert!(player.inventory().held_item().is_empty());
        destination.remove_entity(boats.first().ok_or("Placed boat")?.as_ref());
    }
    if let Some(client) = bedrock.client.bedrock() {
        client.clear_player();
        client.close().await;
    }
    destination.level.shutdown().await;
    origin.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[allow(
    clippy::too_many_lines,
    reason = "The two editions share the protected block interaction and spectator state fixture."
)]
async fn vehicle_spectator_block_use_observes_the_veto_and_current_mode() -> TestResult {
    use crate::block::entities::{chest::ChestBlockEntity, decorated_pot::DecoratedPotBlockEntity};
    use pumpkin_data::{item::Item, item_stack::ItemStack};
    use pumpkin_protocol::{
        bedrock::server::inventory_transaction::TransactionData, java::server::play::SUseItemOn,
    };
    use pumpkin_util::{GameMode, math::position::BlockPos};

    struct SpectatorClick {
        veto: AtomicBool,
        pot: BlockPos,
    }
    impl EventRwHandler<PlayerInteractEvent> for SpectatorClick {
        fn handle<'a>(
            &'a self,
            _: &'a Arc<Server>,
            event: &'a mut PlayerInteractEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                if event.clicked_pos == Some(self.pot) {
                    event.player.gamemode.store(GameMode::Spectator);
                } else {
                    event.cancelled = self.veto.load(Ordering::Relaxed);
                }
            })
        }
    }

    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let chunk = boat_placement_platform(&world);
    let pot_pos = BlockPos::new(8, 63, 8);
    let chest_pos = BlockPos::new(9, 63, 8);
    chunk.set_block_absolute_y(8, 63, 8, Block::DECORATED_POT.default_state.id);
    chunk.set_block_absolute_y(9, 63, 8, Block::CHEST.default_state.id);
    let pot = Arc::new(DecoratedPotBlockEntity::new(pot_pos));
    world.add_block_entity(pot.clone());
    world.add_block_entity(Arc::new(ChestBlockEntity::new(chest_pos)));
    let (java, _peer) = java_rider(&server, &world, "Notch")?;
    let (bedrock, mut packets) = bedrock_rider(&server, &world, "jeb_").await?;
    let java_client = java.client.java().ok_or("Java player")?;
    let bedrock_client = bedrock.client.bedrock().ok_or("Bedrock player")?;
    for player in [&java, &bedrock] {
        player.get_entity().set_pos(BOAT_PLACEMENT_OFFSET);
    }
    let stack = ItemStack::new(1, &Item::OAK_BOAT);
    java.inventory().set_held_item(stack.clone());
    bedrock.inventory().set_held_item(stack.clone());
    bedrock.gamemode.store(GameMode::Spectator);
    let observer = Arc::new(SpectatorClick {
        veto: AtomicBool::new(true),
        pot: pot_pos,
    });
    server
        .plugin_manager
        .register_blocking::<PlayerInteractEvent, _>(observer.clone(), EventPriority::Normal, None);

    java_client.handle_use_item_on(
        &java,
        &SUseItemOn {
            hand: 0.into(),
            position: pot_pos,
            face: 1.into(),
            cursor_pos: Vector3::default(),
            inside_block: false,
            is_against_world_border: false,
            sequence: 1.into(),
        },
        &server,
    )?;
    assert_eq!(java.gamemode.load(), GameMode::Spectator);
    assert_eq!(java.inventory().held_item().uid, stack.uid);
    assert!(pot.get_item().is_none_or(|item| item.is_empty()));

    let click_chest = |id| {
        let mut packet = bedrock_use_item_transaction(0, &stack, 0);
        packet.legacy_request_id = pumpkin_protocol::codec::var_int::VarInt(id);
        if let TransactionData::UseItem(data) = &mut packet.transaction_data {
            data.block_position = chest_pos;
        }
        bedrock_client.handle_inventory_action(&bedrock, packet);
    };
    let window_type = || {
        bedrock
            .current_screen_handler
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .window_type()
    };
    let original_window = window_type();
    click_chest(-7);
    assert_eq!(window_type(), original_window);
    assert_eq!(bedrock_boat_legacy_responses(&mut packets)?, [(-7, 1)]);
    observer.veto.store(false, Ordering::Relaxed);
    click_chest(-9);
    assert_ne!(window_type(), original_window);
    assert_eq!(bedrock_boat_legacy_responses(&mut packets)?, [(-9, 0)]);
    assert_eq!(bedrock.inventory().held_item().uid, stack.uid);
    bedrock_client.clear_player();
    bedrock_client.close().await;
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[allow(
    clippy::too_many_lines,
    reason = "The same offhand stack is tested before and after its cooldown."
)]
async fn bedrock_boat_offhand_firework_commits_before_spawn_observers() -> TestResult {
    use pumpkin_data::{
        data_component_impl::{UseCooldownImpl, UseRemainderImpl},
        item::Item,
        item_stack::ItemStack,
        statistic::StatisticCategory,
    };
    use pumpkin_inventory::player::player_inventory::PlayerInventory;
    use pumpkin_protocol::{
        bedrock::server::inventory_transaction::{HandSlot, TransactionData},
        java::client::play::CSpawnEntity,
    };

    const GROUP: &str = "minecraft:vehicle_rocket_test";

    struct MoveRocketSource {
        rider: Arc<crate::entity::player::Player>,
        saw_empty_source: AtomicBool,
        saw_cooldown: AtomicBool,
    }
    impl EventRwHandler<PacketSentEvent> for MoveRocketSource {
        fn handle<'a>(
            &'a self,
            _: &'a Arc<Server>,
            event: &'a mut PacketSentEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                if event.packet_id == CSpawnEntity::to_id(CURRENT_MC_VERSION)
                    && let Ok(id) = event.payload.as_ref().get_var_int()
                    && let Some(tracked) =
                        self.rider.world().entity_tracker.get_tracked_entity(id.0)
                    && tracked.entity.get_entity().entity_type == &EntityType::FIREWORK_ROCKET
                {
                    let inventory = self.rider.inventory();
                    let offhand = inventory.get_slot(PlayerInventory::OFF_HAND_SLOT);
                    self.saw_empty_source
                        .store(offhand.is_empty(), Ordering::Relaxed);
                    self.saw_cooldown
                        .store(self.rider.is_on_cooldown(GROUP), Ordering::Relaxed);
                    inventory.set_slot(2, offhand);
                    inventory.set_slot(
                        PlayerInventory::OFF_HAND_SLOT,
                        ItemStack::new(1, &Item::STICK),
                    );
                }
            })
        }
    }

    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let (watcher, _peer) = java_equipment_watcher(&server, &world)?;
    watcher
        .chunk_sender
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .mark_sent_out_of_band(Vector2::new(0, 0));
    boat_placement_platform(&world);
    let (player, _packets) = bedrock_rider(&server, &world, "jeb_").await?;
    player.get_entity().set_pos(BOAT_PLACEMENT_OFFSET);
    let boat = ItemStack::new(1, &Item::OAK_BOAT);
    let mut rocket = ItemStack::new(1, &Item::FIREWORK_ROCKET);
    rocket.set_data_component(UseCooldownImpl {
        seconds: 2.0,
        cooldown_group: Some(GROUP.to_owned()),
    });
    rocket.set_data_component(UseRemainderImpl {
        remainder: Some(ItemStack::new(1, &Item::PAPER)),
    });
    player.inventory().set_held_item(boat.clone());
    player
        .inventory()
        .set_slot(PlayerInventory::OFF_HAND_SLOT, rocket.clone());
    let observer = Arc::new(MoveRocketSource {
        rider: player.clone(),
        saw_empty_source: AtomicBool::new(false),
        saw_cooldown: AtomicBool::new(false),
    });
    server
        .plugin_manager
        .register_blocking::<PacketSentEvent, _>(observer.clone(), EventPriority::Normal, None);
    let client = player.client.bedrock().ok_or("Bedrock rider")?;
    let click = || -> TestResult {
        let mut packet = bedrock_use_item_transaction(0, &rocket, 0);
        let TransactionData::UseItem(data) = &mut packet.transaction_data else {
            return Err("expected Bedrock block use".into());
        };
        data.hand = HandSlot::Offhand;
        data.item_in_hand_network_id = Some(rocket.uid);
        client.handle_inventory_action(&player, packet);
        Ok(())
    };
    player.start_cooldown(GROUP.to_owned(), 20);
    click()?;
    assert_eq!(
        player
            .inventory()
            .get_slot(PlayerInventory::OFF_HAND_SLOT)
            .uid,
        rocket.uid
    );
    assert_eq!(
        player.get_stat(StatisticCategory::Used, i32::from(Item::FIREWORK_ROCKET.id)),
        0
    );
    assert!(!observer.saw_empty_source.load(Ordering::Relaxed));
    assert!(player.inventory().get_slot(1).is_empty());
    player.tick_counter.store(20, Ordering::Relaxed);
    click()?;
    assert!(observer.saw_cooldown.load(Ordering::Relaxed));
    assert!(observer.saw_empty_source.load(Ordering::Relaxed));
    assert!(player.inventory().get_slot(2).is_empty());
    assert_eq!(player.inventory().held_item().uid, boat.uid);
    let remainder = player.inventory().get_slot(1);
    assert_eq!(remainder.get_item(), &Item::PAPER);
    assert_eq!(remainder.item_count, 1);
    assert_eq!(
        player
            .inventory()
            .get_slot(PlayerInventory::OFF_HAND_SLOT)
            .get_item(),
        &Item::STICK
    );
    assert_eq!(
        player.get_stat(StatisticCategory::Used, i32::from(Item::FIREWORK_ROCKET.id)),
        1
    );
    client.clear_player();
    client.close().await;
    world.level.shutdown().await;
    Ok(())
}
