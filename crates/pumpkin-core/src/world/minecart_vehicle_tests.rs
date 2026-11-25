use std::sync::{
    Arc, Mutex, PoisonError,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};

use pumpkin_data::{
    Block, BlockId,
    block_properties::{
        PoweredRailLikeProperties, RailLikeProperties, RailShape, RailShapeStraight,
    },
    entity::EntityType,
    game_event::GameEvent,
    item_stack::ItemStack,
    packet::CURRENT_MC_VERSION,
};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::{
    codec::lp_vector_3d::LpVector3d,
    java::{client::play::CEntityVelocity, server::play::SPlayerInput},
    packet::MultiVersionJavaPacket,
    ser::NetworkReadExt,
};
use pumpkin_util::math::{position::BlockPos, vector2::Vector2, vector3::Vector3};
use pumpkin_world::{chunk::ChunkData, cylindrical_chunk_iterator::Cylindrical};

use crate::{
    block::{BlockBehaviour, BlockMetadata, OnEntityCollisionArgs, registry::BlockRegistry},
    entity::{Entity, EntityBase, passive::pig::PigEntity, vehicle::minecart::MinecartEntity},
    plugin::{
        BoxFuture, EventHandler, EventPriority,
        api::events::{
            entity::{entity_dismount::EntityDismountEvent, entity_mount::EntityMountEvent},
            world::generic_game::GenericGameEvent,
        },
        server::packet::PacketSentEvent,
    },
    server::Server,
};

use super::World;
use super::entity_tracker::TrackedEntity;
use super::vehicle_tests::{
    CENTER, TestResult, assert_mounted, java_rider, mount_pig, vehicle_world,
    verify_mount_capacity_after_callbacks,
};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn classic_minecart_speed_and_steps_use_the_rail_state() -> TestResult {
    struct ObserveRailStep(AtomicUsize);
    impl EventHandler<GenericGameEvent> for ObserveRailStep {
        fn handle_blocking<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut GenericGameEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async {
                if event.event_key == GameEvent::Step.name() {
                    self.0.fetch_add(1, Ordering::Relaxed);
                }
            })
        }
    }

    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let steps = Arc::new(ObserveRailStep(AtomicUsize::new(0)));
    server.plugin_manager.register::<GenericGameEvent, _>(
        steps.clone(),
        EventPriority::Normal,
        true,
    );
    let chunk = ChunkData::empty_sync(0, 0);
    let mut rail = RailLikeProperties::default(&Block::RAIL);
    rail.shape = RailShape::EastWest;
    for (z, material) in [(8, &Block::STONE), (11, &Block::SOUL_SAND)] {
        for x in 7..=14 {
            chunk.set_block_absolute_y(x, 63, z, material.default_state.id);
            chunk.set_block_absolute_y(x, 64, z, rail.to_state_id(&Block::RAIL));
        }
    }
    world.level.loaded_chunks.insert(Vector2::new(0, 0), chunk);
    for z in [8.5, 11.5] {
        let cart = MinecartEntity::new(Entity::new(
            world.clone(),
            Vector3::new(8.5, 64.0625, z),
            &EntityType::MINECART,
        ));
        let entity = cart.get_entity();
        entity.velocity.store(Vector3::new(0.2, 0.0, 0.0));
        let initial_steps = steps.0.load(Ordering::Relaxed);
        cart.tick(&cart, &server);
        assert!((entity.velocity.load().x - 0.192).abs() < 1.0e-12);
        let first_x = entity.pos.load().x;
        cart.tick(&cart, &server);
        assert!((entity.pos.load().x - first_x - 0.192).abs() < 1.0e-12);
        assert_eq!(steps.0.load(Ordering::Relaxed), initial_steps);
        // Java accumulates horizontal travel multiplied by 0.6. With the rail's
        // drag, the distance first exceeds one on the tenth move.
        for tick in 3..=10 {
            cart.tick(&cart, &server);
            assert_eq!(
                steps.0.load(Ordering::Relaxed),
                initial_steps + usize::from(tick == 10),
                "rail step emission on tick {tick}",
            );
        }
        assert!(!entity.on_ground.load(Ordering::Relaxed));
    }
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn off_track_cart_rebounds_only_when_it_falls_faster_than_gravity() -> TestResult {
    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let chunk = ChunkData::empty_sync(0, 0);
    chunk.set_block_absolute_y(8, 63, 8, Block::SLIME_BLOCK.default_state.id);
    chunk.set_block_absolute_y(10, 63, 8, Block::STONE.default_state.id);
    world.level.loaded_chunks.insert(Vector2::new(0, 0), chunk);
    for (x, height, speed, rebound) in [
        (8.5, 0.0, 0.0, 0.0),
        (8.5, 0.125, -0.2, 0.203_232_640_621_718_03),
        (10.5, 0.125, -0.2, 0.0),
    ] {
        let cart = MinecartEntity::new(Entity::new(
            world.clone(),
            Vector3::new(x, 64.0 + height, 8.5),
            &EntityType::MINECART,
        ));
        let entity = cart.get_entity();
        entity.velocity.store(Vector3::new(0.0, speed, 0.0));
        cart.tick(&cart, &server);
        assert_eq!(entity.pos.load(), Vector3::new(x, 64.0, 8.5));
        assert!((entity.velocity.load().y - rebound).abs() < 1.0e-12);
    }
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn classic_vehicle_contacts_push_stationary_carts_in_both_tick_orders() -> TestResult {
    let directory = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&directory).await?;
    let chunk = ChunkData::empty_sync(0, 0);
    for x in 7..=11 {
        chunk.set_block_absolute_y(x, 63, 8, Block::STONE.default_state.id);
    }
    world.level.loaded_chunks.insert(Vector2::new(0, 0), chunk);
    let left = Arc::new(MinecartEntity::new(Entity::new(
        world.clone(),
        CENTER,
        &EntityType::MINECART,
    )));
    let right_start = CENTER.add_raw(1.0, 0.0, 0.0);
    let right = Arc::new(MinecartEntity::new(Entity::new(
        world.clone(),
        right_start,
        &EntityType::MINECART,
    )));
    world.spawn_entity(left.clone());
    world.spawn_entity(right.clone());
    assert!(
        !left
            .get_entity()
            .bounding_box
            .load()
            .intersects(&right.get_entity().bounding_box.load())
    );
    for order in [[&left, &right], [&right, &left]] {
        for (cart, start) in [(&left, CENTER), (&right, right_start)] {
            cart.get_entity().set_pos(start);
            cart.get_entity().velocity.store(Vector3::default());
            cart.get_entity().on_ground.store(true, Ordering::Relaxed);
        }
        for cart in order {
            cart.tick(cart.as_ref(), &server);
        }
        assert!(left.get_entity().velocity.load().x < 0.0);
        assert!(right.get_entity().velocity.load().x > 0.0);
    }
    world.level.shutdown().await;
    Ok(())
}

#[derive(Default)]
struct CartMotionTrace {
    collision_velocities: Option<[Vector3<f64>; 2]>,
    packets: Vec<(i32, Vector3<f64>)>,
}

struct ReplaceCartMotionOnPacket {
    first: Arc<MinecartEntity>,
    other: Arc<MinecartEntity>,
    destination: Vector3<f64>,
    replacement: Vector3<f64>,
    trace: Mutex<CartMotionTrace>,
}

impl ReplaceCartMotionOnPacket {
    fn packet_count(&self) -> usize {
        self.trace
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .packets
            .len()
    }

    fn track_collision(
        &self,
        world: &World,
        first: &TrackedEntity,
        other: &TrackedEntity,
        mixed: bool,
    ) {
        first.send_changes(world);
        assert_eq!(
            self.packet_count(),
            1,
            "the pushed cart did not publish its impulse"
        );
        if mixed {
            // The fixture flushed tick zero. The furnace must stay quiet until its next due tick.
            assert!(other.update_interval > 1);
            for _ in 1..other.update_interval {
                other.send_changes(world);
                assert_eq!(
                    self.packet_count(),
                    1,
                    "the drag-only furnace published before its due tick"
                );
            }
        }
        other.send_changes(world);
    }

    fn check_collision(
        &self,
        expected: [Vector3<f64>; 2],
        reverse: bool,
        packets_during_push: usize,
    ) -> TestResult {
        let other = self.other.get_entity();
        assert_eq!(other.pos.load(), self.destination);
        assert_eq!(
            other.velocity.load(),
            self.replacement,
            "collision resumed after the observer, reverse={reverse}"
        );
        assert_eq!(packets_during_push, 0, "tracking ran during the collision");

        let trace = self.trace.lock().unwrap_or_else(PoisonError::into_inner);
        let observed = trace
            .collision_velocities
            .ok_or("First cart velocity packet did not reach the observer")?;
        for (actual, expected) in observed.into_iter().zip(expected) {
            assert!(
                (actual - expected).length_squared() < 1.0e-24,
                "{actual:?} != {expected:?}"
            );
        }
        assert_eq!(trace.packets.len(), 2);
        let (published_id, published_motion) = trace.packets[1];
        assert_eq!(published_id, other.entity_id);
        assert!(
            (published_motion - self.replacement).length_squared() < 1.0e-8,
            "published {published_motion:?}"
        );
        Ok(())
    }
}

impl EventHandler<PacketSentEvent> for ReplaceCartMotionOnPacket {
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut PacketSentEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            if event.packet_id != CEntityVelocity::to_id(CURRENT_MC_VERSION) {
                return;
            }
            let first = self.first.get_entity();
            let other = self.other.get_entity();
            let mut data = event.payload.as_ref();
            let Ok(id) = data.get_var_int() else { return };
            if id.0 != first.entity_id && id.0 != other.entity_id {
                return;
            }
            let Ok(LpVector3d(velocity)) = LpVector3d::read(&mut data) else {
                return;
            };
            let mut trace = self.trace.lock().unwrap_or_else(PoisonError::into_inner);
            trace.packets.push((id.0, velocity));
            if id.0 != first.entity_id || trace.collision_velocities.is_some() {
                return;
            }
            trace.collision_velocities = Some([first.velocity.load(), other.velocity.load()]);
            drop(trace);
            other.velocity.store(self.replacement);
            self.other
                .teleport(self.destination, None, None, other.world.load_full());
        })
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn classic_cart_collision_preserves_an_observers_new_transport_in_both_orders() -> TestResult
{
    let impulse = f64::from(0.1f32) * 0.5;
    check_cart_collision_observer(
        &EntityType::MINECART,
        [
            Vector3::new(0.1125 - impulse, 0.125, -0.01875),
            Vector3::new(0.0375 + impulse, -0.25, -0.05625),
        ],
    )
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn furnace_cart_collision_preserves_an_observers_new_transport_in_both_orders() -> TestResult
{
    let impulse = f64::from(0.1f32) * 0.5;
    check_cart_collision_observer(
        &EntityType::FURNACE_MINECART,
        [
            Vector3::new(0.2375, 0.125, 0.059375),
            Vector3::new(0.225 + impulse, -0.25, 0.0375),
        ],
    )
    .await
}

async fn check_cart_collision_observer(
    left_type: &'static EntityType,
    expected: [Vector3<f64>; 2],
) -> TestResult {
    let mixed = left_type == &EntityType::FURNACE_MINECART;
    for reverse in [false, true] {
        let directory = tempfile::tempdir()?;
        let (server, world) = vehicle_world(&directory).await?;
        let chunk = Vector2::new(0, 0);
        world
            .level
            .loaded_chunks
            .insert(chunk, ChunkData::empty_sync(0, 0));
        let (watcher, _peer) = java_rider(&server, &world, "Notch")?;
        let client = watcher.client.java().ok_or("Missing Java watcher")?;
        client.player.store(Arc::new(Some(watcher.clone())));
        watcher.get_entity().set_pos(CENTER.add_raw(0.0, 0.0, 3.0));
        watcher.watched_section.store(Cylindrical::new(
            chunk,
            std::num::NonZeroU8::new(4).ok_or("Zero view distance")?,
        ));
        watcher
            .chunk_sender
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .mark_sent_out_of_band(chunk);

        let left = Arc::new(MinecartEntity::new(Entity::new(
            world.clone(),
            CENTER,
            left_type,
        )));
        let right = Arc::new(MinecartEntity::new(Entity::new(
            world.clone(),
            CENTER.add_raw(0.875, 0.0, 0.0),
            &EntityType::MINECART,
        )));
        assert!(world.spawn_entity(left.clone()));
        assert!(world.spawn_entity(right.clone()));
        // The ordinary cart receives the only impulse in a mixed collision.
        let (first, other, expected) = if reverse || mixed {
            (&right, &left, [expected[1], expected[0]])
        } else {
            (&left, &right, expected)
        };
        let tracking = |cart: &MinecartEntity| {
            world
                .entity_tracker
                .get_tracked_entity(cart.get_entity().entity_id)
                .ok_or("Cart is not tracked")
        };
        let first_tracking = tracking(first)?;
        let other_tracking = tracking(other)?;
        assert!(first_tracking.is_seen_by(&watcher.gameprofile.id));
        assert!(other_tracking.is_seen_by(&watcher.gameprofile.id));
        if mixed {
            first_tracking.send_changes(&world);
            other_tracking.send_changes(&world);
        }
        left.get_entity()
            .velocity
            .store(Vector3::new(0.25, 0.125, 0.0625));
        right
            .get_entity()
            .velocity
            .store(Vector3::new(-0.125, -0.25, -0.125));

        let observer = Arc::new(ReplaceCartMotionOnPacket {
            first: first.clone(),
            other: other.clone(),
            destination: CENTER.add_raw(0.0, 2.0, 1.0),
            replacement: Vector3::new(0.5, 0.25, -0.75),
            trace: Mutex::default(),
        });
        server.plugin_manager.register::<PacketSentEvent, _>(
            observer.clone(),
            EventPriority::Normal,
            true,
        );
        let (receiver, contact) = if reverse {
            (&right, &left)
        } else {
            (&left, &right)
        };
        receiver.push(contact.as_ref());
        let packets_during_push = observer.packet_count();
        observer.track_collision(&world, &first_tracking, &other_tracking, mixed);

        client.player.store(Arc::new(None));
        observer.check_collision(expected, reverse, packets_during_push)?;
        if mixed {
            let furnace = left.get_entity();
            assert!(!furnace.velocity_dirty.load(Ordering::SeqCst));
            furnace.set_pos(CENTER);
            furnace.velocity_dirty.store(true, Ordering::SeqCst);
            receiver.push(contact.as_ref());
            assert_eq!(
                furnace.velocity.load(),
                observer.replacement.multiply(0.95, 1.0, 0.95)
            );
            assert!(furnace.velocity_dirty.load(Ordering::SeqCst));
        }
        world.level.shutdown().await;
    }
    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
enum ActivatorStep {
    Destination {
        occupied: bool,
    },
    Dismount {
        rider: i32,
        flipped: bool,
    },
    Pickup {
        rider: i32,
        occupied: bool,
        flipped: bool,
    },
}

struct ObserveActivatorContact(Arc<Mutex<Vec<ActivatorStep>>>);

impl BlockMetadata for ObserveActivatorContact {
    fn ids() -> Box<[BlockId]> {
        [Block::ACTIVATOR_RAIL.id].into()
    }
}

impl BlockBehaviour for ObserveActivatorContact {
    fn on_entity_collision(&self, args: OnEntityCollisionArgs<'_>) {
        // The cart straddles two rails. Observe the destination block once per pass.
        if *args.position == BlockPos::new(9, 64, 8) {
            self.0.lock().unwrap_or_else(PoisonError::into_inner).push(
                ActivatorStep::Destination {
                    occupied: args.entity.is_vehicle(),
                },
            );
        }
    }
}

struct ObserveActivatorAttachment {
    cart: Arc<MinecartEntity>,
    trace: Arc<Mutex<Vec<ActivatorStep>>>,
}

impl ObserveActivatorAttachment {
    fn flipped(&self) -> bool {
        let mut saved = NbtCompound::new();
        self.cart.write_custom_nbt(&mut saved);
        saved.get_bool("FlippedRotation").unwrap_or_default()
    }
}

impl EventHandler<EntityDismountEvent> for ObserveActivatorAttachment {
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut EntityDismountEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            if event.dismounted_id == self.cart.get_entity().entity_id {
                self.trace
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .push(ActivatorStep::Dismount {
                        rider: event.entity_id,
                        flipped: self.flipped(),
                    });
            }
        })
    }
}

impl EventHandler<EntityMountEvent> for ObserveActivatorAttachment {
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut EntityMountEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            if event.mount_id == self.cart.get_entity().entity_id {
                self.trace
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .push(ActivatorStep::Pickup {
                        rider: event.entity_id,
                        occupied: self.cart.is_vehicle(),
                        flipped: self.flipped(),
                    });
            }
        })
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn moving_vehicle_activator_ejects_before_cart_pickup() -> TestResult {
    let directory = tempfile::tempdir()?;
    let (server, base_world) = vehicle_world(&directory).await?;
    let trace = Arc::new(Mutex::new(Vec::new()));
    let mut registry = BlockRegistry::default();
    registry.register(ObserveActivatorContact(trace.clone()));
    let world = Arc::new(World::load(
        base_world.level.clone(),
        base_world.level_info.clone(),
        base_world.dimension.clone(),
        Arc::new(registry),
        Arc::downgrade(&server),
    ));
    let chunk = ChunkData::empty_sync(0, 0);
    for x in 8..=10 {
        for z in 7..=9 {
            chunk.set_block_absolute_y(x, 63, z, Block::STONE.default_state.id);
        }
    }
    let mut rail = PoweredRailLikeProperties::default(&Block::ACTIVATOR_RAIL);
    rail.powered = true;
    rail.shape = RailShapeStraight::EastWest;
    chunk.set_block_absolute_y(8, 64, 8, rail.to_state_id(&Block::ACTIVATOR_RAIL));
    chunk.set_block_absolute_y(9, 64, 8, rail.to_state_id(&Block::ACTIVATOR_RAIL));
    let chunk_pos = Vector2::new(0, 0);
    world.level.loaded_chunks.insert(chunk_pos, chunk);
    *world
        .active_chunks
        .write()
        .unwrap_or_else(PoisonError::into_inner) = Arc::new(std::iter::once(chunk_pos).collect());

    let start = Vector3::new(8.88, 64.0625, 8.5);
    let cart = Arc::new(MinecartEntity::new(Entity::new(
        world.clone(),
        start,
        &EntityType::MINECART,
    )));
    world.spawn_entity(cart.clone());
    let original = mount_pig(&world, &(cart.clone() as Arc<dyn EntityBase>));
    let nearby = PigEntity::new(Entity::new(
        world.clone(),
        Vector3::new(9.48, 64.0, 9.58),
        &EntityType::PIG,
    ));
    world.spawn_entity(nearby.clone());
    assert!(!nearby.is_passenger());
    let cart_entity = cart.get_entity();
    let attachment = Arc::new(ObserveActivatorAttachment {
        cart: cart.clone(),
        trace: trace.clone(),
    });
    server.plugin_manager.register::<EntityDismountEvent, _>(
        attachment.clone(),
        EventPriority::Normal,
        true,
    );
    server
        .plugin_manager
        .register::<EntityMountEvent, _>(attachment, EventPriority::Normal, true);
    // The cart starts facing west and rolls east. Its motion direction flips after activation.
    cart_entity.set_rotation(0.0, 0.0);
    cart_entity.velocity.store(Vector3::new(0.32, 0.0, 0.0));
    crate::entity::ride_tick_riders(cart.as_ref());

    cart.tick(cart.as_ref(), &server);
    assert!(cart_entity.pos.load().x >= 9.0);
    assert!(!cart_entity.has_passenger(original.get_entity().entity_id));
    assert!(!original.is_passenger());
    assert_eq!(
        original.get_entity().pos.load(),
        Vector3::new(9.5, 64.0, 7.5)
    );
    assert_mounted(cart_entity, nearby.get_entity());
    assert_eq!(
        trace
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .as_slice(),
        [
            ActivatorStep::Destination { occupied: true },
            ActivatorStep::Dismount {
                rider: original.get_entity().entity_id,
                flipped: false
            },
            ActivatorStep::Destination { occupied: false },
            ActivatorStep::Pickup {
                rider: nearby.get_entity().entity_id,
                occupied: false,
                flipped: true
            },
        ]
    );

    crate::entity::ride_tick_riders(cart.as_ref());
    cart.tick(cart.as_ref(), &server);
    assert!(!cart_entity.has_passengers());
    assert!(!nearby.is_passenger());
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn powered_activator_ejects_last_rider_first_and_keeps_replacement_attachment() -> TestResult
{
    struct EjectionObserver {
        cart: Arc<MinecartEntity>,
        first: Arc<PigEntity>,
        second_id: i32,
        reattach: bool,
        order: Mutex<Vec<i32>>,
    }

    impl EventHandler<EntityDismountEvent> for EjectionObserver {
        fn handle_blocking<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut EntityDismountEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                let cart = self.cart.get_entity();
                if event.dismounted_id != cart.entity_id {
                    return;
                }
                self.order
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .push(event.entity_id);
                if self.reattach && event.entity_id == self.second_id {
                    cart.remove_passenger_on_disconnect(self.first.get_entity().entity_id);
                    cart.add_passenger_silently(self.cart.clone(), self.first.clone());
                }
            })
        }
    }

    for reattach in [false, true] {
        let directory = tempfile::tempdir()?;
        let (server, world) = vehicle_world(&directory).await?;
        let chunk = ChunkData::empty_sync(0, 0);
        chunk.set_block_absolute_y(8, 63, 8, Block::STONE.default_state.id);
        let mut rail = PoweredRailLikeProperties::default(&Block::ACTIVATOR_RAIL);
        rail.powered = true;
        rail.shape = RailShapeStraight::EastWest;
        chunk.set_block_absolute_y(8, 64, 8, rail.to_state_id(&Block::ACTIVATOR_RAIL));
        world.level.loaded_chunks.insert(Vector2::new(0, 0), chunk);
        let cart = Arc::new(MinecartEntity::new(Entity::new(
            world.clone(),
            CENTER,
            &EntityType::MINECART,
        )));
        world.spawn_entity(cart.clone());
        let entity = cart.get_entity();
        let first = PigEntity::new(Entity::new(world.clone(), CENTER, &EntityType::PIG));
        let second = PigEntity::new(Entity::new(world.clone(), CENTER, &EntityType::PIG));
        entity.add_passenger_silently(cart.clone(), first.clone());
        entity.add_passenger_silently(cart.clone(), second.clone());
        assert_mounted(entity, first.get_entity());
        assert_mounted(entity, second.get_entity());
        let original = first.get_entity().riding_revision();
        let observer = Arc::new(EjectionObserver {
            cart: cart.clone(),
            first: first.clone(),
            second_id: second.get_entity().entity_id,
            reattach,
            order: Mutex::new(Vec::new()),
        });
        server.plugin_manager.register::<EntityDismountEvent, _>(
            observer.clone(),
            EventPriority::Normal,
            true,
        );

        cart.tick(cart.as_ref(), &server);
        let mut expected = vec![second.get_entity().entity_id];
        if reattach {
            assert_mounted(entity, first.get_entity());
        } else {
            expected.push(first.get_entity().entity_id);
            assert!(!first.is_passenger());
            assert!(!entity.has_passengers());
        }
        assert_ne!(first.get_entity().riding_revision(), original);
        assert!(!second.is_passenger());
        assert_eq!(
            *observer
                .order
                .lock()
                .unwrap_or_else(PoisonError::into_inner),
            expected
        );
        world.level.shutdown().await;
    }
    Ok(())
}

struct MountEffects {
    cart: Arc<MinecartEntity>,
    destination: Vector3<f64>,
    veto: AtomicBool,
    mounts: AtomicUsize,
}

impl EventHandler<EntityMountEvent> for MountEffects {
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut EntityMountEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async { event.cancelled = self.veto.load(Ordering::Relaxed) })
    }
}

impl EventHandler<GenericGameEvent> for MountEffects {
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut GenericGameEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async {
            if event.event_key == GameEvent::EntityMount.name() {
                let cart = self.cart.get_entity();
                assert_eq!(event.position, cart.pos.load());
                self.mounts.fetch_add(1, Ordering::Relaxed);
                self.cart
                    .teleport(self.destination, None, None, cart.world.load_full());
            }
        })
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn fast_minecart_pickup_runs_mount_effects_only_on_success() -> TestResult {
    use crate::entity::{decoration::leash_knot::LeashKnotEntity, item::ItemEntity};
    use pumpkin_data::{entity::EntityPose, item::Item};

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
    let cart = Arc::new(MinecartEntity::new(Entity::new(
        world.clone(),
        CENTER,
        &EntityType::MINECART,
    )));
    let pig = PigEntity::new(Entity::new(
        world.clone(),
        CENTER.add_raw(0.25, 0.0, 0.0),
        &EntityType::PIG,
    ));
    let holder: Arc<dyn EntityBase> = Arc::new(LeashKnotEntity::new(Entity::new(
        world.clone(),
        CENTER.add_raw(4.0, 0.0, 0.0),
        &EntityType::LEASH_KNOT,
    )));
    world.spawn_entity(cart.clone());
    world.spawn_entity(pig.clone());
    world.spawn_entity(holder.clone());
    let passenger = pig.get_entity();
    passenger.set_pose(EntityPose::Crouching);
    passenger.attach_leash(&holder);
    let destination = CENTER.add_raw(2.0, 0.0, 0.0);
    let effects = Arc::new(MountEffects {
        cart: cart.clone(),
        destination,
        veto: AtomicBool::new(true),
        mounts: AtomicUsize::new(0),
    });
    server.plugin_manager.register::<EntityMountEvent, _>(
        effects.clone(),
        EventPriority::Normal,
        true,
    );
    server.plugin_manager.register::<GenericGameEvent, _>(
        effects.clone(),
        EventPriority::Normal,
        true,
    );
    let cart_entity = cart.get_entity();
    cart_entity.set_has_no_gravity(true);
    let advance = || {
        cart_entity.set_pos(CENTER);
        cart_entity.velocity.store(Vector3::new(0.2, 0.0, 0.0));
        cart.tick(cart.as_ref(), &server);
        assert!(cart_entity.velocity.load().x > 0.1);
    };
    let dropped_leads = || {
        world
            .entities
            .load()
            .iter()
            .filter_map(|entity| entity.cast_any().downcast_ref::<ItemEntity>())
            .filter(|item| {
                item.get_item_stack()
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .get_item()
                    == &Item::LEAD
            })
            .count()
    };

    advance();
    assert!(!cart.is_vehicle());
    assert!(!pig.is_passenger());
    assert_eq!(passenger.pose.load(), EntityPose::Crouching);
    assert!(passenger.is_leashed());
    assert_eq!(dropped_leads(), 0);
    assert_eq!(effects.mounts.load(Ordering::Relaxed), 0);

    effects.veto.store(false, Ordering::Relaxed);
    advance();
    assert_mounted(cart_entity, passenger);
    assert_eq!(cart_entity.pos.load(), destination);
    assert_eq!(passenger.pose.load(), EntityPose::Standing);
    assert!(!passenger.is_leashed());
    assert_eq!(dropped_leads(), 1);
    assert_eq!(effects.mounts.load(Ordering::Relaxed), 1);
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn fast_minecart_pickup_respects_the_collision_disabled_team() -> TestResult {
    use crate::world::scoreboard::{CollisionRule, NameTagVisibility, Team};
    use pumpkin_util::text::{TextComponent, color::NamedColor};

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
    let cart = Arc::new(MinecartEntity::new(Entity::new(
        world.clone(),
        CENTER,
        &EntityType::MINECART,
    )));
    let pig = PigEntity::new(Entity::new(
        world.clone(),
        CENTER.add_raw(0.25, 0.0, 0.0),
        &EntityType::PIG,
    ));
    world.spawn_entity(cart.clone());
    world.spawn_entity(pig.clone());
    let mut team = Team {
        name: "no_pickup".into(),
        display_name: TextComponent::text("No pickup"),
        options: 0,
        nametag_visibility: NameTagVisibility::Always,
        collision_rule: CollisionRule::Never,
        color: NamedColor::White,
        player_prefix: TextComponent::empty(),
        player_suffix: TextComponent::empty(),
        players: vec![pig.get_scoreboard_name()],
    };
    world
        .scoreboard
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .add_team(&world, team.clone());
    let entity = cart.get_entity();
    entity.set_has_no_gravity(true);
    let tick = || {
        entity.set_pos(CENTER);
        entity.velocity.store(Vector3::new(0.2, 0.0, 0.0));
        cart.tick(cart.as_ref(), &server);
        assert!(entity.velocity.load().x >= 0.1);
    };

    tick();
    assert!(!cart.is_vehicle());
    assert!(!pig.is_passenger());
    team.collision_rule = CollisionRule::Always;
    world
        .scoreboard
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .update_team(&world, team);
    tick();
    assert_mounted(entity, pig.get_entity());
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn slow_rideable_vehicle_retains_mob_pickup() -> TestResult {
    let directory = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&directory).await?;
    let chunk = ChunkData::empty_sync(0, 0);
    for x in 8..=9 {
        chunk.set_block_absolute_y(x, 63, 8, Block::STONE.default_state.id);
    }
    world.level.loaded_chunks.insert(Vector2::new(0, 0), chunk);
    let cart = Arc::new(MinecartEntity::new(Entity::new(
        world.clone(),
        CENTER,
        &EntityType::MINECART,
    )));
    let pig = PigEntity::new(Entity::new(
        world.clone(),
        CENTER.add_raw(0.6, 0.0, 0.0),
        &EntityType::PIG,
    ));
    world.spawn_entity(cart.clone());
    world.spawn_entity(pig.clone());
    let cart_entity = cart.get_entity();
    cart_entity.on_ground.store(true, Ordering::Relaxed);
    cart.tick(cart.as_ref(), &server);
    assert!(!cart_entity.has_passengers());
    assert!(!pig.get_entity().has_vehicle());

    cart_entity.velocity.store(Vector3::new(0.02, 0.0, 0.0));
    cart.tick(cart.as_ref(), &server);
    assert_mounted(cart_entity, pig.get_entity());
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tnt_vehicle_impact_uses_speed_after_wall_clipping() -> TestResult {
    let directory = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&directory).await?;
    let chunk = ChunkData::empty_sync(0, 0);
    chunk.set_block_absolute_y(8, 63, 8, Block::STONE.default_state.id);
    chunk.set_block_absolute_y(9, 64, 8, Block::STONE.default_state.id);
    world.level.loaded_chunks.insert(Vector2::new(0, 0), chunk);
    let cart = Arc::new(MinecartEntity::new(Entity::new(
        world.clone(),
        CENTER,
        &EntityType::TNT_MINECART,
    )));
    world.spawn_entity(cart.clone());
    let base = cart.get_entity();
    assert!(world.level_info.load().game_rules.tnt_explodes);
    base.on_ground.store(true, Ordering::Relaxed);
    base.velocity.store(Vector3::new(0.2, 0.0, 0.0));
    cart.tick(cart.as_ref(), &server);
    assert!(base.horizontal_collision.load(Ordering::Relaxed));
    assert!(
        !base.is_removed(),
        "the clipped X velocity cannot detonate the cart"
    );

    base.set_pos(CENTER);
    base.on_ground.store(true, Ordering::Relaxed);
    base.velocity.store(Vector3::new(0.2, 0.0, 0.3));
    cart.tick(cart.as_ref(), &server);
    assert!(base.horizontal_collision.load(Ordering::Relaxed));
    assert!(
        base.is_removed(),
        "the remaining Z velocity can detonate the cart"
    );
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn minecart_vehicle_destination_effect_activates_a_detector_rail() -> TestResult {
    use pumpkin_data::block_properties::DetectorRailProperties;
    use pumpkin_util::math::position::BlockPos;

    let directory = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&directory).await?;
    let chunk = ChunkData::empty_sync(0, 0);
    chunk.set_block_absolute_y(8, 63, 8, Block::STONE.default_state.id);
    chunk.set_block_absolute_y(8, 64, 8, Block::DETECTOR_RAIL.default_state.id);
    world.level.loaded_chunks.insert(Vector2::new(0, 0), chunk);
    let detector = BlockPos::new(8, 64, 8);
    let powered =
        || DetectorRailProperties::from_state_id(world.get_block_state(&detector).id).powered;
    let cart = Arc::new(MinecartEntity::new(Entity::new(
        world.clone(),
        CENTER,
        &EntityType::MINECART,
    )));
    world.spawn_entity(cart.clone());
    assert!(!powered());
    cart.tick(cart.as_ref(), &server);
    assert!(powered());
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn classic_minecart_applies_both_vehicle_contact_passes() -> TestResult {
    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let chunk = ChunkData::empty_sync(0, 0);
    for x in [7, 8] {
        chunk.set_block_absolute_y(x, 63, 8, Block::STONE.default_state.id);
        chunk.set_block_absolute_y(x, 64, 8, Block::POWDER_SNOW.default_state.id);
    }
    world.level.loaded_chunks.insert(Vector2::new(0, 0), chunk);
    let entity = Entity::new(
        world.clone(),
        Vector3::new(8.0, 64.01, 8.5),
        &EntityType::MINECART,
    );
    entity.set_frozen_ticks(5);
    let cart = Arc::new(MinecartEntity::new(entity));
    world.spawn_entity(cart.clone());

    for expected in [7, 9] {
        cart.tick(cart.as_ref(), &server);
        assert_eq!(cart.get_entity().get_frozen_ticks(), expected);
    }
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test]
async fn minecart_dismount_uses_the_next_open_exit() -> TestResult {
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
    let cart = Arc::new(MinecartEntity::new(Entity::new(
        world.clone(),
        center,
        &EntityType::MINECART,
    )));
    cart.get_entity().set_rotation(180.0, 0.0);
    let rider = SnowGolemEntity::new(Entity::new(world.clone(), center, &EntityType::SNOW_GOLEM));
    chunk.set_block_absolute_y(8, 64, 9, Block::STONE.default_state.id);
    chunk.set_block_absolute_y(8, 65, 9, Block::STONE.default_state.id);
    cart.get_entity().add_passenger(cart.clone(), rider.clone());
    assert!(rider.get_entity().has_vehicle());
    cart.get_entity()
        .remove_passenger(rider.get_entity().entity_id);
    assert!(!rider.get_entity().has_vehicle());
    assert_eq!(rider.get_entity().pos.load(), Vector3::new(8.5, 64.0, 7.5));

    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn classic_powered_rail_caps_displacement_before_the_airborne_tick() -> TestResult {
    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let chunk = ChunkData::empty_sync(0, 0);
    chunk.set_block_absolute_y(8, 63, 8, Block::STONE.default_state.id);
    let mut rail = PoweredRailLikeProperties::default(&Block::POWERED_RAIL);
    rail.shape = RailShapeStraight::EastWest;
    rail.powered = true;
    chunk.set_block_absolute_y(8, 64, 8, rail.to_state_id(&Block::POWERED_RAIL));
    world
        .level
        .loaded_chunks
        .insert(Vector2::new(0, 0), chunk.clone());

    let start = Vector3::new(8.25, 64.0625, 8.5);
    let cart = Arc::new(MinecartEntity::new(Entity::new(
        world.clone(),
        start,
        &EntityType::MINECART,
    )));
    world.spawn_entity(cart.clone());
    let entity = cart.get_entity();
    entity.velocity.store(Vector3::new(0.9, 0.0, 0.0));
    cart.tick(cart.as_ref(), &server);
    assert_eq!(entity.pos.load(), Vector3::new(8.65, 64.0625, 8.5));
    assert!((entity.velocity.load().x - 0.924).abs() < 1.0e-12);

    cart.tick(cart.as_ref(), &server);
    let edge = entity.pos.load();
    assert!((edge.x - 9.05).abs() < 1.0e-12);
    assert_eq!(edge.y, 64.0);
    assert!(entity.velocity.load().x > 0.9);

    chunk.set_block_absolute_y(8, 63, 8, Block::AIR.default_state.id);
    cart.tick(cart.as_ref(), &server);
    let air_step = entity.pos.load() - edge;
    assert!((air_step.x - 0.4).abs() < 1.0e-12);
    assert!((air_step.y + 0.04).abs() < 1.0e-12);
    assert!((entity.velocity.load().x - 0.4 * f64::from(0.95f32)).abs() < 1.0e-12);
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn off_track_cart_does_not_creep_when_vertical_clipping_leaves_tiny_motion() -> TestResult {
    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let chunk = ChunkData::empty_sync(0, 0);
    chunk.set_block_absolute_y(8, 63, 8, Block::STONE.default_state.id);
    world
        .level
        .loaded_chunks
        .insert(Vector2::new(0, 0), chunk.clone());
    let cart = Arc::new(MinecartEntity::new(Entity::new(
        world.clone(),
        CENTER,
        &EntityType::MINECART,
    )));
    world.spawn_entity(cart.clone());
    let entity = cart.get_entity();

    for (speed, displacement) in [(0.0004, 0.0), (0.0008, 0.0004)] {
        entity.set_pos(CENTER);
        entity.on_ground.store(true, Ordering::Relaxed);
        entity.velocity.store(Vector3::new(speed, 0.0, 0.0));
        cart.tick(cart.as_ref(), &server);
        assert!(entity.on_ground.load(Ordering::Relaxed));
        assert_eq!(entity.pos.load(), CENTER.add_raw(displacement, 0.0, 0.0));
    }

    chunk.set_block_absolute_y(8, 63, 8, Block::AIR.default_state.id);
    entity.set_pos(CENTER);
    entity.set_has_no_gravity(true);
    entity.velocity.store(Vector3::new(0.0004, 0.0, 0.0));
    cart.tick(cart.as_ref(), &server);
    assert_eq!(entity.pos.load(), CENTER.add_raw(0.0002, 0.0, 0.0));
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn classic_cart_climbs_and_starts_downhill_from_rest() -> TestResult {
    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let chunk = ChunkData::empty_sync(0, 0);
    chunk.set_block_absolute_y(8, 63, 8, Block::STONE.default_state.id);
    chunk.set_block_absolute_y(9, 64, 8, Block::STONE.default_state.id);
    let mut rail = RailLikeProperties::default(&Block::RAIL);
    rail.shape = RailShape::AscendingEast;
    chunk.set_block_absolute_y(8, 64, 8, rail.to_state_id(&Block::RAIL));
    world.level.loaded_chunks.insert(Vector2::new(0, 0), chunk);

    let start = Vector3::new(8.5, 64.5625, 8.5);
    let cart = Arc::new(MinecartEntity::new(Entity::new(
        world.clone(),
        start,
        &EntityType::MINECART,
    )));
    world.spawn_entity(cart.clone());
    let entity = cart.get_entity();
    entity.velocity.store(Vector3::new(0.1, 0.0, 0.0));
    cart.tick(cart.as_ref(), &server);
    let displacement = entity.pos.load() - start;
    assert!((displacement.x - 0.0921875).abs() < 1.0e-12);
    assert!((displacement.y - 0.0921875).abs() < 1.0e-12);
    assert!((entity.velocity.load().x - 0.083890625).abs() < 1.0e-12);

    entity.set_pos(start);
    entity.velocity.store(Vector3::default());
    cart.tick(cart.as_ref(), &server);
    assert_eq!(
        entity.pos.load(),
        start.add_raw(-0.0078125, -0.0078125, 0.0)
    );
    assert!((entity.velocity.load().x + 0.007890625).abs() < 1.0e-12);
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_player_force_mounted_after_a_pig_can_start_the_classic_cart() -> TestResult {
    use crate::command::CommandSender;
    use pumpkin_protocol::java::client::play::CKeepAlive;

    let directory = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&directory).await?;
    let chunk = ChunkData::empty_sync(0, 0);
    chunk.set_block_absolute_y(8, 63, 8, Block::STONE.default_state.id);
    let mut rail = RailLikeProperties::default(&Block::RAIL);
    rail.shape = RailShape::NorthSouth;
    chunk.set_block_absolute_y(8, 64, 8, rail.to_state_id(&Block::RAIL));
    world.level.loaded_chunks.insert(Vector2::new(0, 0), chunk);
    let start = CENTER.add_raw(0.0, 0.0625, 0.0);
    let cart = Arc::new(MinecartEntity::new(Entity::new(
        world.clone(),
        start,
        &EntityType::MINECART,
    )));
    world.spawn_entity(cart.clone());
    let pig = mount_pig(&world, &(cart.clone() as Arc<dyn EntityBase>));
    let (rider, mut peer) = java_rider(&server, &world, "Notch")?;
    rider.get_entity().set_pos(start);
    assert!(!cart.interact(&rider, &mut ItemStack::EMPTY.clone()));
    assert!(!rider.get_entity().has_vehicle());

    let source = CommandSender::Console.into_source(&server).with_silent();
    assert_eq!(
        server
            .command_dispatcher
            .load()
            .execute_input(
                "ride Notch mount @e[type=minecraft:minecart,limit=1]",
                &source
            )
            .map_err(|error| error.message.get_text())?,
        1
    );
    assert!(cart.get_entity().has_passenger(pig.get_entity().entity_id));
    let client = rider.client.java().ok_or("Missing Java rider")?;
    assert!(client.try_send_packet(&CKeepAlive::new(73)));
    let (passengers, seat) = super::boat_vehicle_tests::read_vehicle_passenger_updates(
        &mut peer,
        cart.get_entity().entity_id,
    )
    .await?;
    assert!(seat.is_some());
    assert_eq!(
        passengers,
        [vec![rider.entity_id(), pig.get_entity().entity_id]]
    );

    rider.get_entity().set_rotation(0.0, 0.0);
    client.handle_player_input(
        &rider,
        &SPlayerInput {
            input: SPlayerInput::FORWARD,
        },
        &server,
    );
    assert_eq!(cart.get_entity().velocity.load(), Vector3::default());
    cart.tick(cart.as_ref(), &server);
    let delta_z = cart.get_entity().pos.load().z - start.z;
    assert!(
        (delta_z - 0.00075).abs() < 1.0e-12,
        "actual start displacement {delta_z}"
    );
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn classic_rail_input_only_overrides_braking_below_the_speed_threshold() -> TestResult {
    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let chunk = ChunkData::empty_sync(0, 0);
    chunk.set_block_absolute_y(8, 63, 8, Block::STONE.default_state.id);
    let mut rail = PoweredRailLikeProperties::default(&Block::POWERED_RAIL);
    rail.shape = RailShapeStraight::EastWest;
    rail.powered = false;
    chunk.set_block_absolute_y(8, 64, 8, rail.to_state_id(&Block::POWERED_RAIL));
    world.level.loaded_chunks.insert(Vector2::new(0, 0), chunk);

    let start = CENTER.add_raw(0.0, 0.0625, 0.0);
    let cart = Arc::new(MinecartEntity::new(Entity::new(
        world.clone(),
        start,
        &EntityType::MINECART,
    )));
    world.spawn_entity(cart.clone());
    let (rider, _peer) = java_rider(&server, &world, "Notch")?;
    let entity = cart.get_entity();
    rider.get_entity().set_pos(start);
    assert!(cart.interact(&rider, &mut ItemStack::EMPTY.clone()));
    assert_mounted(entity, rider.get_entity());
    rider.get_entity().set_rotation(-90.0, 0.0);
    let client = rider.client.java().ok_or("Missing Java client")?;
    let diagonal = 0.00075 * std::f64::consts::FRAC_1_SQRT_2;
    for (input, speed, dx, dz, expected_motion) in [
        (0, 0.0, 0.0, 0.0, None),
        (SPlayerInput::FORWARD, 0.0, 0.00075, 0.0, None),
        (SPlayerInput::LEFT, 0.0, 0.0, -0.00075, None),
        (
            SPlayerInput::FORWARD | SPlayerInput::LEFT,
            0.0,
            diagonal,
            -diagonal,
            None,
        ),
        (
            SPlayerInput::FORWARD | SPlayerInput::BACKWARD,
            0.0,
            0.0,
            0.0,
            None,
        ),
        (0, 0.2, 0.075, 0.0, None),
        (SPlayerInput::FORWARD, 0.2, 0.075, 0.0, None),
        (SPlayerInput::FORWARD, 0.099, 0.075, 0.0, Some(0.0997)),
        (SPlayerInput::FORWARD, 0.101, 0.037875, 0.0, Some(0.0503485)),
        (SPlayerInput::FORWARD, -0.099, -0.0735, 0.0, Some(-0.097706)),
        (
            SPlayerInput::FORWARD,
            -0.101,
            -0.037875,
            0.0,
            Some(-0.0503485),
        ),
    ] {
        entity.set_pos(start);
        entity.velocity.store(Vector3::new(speed, 0.0, 0.0));
        client.handle_player_input(&rider, &SPlayerInput { input }, &server);
        cart.tick(cart.as_ref(), &server);
        let delta = entity.pos.load() - start;
        assert!(
            (delta.x - dx).abs() < 1.0e-12,
            "input {input}, speed {speed}: {delta:?}"
        );
        assert!(
            (delta.z - dz).abs() < 1.0e-12,
            "input {input}, speed {speed}: {delta:?}"
        );
        if let Some(expected) = expected_motion {
            let motion = entity.velocity.load();
            assert!(
                (motion - Vector3::new(expected, 0.0, 0.0)).length_squared() < 1.0e-24,
                "input {input}, speed {speed}: motion {motion:?}"
            );
            assert_eq!(delta.y, 0.0);
        }
    }
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn classic_furnace_ignores_a_push_projection_below_the_java_cutoff() -> TestResult {
    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let chunk = ChunkData::empty_sync(0, 0);
    let mut rail = RailLikeProperties::default(&Block::RAIL);
    rail.shape = RailShape::EastWest;
    for x in 7..=10 {
        chunk.set_block_absolute_y(x, 63, 8, Block::STONE.default_state.id);
        chunk.set_block_absolute_y(x, 64, 8, rail.to_state_id(&Block::RAIL));
    }
    world.level.loaded_chunks.insert(Vector2::new(0, 0), chunk);
    let start = Vector3::new(8.5, 64.0625, 8.5);
    for (push_x, projected_x, next_x) in [
        (0.000_001, 0.0, 0.0768),
        (0.000_02, 1.000_000_000_2, 1.036_800_000_192),
    ] {
        let cart = MinecartEntity::new(Entity::new(
            world.clone(),
            start,
            &EntityType::FURNACE_MINECART,
        ));
        let entity = cart.get_entity();
        entity.set_has_no_gravity(true);
        entity.velocity.store(Vector3::new(0.1, 0.0, 0.0));
        let mut nbt = NbtCompound::new();
        nbt.put_short("Fuel", 1200);
        nbt.put_double("PushX", push_x);
        nbt.put_double("PushZ", 1.0);
        cart.read_custom_nbt(&nbt);

        cart.tick(&cart, &server);

        assert!((entity.pos.load().x - 8.6).abs() < 1.0e-12);
        assert!((entity.velocity.load().x - next_x).abs() < 1.0e-12);
        let mut saved = NbtCompound::new();
        cart.write_custom_nbt(&mut saved);
        let stored_x = saved.get_double("PushX").ok_or("Missing furnace push")?;
        assert!((stored_x - projected_x).abs() < 1.0e-12);
        assert_eq!(saved.get_double("PushZ"), Some(0.0));
    }
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn classic_furnace_uses_the_wet_off_track_limit_after_water_contact() -> TestResult {
    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let chunk = ChunkData::empty_sync(0, 0);
    for x in 7..=9 {
        for z in 7..=9 {
            for y in 63..=66 {
                chunk.set_block_absolute_y(x, y, z, Block::WATER.default_state.id);
            }
        }
    }
    world.level.loaded_chunks.insert(Vector2::new(0, 0), chunk);
    let start = Vector3::new(8.5, 64.5, 8.5);
    let cart = Arc::new(MinecartEntity::new(Entity::new(
        world.clone(),
        start,
        &EntityType::FURNACE_MINECART,
    )));
    world.spawn_entity(cart.clone());
    let entity = cart.get_entity();
    entity.set_has_no_gravity(true);
    cart.tick(cart.as_ref(), &server);
    assert_eq!(entity.pos.load(), start);
    assert!(entity.touching_water.load(Ordering::Relaxed));

    entity.set_has_no_gravity(false);
    entity.velocity.store(Vector3::new(1.3, 0.0, 0.0));
    cart.tick(cart.as_ref(), &server);
    let displacement = entity.pos.load() - start;
    assert!((displacement.x - 0.15).abs() < 1.0e-12);
    assert!((displacement.y + 0.005).abs() < 1.0e-12);
    assert!((entity.velocity.load().x - 0.15 * f64::from(0.95f32)).abs() < 1.0e-12);
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn stopped_minecart_dismount_keeps_the_reversal_side_after_loading() -> TestResult {
    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let chunk = ChunkData::empty_sync(0, 0);
    for x in 7..=9 {
        for z in 7..=9 {
            chunk.set_block_absolute_y(x, 63, z, Block::STONE.default_state.id);
        }
    }
    world.level.loaded_chunks.insert(Vector2::new(0, 0), chunk);
    let cart = MinecartEntity::new(Entity::new(world.clone(), CENTER, &EntityType::MINECART));
    cart.get_entity()
        .velocity
        .store(Vector3::new(0.2, 0.0, 0.0));
    cart.tick(&cart, &server);
    let mut saved = NbtCompound::new();
    cart.write_custom_nbt(&mut saved);
    assert_eq!(saved.get_bool("FlippedRotation"), Some(true));

    let restored = MinecartEntity::new(Entity::new(world.clone(), CENTER, &EntityType::MINECART));
    restored
        .get_entity()
        .set_rotation(cart.get_entity().yaw.load(), 0.0);
    let rider = PigEntity::new(Entity::new(world.clone(), CENTER, &EntityType::PIG));
    assert_eq!(
        restored.vehicle_dismount_position(rider.as_ref(), &|| true),
        Some(Vector3::new(8.5, 64.0, 7.5))
    );
    restored.read_custom_nbt(&saved);
    assert_eq!(
        restored.vehicle_dismount_position(rider.as_ref(), &|| true),
        Some(Vector3::new(8.5, 64.0, 9.5))
    );
    restored.read_custom_nbt(&NbtCompound::new());
    assert_eq!(
        restored.vehicle_dismount_position(rider.as_ref(), &|| true),
        Some(Vector3::new(8.5, 64.0, 7.5))
    );
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rideable_minecart_interaction_rechecks_the_last_seat_after_callbacks() -> TestResult {
    verify_mount_capacity_after_callbacks(|cart, rider| {
        rider.get_entity().set_pos(cart.get_entity().pos.load());
        let mut hand = ItemStack::EMPTY.clone();
        cart.interact(rider, &mut hand)
    })
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rideable_minecart_interaction_rechecks_sneaking_after_mount_listener() -> TestResult {
    use crate::entity::player::Player;

    struct SneakOnMount {
        rider: Arc<Player>,
        fired: AtomicBool,
    }

    impl EventHandler<EntityMountEvent> for SneakOnMount {
        fn handle_blocking<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut EntityMountEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                if event.entity_id == self.rider.entity_id()
                    && !self.fired.swap(true, Ordering::Relaxed)
                {
                    self.rider.get_entity().set_sneaking(true);
                }
            })
        }
    }

    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let (rider, _peer) = java_rider(&server, &world, "Notch")?;
    let cart = Arc::new(MinecartEntity::new(Entity::new(
        world.clone(),
        CENTER,
        &EntityType::MINECART,
    )));
    world.spawn_entity(cart.clone());
    let observer = Arc::new(SneakOnMount {
        rider: rider.clone(),
        fired: AtomicBool::new(false),
    });
    server.plugin_manager.register::<EntityMountEvent, _>(
        observer.clone(),
        EventPriority::Normal,
        true,
    );
    let player_entity = rider.get_entity();
    player_entity.set_pos(CENTER);
    assert!(!player_entity.is_sneaking());
    let mut hand = ItemStack::EMPTY.clone();

    assert!(!cart.interact(&rider, &mut hand));
    assert!(observer.fired.load(Ordering::Relaxed));
    assert!(player_entity.is_sneaking());
    assert!(!player_entity.has_vehicle());
    assert!(!cart.get_entity().has_passengers());

    player_entity.set_sneaking(false);
    assert!(cart.interact(&rider, &mut hand));
    assert_mounted(cart.get_entity(), player_entity);
    world.level.shutdown().await;
    Ok(())
}
