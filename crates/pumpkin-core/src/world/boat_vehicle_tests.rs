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
        BoxFuture, EventHandler, EventPriority,
        api::events::{
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

impl EventHandler<GenericGameEvent> for BounceObservations {
    fn handle_blocking<'a>(
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

impl EventHandler<PacketSentEvent> for BounceObservations {
    fn handle_blocking<'a>(
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
    server.plugin_manager.register::<GenericGameEvent, _>(
        observed.clone(),
        EventPriority::Normal,
        true,
    );
    server.plugin_manager.register::<PacketSentEvent, _>(
        observed.clone(),
        EventPriority::Normal,
        true,
    );
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
    impl EventHandler<PacketSentEvent> for ObserveLaunch {
        fn handle_blocking<'a>(
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
        server.plugin_manager.register::<PacketSentEvent, _>(
            observed.clone(),
            EventPriority::Normal,
            true,
        );
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

    impl EventHandler<VehicleMoveEvent> for TeleportBoat {
        fn handle_blocking<'a>(
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
    server.plugin_manager.register::<VehicleMoveEvent, _>(
        listener.clone(),
        EventPriority::Normal,
        true,
    );

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
    impl EventHandler<VehicleEnterEvent> for RelocateOnEnter {
        fn handle_blocking<'a>(
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
    server.plugin_manager.register::<VehicleEnterEvent, _>(
        listener.clone(),
        EventPriority::Normal,
        true,
    );
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

impl EventHandler<VehicleMoveEvent> for ChangeSurface {
    fn handle_blocking<'a>(
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
    server.plugin_manager.register::<VehicleMoveEvent, _>(
        listener.clone(),
        EventPriority::Normal,
        true,
    );

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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn boat_placement_checks_fences_rooted_below_the_hull() -> TestResult {
    use crate::item::{ItemBehaviour, items::boat::BoatItem};
    use pumpkin_data::{item::Item, item_stack::ItemStack};
    use pumpkin_util::Hand;

    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let chunk = ChunkData::empty_sync(0, 0);
    chunk.set_block_absolute_y(8, 63, 8, Block::STONE.default_state.id);
    chunk.set_block_absolute_y(9, 63, 8, Block::OAK_FENCE.default_state.id);
    world
        .level
        .loaded_chunks
        .insert(Vector2::new(0, 0), chunk.clone());
    let (player, _peer) = java_rider(&server, &world, "Notch")?;
    let position = Vector3::new(8.9, 64.0, 8.5);
    player.get_entity().set_pos(position);
    player
        .inventory
        .set_stack_in_hand(Hand::Right, ItemStack::new(1, &Item::OAK_BOAT));
    let boats = || {
        world
            .entities
            .load()
            .iter()
            .filter(|entity| entity.get_entity().entity_type == &EntityType::OAK_BOAT)
            .count()
    };

    BoatItem.normal_use_with_hand(&Item::OAK_BOAT, &player, 0.0, 90.0, Hand::Right);
    assert_eq!(boats(), 0);
    assert_eq!(
        player.inventory.get_stack_in_hand(Hand::Right).item_count,
        1
    );

    chunk.set_block_absolute_y(9, 63, 8, Block::AIR.default_state.id);
    BoatItem.normal_use_with_hand(&Item::OAK_BOAT, &player, 0.0, 90.0, Hand::Right);
    assert_eq!(boats(), 1);
    assert!(player.inventory.get_stack_in_hand(Hand::Right).is_empty());
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
    let directory = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&directory).await?;
    // Keep the boat alive for both inside-block passes so the assertion tests
    // the order of effects independently of the damage threshold.
    server
        .plugin_manager
        .register::<crate::plugin::api::events::vehicle::vehicle_damage::VehicleDamageEvent, _>(
            Arc::new(PreventVehicleDamage),
            EventPriority::Normal,
            true,
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
        boat.get_entity().fire_ticks.store(10, Ordering::Relaxed);
        boat.get_entity().set_frozen_ticks(40);
        boat.tick(&boat, &server);
        assert!(!boat.get_entity().is_removed());
        assert!(!boat.get_entity().is_on_fire());
        assert_eq!(boat.get_entity().get_frozen_ticks(), 0);
    }
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ride_command_uses_the_boat_exit_when_the_seat_is_below_the_floor() -> TestResult {
    use crate::command::CommandSender;

    struct VetoExit(AtomicBool);

    impl EventHandler<VehicleExitEvent> for VetoExit {
        fn handle_blocking<'a>(
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
    server.plugin_manager.register::<VehicleExitEvent, _>(
        listener.clone(),
        EventPriority::Normal,
        true,
    );
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

impl EventHandler<GenericGameEvent> for ObserveBoatMount {
    fn handle_blocking<'a>(
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

impl EventHandler<PacketSentEvent> for ObserveBoatMount {
    fn handle_blocking<'a>(
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
    server.plugin_manager.register::<GenericGameEvent, _>(
        listener.clone(),
        EventPriority::Normal,
        true,
    );
    server.plugin_manager.register::<PacketSentEvent, _>(
        listener.clone(),
        EventPriority::Normal,
        true,
    );

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

    impl EventHandler<GenericGameEvent> for MountObserver {
        fn handle_blocking<'a>(
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
    server.plugin_manager.register::<GenericGameEvent, _>(
        observer.clone(),
        EventPriority::Normal,
        true,
    );
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

impl EventHandler<PacketSentEvent> for ReplaceStandingPose {
    fn handle_blocking<'a>(
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

    impl EventHandler<GenericGameEvent> for DismountOnMount {
        fn handle_blocking<'a>(
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
    server.plugin_manager.register::<GenericGameEvent, _>(
        Arc::new(DismountOnMount {
            boat: boat.clone(),
            rider_id: rider.entity_id,
        }),
        EventPriority::Normal,
        true,
    );
    let observer = Arc::new(ReplaceStandingPose {
        rider: player.clone(),
        fired: AtomicBool::new(false),
    });
    if replace_pose {
        server.plugin_manager.register::<PacketSentEvent, _>(
            observer.clone(),
            EventPriority::Normal,
            true,
        );
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
    pose: Option<i32>,
    width: Option<f32>,
    height: Option<f32>,
    scale: Option<f32>,
    flags: Option<i64>,
    flags_two: Option<i64>,
    name: Option<String>,
    last_equipment_id: Option<i16>,
    last_equipment_uid: Option<std::num::NonZeroI32>,
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
                CMobEquipment,
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

impl EventHandler<PacketSentEvent> for ResizeOnPose {
    fn handle_blocking<'a>(
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
    server.plugin_manager.register::<PacketSentEvent, _>(
        observer.clone(),
        EventPriority::Normal,
        true,
    );

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

impl EventHandler<PacketSentEvent> for ExpireAngerOnMountFlags {
    fn handle_blocking<'a>(
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
    server.plugin_manager.register::<PacketSentEvent, _>(
        observer.clone(),
        EventPriority::Normal,
        true,
    );
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

#[derive(Clone, Copy)]
enum ShieldBreakReplacement {
    Sword,
    SwordDuringClear,
    SwordDuringAttribute,
    Bow,
    RestartOnClear,
    RestartAtFinish,
    Shield(pumpkin_util::Hand),
}

struct DetachOnShieldBreak {
    pig: Arc<PigEntity>,
    boat: Arc<BoatEntity>,
    replacement: ShieldBreakReplacement,
    detached: AtomicBool,
    equipment_packets: std::sync::atomic::AtomicU8,
}

impl DetachOnShieldBreak {
    fn assert_replacement(&self) {
        use crate::entity::mob::Mob;
        use pumpkin_data::{
            attributes::Attributes, data_component_impl::EquipmentSlot, item::Item,
        };

        let living = &self.pig.get_mob_entity().living_entity;
        assert!(self.detached.load(Ordering::Relaxed));
        assert!(!self.pig.get_entity().has_vehicle());
        assert!(
            !self
                .boat
                .get_entity()
                .has_passenger(self.pig.get_entity().entity_id)
        );
        match self.replacement {
            ShieldBreakReplacement::Sword
            | ShieldBreakReplacement::SwordDuringClear
            | ShieldBreakReplacement::SwordDuringAttribute => {
                let active_slot = living
                    .entity_equipment
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .get(&EquipmentSlot::MAIN_HAND);
                let attack_damage = living.get_attribute_value(&Attributes::ATTACK_DAMAGE);
                let base_damage = living.get_attribute_base(&Attributes::ATTACK_DAMAGE);
                if active_slot.item == &Item::STICK {
                    assert_eq!(
                        attack_damage.to_bits(),
                        base_damage.to_bits(),
                        "A stick cannot retain sword damage"
                    );
                }
                assert_eq!(active_slot.item, &Item::DIAMOND_SWORD);
                assert!(attack_damage > base_damage);
                assert!(!living.is_blocking());
                let expected_packets =
                    if matches!(self.replacement, ShieldBreakReplacement::SwordDuringClear) {
                        2
                    } else {
                        1
                    };
                assert_eq!(
                    self.equipment_packets.load(Ordering::Relaxed),
                    expected_packets
                );
            }
            ShieldBreakReplacement::Shield(hand) => {
                let equipment = living
                    .entity_equipment
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner);
                let slot = shield_test_slot(hand);
                assert_eq!(equipment.get(&slot).item, &Item::SHIELD);
                if hand == pumpkin_util::Hand::Left {
                    assert!(equipment.get(&EquipmentSlot::MAIN_HAND).is_empty());
                }
                drop(equipment);
                assert!(living.is_blocking());
                assert_eq!(
                    *living
                        .active_hand
                        .lock()
                        .unwrap_or_else(PoisonError::into_inner),
                    Some(hand)
                );
            }
            ShieldBreakReplacement::Bow => {
                assert!(!living.is_blocking());
                assert_eq!(
                    *living
                        .active_hand
                        .lock()
                        .unwrap_or_else(PoisonError::into_inner),
                    Some(pumpkin_util::Hand::Right)
                );
            }
            ShieldBreakReplacement::RestartOnClear | ShieldBreakReplacement::RestartAtFinish => {
                assert!(living.is_blocking());
            }
        }
    }
}

fn shield_test_slot(hand: pumpkin_util::Hand) -> pumpkin_data::data_component_impl::EquipmentSlot {
    use pumpkin_data::data_component_impl::EquipmentSlot;
    match hand {
        pumpkin_util::Hand::Left => EquipmentSlot::OFF_HAND,
        pumpkin_util::Hand::Right => EquipmentSlot::MAIN_HAND,
    }
}

impl DetachOnShieldBreak {
    fn append_attribute_extra(event: &mut PacketSentEvent, rider_id: i32) {
        use pumpkin_protocol::java::client::play::CUpdateAttributes;
        let properties = [
            (pumpkin_data::attributes::Attributes::ATTACK_SPEED, 77.0),
            (pumpkin_data::attributes::Attributes::MOVEMENT_SPEED, 0.75),
        ]
        .into_iter()
        .map(|(attribute, value)| {
            pumpkin_protocol::java::client::play::Property::new(
                i32::from(attribute.id).into(),
                value,
                Vec::new(),
            )
        })
        .collect();
        let extra = CUpdateAttributes::new(rider_id.into(), properties);
        let mut payload = Vec::new();
        assert!(
            pumpkin_protocol::ClientPacket::write_packet_data(
                &extra,
                &mut payload,
                &CURRENT_MC_VERSION
            )
            .is_ok()
        );
        event.extra_packets.push((event.packet_id, payload.into()));
    }

    fn replace_shield_from_break_status(&self) {
        use crate::entity::mob::Mob;
        use pumpkin_data::{data_component_impl::EquipmentSlot, item::Item, item_stack::ItemStack};

        let mob = self.pig.get_mob_entity();
        match self.replacement {
            ShieldBreakReplacement::Sword => {
                mob.set_item_slot(
                    &EquipmentSlot::MAIN_HAND,
                    ItemStack::new(1, &Item::DIAMOND_SWORD),
                );
            }
            ShieldBreakReplacement::Shield(hand) => {
                let shield = ItemStack::new(1, &Item::SHIELD);
                mob.set_item_slot(&shield_test_slot(hand), shield.clone());
                let use_time = shield.get_max_use_time() - 20;
                mob.living_entity.set_active_hand(hand, shield, use_time);
            }
            ShieldBreakReplacement::Bow => {
                let bow = ItemStack::new(1, &Item::BOW);
                mob.set_item_slot(&EquipmentSlot::MAIN_HAND, bow.clone());
                let duration = bow.get_max_use_time();
                mob.living_entity
                    .set_active_hand(pumpkin_util::Hand::Right, bow, duration);
            }
            ShieldBreakReplacement::SwordDuringClear
            | ShieldBreakReplacement::SwordDuringAttribute
            | ShieldBreakReplacement::RestartOnClear
            | ShieldBreakReplacement::RestartAtFinish => {}
        }
    }
}

impl EventHandler<PacketSentEvent> for DetachOnShieldBreak {
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut PacketSentEvent,
    ) -> BoxFuture<'a, ()> {
        use crate::entity::mob::Mob;
        use pumpkin_data::{data_component_impl::EquipmentSlot, item::Item, item_stack::ItemStack};
        use pumpkin_protocol::java::client::play::{
            CEntityStatus, CSetEquipment, CUpdateAttributes,
        };

        use pumpkin_protocol::ServerPacket;
        Box::pin(async move {
            if event.player.client.java().is_none() {
                return;
            }
            let rider = self.pig.get_entity();
            let mob = self.pig.get_mob_entity();
            if matches!(
                self.replacement,
                ShieldBreakReplacement::SwordDuringAttribute
            ) && event.packet_id == CUpdateAttributes::to_id(CURRENT_MC_VERSION)
                && self.detached.load(Ordering::Relaxed)
                && CUpdateAttributes::read(&mut event.payload.as_ref(), &CURRENT_MC_VERSION)
                    .is_ok_and(|packet| {
                        packet.entity_id.0 == rider.entity_id
                            && packet.properties.iter().any(|property| {
                                property.id.0
                                    == i32::from(
                                        pumpkin_data::attributes::Attributes::ATTACK_SPEED.id,
                                    )
                            })
                    })
                && self.equipment_packets.fetch_add(1, Ordering::Relaxed) == 0
            {
                mob.set_item_slot(
                    &EquipmentSlot::MAIN_HAND,
                    ItemStack::new(1, &Item::DIAMOND_SWORD),
                );
                Self::append_attribute_extra(event, rider.entity_id);
            }
            if event.packet_id == CSetEquipment::to_id(CURRENT_MC_VERSION)
                && self.detached.load(Ordering::Relaxed)
                && event
                    .payload
                    .as_ref()
                    .get_var_int()
                    .is_ok_and(|id| id.0 == rider.entity_id)
            {
                if matches!(
                    self.replacement,
                    ShieldBreakReplacement::SwordDuringAttribute
                ) {
                    return;
                }
                let previous = self.equipment_packets.fetch_add(1, Ordering::Relaxed);
                if matches!(self.replacement, ShieldBreakReplacement::SwordDuringClear)
                    && previous == 0
                {
                    mob.set_item_slot(
                        &EquipmentSlot::MAIN_HAND,
                        ItemStack::new(1, &Item::DIAMOND_SWORD),
                    );
                } else if matches!(self.replacement, ShieldBreakReplacement::Sword) && previous == 1
                {
                    // A replay gives observers another chance to change the slot.
                    mob.set_item_slot(&EquipmentSlot::MAIN_HAND, ItemStack::new(1, &Item::STICK));
                }
                return;
            }
            if event.packet_id == CEntityStatus::to_id(CURRENT_MC_VERSION)
                && event
                    .payload
                    .get(..4)
                    .is_some_and(|id| id == rider.entity_id.to_be_bytes())
                && event.payload.get(4)
                    == Some(
                        &(crate::entity::equipment_break_status(&EquipmentSlot::MAIN_HAND) as u8),
                    )
                && !self.detached.swap(true, Ordering::Relaxed)
            {
                assert!(mob.living_entity.entity_equipment.try_lock().is_ok());
                self.boat.get_entity().remove_passenger(rider.entity_id);
                self.replace_shield_from_break_status();
            }
        })
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn shield_break_observer_can_dismount_and_replace_the_active_item() -> TestResult {
    use pumpkin_util::Hand;
    for replacement in [
        ShieldBreakReplacement::Sword,
        ShieldBreakReplacement::SwordDuringClear,
        ShieldBreakReplacement::SwordDuringAttribute,
        ShieldBreakReplacement::Bow,
        ShieldBreakReplacement::RestartOnClear,
        ShieldBreakReplacement::RestartAtFinish,
        ShieldBreakReplacement::Shield(Hand::Left),
        ShieldBreakReplacement::Shield(Hand::Right),
    ] {
        assert_shield_break_replacement(replacement).await?;
    }
    Ok(())
}

async fn assert_shield_break_replacement(replacement: ShieldBreakReplacement) -> TestResult {
    use crate::entity::mob::Mob;
    use pumpkin_data::damage::DamageType;
    use pumpkin_protocol::java::client::play::CKeepAlive;
    use pumpkin_util::Hand;

    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let (watcher, peer) = java_equipment_watcher(&server, &world)?;
    watcher
        .chunk_sender
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .mark_sent_out_of_band(Vector2::new(0, 0));
    let client = watcher.client.java().ok_or("Java watcher")?;
    let boat = Arc::new(BoatEntity::new(Entity::new(
        world.clone(),
        CENTER,
        &EntityType::OAK_BOAT,
    )));
    let pig = PigEntity::new(Entity::new(world.clone(), CENTER, &EntityType::PIG));
    world.spawn_entity(boat.clone());
    world.spawn_entity(pig.clone());
    boat.get_entity().add_passenger(boat.clone(), pig.clone());
    assert_mounted(boat.get_entity(), pig.get_entity());
    let living = &pig.get_mob_entity().living_entity;
    let shield = test_breaking_shield(&pig, replacement)?;
    let elapsed = shield.get_max_use_time() - 20;
    let flags_check = if matches!(
        replacement,
        ShieldBreakReplacement::Bow
            | ShieldBreakReplacement::RestartOnClear
            | ShieldBreakReplacement::RestartAtFinish
    ) {
        Some(watch_shield_use_flags(&server, &world, &pig, replacement).await?)
    } else {
        None
    };
    living.set_active_hand(Hand::Right, shield, elapsed);
    assert!(living.is_blocking());
    let observer = Arc::new(DetachOnShieldBreak {
        pig: pig.clone(),
        boat: boat.clone(),
        replacement,
        detached: AtomicBool::new(false),
        equipment_packets: std::sync::atomic::AtomicU8::new(0),
    });
    server.plugin_manager.register::<PacketSentEvent, _>(
        observer.clone(),
        EventPriority::Normal,
        true,
    );
    living.damage_with_context(
        pig.as_ref(),
        10.0,
        DamageType::MOB_ATTACK,
        Some(CENTER.add_raw(0.0, 0.0, 1.0)),
        None,
        None,
    );
    if matches!(replacement, ShieldBreakReplacement::SwordDuringAttribute) {
        use pumpkin_data::attributes::Attributes;
        assert!(observer.detached.load(Ordering::Relaxed));
        let speed = living.get_attribute_value(&Attributes::ATTACK_SPEED);
        assert!(speed < living.get_attribute_base(&Attributes::ATTACK_SPEED));
        assert!(client.try_send_packet(&CKeepAlive::new(73)));
        let (attack, movement) =
            read_rider_attribute_updates(peer, pig.get_entity().entity_id).await?;
        assert_eq!(attack.map(f64::to_bits), Some(speed.to_bits()));
        assert_eq!(movement, Some(0.75));
        world.level.shutdown().await;
        return Ok(());
    }
    observer.assert_replacement();
    if let Some((restart, mut packets)) = flags_check {
        restart.assert_outcome(&mut packets, elapsed)?;
    }
    assert!(client.try_send_packet(&CKeepAlive::new(73)));
    assert_shield_replacement_packets(peer, pig.get_entity().entity_id, replacement).await?;
    world.level.shutdown().await;
    Ok(())
}

fn test_breaking_shield(
    pig: &PigEntity,
    replacement: ShieldBreakReplacement,
) -> Result<pumpkin_data::item_stack::ItemStack, Box<dyn std::error::Error>> {
    use crate::entity::mob::Mob;
    use pumpkin_data::{
        data_component_impl::{AttributeModifiersImpl, EquipmentSlot},
        item::Item,
        item_stack::ItemStack,
    };

    let mob = pig.get_mob_entity();
    let mut shield = ItemStack::new(1, &Item::SHIELD);
    shield.set_damage(shield.get_max_damage().ok_or("no durability")? - 1);
    if matches!(replacement, ShieldBreakReplacement::SwordDuringAttribute) {
        let sword = ItemStack::new(1, &Item::DIAMOND_SWORD);
        shield.set_data_component(
            sword
                .get_data_component::<AttributeModifiersImpl>()
                .ok_or("sword modifiers")?
                .clone(),
        );
        mob.set_item_slot(&EquipmentSlot::MAIN_HAND, shield.clone());
    } else {
        mob.living_entity
            .entity_equipment
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .put(&EquipmentSlot::MAIN_HAND, shield.clone());
    }
    Ok(shield)
}

async fn assert_shield_replacement_packets(
    peer: tokio::io::DuplexStream,
    rider_id: i32,
    replacement: ShieldBreakReplacement,
) -> TestResult {
    use pumpkin_data::{data_component_impl::EquipmentSlot, item::Item};
    use pumpkin_util::Hand;

    let (equipment, _) = read_rider_equipment_updates(peer, rider_id).await?;
    let main = EquipmentSlot::MAIN_HAND.discriminant();
    let off = EquipmentSlot::OFF_HAND.discriminant();
    let expected = match replacement {
        ShieldBreakReplacement::Sword
        | ShieldBreakReplacement::SwordDuringClear
        | ShieldBreakReplacement::SwordDuringAttribute => {
            vec![(main, Some(&Item::DIAMOND_SWORD))]
        }
        ShieldBreakReplacement::Bow => vec![(main, Some(&Item::BOW))],
        ShieldBreakReplacement::RestartOnClear | ShieldBreakReplacement::RestartAtFinish => {
            vec![(main, None), (off, Some(&Item::SHIELD))]
        }
        ShieldBreakReplacement::Shield(Hand::Left) => {
            vec![(off, Some(&Item::SHIELD)), (main, None)]
        }
        ShieldBreakReplacement::Shield(Hand::Right) => vec![(main, Some(&Item::SHIELD))],
    };
    let items = equipment
        .iter()
        .map(|(slot, stack)| (*slot, (!stack.is_empty()).then_some(stack.item)))
        .collect::<Vec<_>>();
    assert_eq!(items, expected);
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

async fn read_rider_attribute_updates(
    peer: tokio::io::DuplexStream,
    rider_id: i32,
) -> Result<(Option<f64>, Option<f64>), Box<dyn std::error::Error>> {
    use pumpkin_data::attributes::Attributes;
    use pumpkin_protocol::{
        ServerPacket,
        java::{
            client::play::{CKeepAlive, CUpdateAttributes},
            packet_decoder::TCPNetworkDecoder,
        },
    };
    let mut decoder = TCPNetworkDecoder::new(tokio::io::BufReader::new(peer));
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        let mut speed = None;
        let mut movement = None;
        loop {
            let packet = decoder.get_raw_packet().await?;
            if packet.id == CKeepAlive::to_id(CURRENT_MC_VERSION) {
                return Ok((speed, movement));
            }
            if packet.id == CUpdateAttributes::to_id(CURRENT_MC_VERSION) {
                let packet =
                    CUpdateAttributes::read(&mut packet.payload.as_ref(), &CURRENT_MC_VERSION)?;
                if packet.entity_id.0 == rider_id {
                    for property in packet.properties {
                        if property.id.0 == i32::from(Attributes::MOVEMENT_SPEED.id) {
                            movement = Some(property.value);
                        }
                        if property.id.0 == i32::from(Attributes::ATTACK_SPEED.id) {
                            speed = Some(
                                property.value
                                    + property
                                        .modifiers
                                        .iter()
                                        .map(|modifier| modifier.amount)
                                        .sum::<f64>(),
                            );
                        }
                    }
                }
            }
        }
    })
    .await?
}

struct ObserveShieldUse {
    pig: Arc<PigEntity>,
    replacement: ShieldBreakReplacement,
    start: Mutex<Option<(Option<pumpkin_util::Hand>, i32, bool)>>,
    restarted: AtomicBool,
    finish: Mutex<Option<(Option<pumpkin_util::Hand>, i32, bool)>>,
}

impl ObserveShieldUse {
    const SUCCESSOR_DURATION: i32 = 71_927;
    const SUCCESSOR_DAMAGE: i32 = 7;
    fn assert_outcome(
        &self,
        packets: &mut super::vehicle_tests::BedrockQueue,
        elapsed: i32,
    ) -> TestResult {
        use pumpkin_protocol::bedrock::client::set_actor_data::entity_data_flag::{
            BLOCKING, USING_ITEM,
        };
        use pumpkin_util::Hand;
        let restart = !matches!(self.replacement, ShieldBreakReplacement::Bow);
        let metadata = read_bedrock_seat_metadata(packets, self.pig.get_entity().entity_id)?;
        assert_ne!(
            metadata.flags.ok_or("Missing use flags")? & (1i64 << USING_ITEM),
            0
        );
        assert_eq!(
            metadata.flags_two.ok_or("Missing blocking flags")? & (1i64 << (BLOCKING - 64)) != 0,
            restart,
        );
        assert_eq!(self.restarted.load(Ordering::Relaxed), restart);
        let finish = *self.finish.lock().unwrap_or_else(PoisonError::into_inner);
        assert_eq!(
            finish,
            restart.then_some((Some(Hand::Right), elapsed, false))
        );
        assert_eq!(
            *self.start.lock().unwrap_or_else(PoisonError::into_inner),
            Some((Some(Hand::Right), elapsed, true)),
        );
        if restart {
            use crate::entity::mob::Mob;
            let living = &self.pig.get_mob_entity().living_entity;
            assert_eq!(
                *living
                    .active_hand
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner),
                Some(Hand::Left)
            );
            assert_ne!(elapsed, Self::SUCCESSOR_DURATION);
            assert_eq!(
                living.item_use_time.load(Ordering::Relaxed),
                Self::SUCCESSOR_DURATION
            );
            let off = mob_offhand(living);
            assert_eq!(off.get_damage(), Self::SUCCESSOR_DAMAGE);
            let active = living
                .item_in_use
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            assert!(active.as_ref().is_some_and(|stack| stack.are_equal(&off)));
        }
        Ok(())
    }

    fn restart_shield_use(&self) {
        use crate::entity::mob::Mob;
        use pumpkin_data::{data_component_impl::EquipmentSlot, item::Item, item_stack::ItemStack};
        let mob = self.pig.get_mob_entity();
        let mut shield = ItemStack::new(1, &Item::SHIELD);
        shield.set_damage(Self::SUCCESSOR_DAMAGE);
        mob.set_item_slot(&EquipmentSlot::OFF_HAND, shield.clone());
        mob.living_entity.set_active_hand(
            pumpkin_util::Hand::Left,
            shield,
            Self::SUCCESSOR_DURATION,
        );
    }
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

impl EventHandler<GenericGameEvent> for ObserveShieldUse {
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut GenericGameEvent,
    ) -> BoxFuture<'a, ()> {
        use crate::entity::mob::Mob;
        use pumpkin_protocol::bedrock::client::set_actor_data::entity_data_flag::USING_ITEM;
        Box::pin(async move {
            let start = event.event_key == GameEvent::ItemInteractStart.name();
            if start || event.event_key == GameEvent::ItemInteractFinish.name() {
                let living = &self.pig.get_mob_entity().living_entity;
                let hand = *living
                    .active_hand
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner);
                let using =
                    living.entity.bedrock_flags.load(Ordering::Relaxed) & (1i64 << USING_ITEM) != 0;
                let state = (hand, living.item_use_time.load(Ordering::Relaxed), using);
                if start {
                    self.start
                        .lock()
                        .unwrap_or_else(PoisonError::into_inner)
                        .get_or_insert(state);
                } else {
                    *self.finish.lock().unwrap_or_else(PoisonError::into_inner) = Some(state);
                    if matches!(self.replacement, ShieldBreakReplacement::RestartAtFinish)
                        && !self.restarted.swap(true, Ordering::Relaxed)
                    {
                        self.restart_shield_use();
                    }
                }
            }
        })
    }
}

impl EventHandler<PacketSentEvent> for ObserveShieldUse {
    fn handle_blocking<'a>(
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
            let mob = self.pig.get_mob_entity();
            let living = &mob.living_entity;
            if matches!(self.replacement, ShieldBreakReplacement::RestartOnClear)
                && event.player.client.bedrock().is_some()
                && event.packet_id == CSetActorData::PACKET_ID
                && !living.is_auto_spin_attack()
                && living.item_use_time.load(Ordering::Relaxed) == 0
                && CSetActorData::updates_key(
                    &event.payload,
                    VarULong(self.pig.get_entity().entity_id as u64),
                    pumpkin_world::CURRENT_BEDROCK_MC_PROTOCOL,
                    entity_data_key::FLAGS,
                )
                .unwrap_or(false)
                && !self.restarted.swap(true, Ordering::Relaxed)
            {
                self.restart_shield_use();
            }
        })
    }
}

async fn watch_shield_use_flags(
    server: &Arc<Server>,
    world: &Arc<World>,
    pig: &Arc<PigEntity>,
    replacement: ShieldBreakReplacement,
) -> Result<(Arc<ObserveShieldUse>, super::vehicle_tests::BedrockQueue), Box<dyn std::error::Error>>
{
    let (viewer, mut packets) = bedrock_rider(server, world, "jeb_").await?;
    viewer
        .chunk_sender
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .mark_sent_out_of_band(Vector2::new(0, 0));
    let tracked = world
        .entity_tracker
        .get_tracked_entity(pig.get_entity().entity_id)
        .ok_or("tracker")?;
    tracked.update_player(&viewer, world);
    assert!(tracked.is_seen_by(&viewer.gameprofile.id));
    while packets.try_recv().is_ok() {}
    let observer = Arc::new(ObserveShieldUse {
        pig: pig.clone(),
        replacement,
        start: Mutex::new(None),
        restarted: AtomicBool::new(false),
        finish: Mutex::new(None),
    });
    server.plugin_manager.register::<GenericGameEvent, _>(
        observer.clone(),
        EventPriority::Normal,
        true,
    );
    server.plugin_manager.register::<PacketSentEvent, _>(
        observer.clone(),
        EventPriority::Normal,
        true,
    );
    Ok((observer, packets))
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bedrock_riptide_stop_keeps_item_use_started_by_its_observer() -> TestResult {
    use crate::entity::mob::Mob;
    use pumpkin_data::{item::Item, item_stack::ItemStack};
    use pumpkin_protocol::bedrock::client::set_actor_data::entity_data_flag::{
        BLOCKING, DAMAGE_NEARBY_MOBS, USING_ITEM,
    };
    let directory = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&directory).await?;
    world
        .level
        .loaded_chunks
        .insert(Vector2::new(0, 0), ChunkData::empty_sync(0, 0));
    let pig = PigEntity::new(Entity::new(world.clone(), CENTER, &EntityType::PIG));
    world.spawn_entity(pig.clone());
    let living = &pig.get_mob_entity().living_entity;
    let (observer, mut packets) = watch_shield_use_flags(
        &server,
        &world,
        &pig,
        ShieldBreakReplacement::RestartOnClear,
    )
    .await?;

    living.start_auto_spin_attack(20, 1.0, ItemStack::new(1, &Item::TRIDENT), None);
    assert!(living.is_auto_spin_attack());
    let start = read_bedrock_seat_metadata(&mut packets, pig.get_entity().entity_id)?;
    assert_ne!(
        start.flags.ok_or("Missing Riptide start flags")? & (1 << DAMAGE_NEARBY_MOBS),
        0
    );
    assert!(!observer.restarted.load(Ordering::Relaxed));

    living.stop_auto_spin_attack();

    assert!(observer.restarted.load(Ordering::Relaxed));
    assert!(!living.is_auto_spin_attack());
    assert!(living.is_blocking());
    let metadata = read_bedrock_seat_metadata(&mut packets, pig.get_entity().entity_id)?;
    assert_eq!(
        metadata.flags.ok_or("Missing use flags")?
            & ((1 << USING_ITEM) | (1 << DAMAGE_NEARBY_MOBS)),
        1 << USING_ITEM,
    );
    assert_ne!(
        metadata.flags_two.ok_or("Missing blocking flags")? & (1 << (BLOCKING - 64)),
        0
    );
    world.level.shutdown().await;
    Ok(())
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PairingEquipmentChange {
    Replace,
    Wear,
    Break,
    Retry,
    RePair,
    Armor,
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
            PairingEquipmentChange::RePair => {
                mob.set_item_slot(
                    &EquipmentSlot::MAIN_HAND,
                    ItemStack::new(1, &Item::DIAMOND_SWORD),
                );
                let world = rider.world.load_full();
                if let Some(tracked) = world.entity_tracker.get_tracked_entity(rider.entity_id) {
                    tracked.remove_player(&event.player);
                    tracked.update_player(&event.player, &world);
                }
            }
            PairingEquipmentChange::Replace => {
                mob.set_item_slot(
                    &EquipmentSlot::MAIN_HAND,
                    ItemStack::new(1, &Item::DIAMOND_SWORD),
                );
                if event.player.client.java().is_some() {
                    Self::append_viewer_equipment(event, rider.entity_id);
                }
            }
        }
    }

    fn append_viewer_equipment(event: &mut PacketSentEvent, rider_id: i32) {
        use pumpkin_data::{data_component_impl::EquipmentSlot, item::Item, item_stack::ItemStack};
        use pumpkin_protocol::java::client::play::CSetEquipment;
        use pumpkin_protocol::{ClientPacket, codec::item_stack_seralizer::ItemStackSerializer};
        let slots = [
            (EquipmentSlot::MAIN_HAND, &Item::STICK),
            (EquipmentSlot::OFF_HAND, &Item::CARROT),
            (EquipmentSlot::HEAD, &Item::GOLDEN_HELMET),
        ];
        let extra = CSetEquipment::new(
            rider_id.into(),
            slots
                .into_iter()
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

impl EventHandler<PacketSentEvent> for DetachOnPairingEquipment {
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut PacketSentEvent,
    ) -> BoxFuture<'a, ()> {
        use crate::entity::mob::Mob;
        use pumpkin_data::{data_component_impl::EquipmentSlot, item::Item, item_stack::ItemStack};
        use pumpkin_protocol::{
            bedrock::client::{CMobEquipment, mob_armor_equipment::CMobArmorEquipment},
            codec::var_ulong::VarULong,
            java::client::play::CSetEquipment,
            packet::Packet,
            serial::PacketRead,
        };

        Box::pin(async move {
            let rider = self.pig.get_entity();
            let target = if event.player.client.bedrock().is_some() {
                event.packet_id
                    == if self.change == PairingEquipmentChange::Armor {
                        CMobArmorEquipment::PACKET_ID
                    } else {
                        CMobEquipment::PACKET_ID
                    }
                    && VarULong::read(&mut event.payload.as_ref())
                        .is_ok_and(|id| id.0 == rider.entity_id as u64)
            } else {
                event.packet_id == CSetEquipment::to_id(CURRENT_MC_VERSION)
                    && event
                        .payload
                        .as_ref()
                        .get_var_int()
                        .is_ok_and(|id| id.0 == rider.entity_id)
            };
            if !target {
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
            }
        })
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bedrock_pairing_equipment_observer_can_dismount_the_rider() -> TestResult {
    for change in [
        PairingEquipmentChange::Replace,
        PairingEquipmentChange::Armor,
        PairingEquipmentChange::Retry,
    ] {
        assert_bedrock_pairing_equipment(change).await?;
    }
    Ok(())
}

async fn assert_bedrock_pairing_equipment(change: PairingEquipmentChange) -> TestResult {
    use crate::entity::mob::Mob;
    use pumpkin_data::{data_component_impl::EquipmentSlot, item::Item, item_stack::ItemStack};

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
    let (slot, item) = if change == PairingEquipmentChange::Armor {
        (EquipmentSlot::HEAD, &Item::GOLDEN_HELMET)
    } else {
        (EquipmentSlot::MAIN_HAND, &Item::DIAMOND_SWORD)
    };
    let initial = ItemStack::new(1, item);
    let initial_uid = initial.uid;
    pig.get_mob_entity().set_item_slot(&slot, initial);
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
    server.plugin_manager.register::<PacketSentEvent, _>(
        observer.clone(),
        EventPriority::Normal,
        true,
    );
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
    let metadata = read_bedrock_seat_metadata(&mut packets, rider.entity_id)?;
    let flags = metadata.flags.ok_or("detached flags")?;
    let riding =
        1i64 << pumpkin_protocol::bedrock::client::set_actor_data::entity_data_flag::RIDING;
    assert_eq!(flags & riding, 0);
    let equipped = pig
        .get_mob_entity()
        .living_entity
        .entity_equipment
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .get(&slot);
    if change == PairingEquipmentChange::Armor {
        assert_eq!(equipped.item, &Item::DIAMOND_HELMET);
        assert_eq!(metadata.armor_updates, 1);
        let expected =
            pumpkin_protocol::bedrock::network_item::NetworkItemStackDescriptor::from(&equipped);
        assert_eq!(
            metadata.last_head_equipment,
            Some((expected.id, expected.net_id))
        );
    } else {
        let expected_item = if change == PairingEquipmentChange::Retry {
            &Item::CARROT
        } else {
            &Item::DIAMOND_SWORD
        };
        assert_eq!(equipped.item, expected_item);
        assert_ne!(initial_uid, equipped.uid);
        let expected =
            pumpkin_protocol::bedrock::network_item::NetworkItemStackDescriptor::from(&equipped);
        assert_eq!(metadata.last_equipment_id, Some(expected.id));
        assert_eq!(metadata.last_equipment_uid, expected.net_id);
    }
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn java_pairing_keeps_the_replacement_and_the_unchanged_equipment() -> TestResult {
    for change in [
        PairingEquipmentChange::Replace,
        PairingEquipmentChange::Wear,
        PairingEquipmentChange::Break,
        PairingEquipmentChange::Retry,
        PairingEquipmentChange::RePair,
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
    server.plugin_manager.register::<PacketSentEvent, _>(
        observer.clone(),
        EventPriority::Normal,
        true,
    );
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
    use pumpkin_data::{data_component_impl::EquipmentSlot, item::Item};
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
        let main = if change == PairingEquipmentChange::Retry {
            &Item::CARROT
        } else {
            &Item::DIAMOND_SWORD
        };
        assert_eq!(last_item(&EquipmentSlot::MAIN_HAND), Some(main));
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
    impl EventHandler<PacketSentEvent> for ChangeAge {
        fn handle_blocking<'a>(
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
    server.plugin_manager.register::<PacketSentEvent, _>(
        observer.clone(),
        EventPriority::Normal,
        true,
    );
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

impl EventHandler<PacketSentEvent> for ChangeAiOnDismountFlags {
    fn handle_blocking<'a>(
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
    server.plugin_manager.register::<PacketSentEvent, _>(
        observer.clone(),
        EventPriority::Normal,
        true,
    );
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

impl EventHandler<crate::plugin::api::events::vehicle::vehicle_damage::VehicleDamageEvent>
    for PreventVehicleDamage
{
    fn handle_blocking<'a>(
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

    impl EventHandler<PacketSentEvent> for MountWhenPaddlesClear {
        fn handle_blocking<'a>(
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
    server.plugin_manager.register::<PacketSentEvent, _>(
        listener.clone(),
        EventPriority::Normal,
        true,
    );

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

    impl EventHandler<EntityMountEvent> for FloodOnMount {
        fn handle_blocking<'a>(
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
    server.plugin_manager.register::<EntityMountEvent, _>(
        observer.clone(),
        EventPriority::Normal,
        true,
    );
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

    impl EventHandler<EntityMountEvent> for VetoMount {
        fn handle_blocking<'a>(
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
    server.plugin_manager.register::<EntityMountEvent, _>(
        observer.clone(),
        EventPriority::Normal,
        true,
    );
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

struct ReplacePlayerItemUse {
    player: Arc<crate::entity::player::Player>,
    trigger: GameEvent,
    calls: std::sync::atomic::AtomicU32,
    replace: bool,
}

impl EventHandler<GenericGameEvent> for ReplacePlayerItemUse {
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut GenericGameEvent,
    ) -> BoxFuture<'a, ()> {
        use pumpkin_data::{item::Item, item_stack::ItemStack};
        use pumpkin_util::Hand;
        Box::pin(async move {
            if event.event_key == self.trigger.name()
                && self.calls.fetch_add(1, Ordering::Relaxed) == 0
                && self.replace
            {
                let mut shield = ItemStack::new(1, &Item::SHIELD);
                shield.set_damage(7);
                self.player
                    .inventory()
                    .set_stack_in_hand(Hand::Left, shield.clone());
                self.player
                    .living_entity
                    .set_active_hand(Hand::Left, shield, 61_200);
            }
        })
    }
}

impl ReplacePlayerItemUse {
    fn assert_successor(&self) {
        use pumpkin_util::Hand;
        let living = &self.player.living_entity;
        let equipped = self.player.inventory().get_stack_in_hand(Hand::Left);
        assert_eq!(equipped.get_damage(), 7);
        assert_eq!(living.item_use_time.load(Ordering::Relaxed), 61_200);
        assert_eq!(
            *living
                .active_hand
                .lock()
                .unwrap_or_else(PoisonError::into_inner),
            Some(Hand::Left)
        );
        assert!(
            living
                .item_in_use
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .as_ref()
                .is_some_and(|stack| stack.are_equal(&equipped))
        );
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn java_shield_input_starts_only_once_and_preserves_a_listener_successor() -> TestResult {
    use pumpkin_data::{item::Item, item_stack::ItemStack};
    use pumpkin_protocol::java::server::play::SUseItem;
    for replace in [false, true] {
        let folder = tempfile::tempdir()?;
        let (server, world) = vehicle_world(&folder).await?;
        let (player, _peer) = java_equipment_watcher(&server, &world)?;
        player
            .inventory()
            .set_held_item(ItemStack::new(1, &Item::SHIELD));
        let observer = Arc::new(ReplacePlayerItemUse {
            player: player.clone(),
            trigger: GameEvent::ItemInteractStart,
            calls: std::sync::atomic::AtomicU32::new(0),
            replace,
        });
        server.plugin_manager.register::<GenericGameEvent, _>(
            observer.clone(),
            EventPriority::Normal,
            true,
        );
        let client = player.client.java().ok_or("Java client")?;
        for sequence in 1..=2 {
            client.handle_use_item(
                &player,
                &SUseItem {
                    hand: 0.into(),
                    sequence: sequence.into(),
                    yaw: 0.0,
                    pitch: 0.0,
                },
                &server,
            );
        }
        assert_eq!(
            observer.calls.load(Ordering::Relaxed),
            if replace { 2 } else { 1 }
        );
        if replace {
            observer.assert_successor();
        }
        world.level.shutdown().await;
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn release_packets_keep_the_use_started_by_the_crossbow_finish_listener() -> TestResult {
    use pumpkin_data::{item::Item, item_stack::ItemStack};
    use pumpkin_protocol::{
        bedrock::server::inventory_transaction::{
            HandSlot, ReleaseItemTransactionData, SInventoryTransaction, TransactionData,
        },
        java::server::play::{SPlayerAction, Status},
    };
    use pumpkin_util::{Hand, math::position::BlockPos};

    for bedrock in [false, true] {
        let folder = tempfile::tempdir()?;
        let (server, world) = vehicle_world(&folder).await?;
        let (player, _java_peer, _bedrock_queue) = if bedrock {
            let (player, queue) = bedrock_rider(&server, &world, "jeb_").await?;
            (player, None, Some(queue))
        } else {
            let (player, peer) = java_equipment_watcher(&server, &world)?;
            (player, Some(peer), None)
        };
        let crossbow = ItemStack::new(1, &Item::CROSSBOW);
        player.inventory().set_held_item(crossbow.clone());
        player
            .living_entity
            .set_active_hand(Hand::Right, crossbow, 72_000);
        let observer = Arc::new(ReplacePlayerItemUse {
            player: player.clone(),
            trigger: GameEvent::ItemInteractFinish,
            calls: std::sync::atomic::AtomicU32::new(0),
            replace: true,
        });
        server.plugin_manager.register::<GenericGameEvent, _>(
            observer.clone(),
            EventPriority::Normal,
            true,
        );
        if let Some(client) = player.client.java() {
            client.handle_player_action(
                &player,
                &SPlayerAction {
                    status: (Status::ReleaseItemInUse as i32).into(),
                    position: BlockPos::new(0, 0, 0),
                    face: 0,
                    sequence: 0.into(),
                },
                &server,
            );
        } else {
            let client = player.client.bedrock().ok_or("Bedrock client")?;
            client.handle_inventory_action(
                &player,
                SInventoryTransaction {
                    legacy_request_id: 0.into(),
                    legacy_set_item_slots: Vec::new(),
                    has_value: false,
                    actions: Vec::new(),
                    transaction_type: 4u32.into(),
                    transaction_data: TransactionData::ReleaseItem(ReleaseItemTransactionData {
                        action_type: 0.into(),
                        hot_bar_slot: 0.into(),
                        item_in_hand:
                            pumpkin_protocol::bedrock::network_item::NetworkItemDescriptor::default(
                            ),
                        head_position: Vector3::default(),
                        hand: HandSlot::Mainhand,
                    }),
                },
            );
        }
        assert_eq!(observer.calls.load(Ordering::Relaxed), 1);
        observer.assert_successor();
        world.level.shutdown().await;
    }
    Ok(())
}

struct ReplaceBedrockAttribute {
    pig: Arc<PigEntity>,
    called: AtomicBool,
}

impl EventHandler<PacketSentEvent> for ReplaceBedrockAttribute {
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut PacketSentEvent,
    ) -> BoxFuture<'a, ()> {
        use crate::entity::{attributes::send_attribute_updates_for_living, mob::Mob};
        use pumpkin_data::attributes::Attributes;
        use pumpkin_protocol::{
            bedrock::client::update_attributes::CUpdateAttributes,
            packet::Packet,
            serial::{PacketRead, PacketWrite},
        };
        Box::pin(async move {
            if event.player.client.bedrock().is_none()
                || event.packet_id != CUpdateAttributes::PACKET_ID
            {
                return;
            }
            let Ok(mut original) = CUpdateAttributes::read(&mut event.payload.as_ref()) else {
                return;
            };
            if original.target_runtime_id.0 != self.pig.get_entity().entity_id as u64
                || self.called.swap(true, Ordering::Relaxed)
            {
                return;
            }
            let living = &self.pig.get_mob_entity().living_entity;
            living.set_attribute_base(&Attributes::ATTACK_DAMAGE, 13.0);
            send_attribute_updates_for_living(living, vec![Attributes::ATTACK_DAMAGE]);
            // Include the superseded attack value alongside an unrelated override.
            if let Some(attack) = original.attribute_list.first() {
                original.attribute_list.push(
                    pumpkin_protocol::bedrock::client::update_attributes::AttributeData {
                        min_value: attack.min_value,
                        max_value: 20.0,
                        current_value: 19.0,
                        default_min_value: attack.default_min_value,
                        default_max_value: 20.0,
                        default_value: 20.0,
                        name: "minecraft:health".into(),
                        modifiers: Vec::new(),
                    },
                );
            }
            let mut extra = Vec::new();
            assert!(original.write(&mut extra).is_ok());
            event.extra_packets.push((event.packet_id, extra.into()));
        })
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bedrock_shield_attribute_guard_keeps_an_unrelated_observer_override() -> TestResult {
    use crate::entity::{attributes::send_shield_attribute_updates, mob::Mob};
    use pumpkin_data::attributes::Attributes;
    use pumpkin_protocol::{
        bedrock::client::update_attributes::CUpdateAttributes, packet::Packet, serial::PacketRead,
    };
    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let (_viewer, mut packets) = bedrock_rider(&server, &world, "jeb_").await?;
    let pig = PigEntity::new(Entity::new(world.clone(), CENTER, &EntityType::PIG));
    world.spawn_entity(pig.clone());
    let observer = Arc::new(ReplaceBedrockAttribute {
        pig: pig.clone(),
        called: AtomicBool::new(false),
    });
    server.plugin_manager.register::<PacketSentEvent, _>(
        observer.clone(),
        EventPriority::Normal,
        true,
    );

    let living = &pig.get_mob_entity().living_entity;
    living.set_attribute_base(&Attributes::ATTACK_DAMAGE, 3.0);
    send_shield_attribute_updates(living, &[Attributes::ATTACK_DAMAGE]);

    let mut attack = None;
    let mut health = None;
    while let Ok(packet) = packets.try_recv() {
        let Some((id, payload)) = crate::net::bedrock::split_raw_packet(&packet.data) else {
            continue;
        };
        if i32::from(id) == CUpdateAttributes::PACKET_ID {
            let update = CUpdateAttributes::read(&mut payload.as_ref())?;
            if update.target_runtime_id.0 == pig.get_entity().entity_id as u64 {
                for attribute in update.attribute_list {
                    match attribute.name.as_str() {
                        "minecraft:attack_damage" => attack = Some(attribute.current_value),
                        "minecraft:health" => health = Some(attribute.current_value),
                        _ => {}
                    }
                }
            }
        }
    }
    assert!(observer.called.load(Ordering::Relaxed));
    assert_eq!(attack, Some(13.0));
    assert_eq!(health, Some(19.0));
    world.level.shutdown().await;
    Ok(())
}
