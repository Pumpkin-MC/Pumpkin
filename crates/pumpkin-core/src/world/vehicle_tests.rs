use std::sync::{Arc, Mutex, PoisonError, atomic::AtomicBool, atomic::Ordering};

use crossbeam::atomic::AtomicCell;
use pumpkin_config::{AdvancedConfiguration, BasicConfiguration, TelemetryConfig};
use pumpkin_data::{
    Block,
    damage::DamageType,
    entity::{EntityPose, EntityType},
    game_event::GameEvent,
    packet::CURRENT_MC_VERSION,
    sound::Sound,
};
use pumpkin_protocol::{
    bedrock::client::set_actor_data::entity_data_flag, java::client::play::CSoundEffect,
    packet::MultiVersionJavaPacket, ser::NetworkReadExt,
};
use pumpkin_util::math::{vector2::Vector2, vector3::Vector3};
use pumpkin_world::chunk::ChunkData;

use crate::{
    data::VanillaData,
    entity::{
        Entity, EntityBase,
        passive::pig::PigEntity,
        vehicle::{boat::BoatEntity, minecart::MinecartEntity},
    },
    plugin::{
        BoxFuture, EventPriority, EventRwHandler,
        api::events::{
            Payload,
            block::{
                block_break::BlockBreakEvent, block_drop_item::BlockDropItemEvent,
                block_physics::BlockPhysicsEvent, block_redstone::BlockRedstoneEvent,
                cauldron_level_change::CauldronLevelChangeEvent,
            },
            entity::{
                entity_dismount::EntityDismountEvent, entity_pose_change::EntityPoseChangeEvent,
            },
            vehicle::vehicle_exit::VehicleExitEvent,
            world::generic_game::GenericGameEvent,
        },
        server::packet::PacketSentEvent,
    },
    server::Server,
};

use super::World;

pub(super) type TestResult = Result<(), Box<dyn std::error::Error>>;
pub(super) const CENTER: Vector3<f64> = Vector3::new(8.5, 64.0, 8.5);

pub(super) struct TestContactVehicle(pub(super) Entity);

impl EntityBase for TestContactVehicle {
    fn get_entity(&self) -> &Entity {
        &self.0
    }

    fn get_living_entity(&self) -> Option<&crate::entity::living::LivingEntity> {
        None
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }

    fn vehicle_contact_tick_reach(&self) -> Option<f64> {
        Some(2.0)
    }

    fn get_controlling_passenger(&self) -> Option<Arc<dyn EntityBase>> {
        self.0
            .passengers
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .first()
            .cloned()
    }
}

pub(super) async fn vehicle_world(
    folder: &tempfile::TempDir,
) -> Result<(Arc<Server>, Arc<World>), Box<dyn std::error::Error>> {
    vehicle_world_with_spawn_protection(folder, 0).await
}

pub(super) async fn vehicle_world_with_spawn_protection(
    folder: &tempfile::TempDir,
    spawn_protection: u32,
) -> Result<(Arc<Server>, Arc<World>), Box<dyn std::error::Error>> {
    let basic = BasicConfiguration {
        default_level_name: folder.path().join("world").to_string_lossy().into_owned(),
        spawn_protection,
        allow_nether: false,
        allow_end: false,
        allow_chat_reports: false,
        ..Default::default()
    };
    let mut advanced = AdvancedConfiguration::default();
    advanced.networking.bedrock.online_mode = false;
    let server = Server::new(
        basic,
        advanced,
        TelemetryConfig::default(),
        VanillaData::default(),
        vec![],
    )
    .await?;
    let world = server.worlds.load().first().cloned().ok_or("No world")?;
    Ok((server, world))
}

pub(super) fn mount_pig(world: &Arc<World>, vehicle: &Arc<dyn EntityBase>) -> Arc<PigEntity> {
    let rider = PigEntity::new(Entity::new(world.clone(), CENTER, &EntityType::PIG));
    world.spawn_entity(rider.clone());
    vehicle
        .get_entity()
        .add_passenger(vehicle.clone(), rider.clone());
    assert_mounted(vehicle.get_entity(), rider.get_entity());
    rider
}

pub(super) fn assert_mounted(vehicle: &Entity, rider: &Entity) {
    assert!(vehicle.has_passenger(rider.entity_id));
    assert_ne!(
        rider.bedrock_flags.load(Ordering::Relaxed) & (1i64 << entity_data_flag::RIDING),
        0
    );
    assert_eq!(
        rider
            .get_vehicle()
            .map(|mount| mount.get_entity().entity_id),
        Some(vehicle.entity_id)
    );
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Hook {
    Dismount,
    Exit,
}

type HookCall = (Hook, i32, i32, bool);

struct ExitListener {
    veto: Mutex<Hook>,
    calls: Mutex<Vec<HookCall>>,
}

impl ExitListener {
    fn record(&self, hook: Hook, rider: i32, vehicle: i32) -> bool {
        let cancelled = *self.veto.lock().unwrap_or_else(PoisonError::into_inner) == hook;
        self.calls
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push((hook, rider, vehicle, cancelled));
        cancelled
    }

    fn take_calls(&self) -> Vec<HookCall> {
        std::mem::take(&mut *self.calls.lock().unwrap_or_else(PoisonError::into_inner))
    }
}

impl EventRwHandler<EntityDismountEvent> for ExitListener {
    fn handle<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut EntityDismountEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            event.cancelled = self.record(Hook::Dismount, event.entity_id, event.dismounted_id);
        })
    }
}

impl EventRwHandler<VehicleExitEvent> for ExitListener {
    fn handle<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut VehicleExitEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            event.cancelled = self.record(Hook::Exit, event.exited_id, event.vehicle_id);
        })
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn destroying_occupied_vehicles_detaches_passengers_despite_vetoes() -> TestResult {
    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let listener = Arc::new(ExitListener {
        veto: Mutex::new(Hook::Dismount),
        calls: Mutex::default(),
    });
    server
        .plugin_manager
        .register_blocking::<EntityDismountEvent, _>(listener.clone(), EventPriority::Normal, None);
    server
        .plugin_manager
        .register_blocking::<VehicleExitEvent, _>(listener.clone(), EventPriority::Normal, None);

    let mut stale_links = Vec::new();
    for (kind, veto, rider_count) in [
        (&EntityType::OAK_BOAT, Hook::Dismount, 2),
        (&EntityType::MINECART, Hook::Exit, 1),
    ] {
        *listener.veto.lock().unwrap_or_else(PoisonError::into_inner) = veto;
        let entity = Entity::new(world.clone(), CENTER, kind);
        let vehicle: Arc<dyn EntityBase> = if kind == &EntityType::OAK_BOAT {
            Arc::new(BoatEntity::new(entity))
        } else {
            Arc::new(MinecartEntity::new(entity))
        };
        let base = vehicle.get_entity();
        world.spawn_entity(vehicle.clone());
        let riders: Vec<_> = (0..rider_count)
            .map(|_| mount_pig(&world, &vehicle))
            .collect();
        let expected_calls: Vec<_> = riders
            .iter()
            .flat_map(|rider| {
                let rider_id = rider.get_entity().entity_id;
                [Hook::Dismount, Hook::Exit]
                    .map(|hook| (hook, rider_id, base.entity_id, veto == hook))
            })
            .collect();

        for rider in &riders {
            let entity = rider.get_entity();
            base.remove_passenger(entity.entity_id);
            assert_mounted(base, entity);
        }
        assert_eq!(listener.take_calls(), expected_calls);

        assert!(vehicle.damage(vehicle.as_ref(), 5.0, DamageType::GENERIC));
        assert!(base.is_removed());
        assert!(world.get_entity_by_id(base.entity_id).is_none());
        assert_eq!(listener.take_calls(), expected_calls);
        for rider in riders {
            let rider = rider.get_entity();
            assert!(!rider.is_removed());
            assert!(world.get_entity_by_id(rider.entity_id).is_some());
            let listed = base.has_passenger(rider.entity_id);
            let riding = rider.get_vehicle().is_some();
            assert_eq!(
                rider.bedrock_flags.load(Ordering::Relaxed) & (1i64 << entity_data_flag::RIDING),
                0
            );
            if listed || riding {
                stale_links.push((kind.resource_name, veto, listed, riding));
            }
        }
    }

    world.level.shutdown().await;
    assert!(stale_links.is_empty(), "stale mount links: {stale_links:?}");
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn restored_vehicle_rider_clears_the_bedrock_flag_on_quiet_detach() -> TestResult {
    let folder = tempfile::tempdir()?;
    let (_, world) = vehicle_world(&folder).await?;
    let boat = Arc::new(BoatEntity::new(Entity::new(
        world.clone(),
        CENTER,
        &EntityType::OAK_BOAT,
    )));
    let pig = PigEntity::new(Entity::new(world.clone(), CENTER, &EntityType::PIG));
    let rider = pig.get_entity();
    let vehicle = boat.get_entity();
    vehicle.add_passenger_silently(boat.clone(), pig.clone());
    assert_mounted(vehicle, rider);

    vehicle.remove_passenger_sync(rider.entity_id);
    assert!(!vehicle.has_passenger(rider.entity_id));
    assert!(rider.get_vehicle().is_none());
    assert_eq!(
        rider.bedrock_flags.load(Ordering::Relaxed) & (1i64 << entity_data_flag::RIDING),
        0
    );
    world.level.shutdown().await;
    Ok(())
}

struct RemountDuringExit {
    vehicle: Arc<dyn EntityBase>,
    rider: Arc<PigEntity>,
    ran: AtomicBool,
}

impl EventRwHandler<EntityDismountEvent> for RemountDuringExit {
    fn handle<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut EntityDismountEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            let vehicle = self.vehicle.get_entity();
            let rider = self.rider.get_entity();
            if event.entity_id == rider.entity_id && event.dismounted_id == vehicle.entity_id {
                self.ran.store(true, Ordering::Relaxed);
                vehicle.remove_passenger_sync(rider.entity_id);
                vehicle.add_passenger(self.vehicle.clone(), self.rider.clone());
            }
        })
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn vehicle_exit_preserves_an_attachment_recreated_by_a_listener() -> TestResult {
    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let vehicle: Arc<dyn EntityBase> = Arc::new(BoatEntity::new(Entity::new(
        world.clone(),
        CENTER,
        &EntityType::OAK_BOAT,
    )));
    world.spawn_entity(vehicle.clone());
    let rider = mount_pig(&world, &vehicle);
    let listener = Arc::new(RemountDuringExit {
        vehicle: vehicle.clone(),
        rider: rider.clone(),
        ran: AtomicBool::new(false),
    });
    server
        .plugin_manager
        .register_blocking::<EntityDismountEvent, _>(listener.clone(), EventPriority::Normal, None);

    vehicle
        .get_entity()
        .remove_passenger(rider.get_entity().entity_id);
    assert!(listener.ran.load(Ordering::Relaxed));
    assert_mounted(vehicle.get_entity(), rider.get_entity());
    assert_eq!(rider.get_entity().pos.load(), CENTER);
    world.level.shutdown().await;
    Ok(())
}

struct RelocateRider {
    rider: Arc<PigEntity>,
    destination: Vector3<f64>,
    ran: AtomicBool,
}

impl RelocateRider {
    fn relocate(&self, entity_id: i32) {
        let rider = self.rider.get_entity();
        if entity_id == rider.entity_id {
            self.ran.store(true, Ordering::Relaxed);
            rider.set_pos(self.destination);
        }
    }
}

impl EventRwHandler<EntityPoseChangeEvent> for RelocateRider {
    fn handle<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut EntityPoseChangeEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move { self.relocate(event.entity_id) })
    }
}

impl EventRwHandler<EntityDismountEvent> for RelocateRider {
    fn handle<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut EntityDismountEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move { self.relocate(event.entity_id) })
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn vehicle_exit_preserves_a_relocation_from_the_pose_listener() -> TestResult {
    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let chunk = ChunkData::empty_sync(0, 0);
    chunk.set_block_absolute_y(8, 63, 9, Block::STONE.default_state.id);
    world.level.loaded_chunks.insert(Vector2::new(0, 0), chunk);
    let cart: Arc<dyn EntityBase> = Arc::new(MinecartEntity::new(Entity::new(
        world.clone(),
        CENTER,
        &EntityType::MINECART,
    )));
    cart.get_entity().set_rotation(-90.0, 0.0);
    world.spawn_entity(cart.clone());
    let rider = mount_pig(&world, &cart);
    rider.get_entity().set_pose(EntityPose::Crouching);
    let listener = Arc::new(RelocateRider {
        rider: rider.clone(),
        destination: Vector3::new(14.5, 65.0, 14.5),
        ran: AtomicBool::new(false),
    });
    server
        .plugin_manager
        .register_blocking::<EntityPoseChangeEvent, _>(
            listener.clone(),
            EventPriority::Normal,
            None,
        );

    cart.get_entity()
        .remove_passenger(rider.get_entity().entity_id);
    assert!(listener.ran.load(Ordering::Relaxed));
    assert_eq!(rider.get_entity().pose.load(), EntityPose::Crouching);
    assert_eq!(rider.get_entity().pos.load(), listener.destination);
    assert!(!rider.get_entity().has_vehicle());
    assert!(
        !cart
            .get_entity()
            .has_passenger(rider.get_entity().entity_id)
    );
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn vehicle_exit_preserves_a_relocation_from_the_exit_listener() -> TestResult {
    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let boat: Arc<dyn EntityBase> = Arc::new(BoatEntity::new(Entity::new(
        world.clone(),
        CENTER,
        &EntityType::OAK_BOAT,
    )));
    world.spawn_entity(boat.clone());
    let rider = mount_pig(&world, &boat);
    let listener = Arc::new(RelocateRider {
        rider: rider.clone(),
        destination: Vector3::new(14.5, 65.0, 14.5),
        ran: AtomicBool::new(false),
    });
    server
        .plugin_manager
        .register_blocking::<EntityDismountEvent, _>(listener.clone(), EventPriority::Normal, None);

    boat.get_entity()
        .remove_passenger(rider.get_entity().entity_id);
    assert!(listener.ran.load(Ordering::Relaxed));
    assert_eq!(rider.get_entity().pos.load(), listener.destination);
    assert!(!rider.get_entity().has_vehicle());
    assert!(
        !boat
            .get_entity()
            .has_passenger(rider.get_entity().entity_id)
    );
    world.level.shutdown().await;
    Ok(())
}

pub(super) fn java_rider(
    server: &Arc<Server>,
    world: &Arc<World>,
    name: &str,
) -> Result<(Arc<crate::entity::player::Player>, tokio::io::DuplexStream), Box<dyn std::error::Error>>
{
    use arc_swap::ArcSwap;
    use pumpkin_protocol::ConnectionState;
    use pumpkin_util::GameMode;

    use crate::entity::player::Player;
    use crate::net::{
        ClientPlatform, GameProfile, PacketRateLimiter, PlayerConfig,
        java::{JavaClient, pending::PendingConnection},
    };

    let (transport, peer) = tokio::io::duplex(8192);
    let (read, write) = tokio::io::split(transport);
    let pending = PendingConnection::from_transport(
        Box::new(read),
        Box::new(write),
        "127.0.0.1:25565".parse()?,
        0,
        PacketRateLimiter::new(false, 0.0, 0.0),
        Arc::downgrade(server),
    );
    let profile = GameProfile {
        id: uuid::Uuid::new_v4(),
        name: name.into(),
        properties: ArcSwap::from_pointee(vec![]),
        profile_actions: None,
    };
    let mut client = JavaClient::from_pending(pending, profile.clone(), PlayerConfig::default());
    client.connection_state.store(ConnectionState::Play);
    client.start_outgoing_packet_task();
    let player = Arc::new(Player::new(
        Arc::new(ClientPlatform::Java(client)),
        profile,
        PlayerConfig::default(),
        world,
        GameMode::Survival,
    ));
    player.set_client_loaded(true);
    world.add_player(&player)?;
    Ok((player, peer))
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn manual_vehicle_mount_preserves_teleport_from_mount_hook() -> TestResult {
    use crate::{
        entity::player::Player, plugin::api::events::entity::entity_mount::EntityMountEvent,
    };
    use pumpkin_data::item_stack::ItemStack;
    use pumpkin_protocol::java::client::play::{CKeepAlive, CPlayerPosition};

    struct TeleportOnMount {
        rider: Arc<Player>,
        destination: Vector3<f64>,
        ran: AtomicBool,
    }
    impl EventRwHandler<EntityMountEvent> for TeleportOnMount {
        fn handle<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut EntityMountEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                if event.entity_id == self.rider.entity_id() {
                    assert!(!self.ran.swap(true, Ordering::Relaxed));
                    self.rider.request_teleport(self.destination, 0.0, 0.0);
                }
            })
        }
    }

    for destination in [CENTER, CENTER.add_raw(12.0, 0.0, 0.0)] {
        let folder = tempfile::tempdir()?;
        let (server, world) = vehicle_world(&folder).await?;
        let (rider, peer) = java_rider(&server, &world, "Notch")?;
        let player = rider.get_entity();
        player.set_pos(CENTER);
        let client = rider.client.java().ok_or("Missing Java rider")?;
        client.player.store(Arc::new(Some(rider.clone())));
        let boat = Arc::new(BoatEntity::new(Entity::new(
            world.clone(),
            CENTER,
            &EntityType::OAK_BOAT,
        )));
        world.spawn_entity(boat.clone());
        let listener = Arc::new(TeleportOnMount {
            rider: rider.clone(),
            destination,
            ran: AtomicBool::new(false),
        });
        server
            .plugin_manager
            .register_blocking::<EntityMountEvent, _>(
                listener.clone(),
                EventPriority::Normal,
                None,
            );
        let expected_id = rider
            .teleport_id_count
            .load(Ordering::Relaxed)
            .wrapping_add(1);

        assert!(!boat.interact(&rider, &mut ItemStack::EMPTY.clone()));
        assert!(listener.ran.load(Ordering::Relaxed));
        assert!(!player.has_vehicle());
        assert!(!boat.get_entity().has_passengers());
        assert_eq!(player.pos.load(), destination);
        assert_eq!(
            *rider
                .awaiting_teleport
                .lock()
                .unwrap_or_else(PoisonError::into_inner),
            Some((expected_id.into(), destination))
        );
        assert!(client.try_send_packet(&CKeepAlive::new(73)));
        let mut decoder = pumpkin_protocol::java::packet_decoder::TCPNetworkDecoder::new(
            tokio::io::BufReader::new(peer),
        );
        let packets = outbound_packets_before_marker(
            &mut decoder,
            CPlayerPosition::to_id(CURRENT_MC_VERSION),
        )
        .await?;
        assert_eq!(packets.len(), 1);
        client.player.store(Arc::new(None));
        client.close();
        world.level.shutdown().await;
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ordinary_vehicle_interaction_switches_mounts_after_an_allowed_exit() -> TestResult {
    use crate::plugin::api::events::entity::entity_mount::EntityMountEvent;
    use pumpkin_data::item_stack::ItemStack;

    struct VetoExit(AtomicBool);
    impl EventRwHandler<EntityDismountEvent> for VetoExit {
        fn handle<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut EntityDismountEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                event.cancelled = self.0.load(Ordering::Relaxed);
            })
        }
    }

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

    for cart_target in [false, true] {
        let directory = tempfile::tempdir()?;
        let (server, world) = vehicle_world(&directory).await?;
        let (rider, _peer) = java_rider(&server, &world, "Notch")?;
        let player = rider.get_entity();
        player.set_pos(CENTER);
        let start = Arc::new(BoatEntity::new(Entity::new(
            world.clone(),
            CENTER,
            &EntityType::OAK_BOAT,
        )));
        let destination = CENTER.add_raw(2.0, 0.0, 0.0);
        let target: Arc<dyn EntityBase> = if cart_target {
            Arc::new(MinecartEntity::new(Entity::new(
                world.clone(),
                destination,
                &EntityType::MINECART,
            )))
        } else {
            Arc::new(BoatEntity::new(Entity::new(
                world.clone(),
                destination,
                &EntityType::BAMBOO_CHEST_RAFT,
            )))
        };
        world.spawn_entity(start.clone());
        world.spawn_entity(target.clone());
        let old = start.get_entity();
        old.add_passenger(start.clone(), rider.clone());
        assert_mounted(old, player);
        let veto = Arc::new(VetoExit(AtomicBool::new(true)));
        server
            .plugin_manager
            .register_blocking::<EntityDismountEvent, _>(veto.clone(), EventPriority::Normal, None);
        let mut hand = ItemStack::EMPTY.clone();
        let old_id = rider.teleport_id_count.load(Ordering::Relaxed);

        assert!(!target.interact(&rider, &mut hand));
        assert_mounted(old, player);
        assert!(!target.get_entity().has_passengers());
        assert_eq!(rider.teleport_id_count.load(Ordering::Relaxed), old_id);

        veto.0.store(false, Ordering::Relaxed);
        let veto_mount = Arc::new(VetoMount(AtomicBool::new(true)));
        server
            .plugin_manager
            .register_blocking::<EntityMountEvent, _>(
                veto_mount.clone(),
                EventPriority::Normal,
                None,
            );
        let old_revision = player.riding_revision();
        let old_cooldown = player.riding_cooldown.load(Ordering::Relaxed);
        assert!(!target.interact(&rider, &mut hand));
        assert_mounted(old, player);
        assert!(!target.get_entity().has_passengers());
        assert_eq!(player.riding_revision(), old_revision);
        assert_eq!(player.riding_cooldown.load(Ordering::Relaxed), old_cooldown);
        assert_eq!(rider.teleport_id_count.load(Ordering::Relaxed), old_id);

        veto_mount.0.store(false, Ordering::Relaxed);
        assert!(target.interact(&rider, &mut hand));
        assert!(!old.has_passengers());
        assert_mounted(target.get_entity(), player);
        assert_switch_placement(&rider, target.get_entity(), destination, old_id);
        world.level.shutdown().await;
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn vehicle_switch_keeps_source_when_destination_fills_during_mount_event() -> TestResult {
    use crate::plugin::api::events::entity::entity_mount::EntityMountEvent;

    let directory = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&directory).await?;
    let (rider, _peer) = java_rider(&server, &world, "Notch")?;
    let source = Arc::new(BoatEntity::new(Entity::new(
        world.clone(),
        CENTER,
        &EntityType::OAK_BOAT,
    )));
    let position = CENTER.add_raw(2.0, 0.0, 0.0);
    let target = Arc::new(MinecartEntity::new(Entity::new(
        world.clone(),
        position,
        &EntityType::MINECART,
    )));
    assert!(world.spawn_entity(source.clone()));
    assert!(world.spawn_entity(target.clone()));
    rider.get_entity().set_pos(CENTER);
    source
        .get_entity()
        .add_passenger(source.clone(), rider.clone());

    let occupant = PigEntity::new(Entity::new(world.clone(), position, &EntityType::PIG));
    assert!(world.spawn_entity(occupant.clone()));
    server
        .plugin_manager
        .register_blocking::<EntityMountEvent, _>(
            Arc::new(FillLastSeat {
                original: rider.entity_id(),
                vehicle: target.clone(),
                competitor: occupant.clone(),
            }),
            EventPriority::Normal,
            None,
        );

    assert!(!target.interact(
        &rider,
        &mut pumpkin_data::item_stack::ItemStack::EMPTY.clone()
    ));
    assert_mounted(source.get_entity(), rider.get_entity());
    assert_mounted(target.get_entity(), occupant.get_entity());
    world.level.shutdown().await;
    Ok(())
}

fn assert_switch_placement(
    rider: &crate::entity::player::Player,
    target: &Entity,
    destination: Vector3<f64>,
    old_teleport_id: i32,
) {
    let position = rider.get_entity().pos.load();
    let seat_offset = target
        .entity_type
        .boat
        .map_or(0.0, |seats| f64::from(seats.single_passenger_x_offset));
    assert_eq!(
        (position.x, position.z),
        (destination.x, destination.z + seat_offset)
    );
    assert_eq!(
        *rider
            .awaiting_teleport
            .lock()
            .unwrap_or_else(PoisonError::into_inner),
        Some((old_teleport_id.wrapping_add(1).into(), position))
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn vehicle_switch_preserves_movement_from_the_source_dismount_event() -> TestResult {
    struct MoveOnDismount {
        rider: Arc<crate::entity::player::Player>,
        position: Vector3<f64>,
    }

    impl EventRwHandler<GenericGameEvent> for MoveOnDismount {
        fn handle<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut GenericGameEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async {
                if event.event_key == GameEvent::EntityDismount.name() {
                    self.rider.get_entity().set_pos(self.position);
                }
            })
        }
    }

    let directory = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&directory).await?;
    let (rider, _peer) = java_rider(&server, &world, "Notch")?;
    rider.get_entity().set_pos(CENTER);
    let source = Arc::new(BoatEntity::new(Entity::new(
        world.clone(),
        CENTER,
        &EntityType::OAK_BOAT,
    )));
    let target = Arc::new(MinecartEntity::new(Entity::new(
        world.clone(),
        CENTER.add_raw(2.0, 0.0, 0.0),
        &EntityType::MINECART,
    )));
    assert!(world.spawn_entity(source.clone()));
    assert!(world.spawn_entity(target.clone()));
    source
        .get_entity()
        .add_passenger(source.clone(), rider.clone());
    let old_id = rider.teleport_id_count.load(Ordering::Relaxed);
    let position = CENTER.add_raw(0.0, 3.0, 0.0);
    server
        .plugin_manager
        .register_blocking::<GenericGameEvent, _>(
            Arc::new(MoveOnDismount {
                rider: rider.clone(),
                position,
            }),
            EventPriority::Normal,
            None,
        );

    assert!(target.interact(
        &rider,
        &mut pumpkin_data::item_stack::ItemStack::EMPTY.clone()
    ));
    assert!(!source.get_entity().has_passengers());
    assert_mounted(target.get_entity(), rider.get_entity());
    assert_eq!(rider.get_entity().pos.load(), position);
    assert_eq!(rider.teleport_id_count.load(Ordering::Relaxed), old_id);
    world.level.shutdown().await;
    Ok(())
}

pub(super) fn queue_vehicle_position(
    rider: &crate::entity::player::Player,
    target: Vector3<f64>,
    yaw: f32,
) -> TestResult {
    use pumpkin_data::packet::CURRENT_MC_VERSION;
    use pumpkin_protocol::{
        ClientPacket, RawPacket, java::server::play::SMoveVehicle, packet::MultiVersionJavaPacket,
    };
    let proposal = SMoveVehicle {
        x: target.x,
        y: target.y,
        z: target.z,
        yaw,
        pitch: 0.0,
        on_ground: false,
    };
    let mut data = Vec::new();
    proposal.write_packet_data(&mut data, &CURRENT_MC_VERSION)?;
    rider.inbound_packets.push(RawPacket {
        id: SMoveVehicle::to_id(CURRENT_MC_VERSION),
        payload: data.into(),
    });
    Ok(())
}

pub(super) fn mounted_java_boat(
    world: &Arc<World>,
    rider: &Arc<crate::entity::player::Player>,
    position: Vector3<f64>,
) -> Arc<BoatEntity> {
    let boat = Arc::new(BoatEntity::new(Entity::new(
        world.clone(),
        position,
        &EntityType::OAK_BOAT,
    )));
    world.spawn_entity(boat.clone());
    // Start after the initial seat correction has been acknowledged.
    boat.get_entity()
        .add_passenger_silently(boat.clone(), rider.clone());
    crate::entity::position_rider(boat.as_ref(), rider.as_ref());
    assert_mounted(boat.get_entity(), rider.get_entity());
    boat
}

async fn outbound_packets_before_marker(
    decoder: &mut pumpkin_protocol::java::packet_decoder::TCPNetworkDecoder<
        tokio::io::BufReader<tokio::io::DuplexStream>,
    >,
    packet_id: i32,
) -> Result<Vec<pumpkin_protocol::RawPacket>, Box<dyn std::error::Error>> {
    use pumpkin_data::packet::CURRENT_MC_VERSION;
    use pumpkin_protocol::{java::client::play::CKeepAlive, packet::MultiVersionJavaPacket};

    let observed = tokio::time::timeout(std::time::Duration::from_secs(5), async {
        let mut packets = Vec::new();
        loop {
            let raw = decoder.get_raw_packet().await?;
            if raw.id == CKeepAlive::to_id(CURRENT_MC_VERSION) {
                return Ok::<_, pumpkin_protocol::PacketDecodeError>(packets);
            }
            if raw.id == packet_id {
                packets.push(raw);
            }
        }
    })
    .await??;
    Ok(observed)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn java_vehicle_movement_preserves_a_saddled_horses_cart_collision_rule() -> TestResult {
    use crate::entity::passive::horse::HorseEntity;
    use pumpkin_data::packet::CURRENT_MC_VERSION;
    use pumpkin_protocol::{
        java::{
            client::play::{CKeepAlive, CMoveVehicle},
            packet_decoder::TCPNetworkDecoder,
        },
        packet::MultiVersionJavaPacket,
    };

    let directory = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&directory).await?;
    let (rider, peer) = java_rider(&server, &world, "jeb_")?;
    let client = rider.client.java().ok_or("Missing Java client")?;
    let horse = HorseEntity::new(Entity::new(world.clone(), CENTER, &EntityType::HORSE));
    horse.mob_entity.read_legacy_saddle(true);
    world.spawn_entity(horse.clone());
    horse
        .get_entity()
        .add_passenger(horse.clone(), rider.clone());
    assert_mounted(horse.get_entity(), rider.get_entity());
    assert!(horse.get_controlling_passenger().is_some());

    let target = CENTER.add_raw(1.5, 0.0, 0.0);
    let cart = Arc::new(MinecartEntity::new(Entity::new(
        world.clone(),
        target,
        &EntityType::MINECART,
    )));
    world.spawn_entity(cart.clone());
    assert!(
        !horse
            .get_entity()
            .bounding_box
            .load()
            .intersects(&cart.get_entity().bounding_box.load())
    );
    queue_vehicle_position(&rider, target, 0.0)?;
    World::process_player_packets(&world.players.load());

    assert_eq!(horse.get_entity().pos.load(), target);
    assert!(client.try_send_packet(&CKeepAlive::new(73)));
    let mut decoder = TCPNetworkDecoder::new(tokio::io::BufReader::new(peer));
    assert!(
        outbound_packets_before_marker(&mut decoder, CMoveVehicle::to_id(CURRENT_MC_VERSION))
            .await?
            .is_empty()
    );
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn java_vehicle_movement_passes_cushions_and_still_clips_pushable_mobs() -> TestResult {
    use crate::entity::decoration::cushion::CushionEntity;
    use pumpkin_data::packet::CURRENT_MC_VERSION;
    use pumpkin_protocol::{
        java::{
            client::play::{CKeepAlive, CMoveVehicle},
            packet_decoder::TCPNetworkDecoder,
        },
        packet::MultiVersionJavaPacket,
    };

    let directory = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&directory).await?;
    let (rider, peer) = java_rider(&server, &world, "jeb_")?;
    let client = rider.client.java().ok_or("Missing Java client")?;
    let boat = mounted_java_boat(&world, &rider, CENTER);
    let target = CENTER.add_raw(1.0, 0.0, 0.0);
    let obstacle = CENTER.add_raw(1.5, 0.0, 0.0);
    let cushion = Arc::new(CushionEntity::new(
        Entity::new(world.clone(), obstacle, &EntityType::CUSHION),
        0,
    ));
    world.spawn_entity(cushion.clone());
    let start = boat.get_entity().bounding_box.load();
    let cushion_bounds = cushion.get_entity().bounding_box.load();
    assert!(!start.intersects(&cushion_bounds));
    assert!(start.shift(target - CENTER).intersects(&cushion_bounds));
    queue_vehicle_position(&rider, target, 0.0)?;
    World::process_player_packets(&world.players.load());
    assert_eq!(boat.get_entity().pos.load(), target);
    assert!(client.try_send_packet(&CKeepAlive::new(73)));
    let mut decoder = TCPNetworkDecoder::new(tokio::io::BufReader::new(peer));
    let correction = CMoveVehicle::to_id(CURRENT_MC_VERSION);
    assert!(
        outbound_packets_before_marker(&mut decoder, correction)
            .await?
            .is_empty()
    );

    boat.get_entity().set_pos(CENTER);
    let chunk = Vector2::new(0, 0);
    world
        .level
        .loaded_chunks
        .insert(chunk, ChunkData::empty_sync(0, 0));
    Arc::make_mut(
        &mut world
            .active_chunks
            .write()
            .unwrap_or_else(PoisonError::into_inner),
    )
    .insert(chunk);
    let pig = PigEntity::new(Entity::new(world.clone(), obstacle, &EntityType::PIG));
    world.spawn_entity(pig);
    queue_vehicle_position(&rider, target, 0.0)?;
    World::process_player_packets(&world.players.load());
    assert_eq!(boat.get_entity().pos.load(), CENTER);
    assert!(client.try_send_packet(&CKeepAlive::new(74)));
    assert_eq!(
        outbound_packets_before_marker(&mut decoder, correction)
            .await?
            .len(),
        1
    );
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn vehicle_projection_only_clips_ordinary_mobs_in_ticking_chunks() -> TestResult {
    use crate::entity::passive::cow::CowEntity;

    let directory = tempfile::tempdir()?;
    let (_, world) = vehicle_world(&directory).await?;
    let cart = MinecartEntity::new(Entity::new(
        world.clone(),
        Vector3::new(15.0, 64.0, 8.5),
        &EntityType::MINECART,
    ));
    let cow = CowEntity::new(Entity::new(
        world.clone(),
        Vector3::new(16.5, 64.0, 8.5),
        &EntityType::COW,
    ));
    let chunk = cow.get_entity().chunk_pos.load();
    world
        .level
        .loaded_chunks
        .insert(chunk, ChunkData::empty_sync(chunk.x, chunk.y));
    world.spawn_entity(cow.clone());
    let movement = Vector3::new(1.5, 0.0, 0.0);
    let project = || cart.get_entity().project_movement(movement, &cart).movement;
    assert!(cow.is_pushable());
    assert_eq!(
        project(),
        movement,
        "a loaded but idle cow does not block a cart"
    );
    Arc::make_mut(
        &mut world
            .active_chunks
            .write()
            .unwrap_or_else(PoisonError::into_inner),
    )
    .insert(chunk);
    assert!(
        project().x < movement.x,
        "the same cow blocks movement once it ticks"
    );
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn java_vehicle_inbound_phase_commits_conflicting_moves_in_player_order() -> TestResult {
    use pumpkin_data::packet::CURRENT_MC_VERSION;
    use pumpkin_protocol::{
        ClientPacket, RawPacket, java::server::play::SPlayerPosition,
        packet::MultiVersionJavaPacket,
    };

    let directory = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&directory).await?;
    let chunk = ChunkData::empty_sync(0, 0);
    world.level.loaded_chunks.insert(Vector2::new(0, 0), chunk);
    let (first, _first_peer) = java_rider(&server, &world, "Notch")?;
    let (pedestrian, _pedestrian_peer) = java_rider(&server, &world, "deadmau5")?;
    let (second, _second_peer) = java_rider(&server, &world, "jeb_")?;
    let first_boat = mounted_java_boat(&world, &first, CENTER.add_raw(-5.0, 0.0, 0.0));
    let second_start = CENTER.add_raw(5.0, 0.0, 0.0);
    let second_boat = mounted_java_boat(&world, &second, second_start);
    let walker = pedestrian.get_entity();
    walker.set_pos(CENTER);
    let walks_to = CENTER.add_raw(0.0, 0.0, 3.0);
    let mut payload = Vec::new();
    SPlayerPosition {
        position: walks_to,
        collision: 0,
    }
    .write_packet_data(&mut payload, &CURRENT_MC_VERSION)?;
    pedestrian.inbound_packets.push(RawPacket {
        id: SPlayerPosition::to_id(CURRENT_MC_VERSION),
        payload: payload.into(),
    });

    queue_vehicle_position(&first, CENTER, 45.0)?;
    queue_vehicle_position(&second, CENTER, -45.0)?;
    World::process_player_packets(&[first.clone(), pedestrian.clone(), second.clone()]);

    assert_eq!(walker.pos.load(), walks_to);
    assert_eq!(first_boat.get_entity().pos.load(), CENTER);
    assert_eq!(second_boat.get_entity().pos.load(), second_start);
    assert!(
        second_boat
            .get_entity()
            .horizontal_collision
            .load(Ordering::Relaxed)
    );
    assert!((second_boat.get_entity().yaw.load() + 45.0).abs() < f32::EPSILON);
    assert!(first.get_entity().movement.load().length_squared() > 0.0);
    assert_eq!(second.get_entity().movement.load(), Vector3::default());
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ride_command_uses_the_vehicle_exit_and_respects_vetoes() -> TestResult {
    use crate::command::CommandSender;
    use crate::plugin::api::events::world::generic_game::GenericGameEvent;
    use pumpkin_data::game_event::GameEvent;

    #[derive(Default)]
    struct DismountEvents(Mutex<Vec<Vector3<f64>>>);
    impl EventRwHandler<GenericGameEvent> for DismountEvents {
        fn handle<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut GenericGameEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async {
                if event.event_key == GameEvent::EntityDismount.name() {
                    self.0
                        .lock()
                        .unwrap_or_else(PoisonError::into_inner)
                        .push(event.position);
                }
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
    let cart = Arc::new(MinecartEntity::new(Entity::new(
        world.clone(),
        CENTER,
        &EntityType::MINECART,
    )));
    cart.get_entity().set_rotation(-90.0, 0.0);
    world.spawn_entity(cart.clone());
    cart.get_entity().add_passenger(cart.clone(), rider.clone());
    let player = rider.get_entity();
    crate::entity::position_rider(cart.as_ref(), rider.as_ref());
    let seated_position = player.pos.load();
    assert_eq!(seated_position.x, CENTER.x);
    assert_eq!(seated_position.z, CENTER.z);
    let listener = Arc::new(ExitListener {
        veto: Mutex::new(Hook::Exit),
        calls: Mutex::default(),
    });
    server
        .plugin_manager
        .register_blocking::<VehicleExitEvent, _>(listener.clone(), EventPriority::Normal, None);
    let events = Arc::new(DismountEvents::default());
    server
        .plugin_manager
        .register_blocking::<GenericGameEvent, _>(events.clone(), EventPriority::Normal, None);
    let source = CommandSender::Console.into_source(&server).with_silent();
    let dismount = || {
        server
            .command_dispatcher
            .load()
            .execute_input("ride Notch dismount", &source)
    };
    assert!(dismount().is_err());
    assert_mounted(cart.get_entity(), player);
    assert_eq!(player.pos.load(), seated_position);
    assert!(
        events
            .0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .is_empty()
    );
    *listener.veto.lock().unwrap_or_else(PoisonError::into_inner) = Hook::Dismount;
    let result = dismount().map_err(|error| error.message.get_text())?;

    assert_eq!(result, 1);
    assert_eq!(
        events
            .0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .as_slice(),
        [CENTER],
    );
    assert!(!player.has_vehicle());
    assert!(!cart.get_entity().has_passenger(rider.entity_id()));
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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn java_vehicle_speed_correction_preserves_the_last_committed_look() -> TestResult {
    use pumpkin_data::packet::CURRENT_MC_VERSION;
    use pumpkin_protocol::{
        ServerPacket,
        java::{client::play::CMoveVehicle, packet_decoder::TCPNetworkDecoder},
        packet::MultiVersionJavaPacket,
    };
    use std::time::Duration;

    let directory = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&directory).await?;
    let (rider, peer) = java_rider(&server, &world, "deadmau5")?;
    let boat = mounted_java_boat(&world, &rider, CENTER);
    boat.get_entity().set_rotation(30.0, 0.0);
    boat.get_entity()
        .horizontal_collision
        .store(true, Ordering::Relaxed);
    queue_vehicle_position(&rider, CENTER.add_raw(15.0, 0.0, 0.0), -30.0)?;
    World::process_player_packets(&world.players.load());

    let mut decoder = TCPNetworkDecoder::new(tokio::io::BufReader::new(peer));
    let moved = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let raw = decoder.get_raw_packet().await?;
            if raw.id == CMoveVehicle::to_id(CURRENT_MC_VERSION) {
                return Ok::<_, pumpkin_protocol::PacketDecodeError>(raw);
            }
        }
    })
    .await??;
    let correction = CMoveVehicle::read(&mut moved.payload.as_ref(), &CURRENT_MC_VERSION)?;
    assert_eq!(
        Vector3::new(correction.x, correction.y, correction.z),
        CENTER
    );
    assert!((correction.yaw - 30.0).abs() < f32::EPSILON);
    assert!(
        boat.get_entity()
            .horizontal_collision
            .load(Ordering::Relaxed)
    );
    assert_eq!(boat.get_entity().pos.load(), CENTER);
    world.level.shutdown().await;
    Ok(())
}

struct FillLastSeat {
    vehicle: Arc<dyn EntityBase>,
    competitor: Arc<PigEntity>,
    original: i32,
}

impl EventRwHandler<crate::plugin::api::events::entity::entity_mount::EntityMountEvent>
    for FillLastSeat
{
    fn handle<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut crate::plugin::api::events::entity::entity_mount::EntityMountEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            if event.entity_id == self.original
                && event.mount_id == self.vehicle.get_entity().entity_id
            {
                self.vehicle
                    .get_entity()
                    .add_passenger(self.vehicle.clone(), self.competitor.clone());
            }
        })
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn vehicle_mount_capacity_is_rechecked_after_callbacks() -> TestResult {
    verify_mount_capacity_after_callbacks(|cart, rider| {
        cart.get_entity()
            .add_passenger_with_capacity(cart.clone(), rider.clone(), 1)
    })
    .await
}

pub(super) async fn verify_mount_capacity_after_callbacks(
    board: impl FnOnce(Arc<dyn EntityBase>, &Arc<crate::entity::player::Player>) -> bool,
) -> TestResult {
    use crate::plugin::api::events::entity::entity_mount::EntityMountEvent;

    let directory = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&directory).await?;
    let competitor = PigEntity::new(Entity::new(world.clone(), CENTER, &EntityType::PIG));
    world.spawn_entity(competitor.clone());
    let cart: Arc<dyn EntityBase> = Arc::new(MinecartEntity::new(Entity::new(
        world.clone(),
        CENTER,
        &EntityType::MINECART,
    )));
    world.spawn_entity(cart.clone());
    let (rider, _peer) = java_rider(&server, &world, "jeb_")?;
    server
        .plugin_manager
        .register_blocking::<EntityMountEvent, _>(
            Arc::new(FillLastSeat {
                vehicle: cart.clone(),
                competitor: competitor.clone(),
                original: rider.get_entity().entity_id,
            }),
            EventPriority::Normal,
            None,
        );
    assert!(!cart.get_entity().has_passengers());

    assert!(!board(cart.clone(), &rider));
    assert_mounted(cart.get_entity(), competitor.get_entity());
    assert!(
        !cart
            .get_entity()
            .has_passenger(rider.get_entity().entity_id)
    );
    assert!(!rider.get_entity().has_vehicle());
    world.level.shutdown().await;
    Ok(())
}

enum ExitPacketAction {
    Teleport(Vector3<f64>),
    Relocate(Vector3<f64>),
    Obstruct,
    Cancel,
    Replace,
    ReplaceJavaRelativeX,
}

struct TranslateLegacyVehicleExit;

impl EventRwHandler<crate::plugin::server::packet::PacketSentEvent> for TranslateLegacyVehicleExit {
    fn handle<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut crate::plugin::server::packet::PacketSentEvent,
    ) -> BoxFuture<'a, ()> {
        use pumpkin_data::packet::CURRENT_MC_VERSION;
        use pumpkin_protocol::{
            ClientPacket, ServerPacket, java::client::play::CPlayerPosition,
            packet::MultiVersionJavaPacket,
        };
        use pumpkin_util::version::JavaMinecraftVersion;

        Box::pin(async move {
            if event.packet_id != CPlayerPosition::to_id(CURRENT_MC_VERSION) {
                event.cancelled = true;
                return;
            }
            if let Ok(packet) =
                CPlayerPosition::read(&mut event.payload.as_ref(), &CURRENT_MC_VERSION)
            {
                let mut wire = Vec::new();
                let version = JavaMinecraftVersion::V_1_20_2;
                if packet.write_packet_data(&mut wire, &version).is_ok() {
                    event.packet_id = CPlayerPosition::to_id(version);
                    event.payload = wire.into();
                }
            }
        })
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn translated_java_vehicle_exit_retains_its_pending_confirmation() -> TestResult {
    use pumpkin_protocol::{
        ClientPacket, ServerPacket,
        java::{
            client::play::CPlayerPosition, packet_decoder::TCPNetworkDecoder,
            server::play::SConfirmTeleport,
        },
        packet::MultiVersionJavaPacket,
        ser::NetworkReadExt,
    };
    use pumpkin_util::version::JavaMinecraftVersion;

    let directory = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&directory).await?;
    let (rider, peer) = java_rider(&server, &world, "jeb_")?;
    let client = rider.client.java().ok_or("Missing Java client")?;
    let boat = mounted_java_boat(&world, &rider, CENTER);
    client.player.store(Arc::new(Some(rider.clone())));
    let version = JavaMinecraftVersion::V_1_20_2;
    client.version.store(version);
    server
        .plugin_manager
        .register_blocking::<crate::plugin::server::packet::PacketSentEvent, _>(
            Arc::new(TranslateLegacyVehicleExit),
            EventPriority::Normal,
            None,
        );

    boat.get_entity().remove_passenger(rider.entity_id());
    let mut decoder = TCPNetworkDecoder::new(tokio::io::BufReader::new(peer));
    let outbound =
        tokio::time::timeout(std::time::Duration::from_secs(5), decoder.get_raw_packet()).await??;
    assert_eq!(outbound.id, CPlayerPosition::to_id(version));
    assert_eq!(outbound.payload.len(), 34);
    // The legacy client reads the ID after XYZ, rotation and one byte of flags.
    let teleport_id = (&outbound.payload[33..]).get_var_int()?;
    let destination = rider.get_entity().pos.load();
    let pending = *rider
        .awaiting_teleport
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    let mut acknowledgement = Vec::new();
    SConfirmTeleport {
        teleport_id,
        position: destination,
        yaw: 0.0,
        pitch: 0.0,
    }
    .write_packet_data(&mut acknowledgement, &version)?;
    let confirmation = SConfirmTeleport::read(&mut acknowledgement.as_slice(), &version)?;
    client.handle_confirm_teleport(&rider, &confirmation);
    assert!(
        !client.is_closed(),
        "the client's valid acknowledgement disconnected it"
    );
    assert_eq!(pending, Some((teleport_id, destination)));
    assert_eq!(rider.get_entity().pos.load(), destination);
    assert!(
        rider
            .awaiting_teleport
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .is_none()
    );
    client.close();
    client.player.store(Arc::new(None));
    world.level.shutdown().await;
    Ok(())
}

struct ObserveExitPacket {
    player: uuid::Uuid,
    packet_id: i32,
    action: ExitPacketAction,
    ran: AtomicBool,
}

impl EventRwHandler<crate::plugin::server::packet::PacketSentEvent> for ObserveExitPacket {
    fn handle<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut crate::plugin::server::packet::PacketSentEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            if matches!(
                self.action,
                ExitPacketAction::Relocate(_) | ExitPacketAction::Obstruct
            ) && bedrock_riding_flag(
                event.packet_id,
                &event.payload,
                event.player.entity_id() as u64,
            )
            .is_some()
            {
                return;
            }
            if event.player.gameprofile.id == self.player
                && event.packet_id == self.packet_id
                && !self.ran.swap(true, Ordering::Relaxed)
            {
                match self.action {
                    ExitPacketAction::Teleport(position) => {
                        event.player.request_teleport(position, 20.0, 0.0);
                    }
                    ExitPacketAction::Relocate(position) => {
                        event.player.get_entity().set_pos(position);
                    }
                    ExitPacketAction::Obstruct => {
                        use pumpkin_util::math::position::BlockPos;
                        use pumpkin_world::world::BlockFlags;
                        assert_eq!(event.player.get_entity().pose.load(), EntityPose::Standing);
                        for y in 64..=67 {
                            event.player.world().set_block_state(
                                &BlockPos::new(8, y, 9),
                                Block::STONE.default_state.id,
                                BlockFlags::NOTIFY_ALL,
                            );
                        }
                    }
                    ExitPacketAction::Cancel => event.cancelled = true,
                    ExitPacketAction::Replace | ExitPacketAction::ReplaceJavaRelativeX => {
                        event.cancelled = true;
                        if let Some(client) = event.player.client.bedrock() {
                            use pumpkin_protocol::bedrock::client::move_player::CMovePlayer;

                            let mut packet = event.player.bedrock_move_packet(
                                CMovePlayer::MODE_TELEPORT,
                                false,
                                0,
                            );
                            packet.position.x += 0.5;
                            if let Ok(data) = client.serialize_packet(&packet)
                                && let Some((id, payload)) =
                                    crate::net::bedrock::split_raw_packet(&data)
                            {
                                event.extra_packets.push((i32::from(id), payload));
                            }
                        } else {
                            use pumpkin_data::packet::CURRENT_MC_VERSION;
                            use pumpkin_protocol::{
                                ClientPacket, PositionFlag, ServerPacket,
                                java::client::play::CPlayerPosition,
                            };

                            if let Ok(mut packet) = CPlayerPosition::read(
                                &mut event.payload.as_ref(),
                                &CURRENT_MC_VERSION,
                            ) {
                                if matches!(self.action, ExitPacketAction::ReplaceJavaRelativeX) {
                                    packet.position.x = 0.5;
                                    packet.relatives.push(PositionFlag::X);
                                } else {
                                    packet.position.x += 0.5;
                                }
                                let mut data = Vec::new();
                                if packet
                                    .write_packet_data(&mut data, &CURRENT_MC_VERSION)
                                    .is_ok()
                                {
                                    event.extra_packets.push((event.packet_id, data.into()));
                                }
                            }
                        }
                    }
                }
            }
        })
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn java_vehicle_exit_sends_and_acknowledges_only_the_newer_teleport() -> TestResult {
    java_vehicle_exit_acknowledges_observed_position(ExitPacketAction::Teleport(Vector3::new(
        13.5, 65.0, 13.5,
    )))
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn java_vehicle_exit_acknowledges_a_plugin_replacement_packet() -> TestResult {
    java_vehicle_exit_acknowledges_observed_position(ExitPacketAction::Replace).await?;
    java_vehicle_exit_acknowledges_observed_position(ExitPacketAction::ReplaceJavaRelativeX).await
}

async fn java_vehicle_exit_acknowledges_observed_position(action: ExitPacketAction) -> TestResult {
    use pumpkin_data::packet::CURRENT_MC_VERSION;
    use pumpkin_protocol::{
        ClientPacket, PositionFlag, RawPacket, ServerPacket,
        java::{
            client::play::{CKeepAlive, CPlayerPosition},
            packet_decoder::TCPNetworkDecoder,
            server::play::SConfirmTeleport,
        },
        packet::MultiVersionJavaPacket,
    };
    let directory = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&directory).await?;
    let chunk = ChunkData::empty_sync(0, 0);
    chunk.set_block_absolute_y(8, 63, 9, Block::STONE.default_state.id);
    chunk.set_block_absolute_y(9, 63, 8, Block::STONE.default_state.id);
    world.level.loaded_chunks.insert(Vector2::new(0, 0), chunk);
    let (rider, peer) = java_rider(&server, &world, "jeb_")?;
    let client = rider.client.java().ok_or("Missing Java client")?;
    client.player.store(Arc::new(Some(rider.clone())));
    let boat = mounted_java_boat(&world, &rider, CENTER);
    let relative_x = matches!(action, ExitPacketAction::ReplaceJavaRelativeX);
    if relative_x {
        rider.get_entity().set_rotation(-90.0, 0.0);
    }
    let before_exit = rider.get_entity().pos.load();
    let newer = if let ExitPacketAction::Teleport(destination) = action {
        Some(destination)
    } else {
        None
    };
    let listener = Arc::new(ObserveExitPacket {
        player: rider.gameprofile.id,
        packet_id: CPlayerPosition::to_id(CURRENT_MC_VERSION),
        action,
        ran: AtomicBool::new(false),
    });
    server
        .plugin_manager
        .register_blocking::<crate::plugin::server::packet::PacketSentEvent, _>(
            listener.clone(),
            EventPriority::Normal,
            None,
        );

    boat.get_entity().remove_passenger(rider.entity_id());
    let original_exit = rider.get_entity().pos.load();
    let mut destination = newer.unwrap_or_else(|| original_exit.add_raw(0.5, 0.0, 0.0));
    if relative_x {
        destination.x = before_exit.x + 0.5;
        assert_ne!(before_exit.x, original_exit.x);
    }
    assert!(listener.ran.load(Ordering::Relaxed));
    assert!(!rider.is_passenger());
    assert!(!boat.get_entity().has_passenger(rider.entity_id()));
    assert!(client.try_send_packet(&CKeepAlive::new(73)));
    let mut decoder = TCPNetworkDecoder::new(tokio::io::BufReader::new(peer));
    let observed =
        outbound_packets_before_marker(&mut decoder, CPlayerPosition::to_id(CURRENT_MC_VERSION))
            .await?;
    let [outgoing] = observed.as_slice() else {
        return Err("Expected one winning position packet".into());
    };
    let sent = CPlayerPosition::read(&mut outgoing.payload.as_ref(), &CURRENT_MC_VERSION)?;
    if newer.is_none() {
        assert!(sent.relatives.contains(&PositionFlag::RotateDelta));
    }
    let wire_x = if relative_x { 0.5 } else { destination.x };
    assert_eq!(
        sent.position,
        Vector3::new(wire_x, destination.y, destination.z)
    );
    let pending = || {
        *rider
            .awaiting_teleport
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    };
    assert_eq!(pending(), Some((sent.teleport_id, destination)));
    let mut payload = Vec::new();
    SConfirmTeleport {
        teleport_id: sent.teleport_id,
        position: destination,
        yaw: sent.yaw,
        pitch: sent.pitch,
    }
    .write_packet_data(&mut payload, &CURRENT_MC_VERSION)?;
    rider.inbound_packets.push(RawPacket {
        id: SConfirmTeleport::to_id(CURRENT_MC_VERSION),
        payload: payload.into(),
    });
    World::process_player_packets(&world.players.load());
    assert_eq!(pending(), None);
    assert_eq!(rider.get_entity().pos.load(), destination);
    assert!(!client.is_closed());
    client.close();
    client.player.store(Arc::new(None));
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn java_vehicle_exit_cancelled_position_does_not_block_movement() -> TestResult {
    use pumpkin_data::packet::CURRENT_MC_VERSION;
    use pumpkin_protocol::{
        ClientPacket, RawPacket,
        java::{client::play::CPlayerPosition, server::play::SPlayerPosition},
        packet::MultiVersionJavaPacket,
    };

    let directory = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&directory).await?;
    world
        .level
        .loaded_chunks
        .insert(Vector2::new(0, 0), ChunkData::empty_sync(0, 0));
    let (rider, _peer) = java_rider(&server, &world, "jeb_")?;
    let client = rider.client.java().ok_or("Missing Java client")?;
    client.player.store(Arc::new(Some(rider.clone())));
    let boat = mounted_java_boat(&world, &rider, CENTER);
    let listener = Arc::new(ObserveExitPacket {
        player: rider.gameprofile.id,
        packet_id: CPlayerPosition::to_id(CURRENT_MC_VERSION),
        action: ExitPacketAction::Cancel,
        ran: AtomicBool::new(false),
    });
    server
        .plugin_manager
        .register_blocking::<crate::plugin::server::packet::PacketSentEvent, _>(
            listener.clone(),
            EventPriority::Normal,
            None,
        );

    boat.get_entity().remove_passenger(rider.entity_id());
    assert!(listener.ran.load(Ordering::Relaxed));
    assert!(!rider.get_entity().has_vehicle());
    assert!(
        rider
            .awaiting_teleport
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .is_none()
    );
    let position = rider.get_entity().pos.load().add_raw(0.25, 0.0, 0.0);
    let mut payload = Vec::new();
    SPlayerPosition {
        position,
        collision: 0,
    }
    .write_packet_data(&mut payload, &CURRENT_MC_VERSION)?;
    rider.inbound_packets.push(RawPacket {
        id: SPlayerPosition::to_id(CURRENT_MC_VERSION),
        payload: payload.into(),
    });
    World::process_player_packets(&world.players.load());
    assert_eq!(rider.get_entity().pos.load(), position);
    client.close();
    client.player.store(Arc::new(None));
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn vehicle_exit_preserves_a_relocation_from_outgoing_pose_metadata() -> TestResult {
    vehicle_exit_rechecks_outgoing_pose_metadata(false).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn vehicle_exit_rechecks_clearance_after_outgoing_pose_metadata() -> TestResult {
    vehicle_exit_rechecks_outgoing_pose_metadata(true).await
}

async fn vehicle_exit_rechecks_outgoing_pose_metadata(obstruct: bool) -> TestResult {
    use pumpkin_protocol::{bedrock::client::set_actor_data::CSetActorData, packet::Packet};

    let directory = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&directory).await?;
    let chunk = ChunkData::empty_sync(0, 0);
    for z in 7..=10 {
        chunk.set_block_absolute_y(8, 63, z, Block::STONE.default_state.id);
    }
    world.level.loaded_chunks.insert(Vector2::new(0, 0), chunk);
    let (rider, _packets) = bedrock_rider(&server, &world, "jeb_").await?;
    let client = rider.client.bedrock().ok_or("Missing Bedrock client")?;
    let cart = Arc::new(MinecartEntity::new(Entity::new(
        world.clone(),
        CENTER,
        &EntityType::MINECART,
    )));
    cart.get_entity().set_rotation(-90.0, 0.0);
    cart.get_entity()
        .velocity
        .store(Vector3::new(0.25, 0.0, 0.0));
    world.spawn_entity(cart.clone());
    cart.get_entity().add_passenger(cart.clone(), rider.clone());
    let entity = rider.get_entity();
    entity.set_pose(EntityPose::Crouching);
    let destination = Vector3::new(13.5, 65.0, 13.5);
    let listener = Arc::new(ObserveExitPacket {
        player: rider.gameprofile.id,
        packet_id: CSetActorData::PACKET_ID,
        action: if obstruct {
            ExitPacketAction::Obstruct
        } else {
            ExitPacketAction::Relocate(destination)
        },
        ran: AtomicBool::new(false),
    });
    server
        .plugin_manager
        .register_blocking::<crate::plugin::server::packet::PacketSentEvent, _>(
            listener.clone(),
            EventPriority::Normal,
            None,
        );

    cart.get_entity().remove_passenger(rider.entity_id());
    assert!(listener.ran.load(Ordering::Relaxed));
    assert_eq!(entity.pose.load(), EntityPose::Standing);
    if obstruct {
        assert_eq!(entity.pos.load(), Vector3::new(8.5, 64.0, 7.5));
    } else {
        assert_eq!(entity.pos.load(), destination);
        assert!(
            rider
                .awaiting_teleport
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .is_none()
        );
    }
    assert!(!entity.has_vehicle());
    client.clear_player();
    client.close().await;
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn locked_java_vehicle_keeps_its_position_without_a_rider_teleport() -> TestResult {
    use pumpkin_data::packet::CURRENT_MC_VERSION;
    use pumpkin_protocol::{
        ClientPacket, RawPacket, ServerPacket,
        java::{
            client::play::{CKeepAlive, CMoveVehicle},
            packet_decoder::TCPNetworkDecoder,
            server::play::SPlayerRotation,
        },
        packet::MultiVersionJavaPacket,
    };

    let directory = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&directory).await?;
    let (rider, peer) = java_rider(&server, &world, "Notch")?;
    let boat = mounted_java_boat(&world, &rider, CENTER);
    rider.set_movement_locked(true);
    let mut data = Vec::new();
    SPlayerRotation {
        yaw: 35.0,
        pitch: 0.0,
        ground: false,
        horizontal_collision: false,
    }
    .write_packet_data(&mut data, &CURRENT_MC_VERSION)?;
    rider.inbound_packets.push(RawPacket {
        id: SPlayerRotation::to_id(CURRENT_MC_VERSION),
        payload: data.into(),
    });
    queue_vehicle_position(&rider, CENTER.add_raw(1.0, 0.0, 0.0), 35.0)?;
    World::process_player_packets(&world.players.load());
    assert_eq!(boat.get_entity().pos.load(), CENTER);
    assert!((rider.get_entity().yaw.load() - 35.0).abs() < f32::EPSILON);
    assert!(
        rider
            .awaiting_teleport
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .is_none()
    );
    let client = rider.client.java().ok_or("Missing Java client")?;
    let mut decoder = TCPNetworkDecoder::new(tokio::io::BufReader::new(peer));
    assert!(client.try_send_packet(&CKeepAlive::new(73)));
    let move_id = CMoveVehicle::to_id(CURRENT_MC_VERSION);
    let packets = outbound_packets_before_marker(&mut decoder, move_id).await?;
    let [correction] = packets.as_slice() else {
        return Err("Expected exactly one vehicle correction".into());
    };
    let correction = CMoveVehicle::read(&mut correction.payload.as_ref(), &CURRENT_MC_VERSION)?;
    let corrected = Vector3::new(correction.x, correction.y, correction.z);
    assert_eq!(corrected, CENTER);
    queue_vehicle_position(&rider, corrected, correction.yaw)?;
    queue_vehicle_position(
        &rider,
        corrected.add_raw(0.000_005, 0.0, 0.0),
        correction.yaw,
    )?;
    World::process_player_packets(&world.players.load());
    assert!(client.try_send_packet(&CKeepAlive::new(74)));
    assert!(
        outbound_packets_before_marker(&mut decoder, move_id)
            .await?
            .is_empty()
    );
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn fully_blocked_java_vehicle_proposal_drops_the_last_recorded_crossing() -> TestResult {
    use pumpkin_util::math::position::BlockPos;

    let directory = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&directory).await?;
    let chunk = ChunkData::empty_sync(0, 0);
    let pad = BlockPos::new(7, 64, 8);
    chunk.set_block_absolute_y(7, 63, 8, Block::WATER.default_state.id);
    chunk.set_block_absolute_y(7, 64, 8, Block::LILY_PAD.default_state.id);
    chunk.set_block_absolute_y(12, 64, 8, Block::STONE.default_state.id);
    world.level.loaded_chunks.insert(Vector2::new(0, 0), chunk);
    let (rider, _peer) = java_rider(&server, &world, "Notch")?;
    let start = Vector3::new(4.5, 64.125, 8.5);
    let boat = Arc::new(TestContactVehicle(Entity::new(
        world.clone(),
        start,
        &EntityType::OAK_BOAT,
    )));
    let entity = boat.get_entity();
    world.spawn_entity(boat.clone());
    entity.add_passenger_silently(boat.clone(), rider.clone());
    assert_mounted(entity, rider.get_entity());
    let half_width = entity.bounding_box.load().max.x - start.x;
    let beside_wall = Vector3::new(12.0 - half_width, start.y, start.z);
    World::process_player_packets(&world.players.load());
    queue_vehicle_position(&rider, beside_wall, 25.0)?;
    World::process_player_packets(&world.players.load());
    assert_eq!(entity.pos.load(), beside_wall);
    assert!(entity.vehicle_block_effect_reach() > 0.0);
    queue_vehicle_position(&rider, beside_wall.add_raw(0.5, 0.0, 0.0), 70.0)?;
    World::process_player_packets(&world.players.load());
    assert_eq!(entity.pos.load(), beside_wall);
    assert!(entity.horizontal_collision.load(Ordering::Relaxed));
    assert!((entity.yaw.load() - 70.0).abs() < f32::EPSILON);
    assert_eq!(world.get_block(&pad), &Block::LILY_PAD);
    assert!(entity.apply_recorded_vehicle_block_effects(boat.as_ref(), beside_wall));
    assert_eq!(world.get_block(&pad), &Block::LILY_PAD);
    world.level.shutdown().await;
    Ok(())
}

struct InvertMountFromListener {
    rider: Arc<dyn EntityBase>,
    mount: Arc<dyn EntityBase>,
}

impl EventRwHandler<crate::plugin::api::events::entity::entity_mount::EntityMountEvent>
    for InvertMountFromListener
{
    fn handle<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut crate::plugin::api::events::entity::entity_mount::EntityMountEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            if event.entity_id == self.rider.get_entity().entity_id
                && event.mount_id == self.mount.get_entity().entity_id
            {
                self.rider
                    .get_entity()
                    .add_passenger(self.rider.clone(), self.mount.clone());
            }
        })
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn vehicle_mount_rechecks_ancestry_after_callbacks() -> TestResult {
    use crate::plugin::api::events::entity::entity_mount::EntityMountEvent;

    let directory = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&directory).await?;
    let rider = Arc::new(BoatEntity::new(Entity::new(
        world.clone(),
        CENTER,
        &EntityType::OAK_BOAT,
    )));
    let mount = Arc::new(BoatEntity::new(Entity::new(
        world.clone(),
        CENTER,
        &EntityType::OAK_BOAT,
    )));
    world.spawn_entity(rider.clone());
    world.spawn_entity(mount.clone());
    server
        .plugin_manager
        .register_blocking::<EntityMountEvent, _>(
            Arc::new(InvertMountFromListener {
                rider: rider.clone(),
                mount: mount.clone(),
            }),
            EventPriority::Normal,
            None,
        );

    mount
        .get_entity()
        .add_passenger(mount.clone(), rider.clone());
    assert!(!rider.get_entity().has_vehicle());
    assert!(!mount.get_entity().has_passengers());
    assert_mounted(rider.get_entity(), mount.get_entity());
    world.level.shutdown().await;
    Ok(())
}

struct RemountFromPassengerPacket {
    rider: Arc<crate::entity::player::Player>,
    boat: Arc<BoatEntity>,
    ran: AtomicBool,
    dismounts: Mutex<Vec<(Vector3<f64>, bool)>>,
}

impl EventRwHandler<GenericGameEvent> for RemountFromPassengerPacket {
    fn handle<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut GenericGameEvent,
    ) -> BoxFuture<'a, ()> {
        use pumpkin_data::game_event::GameEvent;

        Box::pin(async move {
            if event.event_key == GameEvent::EntityDismount.name() {
                let attached = self.rider.is_passenger()
                    || self
                        .boat
                        .get_entity()
                        .has_passenger(self.rider.get_entity().entity_id);
                self.dismounts
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .push((event.position, attached));
            }
        })
    }
}

impl EventRwHandler<crate::plugin::server::packet::PacketSentEvent> for RemountFromPassengerPacket {
    fn handle<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut crate::plugin::server::packet::PacketSentEvent,
    ) -> BoxFuture<'a, ()> {
        use pumpkin_data::packet::CURRENT_MC_VERSION;
        use pumpkin_protocol::{
            java::client::play::CSetPassengers, packet::MultiVersionJavaPacket, ser::NetworkReadExt,
        };

        Box::pin(async move {
            let mut data = event.payload.as_ref();
            if event.player.get_entity().entity_id == self.rider.get_entity().entity_id
                && event.packet_id == CSetPassengers::to_id(CURRENT_MC_VERSION)
                && data
                    .get_var_int()
                    .is_ok_and(|id| id.0 == self.boat.get_entity().entity_id)
                && data.get_var_int().is_ok_and(|count| count.0 == 0)
                && !self.ran.swap(true, Ordering::Relaxed)
            {
                self.boat
                    .get_entity()
                    .add_passenger(self.boat.clone(), self.rider.clone());
                self.boat
                    .get_entity()
                    .set_pos(CENTER.add_raw(1.0, 0.0, 0.0));
                event.cancelled = true;
                event
                    .extra_packets
                    .push((event.packet_id, event.payload.clone()));
            }
        })
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn java_vehicle_remount_wins_over_the_older_passenger_list_for_all_recipients() -> TestResult
{
    use crate::plugin::server::packet::PacketSentEvent;
    use pumpkin_data::packet::CURRENT_MC_VERSION;
    use pumpkin_protocol::{
        java::{
            client::play::{CKeepAlive, CSetPassengers},
            packet_decoder::TCPNetworkDecoder,
        },
        packet::MultiVersionJavaPacket,
        ser::NetworkReadExt,
    };

    let directory = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&directory).await?;
    let (rider, rider_peer) = java_rider(&server, &world, "Notch")?;
    let (watcher, watcher_peer) = java_rider(&server, &world, "jeb_")?;
    let rider_client = rider.client.java().ok_or("Missing rider's Java client")?;
    let watcher_client = watcher
        .client
        .java()
        .ok_or("Missing watcher's Java client")?;
    let watched = pumpkin_world::cylindrical_chunk_iterator::Cylindrical::new(
        Vector2::new(0, 0),
        std::num::NonZeroU8::new(4).ok_or("Zero view distance")?,
    );
    rider.watched_section.store(watched);
    watcher.watched_section.store(watched);
    rider_client.player.store(Arc::new(Some(rider.clone())));
    let boat = Arc::new(BoatEntity::new(Entity::new(
        world.clone(),
        CENTER,
        &EntityType::OAK_BOAT,
    )));
    world.spawn_entity(boat.clone());
    boat.get_entity().add_passenger(boat.clone(), rider.clone());
    let listener = Arc::new(RemountFromPassengerPacket {
        rider: rider.clone(),
        boat: boat.clone(),
        ran: AtomicBool::new(false),
        dismounts: Mutex::default(),
    });
    server
        .plugin_manager
        .register_blocking::<PacketSentEvent, _>(listener.clone(), EventPriority::Normal, None);

    server
        .plugin_manager
        .register_blocking::<GenericGameEvent, _>(listener.clone(), EventPriority::Normal, None);

    boat.get_entity()
        .remove_passenger(rider.get_entity().entity_id);
    assert!(listener.ran.load(Ordering::Relaxed));
    assert_eq!(
        listener
            .dismounts
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .as_slice(),
        [(CENTER, false)]
    );
    assert_mounted(boat.get_entity(), rider.get_entity());
    let packet_id = CSetPassengers::to_id(CURRENT_MC_VERSION);
    for (client, peer) in [(rider_client, rider_peer), (watcher_client, watcher_peer)] {
        assert!(client.try_send_packet(&CKeepAlive::new(73)));
        let mut decoder = TCPNetworkDecoder::new(tokio::io::BufReader::new(peer));
        let packets = outbound_packets_before_marker(&mut decoder, packet_id).await?;
        assert!(
            packets.len() >= 2,
            "expected the initial mount and the remount"
        );
        for raw in packets {
            let mut data = raw.payload.as_ref();
            assert_eq!(data.get_var_int()?.0, boat.get_entity().entity_id);
            assert_eq!(
                data.get_var_int()?.0,
                1,
                "a stale passenger list reached the client"
            );
            assert_eq!(data.get_var_int()?.0, rider.get_entity().entity_id);
        }
    }
    assert!(
        rider
            .awaiting_teleport
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .is_some_and(|(_, pending)| pending == rider.get_entity().pos.load())
    );
    rider_client.close();
    rider_client.player.store(Arc::new(None));
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn automatic_vehicle_boarding_rechecks_rider_eligibility() -> TestResult {
    use crate::plugin::api::events::vehicle::vehicle_enter::VehicleEnterEvent;

    struct SneakOnEnter {
        rider: Arc<dyn EntityBase>,
        ran: AtomicBool,
    }

    impl EventRwHandler<VehicleEnterEvent> for SneakOnEnter {
        fn handle<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            _event: &'a mut VehicleEnterEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async {
                if !self.ran.swap(true, Ordering::Relaxed) {
                    self.rider.get_entity().set_sneaking(true);
                }
            })
        }
    }

    for kind in [&EntityType::OAK_BOAT, &EntityType::MINECART] {
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
        let base = Entity::new(world.clone(), CENTER, kind);
        base.set_has_no_gravity(true);
        let vehicle: Arc<dyn EntityBase> = if kind == &EntityType::OAK_BOAT {
            Arc::new(BoatEntity::new(base))
        } else {
            Arc::new(MinecartEntity::new(base))
        };
        let pig = PigEntity::new(Entity::new(
            world.clone(),
            CENTER.add_raw(0.25, 0.0, 0.0),
            &EntityType::PIG,
        ));
        world.spawn_entity(vehicle.clone());
        world.spawn_entity(pig.clone());
        let listener = Arc::new(SneakOnEnter {
            rider: pig.clone(),
            ran: AtomicBool::new(false),
        });
        server
            .plugin_manager
            .register_blocking::<VehicleEnterEvent, _>(
                listener.clone(),
                EventPriority::Normal,
                None,
            );
        let entity = vehicle.get_entity();
        let tick = || {
            entity.set_pos(CENTER);
            if kind == &EntityType::MINECART {
                entity.velocity.store(Vector3::new(0.2, 0.0, 0.0));
            }
            vehicle.tick(vehicle.as_ref(), &server);
        };

        assert!(pig.can_ride(vehicle.as_ref()));
        tick();
        assert!(listener.ran.load(Ordering::Relaxed));
        assert!(pig.get_entity().is_sneaking());
        assert!(
            !vehicle.is_vehicle(),
            "{} linked an ineligible rider",
            kind.resource_name
        );
        assert!(!pig.is_passenger());

        pig.get_entity().set_sneaking(false);
        tick();
        assert_mounted(entity, pig.get_entity());
        world.level.shutdown().await;
    }
    Ok(())
}

struct ObserveVehicleContact;

impl<E: crate::plugin::Payload> EventRwHandler<E> for ObserveVehicleContact {
    fn handle<'a>(&'a self, _server: &'a Arc<Server>, _event: &'a mut E) -> BoxFuture<'a, ()> {
        Box::pin(async {})
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn vehicle_ticks_with_unbounded_actions_request_serial_execution() -> TestResult {
    use super::entity_islands::{EntityReach, resolve};

    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let config = pumpkin_config::world::EntityIslandsConfig::default();
    let runs_parallel = |kind| {
        let cart = MinecartEntity::new(Entity::new(world.clone(), CENTER, kind));
        resolve(vec![EntityReach::of(&cart, &config)], &config).is_some()
    };
    assert!(runs_parallel(&EntityType::HOPPER_MINECART));
    assert!(!runs_parallel(&EntityType::TNT_MINECART));
    assert!(!runs_parallel(&EntityType::COMMAND_BLOCK_MINECART));
    assert!(runs_parallel(&EntityType::MINECART));
    register_vehicle_contact_listener::<
        crate::plugin::api::events::entity::entity_mount::EntityMountEvent,
    >(&server);
    assert!(!runs_parallel(&EntityType::MINECART));
    assert!(runs_parallel(&EntityType::HOPPER_MINECART));
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn vehicle_history_reaches_unbounded_block_break_listeners() -> TestResult {
    exercise_vehicle_history_listener(
        register_vehicle_contact_listener::<BlockBreakEvent>,
        &[&Block::LILY_PAD, &Block::POWDER_SNOW],
    )
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn vehicle_history_reaches_unbounded_block_drop_listeners() -> TestResult {
    exercise_vehicle_history_listener(
        register_vehicle_contact_listener::<BlockDropItemEvent>,
        &[&Block::LILY_PAD],
    )
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn vehicle_history_reaches_unbounded_block_physics_listeners() -> TestResult {
    exercise_vehicle_history_listener(
        register_vehicle_contact_listener::<BlockPhysicsEvent>,
        &[&Block::LILY_PAD, &Block::STONE_PRESSURE_PLATE],
    )
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn vehicle_history_reaches_unbounded_block_redstone_listeners() -> TestResult {
    exercise_vehicle_history_listener(
        register_vehicle_contact_listener::<BlockRedstoneEvent>,
        &[&Block::STONE_PRESSURE_PLATE],
    )
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn vehicle_history_reaches_unbounded_cauldron_listeners() -> TestResult {
    exercise_vehicle_history_listener(
        register_vehicle_contact_listener::<CauldronLevelChangeEvent>,
        &[&Block::WATER_CAULDRON, &Block::POWDER_SNOW_CAULDRON],
    )
    .await
}

fn register_vehicle_contact_listener<E>(server: &Server)
where
    E: Payload + Send + Sync + 'static,
    ObserveVehicleContact: EventRwHandler<E>,
{
    server.plugin_manager.register_blocking::<E, _>(
        Arc::new(ObserveVehicleContact),
        EventPriority::Normal,
        None,
    );
}

async fn exercise_vehicle_history_listener(
    register_listener: fn(&Server),
    blocks: &[&Block],
) -> TestResult {
    use super::entity_islands::{self, BlockWriteCheck, EntityReach, TickReach};
    use pumpkin_util::math::boundingbox::BoundingBox;

    let directory = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&directory).await?;
    let center = ChunkData::empty_sync(0, 0);
    center.set_block_absolute_y(8, 64, 8, Block::POWDER_SNOW.default_state.id);
    for (x, z) in (0..=3).flat_map(|x| (-1..=1).map(move |z| (x, z))) {
        let chunk = if (x, z) == (0, 0) {
            center.clone()
        } else {
            ChunkData::empty_sync(x, z)
        };
        world.level.loaded_chunks.insert(Vector2::new(x, z), chunk);
    }
    let shift = Vector3::new(19.0, 0.0, 0.0);
    let vehicle = TestContactVehicle(Entity::new(
        world.clone(),
        CENTER + shift,
        &EntityType::MINECART,
    ));
    vehicle
        .get_entity()
        .fire_ticks
        .store(100, Ordering::Relaxed);
    let config = pumpkin_config::world::EntityIslandsConfig {
        unaudited_reach: 0.0,
        ..Default::default()
    };
    let needs_serial = |peer_writes_near_vehicle| -> Result<bool, Box<dyn std::error::Error>> {
        let report = EntityReach::of(&vehicle, &config);
        let checks = [
            report.block_write_check(&vehicle),
            BlockWriteCheck::Unconditional,
        ];
        let mut reaches =
            entity_islands::resolve(vec![report], &config).ok_or("Unexpected unresolved reach")?;
        let mut writer = TickReach::conservative(
            &BoundingBox::new_array([128.0, 64.0, 8.0], [129.0, 65.0, 9.0]),
            0.0,
            vec![],
        );
        if peer_writes_near_vehicle {
            writer.block_writes =
                Some(BoundingBox::new_array([27.0, 64.0, 8.0], [28.0, 65.0, 9.0]));
        }
        reaches.push(writer);
        entity_islands::prune_block_writes(&world, &mut reaches, &checks)
            .ok_or("Unexpected contact planning budget")?;
        let nodes = reaches
            .into_iter()
            .enumerate()
            .map(|(i, reach)| (i as i32, reach))
            .collect::<Vec<_>>();
        Ok(entity_islands::partition(&nodes, &[], &[]).is_none())
    };
    assert!(!needs_serial(false)?);
    register_listener(&server);
    assert!(!needs_serial(false)?);
    assert!(needs_serial(true)?);
    vehicle
        .get_entity()
        .record_vehicle_movement(CENTER, shift, shift);
    for block in blocks {
        center.set_block_absolute_y(8, 64, 8, block.default_state.id);
        assert!(needs_serial(false)?, "{block:?} reached the listener");
    }
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn vehicle_destination_compares_all_collision_boxes_of_the_same_block() -> TestResult {
    use pumpkin_util::math::{boundingbox::BoundingBox, position::BlockPos};

    let directory = tempfile::tempdir()?;
    let (_server, world) = vehicle_world(&directory).await?;
    let chunk = ChunkData::empty_sync(0, 0);
    chunk.set_block_absolute_y(8, 64, 8, Block::OAK_STAIRS.default_state.id);
    chunk.set_block_absolute_y(9, 64, 8, Block::OAK_STAIRS.default_state.id);
    world.level.loaded_chunks.insert(Vector2::new(0, 0), chunk);
    let block = BlockPos::new(8, 64, 8);
    let upper = world
        .get_block_state(&block)
        .get_block_collision_shapes_at(&block)
        .map(|shape| shape.at_pos(block))
        .find(|bounds| bounds.min.y > 64.0)
        .ok_or("Expected an upper stair box")?;
    let center = (upper.min + upper.max) * 0.5;
    let original = BoundingBox::new(
        Vector3::new(center.x - 0.1, 64.2, center.z - 0.1),
        Vector3::new(center.x + 0.1, 64.3, center.z + 0.1),
    );
    let boat = BoatEntity::new(Entity::new(world.clone(), CENTER, &EntityType::OAK_BOAT));
    assert!(!upper.intersects(&original));
    assert!(upper.intersects(&original.shift(Vector3::new(0.0, 0.5, 0.0))));
    assert!(!world.has_new_vehicle_collision(&boat, original, Vector3::new(0.0, 0.5, 0.0)));
    assert!(world.has_new_vehicle_collision(&boat, original, Vector3::new(1.0, 0.0, 0.0)));
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn mounted_java_look_packets_do_not_overwrite_accepted_vehicle_movement() -> TestResult {
    use pumpkin_data::packet::CURRENT_MC_VERSION;
    use pumpkin_protocol::{
        ClientPacket, RawPacket,
        java::server::play::{SClientTickEnd, SPlayerRotation},
        packet::MultiVersionJavaPacket,
    };

    let directory = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&directory).await?;
    let (rider, _peer) = java_rider(&server, &world, "Notch")?;
    let boat = mounted_java_boat(&world, &rider, CENTER);
    let queue_look = || -> TestResult {
        let mut payload = Vec::new();
        SPlayerRotation {
            yaw: 20.0,
            pitch: 0.0,
            ground: false,
            horizontal_collision: false,
        }
        .write_packet_data(&mut payload, &CURRENT_MC_VERSION)?;
        rider.inbound_packets.push(RawPacket {
            id: SPlayerRotation::to_id(CURRENT_MC_VERSION),
            payload: payload.into(),
        });
        Ok(())
    };
    let end_tick = || {
        rider.inbound_packets.push(RawPacket {
            id: SClientTickEnd::to_id(CURRENT_MC_VERSION),
            payload: bytes::Bytes::new(),
        });
        World::process_player_packets(&world.players.load());
    };

    let movement = Vector3::new(1.0, 0.0, 0.0);
    for look_first in [true, false] {
        if look_first {
            queue_look()?;
        }
        queue_vehicle_position(&rider, boat.get_entity().pos.load() + movement, 0.0)?;
        if !look_first {
            queue_look()?;
        }
        end_tick();
        assert_eq!(rider.get_entity().movement.load(), movement);
    }
    queue_look()?;
    end_tick();
    assert_eq!(rider.get_entity().movement.load(), Vector3::default());
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn java_vehicle_riding_distance_counts_only_movement_after_boarding() -> TestResult {
    use pumpkin_data::{
        packet::CURRENT_MC_VERSION,
        statistic::{CustomStatistic, StatisticCategory},
    };
    use pumpkin_protocol::{
        ClientPacket, RawPacket, java::server::play::SInteract, packet::MultiVersionJavaPacket,
    };

    let directory = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&directory).await?;
    let chunk = Vector2::new(0, 0);
    world
        .level
        .loaded_chunks
        .insert(chunk, ChunkData::empty_sync(0, 0));
    world
        .forced_chunks
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .insert(chunk);
    let (rider, _peer) = java_rider(&server, &world, "Notch")?;
    let entry = CENTER.add_raw(-1.0, 0.0, 0.0);
    rider.get_entity().set_pos(entry);
    let boat = Arc::new(BoatEntity::new(Entity::new(
        world.clone(),
        CENTER,
        &EntityType::OAK_BOAT,
    )));
    world.spawn_entity(boat.clone());
    assert!(!rider.is_passenger());
    let board = |player: &crate::entity::player::Player, vehicle: &Entity| -> TestResult {
        let interact = SInteract {
            entity_id: vehicle.entity_id.into(),
            r#type: 2.into(),
            target_position: Some(Vector3::default()),
            hand: Some(0.into()),
            sneaking: false,
        };
        let mut data = Vec::new();
        interact.write_packet_data(&mut data, &CURRENT_MC_VERSION)?;
        player.inbound_packets.push(RawPacket {
            id: SInteract::to_id(CURRENT_MC_VERSION),
            payload: data.into(),
        });
        Ok(())
    };
    board(&rider, boat.get_entity())?;

    world.tick(&server);
    assert_mounted(boat.get_entity(), rider.get_entity());
    let distance = ((rider.get_entity().pos.load() - entry).length() as f32 * 100.0).round() as i32;
    assert!(distance > 0, "the first ride tick reached the seat");
    let stat = || {
        rider
            .stats
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(StatisticCategory::Custom, CustomStatistic::BoatOneCm as i32)
    };
    assert_eq!(stat(), 0, "boarding a stationary boat adds no distance");
    world.tick(&server);
    assert_eq!(stat(), 0, "the stationary second tick adds no distance");

    let (cart_rider, _cart_peer) = java_rider(&server, &world, "jeb_")?;
    let cart_start = CENTER.add_raw(4.0, 0.0, 0.0);
    cart_rider
        .get_entity()
        .set_pos(cart_start.add_raw(-1.0, 0.0, 0.0));
    let cart = Arc::new(MinecartEntity::new(Entity::new(
        world.clone(),
        cart_start,
        &EntityType::MINECART,
    )));
    let cart_entity = cart.get_entity();
    cart_entity.velocity.store(Vector3::new(0.2, 0.0, 0.0));
    world.spawn_entity(cart.clone());
    board(&cart_rider, cart_entity)?;
    world.tick(&server);
    assert_mounted(cart_entity, cart_rider.get_entity());
    let cart_distance =
        ((cart_entity.pos.load() - cart_start).length() as f32 * 100.0).round() as i32;
    assert!(cart_distance > 0, "the cart moved after boarding");
    assert_eq!(
        cart_rider
            .stats
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(
                StatisticCategory::Custom,
                CustomStatistic::MinecartOneCm as i32
            ),
        cart_distance,
        "only cart movement after boarding adds distance",
    );
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn java_vehicle_riding_distance_follows_each_rider_once_per_tick() -> TestResult {
    use pumpkin_data::statistic::{CustomStatistic, StatisticCategory};

    let directory = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&directory).await?;
    let (driver, _driver_peer) = java_rider(&server, &world, "Notch")?;
    let (passenger, _passenger_peer) = java_rider(&server, &world, "jeb_")?;
    let (cart_rider, _cart_peer) = java_rider(&server, &world, "deadmau5")?;
    let boat = mounted_java_boat(&world, &driver, CENTER);
    boat.get_entity()
        .add_passenger(boat.clone(), passenger.clone());
    let cart_pos = CENTER.add_raw(0.0, 0.0, 4.0);
    let cart = Arc::new(MinecartEntity::new(Entity::new(
        world.clone(),
        cart_pos,
        &EntityType::MINECART,
    )));
    world.spawn_entity(cart.clone());
    cart.get_entity()
        .add_passenger(cart.clone(), cart_rider.clone());
    crate::entity::ride_tick_riders(boat.as_ref());
    crate::entity::ride_tick_riders(cart.as_ref());

    let players = world.players.load();
    let starts = world.capture_vehicle_rides(&players);
    queue_vehicle_position(&driver, CENTER.add_raw(1.5, 0.0, 0.0), 0.0)?;
    queue_vehicle_position(&driver, CENTER.add_raw(1.0, 0.0, 0.0), 0.0)?;
    World::process_player_packets(&players);
    cart.get_entity()
        .move_entity(cart.as_ref(), Vector3::new(1.0, 0.0, 0.0));
    crate::entity::ride_tick_riders(boat.as_ref());
    crate::entity::ride_tick_riders(cart.as_ref());
    world.record_vehicle_rides(starts);

    for (rider, stat) in [
        (&driver, CustomStatistic::BoatOneCm),
        (&passenger, CustomStatistic::BoatOneCm),
        (&cart_rider, CustomStatistic::MinecartOneCm),
    ] {
        let stats = rider.stats.lock().unwrap_or_else(PoisonError::into_inner);
        assert_eq!(stats.get(StatisticCategory::Custom, stat as i32), 100);
    }
    let starts = world.capture_vehicle_rides(&players);
    queue_vehicle_position(&driver, CENTER.add_raw(2.0, 0.0, 0.0), 0.0)?;
    World::process_player_packets(&players);
    assert_eq!(boat.get_entity().pos.load(), CENTER.add_raw(2.0, 0.0, 0.0));
    boat.get_entity().remove_passenger(driver.entity_id());
    assert!(!driver.get_entity().has_vehicle());
    world.record_vehicle_rides(starts);
    assert_eq!(
        driver.get_stat(StatisticCategory::Custom, CustomStatistic::BoatOneCm as i32),
        200,
    );
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn java_vehicle_speed_factor_uses_the_resolved_endpoint_before_correction() -> TestResult {
    use pumpkin_data::packet::CURRENT_MC_VERSION;
    use pumpkin_protocol::{
        ServerPacket,
        java::{
            client::play::{CKeepAlive, CMoveVehicle},
            packet_decoder::TCPNetworkDecoder,
        },
        packet::MultiVersionJavaPacket,
    };

    let directory = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&directory).await?;
    let chunk = ChunkData::empty_sync(0, 0);
    chunk.set_block_absolute_y(9, 63, 8, Block::SOUL_SAND.default_state.id);
    world
        .level
        .loaded_chunks
        .insert(Vector2::new(0, 0), chunk.clone());
    let (rider, peer) = java_rider(&server, &world, "Notch")?;
    let boat = mounted_java_boat(&world, &rider, CENTER);
    let vehicle = boat.get_entity();
    let velocity = Vector3::new(0.2, 0.1, 0.1);
    let factor = f64::from(Block::SOUL_SAND.get_speed_factor());
    assert!(factor < 1.0);
    let slowed = velocity.multiply(factor, 1.0, factor);

    vehicle.velocity.store(velocity);
    let target = CENTER.add_raw(1.0, 0.0, 0.0);
    queue_vehicle_position(&rider, target, 0.0)?;
    World::process_player_packets(&world.players.load());
    assert_eq!(vehicle.pos.load(), target);
    assert_eq!(vehicle.velocity.load(), slowed);

    vehicle.set_pos(CENTER);
    vehicle.velocity.store(velocity);
    chunk.set_block_absolute_y(10, 64, 8, Block::STONE.default_state.id);
    queue_vehicle_position(&rider, CENTER.add_raw(2.0, 0.0, 0.0), 0.0)?;
    World::process_player_packets(&world.players.load());
    assert_eq!(vehicle.pos.load(), CENTER);
    assert_eq!(vehicle.velocity.load(), slowed);

    vehicle.velocity.store(velocity);
    queue_vehicle_position(&rider, CENTER.add_raw(15.0, 0.0, 0.0), 0.0)?;
    World::process_player_packets(&world.players.load());
    assert_eq!(vehicle.pos.load(), CENTER);
    assert_eq!(vehicle.velocity.load(), velocity);

    let near_boundary = Vector3::new(8.9999, 64.0, 8.3125);
    vehicle.set_pos(near_boundary);
    for x in 8..=9 {
        chunk.set_block_absolute_y(x, 64, 9, Block::STONE.default_state.id);
    }
    queue_vehicle_position(&rider, near_boundary.add_raw(0.0002, 0.0, 0.75), 0.0)?;
    World::process_player_packets(&world.players.load());
    assert_eq!(vehicle.pos.load(), near_boundary);
    assert_eq!(vehicle.velocity.load(), velocity);

    for x in 8..=9 {
        chunk.set_block_absolute_y(x, 64, 9, Block::AIR.default_state.id);
    }
    let free_step = near_boundary.add_raw(0.0002, 0.0, 0.0);
    queue_vehicle_position(&rider, free_step, 0.0)?;
    World::process_player_packets(&world.players.load());
    assert_eq!(vehicle.pos.load(), free_step);
    assert_eq!(vehicle.velocity.load(), slowed);
    let client = rider.client.java().ok_or("Missing Java client")?;
    assert!(client.try_send_packet(&CKeepAlive::new(73)));
    let mut decoder = TCPNetworkDecoder::new(tokio::io::BufReader::new(peer));
    let corrections =
        outbound_packets_before_marker(&mut decoder, CMoveVehicle::to_id(CURRENT_MC_VERSION))
            .await?;
    assert_eq!(corrections.len(), 3);
    for (packet, expected) in corrections.into_iter().zip([CENTER, CENTER, near_boundary]) {
        let correction = CMoveVehicle::read(&mut packet.payload.as_ref(), &CURRENT_MC_VERSION)?;
        assert_eq!(
            Vector3::new(correction.x, correction.y, correction.z),
            expected
        );
    }
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn lava_ignites_both_carts_but_only_destroys_the_unprotected_vehicle() -> TestResult {
    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let chunk = ChunkData::empty_sync(0, 0);
    for (z, lava) in [(8, &Block::LAVA), (11, &Block::LAVA_CAULDRON)] {
        for x in [8, 11] {
            chunk.set_block_absolute_y(x, 63, z, Block::STONE.default_state.id);
            chunk.set_block_absolute_y(x, 64, z, lava.default_state.id);
        }
    }
    world.level.loaded_chunks.insert(Vector2::new(0, 0), chunk);
    let make_cart = |x, y, z, invulnerable| {
        let entity = Entity::new(world.clone(), Vector3::new(x, y, z), &EntityType::MINECART);
        entity.set_invulnerable(invulnerable);
        entity.set_has_no_gravity(true);
        let cart = Arc::new(MinecartEntity::new(entity));
        world.spawn_entity(cart.clone());
        cart
    };
    for (y, z) in [(64.01, 8.5), (64.35, 11.5)] {
        let protected = make_cart(8.5, y, z, true);
        let ordinary = make_cart(11.5, y, z, false);
        assert!(!protected.get_entity().is_on_fire());
        assert!(!ordinary.get_entity().is_removed());
        for cart in [&protected, &ordinary] {
            cart.get_entity()
                .apply_vehicle_destination_block_effects(cart.as_ref());
        }
        assert!(!ordinary.get_entity().is_removed());
        assert_eq!(ordinary.vehicle.damage.load(), 40.0);
        for cart in [&protected, &ordinary] {
            cart.get_entity()
                .apply_vehicle_destination_block_effects(cart.as_ref());
        }
        assert!(
            !protected.get_entity().is_removed(),
            "protected cart at Z {z}"
        );
        assert_eq!(
            protected.get_entity().fire_ticks.load(Ordering::Relaxed),
            300
        );
        assert!(ordinary.get_entity().is_removed());
        for _ in 0..3 {
            protected.tick(protected.as_ref(), &server);
            assert!(
                !protected.get_entity().is_removed(),
                "protected cart at Z {z}"
            );
            assert_eq!(
                protected.vehicle.damage.load(),
                0.0,
                "protected cart at Z {z}"
            );
        }
    }
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn classic_minecart_takes_a_final_lava_hit_only_after_its_first_tick() -> TestResult {
    struct VetoDestinationHits {
        cart: Arc<MinecartEntity>,
        attempts: std::sync::atomic::AtomicUsize,
    }

    impl EntityBase for VetoDestinationHits {
        fn get_entity(&self) -> &Entity {
            self.cart.get_entity()
        }

        fn get_living_entity(&self) -> Option<&crate::entity::living::LivingEntity> {
            None
        }

        fn cast_any(&self) -> &dyn std::any::Any {
            self.cart.as_ref()
        }

        fn damage_with_context(
            &self,
            _caller: &dyn EntityBase,
            amount: f32,
            damage_type: DamageType,
            position: Option<Vector3<f64>>,
            source: Option<&dyn EntityBase>,
            cause: Option<&dyn EntityBase>,
        ) -> bool {
            assert_eq!(damage_type, DamageType::LAVA);
            assert_eq!(amount, 4.0);
            self.attempts.fetch_add(1, Ordering::Relaxed) >= 4
                && self.cart.damage_with_context(
                    self.cart.as_ref(),
                    amount,
                    damage_type,
                    position,
                    source,
                    cause,
                )
        }
    }

    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let chunk = ChunkData::empty_sync(0, 0);
    chunk.set_block_absolute_y(8, 63, 8, Block::STONE.default_state.id);
    chunk.set_block_absolute_y(8, 64, 8, Block::LAVA.default_state.id);
    world.level.loaded_chunks.insert(Vector2::new(0, 0), chunk);
    let entity = Entity::new(
        world.clone(),
        CENTER.add_raw(0.0, 0.01, 0.0),
        &EntityType::MINECART,
    );
    entity.set_has_no_gravity(true);
    let cart = Arc::new(MinecartEntity::new(entity));
    world.spawn_entity(cart.clone());
    let caller = VetoDestinationHits {
        cart: cart.clone(),
        attempts: std::sync::atomic::AtomicUsize::new(0),
    };
    let mut fresh = pumpkin_nbt::compound::NbtCompound::new();
    cart.write_custom_nbt(&mut fresh);
    assert_eq!(fresh.get_bool("HasTicked"), Some(true));
    assert_eq!(cart.vehicle.damage.load(), 0.0);
    cart.tick(&caller, &server);
    assert_eq!(caller.attempts.load(Ordering::Relaxed), 2);
    assert_eq!(cart.vehicle.damage.load(), 0.0);
    cart.tick(&caller, &server);
    assert_eq!(caller.attempts.load(Ordering::Relaxed), 5);
    assert_eq!(cart.vehicle.damage.load(), 40.0);
    assert!(!cart.get_entity().is_removed());
    let mut saved = pumpkin_nbt::compound::NbtCompound::new();
    cart.write_custom_nbt(&mut saved);
    assert_eq!(saved.get_bool("HasTicked"), Some(false));
    cart.read_custom_nbt(&fresh);
    assert!(cart.get_entity().first_tick.load(Ordering::Relaxed));
    cart.read_custom_nbt(&pumpkin_nbt::compound::NbtCompound::new());
    assert!(!cart.get_entity().first_tick.load(Ordering::Relaxed));
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn powder_snow_advances_freezing_once_per_vehicle_effect_pass() -> TestResult {
    let folder = tempfile::tempdir()?;
    let (_server, world) = vehicle_world(&folder).await?;
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

    for expected in [6, 7] {
        cart.get_entity()
            .apply_vehicle_destination_block_effects(cart.as_ref());
        assert_eq!(cart.get_entity().get_frozen_ticks(), expected);
    }
    let mut saved = pumpkin_nbt::compound::NbtCompound::new();
    saved.put_int("TicksFrozen", Entity::MAX_FROZEN_TICKS + 60);
    cart.get_entity().read_nbt_non_mut(&saved);
    assert_eq!(
        cart.get_entity().get_frozen_ticks(),
        Entity::MAX_FROZEN_TICKS + 60
    );
    cart.get_entity()
        .apply_vehicle_destination_block_effects(cart.as_ref());
    assert_eq!(
        cart.get_entity().get_frozen_ticks(),
        Entity::MAX_FROZEN_TICKS
    );
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn vehicle_cauldron_uses_fire_ignited_in_the_same_contact_step() -> TestResult {
    use pumpkin_data::block_properties::WaterCauldronLikeProperties;
    use pumpkin_util::math::position::BlockPos;

    let folder = tempfile::tempdir()?;
    let (_server, world) = vehicle_world(&folder).await?;
    let chunk = ChunkData::empty_sync(0, 0);
    let mut full = WaterCauldronLikeProperties::default(&Block::WATER_CAULDRON);
    full.level = 3;
    for (z, cauldron) in [
        (4, &Block::WATER_CAULDRON),
        (9, &Block::POWDER_SNOW_CAULDRON),
    ] {
        for x in [8, 9, 12] {
            chunk.set_block_absolute_y(x, 63, z, Block::STONE.default_state.id);
        }
        chunk.set_block_absolute_y(8, 64, z, full.to_state_id(cauldron));
        chunk.set_block_absolute_y(9, 64, z, Block::FIRE.default_state.id);
        chunk.set_block_absolute_y(12, 64, z, full.to_state_id(cauldron));
    }
    world.level.loaded_chunks.insert(Vector2::new(0, 0), chunk);
    for (z, cauldron) in [
        (4, &Block::WATER_CAULDRON),
        (9, &Block::POWDER_SNOW_CAULDRON),
    ] {
        world.level_info.rcu(|current| {
            let mut updated = (**current).clone();
            updated.game_rules.mob_griefing = cauldron == &Block::WATER_CAULDRON;
            updated
        });
        for (cart_x, cauldron_x, expected_level) in [(9.0, 8, 2), (12.5, 12, 3)] {
            let entity = Entity::new(
                world.clone(),
                Vector3::new(cart_x, 64.35, f64::from(z) + 0.5),
                &EntityType::MINECART,
            );
            entity.set_invulnerable(true);
            entity.fire_ticks.store(0, Ordering::Relaxed);
            let cart = Arc::new(MinecartEntity::new(entity));
            world.spawn_entity(cart.clone());
            let pos = BlockPos::new(cauldron_x, 64, z);
            assert_eq!(world.get_block_state_id(&pos), full.to_state_id(cauldron));
            assert_eq!(cart.get_entity().fire_ticks.load(Ordering::Relaxed), 0);
            cart.get_entity()
                .apply_vehicle_destination_block_effects(cart.as_ref());
            assert_eq!(cart.get_entity().fire_ticks.load(Ordering::Relaxed), 0);
            let expected_block = if expected_level == 2 {
                &Block::WATER_CAULDRON
            } else {
                cauldron
            };
            let mut expected = WaterCauldronLikeProperties::default(expected_block);
            expected.level = expected_level;
            assert_eq!(
                world.get_block_state_id(&pos),
                expected.to_state_id(expected_block)
            );
        }
    }
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn fire_clears_freezing_after_powder_snow_in_the_same_vehicle_contact_step() -> TestResult {
    use pumpkin_util::math::position::BlockPos;

    let folder = tempfile::tempdir()?;
    let (_server, world) = vehicle_world(&folder).await?;
    let chunk = ChunkData::empty_sync(0, 0);
    for x in [8, 9] {
        chunk.set_block_absolute_y(x, 63, 4, Block::STONE.default_state.id);
    }
    chunk.set_block_absolute_y(8, 64, 4, Block::POWDER_SNOW.default_state.id);
    chunk.set_block_absolute_y(9, 64, 4, Block::FIRE.default_state.id);
    world.level.loaded_chunks.insert(Vector2::new(0, 0), chunk);
    let snow = BlockPos::new(8, 64, 4);
    assert_eq!(world.get_block(&snow), &Block::POWDER_SNOW);
    let entity = Entity::new(
        world.clone(),
        Vector3::new(9.0, 64.01, 4.5),
        &EntityType::MINECART,
    );
    entity.set_invulnerable(true);
    entity.set_frozen_ticks(5);
    let cart = Arc::new(MinecartEntity::new(entity));
    world.spawn_entity(cart.clone());
    cart.get_entity()
        .apply_vehicle_destination_block_effects(cart.as_ref());
    assert_eq!(cart.get_entity().get_frozen_ticks(), 0);
    // The same contact step ignites the cart before powder snow melts and extinguishes it.
    assert_eq!(cart.get_entity().fire_ticks.load(Ordering::Relaxed), 0);
    assert_eq!(world.get_block(&snow), &Block::AIR);
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn java_vehicle_uses_the_shrinking_borders_tick_extent() -> TestResult {
    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let (rider, _peer) = java_rider(&server, &world, "Notch")?;
    let start = Vector3::new(7.05, 64.01, 4.5);
    let boat = mounted_java_boat(&world, &rider, start);
    let current_diameter = {
        let mut border = world
            .worldborder
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        border.set_diameter(&world, 32.0, None);
        border.set_diameter(&world, 16.0, Some(100));
        border.tick();
        border.current_diameter()
    };
    assert_eq!(current_diameter, 24.0);
    let target = start.add_raw(5.5, 0.0, 0.0);
    queue_vehicle_position(&rider, target, 60.0)?;
    World::process_player_packets(&world.players.load());
    assert_eq!(boat.get_entity().pos.load(), target);

    boat.get_entity().set_pos(start);
    world
        .worldborder
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .tick();
    queue_vehicle_position(&rider, start.add_raw(2.45, 0.0, 0.0), 60.0)?;
    World::process_player_packets(&world.players.load());
    assert_eq!(boat.get_entity().pos.load(), start);
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn java_vehicle_moved_wrongly_detects_a_fence_rooted_below_the_starting_hull() -> TestResult {
    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let chunk = ChunkData::empty_sync(0, 0);
    chunk.set_block_absolute_y(4, 63, 10, Block::OAK_FENCE.default_state.id);
    for z in [7, 10] {
        for y in [64, 65] {
            chunk.set_block_absolute_y(6, y, z, Block::STONE.default_state.id);
        }
    }
    world.level.loaded_chunks.insert(Vector2::new(0, 0), chunk);
    for (z, name, expected) in [(7.5, "Notch", 4.55), (10.5, "jeb_", 8.5)] {
        let (rider, _peer) = java_rider(&server, &world, name)?;
        let start = Vector3::new(4.5, 64.1, z);
        let boat = mounted_java_boat(&world, &rider, start);
        let first_target = start.add_raw(0.05, 0.0, 0.0);
        queue_vehicle_position(&rider, first_target, 35.0)?;
        World::process_player_packets(&world.players.load());
        assert_eq!(boat.get_entity().pos.load(), first_target);

        queue_vehicle_position(&rider, Vector3::new(8.5, start.y, z), 75.0)?;
        World::process_player_packets(&world.players.load());
        assert_eq!(
            boat.get_entity().pos.load(),
            Vector3::new(expected, start.y, z)
        );
    }
    world.level.shutdown().await;
    Ok(())
}

struct ReplaceVehicleFromCorrection {
    rider: Arc<crate::entity::player::Player>,
    previous: Arc<BoatEntity>,
    replacement: Arc<BoatEntity>,
    ran: AtomicBool,
}

impl EventRwHandler<crate::plugin::server::packet::PacketSentEvent>
    for ReplaceVehicleFromCorrection
{
    fn handle<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut crate::plugin::server::packet::PacketSentEvent,
    ) -> BoxFuture<'a, ()> {
        use pumpkin_data::packet::CURRENT_MC_VERSION;
        use pumpkin_protocol::{
            java::client::play::{CKeepAlive, CMoveVehicle},
            packet::MultiVersionJavaPacket,
        };

        Box::pin(async move {
            let rider_id = self.rider.get_entity().entity_id;
            if event.player.get_entity().entity_id == rider_id
                && event.packet_id == CMoveVehicle::to_id(CURRENT_MC_VERSION)
                && !self.ran.swap(true, Ordering::Relaxed)
            {
                self.previous.get_entity().remove_passenger_sync(rider_id);
                self.replacement
                    .get_entity()
                    .add_passenger(self.replacement.clone(), self.rider.clone());
                event.cancelled = true;
                event
                    .extra_packets
                    .push((event.packet_id, event.payload.clone()));
                event.extra_packets.push((
                    CKeepAlive::to_id(CURRENT_MC_VERSION),
                    bytes::Bytes::copy_from_slice(&75i64.to_be_bytes()),
                ));
            }
        })
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn java_vehicle_correction_cannot_reach_the_replacement_mount() -> TestResult {
    use pumpkin_data::packet::CURRENT_MC_VERSION;
    use pumpkin_protocol::{
        java::{
            client::play::{CKeepAlive, CMoveVehicle, CSetPassengers},
            packet_decoder::TCPNetworkDecoder,
        },
        packet::MultiVersionJavaPacket,
        ser::NetworkReadExt,
    };

    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let (rider, peer) = java_rider(&server, &world, "Notch")?;
    let client = rider.client.java().ok_or("Missing Java client")?;
    rider
        .watched_section
        .store(pumpkin_world::cylindrical_chunk_iterator::Cylindrical::new(
            Vector2::new(0, 0),
            std::num::NonZeroU8::new(4).ok_or("Zero view distance")?,
        ));
    client.player.store(Arc::new(Some(rider.clone())));
    let previous = mounted_java_boat(&world, &rider, CENTER);
    let replacement_pos = CENTER.add_raw(4.0, 0.0, 0.0);
    let replacement = Arc::new(BoatEntity::new(Entity::new(
        world.clone(),
        replacement_pos,
        &EntityType::OAK_BOAT,
    )));
    world.spawn_entity(replacement.clone());
    let listener = Arc::new(ReplaceVehicleFromCorrection {
        rider: rider.clone(),
        previous,
        replacement: replacement.clone(),
        ran: AtomicBool::new(false),
    });
    server
        .plugin_manager
        .register_blocking::<crate::plugin::server::packet::PacketSentEvent, _>(
            listener.clone(),
            EventPriority::Normal,
            None,
        );
    rider.set_movement_locked(true);
    queue_vehicle_position(&rider, CENTER.add_raw(0.5, 0.0, 0.0), 0.0)?;
    World::process_player_packets(&world.players.load());
    assert!(listener.ran.load(Ordering::Relaxed));
    assert_mounted(replacement.get_entity(), rider.get_entity());
    assert_eq!(replacement.get_entity().pos.load(), replacement_pos);
    assert!(client.try_send_packet(&CKeepAlive::new(74)));
    let mut decoder = TCPNetworkDecoder::new(tokio::io::BufReader::new(peer));
    let mut attached = false;
    let mut unrelated_extra = false;
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        loop {
            let packet = decoder.get_raw_packet().await?;
            if packet.id == CKeepAlive::to_id(CURRENT_MC_VERSION) {
                match packet.payload.as_ref().get_i64_be()? {
                    74 => break,
                    75 => unrelated_extra = true,
                    _ => {}
                }
            }
            assert_ne!(packet.id, CMoveVehicle::to_id(CURRENT_MC_VERSION));
            if packet.id == CSetPassengers::to_id(CURRENT_MC_VERSION) {
                let mut payload = packet.payload.as_ref();
                if payload.get_var_int()?.0 == replacement.get_entity().entity_id {
                    assert_eq!(payload.get_var_int()?.0, 1);
                    assert_eq!(payload.get_var_int()?.0, rider.get_entity().entity_id);
                    attached = true;
                }
            }
        }
        Ok::<(), Box<dyn std::error::Error>>(())
    })
    .await??;
    assert!(attached, "the replacement attachment reached the client");
    assert!(
        unrelated_extra,
        "the observer's unrelated extra reached the client"
    );
    client.close();
    client.player.store(Arc::new(None));
    world.level.shutdown().await;
    Ok(())
}

pub(super) type BedrockQueue =
    tokio::sync::mpsc::UnboundedReceiver<crate::net::bedrock::OutgoingPacket>;

pub(super) async fn bedrock_rider(
    server: &Arc<Server>,
    world: &Arc<World>,
    name: &str,
) -> Result<(Arc<crate::entity::player::Player>, BedrockQueue), Box<dyn std::error::Error>> {
    use crate::entity::player::Player;
    use crate::net::{ClientPlatform, GameProfile, PlayerConfig, bedrock::BedrockClient};
    use arc_swap::ArcSwap;
    use pumpkin_util::GameMode;

    let (client, packets) = BedrockClient::test_outgoing_queue(server).await?;
    let player = Arc::new(Player::new(
        Arc::new(ClientPlatform::Bedrock(client.clone())),
        GameProfile {
            id: uuid::Uuid::new_v4(),
            name: name.into(),
            properties: ArcSwap::from_pointee(vec![]),
            profile_actions: None,
        },
        PlayerConfig::default(),
        world,
        GameMode::Survival,
    ));
    player.get_entity().set_pos(CENTER);
    player
        .watched_section
        .store(pumpkin_world::cylindrical_chunk_iterator::Cylindrical::new(
            Vector2::new(0, 0),
            std::num::NonZeroU8::new(4).ok_or("Zero view distance")?,
        ));
    client.set_player(player.clone());
    player.set_client_loaded(true);
    world.add_player(&player)?;
    Ok((player, packets))
}

pub(super) fn bedrock_links(
    packets: &mut BedrockQueue,
    vehicle_id: i32,
) -> Result<Vec<(i64, u8)>, Box<dyn std::error::Error>> {
    use pumpkin_protocol::{
        bedrock::client::CSetActorLink, codec::var_long::VarLong, packet::Packet,
        serial::PacketRead,
    };

    let mut links = Vec::new();
    while let Ok(packet) = packets.try_recv() {
        if let Some((id, payload)) = crate::net::bedrock::split_raw_packet(&packet.data)
            && i32::from(id) == CSetActorLink::PACKET_ID
        {
            let mut data = payload.as_ref();
            if VarLong::read(&mut data)?.0 == i64::from(vehicle_id) {
                links.push((VarLong::read(&mut data)?.0, u8::read(&mut data)?));
            }
        }
    }
    Ok(links)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bedrock_vehicle_updates_the_remaining_riders_role_for_existing_watchers() -> TestResult {
    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let (watcher, mut packets) = bedrock_rider(&server, &world, "Notch").await?;
    let boat: Arc<dyn EntityBase> = Arc::new(BoatEntity::new(Entity::new(
        world.clone(),
        CENTER,
        &EntityType::OAK_BOAT,
    )));
    world.spawn_entity(boat.clone());
    let front = mount_pig(&world, &boat);
    let back = mount_pig(&world, &boat);
    let vehicle = boat.get_entity();
    let front_id = front.get_entity().entity_id;
    let back_id = i64::from(back.get_entity().entity_id);
    assert_eq!(
        bedrock_links(&mut packets, vehicle.entity_id)?,
        [(i64::from(front_id), 1), (back_id, 2)]
    );

    vehicle.remove_passenger(front_id);
    assert_eq!(
        bedrock_links(&mut packets, vehicle.entity_id)?,
        [(i64::from(front_id), 0), (back_id, 1)]
    );
    let (fresh, mut fresh_packets) = bedrock_rider(&server, &world, "jeb_").await?;
    let tracked = world
        .entity_tracker
        .get_tracked_entity(vehicle.entity_id)
        .ok_or("Boat is not tracked")?;
    tracked.add_pairing(&fresh);
    assert!(bedrock_links(&mut fresh_packets, vehicle.entity_id)?.is_empty());
    let tracked_rider = world
        .entity_tracker
        .get_tracked_entity(back.get_entity().entity_id)
        .ok_or("Remaining rider is not tracked")?;
    tracked_rider.add_pairing(&fresh);
    assert_eq!(
        bedrock_links(&mut fresh_packets, vehicle.entity_id)?,
        [(back_id, 1)]
    );
    for player in [&watcher, &fresh] {
        let client = player.client.bedrock().ok_or("Missing Bedrock client")?;
        client.clear_player();
        client.close().await;
    }
    world.level.shutdown().await;
    Ok(())
}

fn bedrock_riding_flag(packet_id: i32, payload: &[u8], rider_id: u64) -> Option<bool> {
    use pumpkin_protocol::{
        bedrock::client::set_actor_data::{CSetActorData, entity_data_key},
        codec::{var_long::VarLong, var_uint::VarUInt, var_ulong::VarULong},
        packet::Packet,
        serial::PacketRead,
    };

    let mut data = payload;
    if packet_id != CSetActorData::PACKET_ID || VarULong::read(&mut data).ok()?.0 != rider_id {
        return None;
    }
    let mut riding = None;
    for _ in 0..VarUInt::read(&mut data).ok()?.0 {
        let key = VarUInt::read(&mut data).ok()?.0;
        if VarUInt::read(&mut data).ok()?.0 != 7 || u8::read(&mut data).ok()? != 7 {
            return None;
        }
        let word = VarLong::read(&mut data).ok()?.0;
        match key {
            entity_data_key::FLAGS => riding = Some(word & (1i64 << entity_data_flag::RIDING) != 0),
            entity_data_key::FLAGS_TWO => {}
            _ => return None,
        }
    }
    riding
}

#[derive(Clone, Copy)]
enum BedrockAttachmentUpdate {
    Flags,
    Link,
}

struct RemountFromBedrockAttachment {
    boat: Arc<BoatEntity>,
    rider: Arc<crate::entity::player::Player>,
    update: BedrockAttachmentUpdate,
    unrelated: (i32, bytes::Bytes),
    ran: AtomicBool,
}

impl EventRwHandler<crate::plugin::server::packet::PacketSentEvent>
    for RemountFromBedrockAttachment
{
    fn handle<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut crate::plugin::server::packet::PacketSentEvent,
    ) -> BoxFuture<'a, ()> {
        use pumpkin_protocol::{
            bedrock::client::CSetActorLink, codec::var_long::VarLong, packet::Packet,
            serial::PacketRead,
        };

        Box::pin(async move {
            let rider_id = self.rider.entity_id();
            let vehicle_id = self.boat.get_entity().entity_id;
            let mut payload = event.payload.as_ref();
            let is_detach = match self.update {
                BedrockAttachmentUpdate::Flags => {
                    bedrock_riding_flag(event.packet_id, payload, rider_id as u64) == Some(false)
                }
                BedrockAttachmentUpdate::Link => {
                    event.packet_id == CSetActorLink::PACKET_ID
                        && VarLong::read(&mut payload).is_ok_and(|id| id.0 == i64::from(vehicle_id))
                        && VarLong::read(&mut payload).is_ok_and(|id| id.0 == i64::from(rider_id))
                        && u8::read(&mut payload).is_ok_and(|role| role == 0)
                }
            };
            if is_detach && !self.ran.swap(true, Ordering::Relaxed) {
                self.boat
                    .get_entity()
                    .add_passenger(self.boat.clone(), self.rider.clone());
                event.cancelled = true;
                event
                    .extra_packets
                    .push((event.packet_id, event.payload.clone()));
                event.extra_packets.push(self.unrelated.clone());
            }
        })
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bedrock_vehicle_remount_wins_over_older_riding_metadata() -> TestResult {
    bedrock_remount_suppresses_replayed_attachment(BedrockAttachmentUpdate::Flags).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bedrock_vehicle_remount_wins_over_replayed_unlink() -> TestResult {
    bedrock_remount_suppresses_replayed_attachment(BedrockAttachmentUpdate::Link).await
}

async fn bedrock_remount_suppresses_replayed_attachment(
    update: BedrockAttachmentUpdate,
) -> TestResult {
    use pumpkin_protocol::{
        bedrock::client::{
            CSetActorLink,
            set_actor_data::{
                CSetActorData, MetadataValue, PropertySyncData, SyncedActorDataList,
                entity_data_key,
            },
        },
        codec::{var_long::VarLong, var_ulong::VarULong},
        packet::Packet,
        serial::PacketRead,
    };

    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let (rider, mut packets) = bedrock_rider(&server, &world, "Notch").await?;
    let client = rider.client.bedrock().ok_or("Missing Bedrock client")?;
    let boat = Arc::new(BoatEntity::new(Entity::new(
        world.clone(),
        CENTER,
        &EntityType::OAK_BOAT,
    )));
    world.spawn_entity(boat.clone());
    boat.get_entity().add_passenger(boat.clone(), rider.clone());
    while packets.try_recv().is_ok() {}
    let mut metadata = SyncedActorDataList::new();
    metadata.set(entity_data_key::NAME, MetadataValue::String("Rider".into()));
    let extra = client.serialize_packet(&CSetActorData {
        target_runtime_id: VarULong(rider.entity_id() as u64),
        actor_data: metadata,
        synced_properties: PropertySyncData::default(),
        tick: VarULong(0),
    })?;
    let (extra_id, extra_payload) =
        crate::net::bedrock::split_raw_packet(&extra).ok_or("Missing extra packet ID")?;
    let listener = Arc::new(RemountFromBedrockAttachment {
        boat: boat.clone(),
        rider: rider.clone(),
        update,
        unrelated: (i32::from(extra_id), extra_payload),
        ran: AtomicBool::new(false),
    });
    server
        .plugin_manager
        .register_blocking::<crate::plugin::server::packet::PacketSentEvent, _>(
            listener.clone(),
            EventPriority::Normal,
            None,
        );

    boat.get_entity().remove_passenger(rider.entity_id());
    assert!(listener.ran.load(Ordering::Relaxed));
    assert_mounted(boat.get_entity(), rider.get_entity());
    let mut attachment = Vec::new();
    let mut saw_unrelated = false;
    while let Ok(packet) = packets.try_recv() {
        saw_unrelated |= packet.data == extra;
        let Some((id, data)) = crate::net::bedrock::split_raw_packet(&packet.data) else {
            continue;
        };
        match update {
            BedrockAttachmentUpdate::Flags => {
                if let Some(flag) =
                    bedrock_riding_flag(i32::from(id), &data, rider.entity_id() as u64)
                {
                    attachment.push(u8::from(flag));
                }
            }
            BedrockAttachmentUpdate::Link if i32::from(id) == CSetActorLink::PACKET_ID => {
                let mut data = data.as_ref();
                if VarLong::read(&mut data)?.0 == i64::from(boat.get_entity().entity_id)
                    && VarLong::read(&mut data)?.0 == i64::from(rider.entity_id())
                {
                    attachment.push(u8::read(&mut data)?);
                }
            }
            BedrockAttachmentUpdate::Link => {}
        }
    }
    assert!(saw_unrelated, "the name update must reach the client");
    assert_eq!(
        attachment,
        [1],
        "a stale attachment update reached the client"
    );
    client.clear_player();
    client.close().await;
    world.level.shutdown().await;
    Ok(())
}

fn bedrock_exit_positions(
    packets: &mut BedrockQueue,
    rider_id: i32,
) -> Result<Vec<Vector3<f32>>, Box<dyn std::error::Error>> {
    use pumpkin_protocol::{
        bedrock::client::move_player::CMovePlayer, codec::var_ulong::VarULong, packet::Packet,
        serial::PacketRead,
    };

    let mut positions = Vec::new();
    while let Ok(packet) = packets.try_recv() {
        if let Some((id, payload)) = crate::net::bedrock::split_raw_packet(&packet.data)
            && i32::from(id) == CMovePlayer::PACKET_ID
        {
            let mut payload = payload.as_ref();
            assert_eq!(VarULong::read(&mut payload)?.0, rider_id as u64);
            let position = Vector3::<f32>::read(&mut payload)?;
            for _ in 0..3 {
                f32::read(&mut payload)?;
            }
            if u8::read(&mut payload)? == CMovePlayer::MODE_TELEPORT {
                positions.push(position);
            }
        }
    }
    Ok(positions)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bedrock_vehicle_dismount_sends_only_the_newer_teleport() -> TestResult {
    bedrock_vehicle_exit_tracks_the_observed_position(false).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bedrock_vehicle_dismount_tracks_a_plugin_replacement_packet() -> TestResult {
    bedrock_vehicle_exit_tracks_the_observed_position(true).await
}

async fn bedrock_vehicle_exit_tracks_the_observed_position(replace: bool) -> TestResult {
    use pumpkin_protocol::{bedrock::client::move_player::CMovePlayer, packet::Packet};

    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let (rider, mut packets) = bedrock_rider(&server, &world, "Notch").await?;
    let client = rider.client.bedrock().ok_or("Missing Bedrock client")?;
    let boat = Arc::new(BoatEntity::new(Entity::new(
        world.clone(),
        CENTER,
        &EntityType::OAK_BOAT,
    )));
    world.spawn_entity(boat.clone());
    boat.get_entity().add_passenger(boat.clone(), rider.clone());
    while packets.try_recv().is_ok() {}

    let destination = Vector3::new(13.5, 65.0, 13.5);
    let listener = Arc::new(ObserveExitPacket {
        player: rider.gameprofile.id,
        packet_id: CMovePlayer::PACKET_ID,
        action: if replace {
            ExitPacketAction::Replace
        } else {
            ExitPacketAction::Teleport(destination)
        },
        ran: AtomicBool::new(false),
    });
    server
        .plugin_manager
        .register_blocking::<crate::plugin::server::packet::PacketSentEvent, _>(
            listener.clone(),
            EventPriority::Normal,
            None,
        );
    boat.get_entity().remove_passenger(rider.entity_id());
    assert!(listener.ran.load(Ordering::Relaxed));
    assert!(!rider.get_entity().has_vehicle());
    let destination = if replace {
        let position = rider.get_entity().pos.load();
        assert_ne!(position, destination);
        position
    } else {
        assert_eq!(rider.get_entity().pos.load(), destination);
        destination
    };
    let mut target = rider.bedrock_pos().to_f32_lossy();
    if replace {
        target.x += 0.5;
    }
    assert_eq!(
        bedrock_exit_positions(&mut packets, rider.entity_id())?,
        [target]
    );
    assert_eq!(client.teleport_sent_pos.load(), target);
    assert!(
        rider
            .awaiting_teleport
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .as_ref()
            .is_some_and(|(_, position)| *position == destination)
    );

    client.clear_player();
    client.close().await;
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bedrock_vehicle_dismount_acknowledges_the_transmitted_float_position() -> TestResult {
    use pumpkin_protocol::{
        bedrock::server::player_auth_input::SPlayerAuthInput,
        codec::{bitset::Bitset, var_ulong::VarULong},
    };

    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let (rider, mut packets) = bedrock_rider(&server, &world, "Notch").await?;
    let client = rider.client.bedrock().ok_or("Missing Bedrock client")?;
    // At this coordinate adjacent f32 values are farther apart than the acknowledgement tolerance.
    let start = Vector3::new(4_194_304.125, 64.0, 8.5);
    rider.get_entity().set_pos(start);
    let chunk = ChunkData::empty_sync(262_144, 0);
    chunk.set_block_absolute_y(0, 63, 8, Block::STONE.default_state.id);
    world
        .level
        .loaded_chunks
        .insert(Vector2::new(262_144, 0), chunk);
    let cart = Arc::new(MinecartEntity::new(Entity::new(
        world.clone(),
        start,
        &EntityType::MINECART,
    )));
    world.spawn_entity(cart.clone());
    cart.get_entity().add_passenger(cart.clone(), rider.clone());
    while packets.try_recv().is_ok() {}
    cart.get_entity().remove_passenger(rider.entity_id());
    assert!(!rider.get_entity().has_vehicle());

    let [position] = bedrock_exit_positions(&mut packets, rider.entity_id())?
        .try_into()
        .map_err(|_| "Expected one Bedrock dismount teleport")?;
    let pending = || {
        *rider
            .awaiting_teleport
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    };
    let (_, exact) = pending().ok_or("Missing pending dismount")?;
    assert!((f64::from(position.x) - exact.x).abs() > 0.1);
    assert_eq!(client.teleport_sent_pos.load(), position);

    let input = |position| SPlayerAuthInput {
        pitch: 0.0,
        yaw: 0.0,
        position,
        move_vec: Vector2::default(),
        head_yaw: 0.0,
        input_data: Bitset::default(),
        input_mode: 0.into(),
        play_mode: 0.into(),
        interaction_model: 0.into(),
        interact_pitch: 0.0,
        interact_yaw: 0.0,
        tick: VarULong(0),
        delta: Vector3::default(),
        block_actions: None,
        item_interaction: None,
        item_stack_request: None,
        vehicle_rotation: None,
        vehicle_unique_id: None,
        analog_move: Vector2::default(),
        camera_orientation: Vector3::default(),
        raw_move: Vector2::default(),
    };
    client.handle_player_auth_input(
        &rider,
        input(Vector3::new(position.x + 1.0, position.y, position.z)),
        &server,
    );
    assert!(pending().is_some());
    client.handle_player_auth_input(&rider, input(position), &server);
    assert!(pending().is_none());
    client.clear_player();
    client.close().await;
    world.level.shutdown().await;
    Ok(())
}

const SPLASH_REPLACEMENT_MOTION: Vector3<f64> = Vector3::new(-0.25, 0.125, 0.0);

#[derive(Debug, PartialEq)]
enum SplashObservation {
    Sound(f32),
    Splash(Vector3<f64>, bool),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SplashCallback {
    Sound,
    Splash,
}

struct SplashObserver {
    subject: Arc<dyn EntityBase>,
    destination: Option<Vector3<f64>>,
    observations: Mutex<Vec<SplashObservation>>,
    teleport_on: AtomicCell<Option<SplashCallback>>,
}

impl SplashObserver {
    fn register(
        server: &Server,
        subject: Arc<dyn EntityBase>,
        destination: Option<Vector3<f64>>,
    ) -> Arc<Self> {
        let observer = Arc::new(Self {
            subject,
            destination,
            observations: Mutex::default(),
            teleport_on: AtomicCell::new(None),
        });
        server
            .plugin_manager
            .register_blocking::<GenericGameEvent, _>(
                observer.clone(),
                EventPriority::Normal,
                None,
            );
        server
            .plugin_manager
            .register_blocking::<PacketSentEvent, _>(observer.clone(), EventPriority::Normal, None);
        observer
    }

    fn teleport_from(&self, callback: SplashCallback) {
        if self.teleport_on.load() == Some(callback) {
            let entity = self.subject.get_entity();
            entity.velocity.store(SPLASH_REPLACEMENT_MOTION);
            self.subject.teleport(
                self.destination.unwrap_or_else(|| entity.pos.load()),
                None,
                None,
                entity.world.load_full(),
            );
        }
    }

    fn take_observations(&self) -> Vec<SplashObservation> {
        std::mem::take(
            &mut *self
                .observations
                .lock()
                .unwrap_or_else(PoisonError::into_inner),
        )
    }
}

impl EventRwHandler<GenericGameEvent> for SplashObserver {
    fn handle<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut GenericGameEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async {
            if event.event_key != GameEvent::Splash.name() {
                return;
            }
            let entity = self.subject.get_entity();
            assert!(entity.water_height.load() > 0.0);
            self.observations
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push(SplashObservation::Splash(
                    entity.velocity.load(),
                    entity.touching_water.load(Ordering::Relaxed),
                ));
            self.teleport_from(SplashCallback::Splash);
        })
    }
}

impl EventRwHandler<PacketSentEvent> for SplashObserver {
    fn handle<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut PacketSentEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async {
            let mut payload = event.payload.as_ref();
            let mut packed_position = [0; 12];
            if event.packet_id == CSoundEffect::to_id(CURRENT_MC_VERSION)
                && payload
                    .get_var_int()
                    .is_ok_and(|id| id.0 == Sound::EntityGenericSplash as i32 + 1)
                && payload.get_var_int().is_ok()
                && payload.read_bytes_to_buf(&mut packed_position).is_ok()
                && let Ok(volume) = payload.get_f32_be()
            {
                self.observations
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .push(SplashObservation::Sound(volume));
                self.teleport_from(SplashCallback::Sound);
            }
        })
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cart_water_entry_splash_precedes_current_and_respects_teleport() -> TestResult {
    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let chunk = ChunkData::empty_sync(0, 0);
    chunk.set_block_absolute_y(8, 64, 8, Block::WATER.default_state.id);
    // The empty eastern neighbor lets this source pull down toward the lower water.
    chunk.set_block_absolute_y(9, 63, 8, Block::WATER.default_state.id);
    world.level.loaded_chunks.insert(Vector2::new(0, 0), chunk);
    let (watcher, _peer) = java_rider(&server, &world, "Notch")?;
    watcher.get_entity().set_pos(CENTER.add_raw(4.0, 0.0, 0.0));
    let client = watcher.client.java().ok_or("Missing Java watcher")?;
    client.player.store(Arc::new(Some(watcher.clone())));

    let start = CENTER.add_raw(0.0, 1.25, 0.0);
    let cart = Arc::new(MinecartEntity::new(Entity::new(
        world.clone(),
        start,
        &EntityType::MINECART,
    )));
    let entity = cart.get_entity();
    entity.set_has_no_gravity(true);
    let observer = SplashObserver::register(&server, cart.clone(), None);

    let impact_y = -0.55 * f64::from(0.95f32);
    let incoming = Vector3::new(0.0, impact_y, 0.0);
    for teleport_on in [
        None,
        Some(SplashCallback::Splash),
        Some(SplashCallback::Sound),
    ] {
        entity.set_pos(start);
        entity.velocity.store(Vector3::default());
        cart.tick(cart.as_ref(), &server);
        assert!(!entity.first_tick.load(Ordering::Relaxed));
        assert!(!entity.touching_water.load(Ordering::Relaxed));
        assert_eq!(entity.water_height.load().to_bits(), 0.0f64.to_bits());
        observer.teleport_on.store(teleport_on);
        entity.velocity.store(Vector3::new(0.0, -0.55, 0.0));

        cart.tick(cart.as_ref(), &server);

        let events = observer.take_observations();
        let [SplashObservation::Sound(volume), after_sound @ ..] = events.as_slice() else {
            return Err(format!("Expected a splash sound first: {events:?}").into());
        };
        assert!(
            (volume - impact_y.abs() as f32 * 0.2).abs() < 1.0e-7,
            "splash volume {volume}"
        );
        if teleport_on == Some(SplashCallback::Sound) {
            assert!(after_sound.is_empty());
        } else {
            assert_eq!(after_sound, [SplashObservation::Splash(incoming, false)]);
        }
        assert_eq!(
            entity.touching_water.load(Ordering::Relaxed),
            teleport_on.is_none()
        );
        assert_eq!(
            entity.velocity.load(),
            if teleport_on.is_some() {
                SPLASH_REPLACEMENT_MOTION
            } else {
                incoming.add_raw(0.014, 0.0, 0.0)
            }
        );
    }
    client.player.store(Arc::new(None));
    client.close();
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn restored_cart_dispatches_splash_on_its_first_wet_tick() -> TestResult {
    use std::sync::atomic::AtomicUsize;

    use pumpkin_nbt::compound::NbtCompound;

    use crate::entity::r#type::load_entity_from_nbt;

    struct SplashCount(AtomicUsize);

    impl EventRwHandler<GenericGameEvent> for SplashCount {
        fn handle<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut GenericGameEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async {
                if event.event_key == GameEvent::Splash.name() {
                    self.0.fetch_add(1, Ordering::Relaxed);
                }
            })
        }
    }

    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let chunk = ChunkData::empty_sync(0, 0);
    chunk.set_block_absolute_y(8, 64, 8, Block::WATER.default_state.id);
    world.level.loaded_chunks.insert(Vector2::new(0, 0), chunk);
    let observer = Arc::new(SplashCount(AtomicUsize::new(0)));
    server
        .plugin_manager
        .register_blocking::<GenericGameEvent, _>(observer.clone(), EventPriority::Normal, None);

    let position = CENTER.add_raw(0.0, 0.25, 0.0);
    let original = MinecartEntity::new(Entity::new(world.clone(), position, &EntityType::MINECART));
    original.get_entity().set_has_no_gravity(true);
    original.tick(&original, &server);
    assert!(original.get_entity().touching_water.load(Ordering::Relaxed));
    assert_eq!(observer.0.load(Ordering::Relaxed), 0);
    let mut saved = NbtCompound::new();
    original.write_nbt(&mut saved);
    assert_eq!(saved.get_bool("HasTicked"), Some(false));

    for (tag, expected) in [(Some(false), 1), (None, 1), (Some(true), 0)] {
        let mut nbt = saved.clone();
        match tag {
            Some(value) => nbt.put_bool("HasTicked", value),
            None => {
                nbt.child_tags.remove("HasTicked");
            }
        }
        let restored =
            load_entity_from_nbt(&nbt, position, &world).ok_or("Missing saved minecart")?;
        assert_eq!(restored.get_entity().pos.load(), position);
        assert!(!restored.get_entity().touching_water.load(Ordering::Relaxed));
        observer.0.store(0, Ordering::Relaxed);

        restored.tick(restored.as_ref(), &server);
        assert!(restored.get_entity().touching_water.load(Ordering::Relaxed));
        assert_eq!(
            observer.0.load(Ordering::Relaxed),
            expected,
            "HasTicked={tag:?}"
        );
        restored.tick(restored.as_ref(), &server);
        assert_eq!(
            observer.0.load(Ordering::Relaxed),
            expected,
            "second wet tick with HasTicked={tag:?}"
        );
    }
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn pig_water_entry_discards_contact_after_splash_teleport() -> TestResult {
    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let chunk = ChunkData::empty_sync(0, 0);
    chunk.set_block_absolute_y(8, 64, 8, Block::WATER.default_state.id);
    chunk.set_block_absolute_y(9, 63, 8, Block::WATER.default_state.id);
    world.level.loaded_chunks.insert(Vector2::new(0, 0), chunk);
    let (watcher, _peer) = java_rider(&server, &world, "Notch")?;
    watcher.get_entity().set_pos(CENTER.add_raw(4.0, 0.0, 0.0));
    let client = watcher.client.java().ok_or("Missing Java watcher")?;
    client.player.store(Arc::new(Some(watcher.clone())));

    let dry = CENTER.add_raw(0.0, 1.25, 0.0);
    let wet = CENTER.add_raw(0.0, 0.25, 0.0);
    let destination = CENTER.add_raw(-4.0, 0.0, 0.0);
    let incoming = Vector3::new(0.0, -0.5, 0.0);
    let pig = PigEntity::new(Entity::new(world.clone(), dry, &EntityType::PIG));
    let observer = SplashObserver::register(&server, pig.clone(), Some(destination));
    let entity = pig.get_entity();
    for teleport_on in [
        None,
        Some(SplashCallback::Splash),
        Some(SplashCallback::Sound),
    ] {
        entity.set_pos(dry);
        entity.fire_ticks.store(0, Ordering::Relaxed);
        entity.tick(pig.as_ref(), &server);
        assert!(!entity.touching_water.load(Ordering::Relaxed));
        assert!(!entity.first_tick.load(Ordering::Relaxed));

        observer.teleport_on.store(teleport_on);
        entity.set_pos(wet);
        entity.velocity.store(incoming);
        entity.fire_ticks.store(41, Ordering::Relaxed);
        entity.tick(pig.as_ref(), &server);

        let observations = observer.take_observations();
        let [SplashObservation::Sound(_), after_sound @ ..] = observations.as_slice() else {
            return Err(format!("Expected a splash sound first: {observations:?}").into());
        };
        if teleport_on == Some(SplashCallback::Sound) {
            assert!(after_sound.is_empty());
        } else {
            assert_eq!(after_sound, [SplashObservation::Splash(incoming, false)]);
        }
        if teleport_on.is_some() {
            assert_eq!(entity.pos.load(), destination);
            assert_eq!(entity.velocity.load(), SPLASH_REPLACEMENT_MOTION);
            assert!(!entity.touching_water.load(Ordering::Relaxed));
            assert_eq!(entity.water_height.load(), 0.0);
            // The dry destination still burns after the ordinary fire timer advances.
            assert_eq!(entity.fire_ticks.load(Ordering::Relaxed), 40);
        } else {
            assert_eq!(entity.pos.load(), wet);
            assert_eq!(entity.velocity.load(), incoming.add_raw(0.014, 0.0, 0.0));
            assert!(entity.touching_water.load(Ordering::Relaxed));
            assert!(entity.water_height.load() > 0.0);
            assert_eq!(entity.fire_ticks.load(Ordering::Relaxed), 0);
        }
    }
    client.player.store(Arc::new(None));
    client.close();
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[allow(
    clippy::too_many_lines,
    reason = "The three denied clicks and the two swing outcomes share one tracked cart fixture."
)]
async fn java_vehicle_block_use_waits_for_world_and_teleport_eligibility() -> TestResult {
    use pumpkin_data::{
        data_component_impl::{InteractAnimationImpl, SwingAnimationType},
        item::Item,
        item_stack::ItemStack,
        statistic::StatisticCategory,
    };
    use pumpkin_protocol::java::{client::play::CSwingArm, server::play::SUseItemOn};
    use pumpkin_util::{math::position::BlockPos, permission::PermissionLvl};
    use pumpkin_world::cylindrical_chunk_iterator::Cylindrical;
    use std::{num::NonZeroU8, sync::atomic::AtomicUsize};

    struct ObservePlacementSwing(AtomicUsize);
    impl EventRwHandler<PacketSentEvent> for ObservePlacementSwing {
        fn handle<'a>(
            &'a self,
            _: &'a Arc<Server>,
            event: &'a mut PacketSentEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                if event.packet_id == CSwingArm::to_id(CURRENT_MC_VERSION) {
                    assert_eq!(event.player.gameprofile.name, "jeb_");
                    let mut payload = event.payload.as_ref();
                    assert!(payload.get_var_int().is_ok());
                    for expected in [0, 2, 9] {
                        assert_eq!(
                            payload.get_var_int().map(|value| value.0).ok(),
                            Some(expected)
                        );
                    }
                    assert!(payload.is_empty());
                    self.0.fetch_add(1, Ordering::Relaxed);
                }
            })
        }
    }

    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world_with_spawn_protection(&folder, 4).await?;
    world.level_info.rcu(|old| {
        let mut level_info = (**old).clone();
        level_info.spawn_x = 8;
        level_info.spawn_z = 8;
        level_info
    });
    let (placer, _placer_peer) = java_rider(&server, &world, "Notch")?;
    let (watcher, _watcher_peer) = java_rider(&server, &world, "jeb_")?;
    let watcher_client = watcher.client.java().ok_or("Java watcher")?;
    watcher_client.player.store(Arc::new(Some(watcher.clone())));
    let placement_position = Vector3::new(8.5, 65.0, 6.5);
    placer.get_entity().set_pos(placement_position);
    watcher.get_entity().set_pos(Vector3::new(10.0, 65.0, 6.5));
    let chunk_pos = Vector2::new(0, 0);
    watcher.watched_section.store(Cylindrical::new(
        chunk_pos,
        NonZeroU8::new(3).ok_or("watch distance")?,
    ));
    watcher
        .chunk_sender
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .mark_sent_out_of_band(chunk_pos);
    let rail = BlockPos::new(8, 65, 8);
    let chunk = ChunkData::empty_sync(0, 0);
    chunk.set_block_absolute_y(8, 64, 8, Block::STONE.default_state.id);
    chunk.set_block_absolute_y(8, 65, 8, Block::RAIL.default_state.id);
    world.level.loaded_chunks.insert(chunk_pos, chunk);
    let tracked: Arc<dyn EntityBase> = placer.clone();
    world.entity_tracker.add_entity(&tracked, &world);
    let swing = Arc::new(ObservePlacementSwing(AtomicUsize::new(0)));
    server
        .plugin_manager
        .register_blocking::<PacketSentEvent, _>(swing.clone(), EventPriority::Normal, None);
    let mut cart = ItemStack::new(1, &Item::MINECART);
    cart.set_data_component(InteractAnimationImpl {
        animation_type: SwingAnimationType::Stab,
        duration: 9,
    });
    placer.inventory().set_held_item(cart.clone());
    placer.last_attacked_ticks.store(20, Ordering::Relaxed);
    let client = placer.client.java().ok_or("Java placer")?;
    let click = |sequence| {
        client.handle_use_item_on(
            &placer,
            &SUseItemOn {
                hand: 0.into(),
                position: rail,
                face: 1.into(),
                cursor_pos: Vector3::new(0.5, 0.5, 0.5),
                inside_block: false,
                is_against_world_border: false,
                sequence,
            },
            &server,
        )
    };
    let assert_unspent = || {
        assert_eq!(placer.inventory().held_item().uid, cart.uid);
        assert_eq!(
            placer.get_stat(StatisticCategory::Used, i32::from(Item::MINECART.id)),
            0
        );
        assert_eq!(placer.last_attacked_ticks.load(Ordering::Relaxed), 20);
        assert_eq!(swing.0.load(Ordering::Relaxed), 0);
    };

    placer.permission_lvl.store(PermissionLvl::Zero);
    server
        .data
        .operator_config
        .write()
        .unwrap_or_else(PoisonError::into_inner)
        .ops
        .push(pumpkin_config::op::Op::new(
            watcher.gameprofile.id,
            "jeb_".into(),
            PermissionLvl::Two,
            false,
        ));
    assert!(world.is_in_spawn_protection(&placer, &rail));
    click(1.into())?;
    assert_unspent();
    server
        .data
        .operator_config
        .write()
        .unwrap_or_else(PoisonError::into_inner)
        .ops
        .push(pumpkin_config::op::Op::new(
            placer.gameprofile.id,
            "Notch".into(),
            PermissionLvl::Two,
            false,
        ));
    let set_border = |diameter| {
        let mut border = world
            .worldborder
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        border.old_diameter = diameter;
        border.new_diameter = diameter;
    };
    set_border(16.0);
    click(2.into())?;
    assert_unspent();
    set_border(32.0);
    *placer
        .awaiting_teleport
        .lock()
        .unwrap_or_else(PoisonError::into_inner) = Some((5.into(), placement_position));
    click(3.into())?;
    assert_unspent();
    *placer
        .awaiting_teleport
        .lock()
        .unwrap_or_else(PoisonError::into_inner) = None;
    click(4.into())?;
    assert!(placer.inventory().held_item().is_empty());
    assert_eq!(
        placer.get_stat(StatisticCategory::Used, i32::from(Item::MINECART.id)),
        1
    );
    assert_eq!(placer.last_attacked_ticks.load(Ordering::Relaxed), 0);
    assert_eq!(swing.0.load(Ordering::Relaxed), 2);
    placer.tick_counter.store(1, Ordering::Relaxed);
    placer.last_attacked_ticks.store(9, Ordering::Relaxed);
    placer.inventory().set_held_item(cart.copy_with_count(1));
    click(5.into())?;
    assert!(placer.inventory().held_item().is_empty());
    assert_eq!(placer.last_attacked_ticks.load(Ordering::Relaxed), 9);
    assert_eq!(swing.0.load(Ordering::Relaxed), 2);
    watcher_client.close();
    watcher_client.player.store(Arc::new(None));
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[allow(
    clippy::too_many_lines,
    reason = "Both editions hold the same configured cart through the air and rail packet sequences."
)]
async fn classic_minecart_air_pass_does_not_spend_the_following_rail_use() -> TestResult {
    use pumpkin_data::{
        data_component_impl::UseCooldownImpl, item::Item, item_stack::ItemStack,
        statistic::StatisticCategory,
    };
    use pumpkin_protocol::{
        bedrock::server::inventory_transaction::TransactionData,
        java::server::play::{SUseItem, SUseItemOn},
    };
    use pumpkin_util::math::position::BlockPos;

    use crate::plugin::api::events::player::player_interact_event::{
        InteractAction, PlayerInteractEvent,
    };
    use std::sync::atomic::AtomicUsize;

    use super::boat_vehicle_tests::bedrock_use_item_transaction;

    struct ObserveCartAirUse(AtomicUsize);
    impl EventRwHandler<PlayerInteractEvent> for ObserveCartAirUse {
        fn handle<'a>(
            &'a self,
            _: &'a Arc<Server>,
            event: &'a mut PlayerInteractEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                if event.action == InteractAction::RightClickAir {
                    self.0.fetch_add(1, Ordering::Relaxed);
                }
            })
        }
    }

    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let (java, _peer) = java_rider(&server, &world, "Notch")?;
    let (bedrock, _packets) = bedrock_rider(&server, &world, "jeb_").await?;
    let events = Arc::new(ObserveCartAirUse(AtomicUsize::new(0)));
    server
        .plugin_manager
        .register_blocking::<PlayerInteractEvent, _>(events.clone(), EventPriority::Normal, None);
    let java_client = java.client.java().ok_or("Java player")?;
    let bedrock_client = bedrock.client.bedrock().ok_or("Bedrock player")?;
    let chunk = ChunkData::empty_sync(0, 0);
    for x in [8, 11] {
        chunk.set_block_absolute_y(x, 64, 8, Block::STONE.default_state.id);
        chunk.set_block_absolute_y(x, 65, 8, Block::RAIL.default_state.id);
    }
    world.level.loaded_chunks.insert(Vector2::new(0, 0), chunk);
    for (player, pos) in [(&java, 8.5), (&bedrock, 11.5)] {
        player.get_entity().set_pos(Vector3::new(pos, 65.0, 6.5));
    }
    let mut cart = ItemStack::new(1, &Item::MINECART);
    let group = "minecraft:minecart";
    cart.set_data_component(UseCooldownImpl {
        seconds: 1.0,
        cooldown_group: Some(group.to_owned()),
    });
    java.inventory().set_held_item(cart.clone());
    let bedrock_cart = cart.copy_with_count(1);
    bedrock.inventory().set_held_item(bedrock_cart.clone());

    java_client.handle_use_item(
        &java,
        &SUseItem {
            hand: 0.into(),
            sequence: 1.into(),
            yaw: 0.0,
            pitch: 0.0,
        },
        &server,
    );
    bedrock_client
        .handle_inventory_action(&bedrock, bedrock_use_item_transaction(0, &bedrock_cart, 1));
    assert_eq!(events.0.load(Ordering::Relaxed), 2);
    for player in [&java, &bedrock] {
        assert_eq!(
            player.get_stat(StatisticCategory::Used, i32::from(Item::MINECART.id)),
            0
        );
        assert!(!player.is_on_cooldown(group));
        assert_eq!(player.inventory().held_item().get_item(), &Item::MINECART);
    }
    java_client.handle_use_item_on(
        &java,
        &SUseItemOn {
            hand: 0.into(),
            position: BlockPos::new(8, 65, 8),
            face: 1.into(),
            cursor_pos: Vector3::new(0.5, 0.5, 0.5),
            inside_block: false,
            is_against_world_border: false,
            sequence: 2.into(),
        },
        &server,
    )?;
    let mut packet = bedrock_use_item_transaction(0, &bedrock_cart, 0);
    let TransactionData::UseItem(data) = &mut packet.transaction_data else {
        return Err("expected Bedrock block use".into());
    };
    data.block_position = BlockPos::new(11, 65, 8);
    bedrock_client.handle_inventory_action(&bedrock, packet);
    for player in [&java, &bedrock] {
        assert_eq!(
            player.get_stat(StatisticCategory::Used, i32::from(Item::MINECART.id)),
            1
        );
        assert!(player.is_on_cooldown(group));
        assert!(player.inventory().held_item().is_empty());
    }
    bedrock_client.clear_player();
    bedrock_client.close().await;
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[allow(
    clippy::too_many_lines,
    reason = "One tracked player exercises invalid hits, punch timing, pot feedback and legacy prediction."
)]
async fn java_vehicle_block_hit_and_swing_feedback_follow_the_origin_protocol() -> TestResult {
    use crate::{
        block::entities::decorated_pot::DecoratedPotBlockEntity,
        plugin::api::events::player::player_interact_event::{InteractAction, PlayerInteractEvent},
    };
    use pumpkin_data::{
        data_component_impl::{InteractAnimationImpl, SwingAnimationImpl, SwingAnimationType},
        item::Item,
        item_stack::ItemStack,
        statistic::StatisticCategory,
    };
    use pumpkin_protocol::java::{
        client::play::CSwingArm,
        server::play::{SSwingArm, SUseItemOn},
    };
    use pumpkin_util::{math::position::BlockPos, version::JavaMinecraftVersion};
    use pumpkin_world::cylindrical_chunk_iterator::Cylindrical;
    use std::{num::NonZeroU8, sync::atomic::AtomicUsize};

    #[derive(Default)]
    struct UseFeedback {
        swings: Mutex<Vec<(i32, i32, i32)>>,
        block_interactions: AtomicUsize,
    }
    impl EventRwHandler<PacketSentEvent> for UseFeedback {
        fn handle<'a>(
            &'a self,
            _: &'a Arc<Server>,
            event: &'a mut PacketSentEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                if event.packet_id == CSwingArm::to_id(CURRENT_MC_VERSION) {
                    assert_eq!(event.player.gameprofile.name, "jeb_");
                    let mut payload = event.payload.as_ref();
                    assert!(payload.get_var_int().is_ok());
                    if let (Ok(hand), Ok(animation), Ok(duration)) = (
                        payload.get_var_int(),
                        payload.get_var_int(),
                        payload.get_var_int(),
                    ) {
                        self.swings
                            .lock()
                            .unwrap_or_else(PoisonError::into_inner)
                            .push((hand.0, animation.0, duration.0));
                    }
                    assert!(payload.is_empty());
                }
            })
        }
    }
    impl EventRwHandler<PlayerInteractEvent> for UseFeedback {
        fn handle<'a>(
            &'a self,
            _: &'a Arc<Server>,
            event: &'a mut PlayerInteractEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                if event.action == InteractAction::RightClickBlock {
                    self.block_interactions.fetch_add(1, Ordering::Relaxed);
                }
            })
        }
    }

    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let (placer, _placer_peer) = java_rider(&server, &world, "Notch")?;
    let (watcher, _watcher_peer) = java_rider(&server, &world, "jeb_")?;
    let client = placer.client.java().ok_or("Java placer")?;
    let watcher_client = watcher.client.java().ok_or("Java watcher")?;
    client.player.store(Arc::new(Some(placer.clone())));
    watcher_client.player.store(Arc::new(Some(watcher.clone())));
    placer.get_entity().set_pos(Vector3::new(8.5, 65.0, 6.5));
    watcher.get_entity().set_pos(Vector3::new(10.0, 65.0, 6.5));
    let chunk_pos = Vector2::new(0, 0);
    watcher.watched_section.store(Cylindrical::new(
        chunk_pos,
        NonZeroU8::new(3).ok_or("watch distance")?,
    ));
    watcher
        .chunk_sender
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .mark_sent_out_of_band(chunk_pos);
    let rail = BlockPos::new(8, 65, 8);
    let pot_pos = BlockPos::new(9, 65, 8);
    let chunk = ChunkData::empty_sync(0, 0);
    chunk.set_block_absolute_y(8, 64, 8, Block::STONE.default_state.id);
    chunk.set_block_absolute_y(8, 65, 8, Block::RAIL.default_state.id);
    chunk.set_block_absolute_y(9, 65, 8, Block::DECORATED_POT.default_state.id);
    world.level.loaded_chunks.insert(chunk_pos, chunk);
    let pot = Arc::new(DecoratedPotBlockEntity::new(pot_pos));
    world.add_block_entity(pot.clone());
    let tracked: Arc<dyn EntityBase> = placer.clone();
    world.entity_tracker.add_entity(&tracked, &world);

    let feedback = Arc::new(UseFeedback::default());
    server
        .plugin_manager
        .register_blocking::<PacketSentEvent, _>(feedback.clone(), EventPriority::Normal, None);
    server
        .plugin_manager
        .register_blocking::<PlayerInteractEvent, _>(feedback.clone(), EventPriority::Normal, None);
    let take_swings = || {
        std::mem::take(
            &mut *feedback
                .swings
                .lock()
                .unwrap_or_else(PoisonError::into_inner),
        )
    };
    let click = |sequence, position, cursor_pos| {
        client.handle_use_item_on(
            &placer,
            &SUseItemOn {
                hand: 0.into(),
                position,
                face: 1.into(),
                cursor_pos,
                inside_block: false,
                is_against_world_border: false,
                sequence: pumpkin_protocol::VarInt(sequence),
            },
            &server,
        )
    };
    let mut cart = ItemStack::new(1, &Item::MINECART);
    cart.set_data_component(SwingAnimationImpl {
        animation_type: SwingAnimationType::Whack,
        duration: 7,
    });
    placer.inventory().set_held_item(cart.clone());
    click(1, rail, Vector3::new(0.5, 2.0, 0.5))?;
    assert_eq!(feedback.block_interactions.load(Ordering::Relaxed), 0);
    assert_eq!(placer.inventory().held_item().uid, cart.uid);
    assert_eq!(
        placer.get_stat(StatisticCategory::Used, i32::from(Item::MINECART.id)),
        0
    );

    let swing = SSwingArm { hand: 0.into() };
    client.handle_swing_arm(&server, &placer, &swing);
    assert_eq!(take_swings(), [(0, 1, 7)]);
    placer.tick_counter.store(1, Ordering::Relaxed);
    placer.last_attacked_ticks.store(1, Ordering::Relaxed);
    let center = Vector3::new(0.5, 0.5, 0.5);
    click(2, rail, center)?;
    assert!(placer.inventory().held_item().is_empty());
    assert_eq!(placer.last_attacked_ticks.load(Ordering::Relaxed), 1);
    assert!(take_swings().is_empty());

    placer.tick_counter.store(20, Ordering::Relaxed);
    placer.last_attacked_ticks.store(20, Ordering::Relaxed);
    let mut boat = ItemStack::new(1, &Item::OAK_BOAT);
    boat.set_data_component(InteractAnimationImpl {
        animation_type: SwingAnimationType::Stab,
        duration: 9,
    });
    placer.inventory().set_held_item(boat);
    click(3, pot_pos, center)?;
    assert!(
        pot.get_item()
            .is_some_and(|stack| stack.item == &Item::OAK_BOAT)
    );
    assert!(placer.inventory().held_item().is_empty());
    assert_eq!(take_swings(), [(0, 2, 9), (0, 2, 9)]);
    assert_eq!(placer.last_attacked_ticks.load(Ordering::Relaxed), 0);

    client.version.store(JavaMinecraftVersion::V_1_21_11);
    placer.tick_counter.store(40, Ordering::Relaxed);
    placer.last_attacked_ticks.store(20, Ordering::Relaxed);
    placer.inventory().set_held_item(cart.copy_with_count(1));
    click(4, rail, center)?;
    assert!(placer.inventory().held_item().is_empty());
    assert!(take_swings().is_empty());
    assert_eq!(placer.last_attacked_ticks.load(Ordering::Relaxed), 20);
    client.handle_swing_arm(&server, &placer, &swing);
    assert_eq!(take_swings(), [(0, 1, 6)]);

    for client in [client, watcher_client] {
        client.close();
        client.player.store(Arc::new(None));
    }
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[allow(
    clippy::too_many_lines,
    reason = "The same connection checks vehicle completion, denial feedback, and callback revalidation."
)]
async fn java_vehicle_block_corrections_keep_their_origin_after_callbacks() -> TestResult {
    use crate::plugin::api::events::player::player_interact_event::PlayerInteractEvent;
    use pumpkin_data::{item::Item, item_stack::ItemStack, translation};
    use pumpkin_protocol::{
        java::{
            client::play::{CBlockUpdate, CKeepAlive, CSystemChatMessage},
            packet_decoder::TCPNetworkDecoder,
            server::play::SUseItemOn,
        },
        ser::NetworkReadSliceExt,
    };
    use pumpkin_util::{
        PermissionLvl,
        math::position::BlockPos,
        text::{TextComponent, color::NamedColor},
        version::JavaMinecraftVersion,
    };

    // Official Java 1.20.1 clientbound play packet IDs used by the translation observer.
    const OLD_BLOCK_UPDATE: i32 = 10;
    const OLD_KEEP_ALIVE: i32 = 35;

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Interaction {
        Success,
        ItemFailure,
        Veto,
        SpawnProtected,
        TransferDuringInteract,
        TransferDuringVeto,
        TransferDuringLegacyCorrection,
        StateChangeDuringCorrection,
    }

    struct ReplaceCorrectionOrigin {
        rail: BlockPos,
        destination: Arc<World>,
        origin_chunk: Arc<ChunkData>,
        interaction: AtomicCell<Interaction>,
        changed: AtomicBool,
    }
    impl EventRwHandler<PlayerInteractEvent> for ReplaceCorrectionOrigin {
        fn handle<'a>(
            &'a self,
            _: &'a Arc<Server>,
            event: &'a mut PlayerInteractEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                if event.clicked_pos != Some(self.rail) {
                    return;
                }
                let interaction = self.interaction.load();
                if matches!(
                    interaction,
                    Interaction::TransferDuringInteract | Interaction::TransferDuringVeto
                ) {
                    event
                        .player
                        .get_entity()
                        .set_world(self.destination.clone());
                }
                if matches!(
                    interaction,
                    Interaction::Veto
                        | Interaction::TransferDuringVeto
                        | Interaction::TransferDuringLegacyCorrection
                        | Interaction::StateChangeDuringCorrection
                ) {
                    event.cancelled = true;
                }
            })
        }
    }
    impl EventRwHandler<PacketSentEvent> for ReplaceCorrectionOrigin {
        fn handle<'a>(
            &'a self,
            _: &'a Arc<Server>,
            event: &'a mut PacketSentEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                if event.packet_id != CBlockUpdate::to_id(CURRENT_MC_VERSION)
                    || !event
                        .payload
                        .as_ref()
                        .get_block_pos(&CURRENT_MC_VERSION)
                        .is_ok_and(|pos| pos == self.rail)
                {
                    return;
                }
                match self.interaction.load() {
                    Interaction::TransferDuringLegacyCorrection => {
                        event.cancelled = true;
                        event
                            .extra_packets
                            .push((OLD_BLOCK_UPDATE, event.payload.clone()));
                        event
                            .extra_packets
                            .push((OLD_KEEP_ALIVE, 904i64.to_be_bytes().to_vec().into()));
                        event
                            .player
                            .get_entity()
                            .set_world(self.destination.clone());
                    }
                    Interaction::StateChangeDuringCorrection
                        if !self.changed.swap(true, Ordering::Relaxed) =>
                    {
                        self.origin_chunk.set_block_absolute_y(
                            8,
                            65,
                            8,
                            Block::GOLD_BLOCK.default_state.id,
                        );
                    }
                    _ => {}
                }
            })
        }
    }

    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world_with_spawn_protection(&folder, 4).await?;
    world.level_info.rcu(|old| {
        let mut level_info = (**old).clone();
        level_info.spawn_x = 8;
        level_info.spawn_z = 8;
        level_info
    });
    let destination =
        server.create_world("feedback-replacement".to_owned(), world.dimension.clone());
    let rail = BlockPos::new(8, 65, 8);
    let above = BlockPos::new(8, 66, 8);
    let origin_chunk = ChunkData::empty_sync(0, 0);
    origin_chunk.set_block_absolute_y(8, 64, 8, Block::STONE.default_state.id);
    world
        .level
        .loaded_chunks
        .insert(Vector2::new(0, 0), origin_chunk.clone());
    let destination_chunk = ChunkData::empty_sync(0, 0);
    destination_chunk.set_block_absolute_y(8, 65, 8, Block::STONE.default_state.id);
    destination
        .level
        .loaded_chunks
        .insert(Vector2::new(0, 0), destination_chunk);

    let (player, peer) = java_rider(&server, &world, "Notch")?;
    player.get_entity().set_pos(Vector3::new(8.5, 65.0, 6.5));
    let client = player.client.java().ok_or("Java player")?;
    client.player.store(Arc::new(Some(player.clone())));
    player.permission_lvl.store(PermissionLvl::Zero);
    let listener = Arc::new(ReplaceCorrectionOrigin {
        rail,
        destination: destination.clone(),
        origin_chunk: origin_chunk.clone(),
        interaction: AtomicCell::new(Interaction::Success),
        changed: AtomicBool::new(false),
    });
    server
        .plugin_manager
        .register_blocking::<PlayerInteractEvent, _>(listener.clone(), EventPriority::Normal, None);
    server
        .plugin_manager
        .register_blocking::<PacketSentEvent, _>(listener.clone(), EventPriority::Normal, None);
    let mut decoder = TCPNetworkDecoder::new(tokio::io::BufReader::new(peer));
    for (sequence, interaction) in [
        (1, Interaction::Success),
        (2, Interaction::ItemFailure),
        (3, Interaction::Veto),
        (4, Interaction::TransferDuringInteract),
        (5, Interaction::TransferDuringVeto),
        (6, Interaction::TransferDuringLegacyCorrection),
        (7, Interaction::StateChangeDuringCorrection),
        (8, Interaction::SpawnProtected),
    ] {
        player.get_entity().set_world(world.clone());
        let cart = ItemStack::new(1, &Item::MINECART);
        player.inventory().set_held_item(cart.clone());
        let old_connection = interaction == Interaction::TransferDuringLegacyCorrection;
        client.version.store(if old_connection {
            JavaMinecraftVersion::V_1_20
        } else {
            CURRENT_MC_VERSION
        });
        let initial_state = if interaction == Interaction::ItemFailure {
            Block::STONE.default_state.id
        } else {
            Block::RAIL.default_state.id
        };
        origin_chunk.set_block_absolute_y(8, 65, 8, initial_state);
        let protected = interaction == Interaction::SpawnProtected;
        if protected {
            server
                .data
                .operator_config
                .write()
                .unwrap_or_else(PoisonError::into_inner)
                .ops
                .push(pumpkin_config::op::Op::new(
                    uuid::Uuid::nil(),
                    "jeb_".into(),
                    PermissionLvl::Two,
                    false,
                ));
        }
        listener.interaction.store(interaction);
        client.handle_use_item_on(
            &player,
            &SUseItemOn {
                hand: 0.into(),
                position: rail,
                face: 1.into(),
                cursor_pos: Vector3::new(0.5, 0.5, 0.5),
                inside_block: false,
                is_against_world_border: false,
                sequence: pumpkin_protocol::VarInt(sequence),
            },
            &server,
        )?;
        if interaction == Interaction::Success {
            assert!(player.inventory().get_slot(0).is_empty());
        } else {
            assert_eq!(player.inventory().get_slot(0).uid, cart.uid);
        }
        let transferred = matches!(
            interaction,
            Interaction::TransferDuringInteract
                | Interaction::TransferDuringVeto
                | Interaction::TransferDuringLegacyCorrection
        );
        let expected_world = if transferred { &destination } else { &world };
        assert!(Arc::ptr_eq(expected_world, &player.world()));
        assert!(client.try_send_packet(&CKeepAlive::new(i64::from(sequence))));
        let (updates, overlays, old_keep_alives) =
            tokio::time::timeout(std::time::Duration::from_secs(5), async {
                let mut overlays = Vec::new();
                let mut updates = Vec::new();
                let mut old_keep_alives = Vec::new();
                loop {
                    let packet = decoder.get_raw_packet().await?;
                    let mut payload = packet.payload.as_ref();
                    if packet.id == CKeepAlive::to_id(CURRENT_MC_VERSION) {
                        assert_eq!(payload.get_i64_be()?, i64::from(sequence));
                        return Ok::<_, Box<dyn std::error::Error>>((
                            updates,
                            overlays,
                            old_keep_alives,
                        ));
                    }
                    if packet.id == CBlockUpdate::to_id(CURRENT_MC_VERSION)
                        || (old_connection && packet.id == OLD_BLOCK_UPDATE)
                    {
                        updates.push((
                            payload.get_block_pos(&CURRENT_MC_VERSION)?,
                            payload.get_var_int()?.0,
                        ));
                    } else if packet.id == CSystemChatMessage::to_id(CURRENT_MC_VERSION) {
                        overlays.push((
                            payload.get_component(&CURRENT_MC_VERSION)?,
                            payload.get_bool()?,
                        ));
                    } else if old_connection && packet.id == OLD_KEEP_ALIVE {
                        old_keep_alives.push(payload.get_i64_be()?);
                    }
                }
            })
            .await??;
        if transferred {
            assert!(updates.is_empty());
        } else {
            let final_state = if interaction == Interaction::StateChangeDuringCorrection {
                Block::GOLD_BLOCK.default_state.id
            } else {
                initial_state
            };
            assert_eq!(
                updates,
                [
                    (rail, i32::from(final_state.as_u16())),
                    (above, i32::from(Block::AIR.default_state.id.as_u16())),
                ],
            );
        }
        if protected {
            assert_eq!(
                overlays,
                [(
                    TextComponent::translate(
                        translation::java::BUILD_SPAWN_PROTECTION,
                        [TextComponent::text("8, 65, 8")],
                    )
                    .color_named(NamedColor::Red),
                    true,
                )],
            );
        } else {
            assert!(overlays.is_empty());
        }
        if old_connection {
            assert_eq!(old_keep_alives, [904]);
        } else {
            assert!(old_keep_alives.is_empty());
        }
    }

    client.close();
    client.player.store(Arc::new(None));
    destination.level.shutdown().await;
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn java_minecart_advancement_matches_the_clicked_rail_state() -> TestResult {
    use crate::entity::player::advancement::{
        ActiveCriterionListener, AdvancementProgress,
        predicate::{CriterionConditions, ItemUsedOnBlockPredicate, TagOrIdMatcher},
    };
    use pumpkin_data::{Advancement, Property, item::Item, item_stack::ItemStack};
    use pumpkin_protocol::java::server::play::SUseItemOn;
    use pumpkin_util::math::position::BlockPos;

    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let (player, _peer) = java_rider(&server, &world, "Notch")?;
    let client = player.client.java().ok_or("Java player")?;
    player.get_entity().set_pos(Vector3::new(8.5, 65.0, 6.5));
    let rail = BlockPos::new(8, 65, 8);
    let chunk = ChunkData::empty_sync(0, 0);
    chunk.set_block_absolute_y(8, 64, 8, Block::STONE.default_state.id);
    world
        .level
        .loaded_chunks
        .insert(Vector2::new(0, 0), chunk.clone());

    let advancement = Advancement::HUSBANDRY_RIDE_A_BOAT_WITH_A_GOAT;
    let matcher = |pattern: &str| TagOrIdMatcher {
        patterns: vec![pattern.into()],
    };
    let predicate = ItemUsedOnBlockPredicate {
        blocks: Some(matcher("minecraft:powered_rail")),
        items: Some(matcher("minecraft:minecart")),
        state_key: Some("powered".into()),
        state_value: Some("true".into()),
        ..ItemUsedOnBlockPredicate::default()
    };
    {
        let mut advancements = player
            .advancements
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        advancements.set_player(&player);
        advancements.active_listeners.insert(
            "minecraft:item_used_on_block",
            vec![ActiveCriterionListener {
                advancement,
                criterion: advancement.criteria[0],
                conditions: Arc::new(CriterionConditions::ItemUsedOnBlock(predicate)),
            }],
        )
    };
    for (sequence, powered) in [(1, false), (2, true)] {
        let state = Block::POWERED_RAIL
            .with_property(
                Block::POWERED_RAIL.default_state.id,
                Property::Powered(powered),
            )
            .ok_or("Powered rail property")?;
        chunk.set_block_absolute_y(8, 65, 8, state);
        player
            .inventory()
            .set_held_item(ItemStack::new(1, &Item::MINECART));
        client.handle_use_item_on(
            &player,
            &SUseItemOn {
                hand: 0.into(),
                position: rail,
                face: 1.into(),
                cursor_pos: Vector3::new(0.5, 0.5, 0.5),
                inside_block: false,
                is_against_world_border: false,
                sequence: pumpkin_protocol::VarInt(sequence),
            },
            &server,
        )?;
        assert!(player.inventory().held_item().is_empty());
        let advancements = player
            .advancements
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        assert_eq!(
            advancements
                .progress
                .map
                .get(advancement)
                .is_some_and(AdvancementProgress::is_done),
            powered,
        );
    }
    client.close();
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[expect(
    clippy::too_many_lines,
    reason = "One equipment fixture compares veto, component eligibility and the authoritative swap."
)]
async fn bedrock_equippable_minecart_waits_for_veto_and_commits_one_item() -> TestResult {
    use crate::plugin::api::events::player::player_interact_event::{
        InteractAction, PlayerInteractEvent,
    };
    use pumpkin_data::{
        data_component_impl::{
            EquipmentSlot, EquippableImpl, IDSet, MaxStackSizeImpl, UseCooldownImpl,
        },
        item::Item,
        item_stack::ItemStack,
        statistic::StatisticCategory,
    };
    use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};
    use pumpkin_protocol::bedrock::{
        network_item::NetworkItemDescriptor, server::inventory_transaction::InventoryAction,
    };

    struct VetoCartEquip(AtomicBool);
    impl EventRwHandler<PlayerInteractEvent> for VetoCartEquip {
        fn handle<'a>(
            &'a self,
            _: &'a Arc<Server>,
            event: &'a mut PlayerInteractEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                if event.action == InteractAction::RightClickAir {
                    event.cancelled = self.0.load(Ordering::Relaxed);
                }
            })
        }
    }

    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let (player, _packets) = bedrock_rider(&server, &world, "jeb_").await?;
    let client = player.client.bedrock().ok_or("Bedrock player")?;
    let inventory = player.inventory();
    let equipment_slot = inventory
        .equipment_slot_index(&EquipmentSlot::HEAD)
        .ok_or("head slot")?;
    let untouched = ItemStack::new(1, &Item::STONE);
    inventory.set_slot(1, untouched.clone());
    let group = "exercise:cart_equip";
    let mut cart = ItemStack::new(2, &Item::MINECART);
    cart.set_data_component(MaxStackSizeImpl { size: 2 });
    cart.set_data_component(UseCooldownImpl {
        seconds: 1.0,
        cooldown_group: Some(group.to_owned()),
    });
    let mut data = NbtCompound::new();
    data.put_string("slot", "head".to_owned());
    let equippable = EquippableImpl::read_data(&NbtTag::Compound(data)).ok_or("equippable cart")?;
    let veto = Arc::new(VetoCartEquip(AtomicBool::new(true)));
    server
        .plugin_manager
        .register_blocking::<PlayerInteractEvent, _>(veto.clone(), EventPriority::Normal, None);

    let send = |stack: &ItemStack| {
        inventory.set_held_item(stack.clone());
        let mut packet = super::boat_vehicle_tests::bedrock_use_item_transaction(0, stack, 1);
        packet.actions.push(InventoryAction {
            source_type: 0,
            window_id: Some(0),
            source_flags: None,
            inventory_slot: 1,
            old_item: NetworkItemDescriptor::from(&untouched),
            new_item: NetworkItemDescriptor::from(ItemStack::EMPTY),
        });
        client.handle_inventory_action(&player, packet);
    };
    let uses = || player.get_stat(StatisticCategory::Used, i32::from(Item::MINECART.id));
    let assert_unchanged = || {
        assert_eq!(inventory.held_item().item_count, 2);
        assert_eq!(inventory.get_slot(1).uid, untouched.uid);
        assert!(inventory.get_slot(equipment_slot).is_empty());
        assert_eq!(uses(), 0);
        assert!(!player.is_on_cooldown(group));
    };

    cart.set_data_component(equippable.clone());
    send(&cart);
    assert_unchanged();
    veto.0.store(false, Ordering::Relaxed);
    cart.set_data_component(EquippableImpl {
        swappable: false,
        ..equippable.clone()
    });
    send(&cart);
    assert_unchanged();
    cart.set_data_component(EquippableImpl {
        allowed_entities: Some(IDSet::IDs(std::borrow::Cow::Owned(vec![&EntityType::PIG]))),
        ..equippable.clone()
    });
    send(&cart);
    assert_unchanged();

    cart.set_data_component(equippable);
    send(&cart);
    assert_eq!(inventory.held_item().item_count, 1);
    assert_eq!(inventory.get_slot(1).uid, untouched.uid);
    let equipped = inventory.get_slot(equipment_slot);
    assert_eq!(equipped.get_item(), &Item::MINECART);
    assert_eq!(equipped.item_count, 1);
    assert_eq!(uses(), 1);
    assert!(player.is_on_cooldown(group));
    client.clear_player();
    client.close().await;
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[allow(
    clippy::too_many_lines,
    reason = "The edition packet paths share consumption, cooldown, sounds and active hand ownership."
)]
async fn configured_minecart_air_use_completes_consumption_and_dispatches_kinetic_use() -> TestResult
{
    use pumpkin_data::{
        data_component::DataComponent,
        data_component_impl::{
            ConsumableImpl, ConsumeAnimation, ConsumeEffect, IdOr, KineticWeaponImpl,
            PotionContentsImpl, UseCooldownImpl, UseRemainderImpl,
        },
        effect::StatusEffect,
        entity::EntityStatus,
        item::Item,
        item_stack::ItemStack,
        potion::Potion,
        statistic::StatisticCategory,
    };
    use pumpkin_protocol::{
        bedrock::client::CPlaySound,
        java::{
            client::play::CEntityStatus,
            server::play::{SSetHeldItem, SUseItem},
        },
        packet::Packet,
    };
    use std::{borrow::Cow, sync::atomic::AtomicUsize};

    #[derive(Default)]
    struct ObserveCartFeedback {
        near_eat: AtomicUsize,
        near_kinetic: AtomicUsize,
        far_kinetic: AtomicUsize,
        actor_kinetic: AtomicUsize,
        actor_completion: AtomicUsize,
        observer_completion: AtomicUsize,
        starts: AtomicUsize,
        stops: AtomicUsize,
    }

    impl EventRwHandler<PacketSentEvent> for ObserveCartFeedback {
        fn handle<'a>(
            &'a self,
            _: &'a Arc<Server>,
            event: &'a mut PacketSentEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                let name = event.player.gameprofile.name.as_str();
                if event.packet_id == CSoundEffect::to_id(CURRENT_MC_VERSION)
                    && let Ok(id) = event.payload.as_ref().get_var_int()
                {
                    if id.0 == Sound::EntityGenericEat as i32 + 1 && name == "deadmau5" {
                        self.near_eat.fetch_add(1, Ordering::Relaxed);
                    }
                    if id.0 == Sound::ItemSpearUse as i32 + 1 {
                        let count = match name {
                            "deadmau5" => &self.near_kinetic,
                            "Grumm" => &self.far_kinetic,
                            _ => &self.actor_kinetic,
                        };
                        count.fetch_add(1, Ordering::Relaxed);
                    }
                }
                if event.packet_id == CEntityStatus::to_id(CURRENT_MC_VERSION)
                    && event.payload.last().copied() == Some(EntityStatus::UseItemComplete as u8)
                {
                    let count = if name == "Notch" {
                        &self.actor_completion
                    } else {
                        &self.observer_completion
                    };
                    count.fetch_add(1, Ordering::Relaxed);
                }
            })
        }
    }

    impl EventRwHandler<GenericGameEvent> for ObserveCartFeedback {
        fn handle<'a>(
            &'a self,
            _: &'a Arc<Server>,
            event: &'a mut GenericGameEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                if event.event_key == GameEvent::ItemInteractStart.name() {
                    self.starts.fetch_add(1, Ordering::Relaxed);
                } else if event.event_key == GameEvent::ItemInteractFinish.name() {
                    self.stops.fetch_add(1, Ordering::Relaxed);
                }
            })
        }
    }

    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let (java, _peer) = java_rider(&server, &world, "Notch")?;
    let (near, _near_peer) = java_rider(&server, &world, "deadmau5")?;
    let (far, _far_peer) = java_rider(&server, &world, "Grumm")?;
    let (bedrock, mut actor_packets) = bedrock_rider(&server, &world, "jeb_").await?;
    let (observer, mut observer_packets) = bedrock_rider(&server, &world, "Dinnerbone").await?;
    java.get_entity().set_pos(CENTER);
    // Player consumption remains audible to observers even when the player is silent.
    java.get_entity().set_silent(true);
    near.get_entity().set_pos(CENTER);
    far.get_entity().set_pos(CENTER.add_raw(16.0, 0.0, 0.0));
    for player in [&java, &near, &far] {
        player
            .client
            .java()
            .ok_or("Java player")?
            .player
            .store(Arc::new(Some(player.clone())));
    }
    let java_client = java.client.java().ok_or("Java player")?;
    let bedrock_client = bedrock.client.bedrock().ok_or("Bedrock player")?;
    let feedback = Arc::new(ObserveCartFeedback::default());
    server
        .plugin_manager
        .register_blocking::<PacketSentEvent, _>(feedback.clone(), EventPriority::Normal, None);
    server
        .plugin_manager
        .register_blocking::<GenericGameEvent, _>(feedback.clone(), EventPriority::Normal, None);

    let send = |player: &Arc<crate::entity::player::Player>, stack: &ItemStack, sequence| {
        player.inventory().set_held_item(stack.clone());
        if Arc::ptr_eq(player, &java) {
            java_client.handle_use_item(
                player,
                &SUseItem {
                    hand: 0.into(),
                    sequence,
                    yaw: 0.0,
                    pitch: -90.0,
                },
                &server,
            );
        } else {
            bedrock_client.handle_inventory_action(
                player,
                super::boat_vehicle_tests::bedrock_use_item_transaction(0, stack, 1),
            );
        }
    };
    let consumable = |duration| {
        ConsumableImpl::new(
            duration,
            ConsumeAnimation::Eat,
            IdOr::Id(Sound::EntityGenericEat),
            false,
            Cow::Borrowed(&[ConsumeEffect::ClearAllEffects]),
        )
    };
    let group = "exercise:instant_cart";
    let timed_group = "exercise:timed_cart";
    let mut instant = ItemStack::new(1, &Item::MINECART);
    instant.set_data_component(consumable(0.0));
    instant.set_data_component(UseCooldownImpl {
        seconds: 1.0,
        cooldown_group: Some(group.to_owned()),
    });
    instant.set_data_component(UseRemainderImpl {
        remainder: Some(ItemStack::new(1, &Item::STICK)),
    });
    // The explicit clear action runs after the potion component awards Speed.
    instant.set_data_component(PotionContentsImpl {
        potion_id: Some(i32::from(Potion::SWIFTNESS.id)),
        custom_color: None,
        custom_effects: vec![],
        custom_name: None,
    });
    let mut timed = ItemStack::new(1, &Item::MINECART);
    timed.set_data_component(consumable(0.5));
    timed.set_data_component(UseRemainderImpl {
        remainder: Some(ItemStack::new(1, &Item::STICK)),
    });
    timed.set_data_component(UseCooldownImpl {
        seconds: 1.0,
        cooldown_group: Some(timed_group.to_owned()),
    });
    let mut kinetic = ItemStack::new(1, &Item::MINECART);
    kinetic.set_data_component(KineticWeaponImpl {
        contact_cooldown_ticks: 10,
        delay_ticks: 0,
        dismount_conditions: None,
        knockback_conditions: None,
        damage_conditions: None,
        forward_movement: 0.0,
        damage_multiplier: 1.0,
        sound: Some(IdOr::Id(Sound::ItemSpearUse)),
        hit_sound: None,
    });

    for (edition, player) in [&java, &bedrock].into_iter().enumerate() {
        let uses = || player.get_stat(StatisticCategory::Used, i32::from(Item::MINECART.id));
        send(player, &instant.copy_with_count(1), 1.into());
        assert_eq!(player.inventory().held_item().get_item(), &Item::STICK);
        assert_eq!(uses(), 1);
        assert!(player.is_on_cooldown(group));
        assert!(!player.living_entity.has_effect(&StatusEffect::SPEED));

        let held = timed.copy_with_count(1);
        send(player, &held, 2.into());
        assert_eq!(player.inventory().held_item().uid, held.uid);
        assert_eq!(uses(), 1);
        let timer = &player.living_entity.item_use_time;
        assert_eq!(timer.load(Ordering::Relaxed), 10);
        for _ in 0..4 {
            player.living_entity.tick(player.as_ref(), &server);
        }
        player.start_cooldown(timed_group.to_owned(), 5);
        for _ in 0..5 {
            player.living_entity.tick(player.as_ref(), &server);
        }
        assert_eq!(timer.load(Ordering::Relaxed), 1);
        assert_eq!(feedback.near_eat.load(Ordering::Relaxed), edition * 3 + 2);
        let completions = feedback.actor_completion.load(Ordering::Relaxed);
        player.living_entity.tick(player.as_ref(), &server);
        assert_eq!(player.inventory().held_item().get_item(), &Item::STICK);
        assert_eq!(uses(), 2);
        assert_eq!(
            feedback.actor_completion.load(Ordering::Relaxed),
            completions + usize::from(edition == 0)
        );
        assert_eq!(feedback.observer_completion.load(Ordering::Relaxed), 0);
        let cooldowns = player
            .item_cooldowns
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        assert_eq!(
            cooldowns.get(timed_group).map(|cooldown| cooldown.duration),
            Some(20)
        );
        drop(cooldowns);

        let held = kinetic.copy_with_count(1);
        let starts = feedback.starts.load(Ordering::Relaxed);
        let stops = feedback.stops.load(Ordering::Relaxed);
        send(player, &held, 3.into());
        assert_eq!(timer.load(Ordering::Relaxed), held.get_max_use_time());
        player.living_entity.tick(player.as_ref(), &server);
        send(player, &held, 4.into());
        assert_eq!(timer.load(Ordering::Relaxed), held.get_max_use_time() - 1);
        assert_eq!(
            feedback.near_kinetic.load(Ordering::Relaxed),
            edition * 2 + 1
        );
        assert_eq!(feedback.far_kinetic.load(Ordering::Relaxed), 0);
        // Notch is the initiator in Java and a nearby observer of the Bedrock use.
        assert_eq!(feedback.actor_kinetic.load(Ordering::Relaxed), edition);
        assert_eq!(feedback.starts.load(Ordering::Relaxed), starts + 1);
        let replacement = held.copy_with_count(1);
        player.inventory().set_held_item(replacement);
        player.living_entity.tick(player.as_ref(), &server);
        assert_eq!(timer.load(Ordering::Relaxed), 0);
        assert_eq!(feedback.stops.load(Ordering::Relaxed), stops + 1);
        assert_eq!(uses(), 2);

        let mut silent = kinetic.copy_with_count(1);
        silent.remove_data_component(DataComponent::UseEffects);
        send(player, &silent, 5.into());
        assert!(timer.load(Ordering::Relaxed) > 0);
        if edition == 0 {
            let untouched = ItemStack::new(1, &Item::STICK);
            player.inventory().set_slot(1, untouched);
            java_client.handle_set_held_item(&server, player, &SSetHeldItem { slot: 1 });
            assert_eq!(player.inventory().get_slot(0).uid, silent.uid);
        } else {
            player.living_entity.clear_active_hand();
        }
        assert_eq!(timer.load(Ordering::Relaxed), 0);
        assert_eq!(feedback.starts.load(Ordering::Relaxed), starts + 1);
        assert_eq!(feedback.stops.load(Ordering::Relaxed), stops + 1);
    }

    let sounds = |packets: &mut BedrockQueue| {
        let mut count = 0;
        while let Ok(packet) = packets.try_recv() {
            if let Some((id, payload)) = crate::net::bedrock::split_raw_packet(&packet.data)
                && i32::from(id) == CPlaySound::PACKET_ID
                && payload
                    .as_ref()
                    .get_str()
                    .is_ok_and(|name| name.as_ref() == "item.spear.use")
            {
                count += 1;
            }
        }
        count
    };
    assert_eq!(sounds(&mut observer_packets), 4);
    assert_eq!(sounds(&mut actor_packets), 2);
    for player in [&bedrock, &observer] {
        let client = player.client.bedrock().ok_or("Bedrock player")?;
        client.clear_player();
        client.close().await;
    }
    for player in [&java, &near, &far] {
        let client = player.client.java().ok_or("Java player")?;
        client.close();
        client.player.store(Arc::new(None));
    }
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn configured_cart_tick_does_not_decrement_reentrant_replacement_use() -> TestResult {
    use crate::{
        entity::player::Player,
        item::items::cart_air_use::{CartAirUseResult, use_cart_in_air},
    };
    use pumpkin_data::{
        data_component_impl::{ConsumableImpl, ConsumeAnimation, IdOr},
        item::Item,
        item_stack::ItemStack,
    };
    use pumpkin_util::Hand;
    use std::borrow::Cow;

    struct ReplaceUse {
        actor: Arc<Player>,
        stack: ItemStack,
        pending: AtomicBool,
    }

    impl EventRwHandler<PacketSentEvent> for ReplaceUse {
        fn handle<'a>(
            &'a self,
            _: &'a Arc<Server>,
            event: &'a mut PacketSentEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                if event.player.gameprofile.name == "deadmau5"
                    && event.packet_id == CSoundEffect::to_id(CURRENT_MC_VERSION)
                    && event
                        .payload
                        .as_ref()
                        .get_var_int()
                        .is_ok_and(|id| id.0 == Sound::EntityGenericEat as i32 + 1)
                    && self.pending.swap(false, Ordering::Relaxed)
                {
                    let actor = &self.actor;
                    actor.living_entity.clear_active_hand();
                    actor.inventory().set_slot(0, self.stack.clone());
                    let result =
                        use_cart_in_air(actor, Hand::Right, 0, &self.stack, &actor.world());
                    assert!(result == CartAirUseResult::Use);
                }
            })
        }
    }

    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let (actor, _actor_peer) = java_rider(&server, &world, "Notch")?;
    let (observer, _observer_peer) = java_rider(&server, &world, "deadmau5")?;
    actor.get_entity().set_pos(CENTER);
    observer.get_entity().set_pos(CENTER);
    let observer_client = observer.client.java().ok_or("Java observer")?;
    observer_client
        .player
        .store(Arc::new(Some(observer.clone())));

    let mut stack = ItemStack::new(1, &Item::MINECART);
    let consumable = |seconds| {
        ConsumableImpl::new(
            seconds,
            ConsumeAnimation::Eat,
            IdOr::Id(Sound::EntityGenericEat),
            false,
            Cow::Borrowed(&[]),
        )
    };
    stack.set_data_component(consumable(0.5));
    actor.inventory().set_slot(0, stack.clone());
    assert!(use_cart_in_air(&actor, Hand::Right, 0, &stack, &world) == CartAirUseResult::Use);
    // Reuse the physical stack and give the replacement the same duration
    // as the old action has left. Only the revision distinguishes the timers.
    let mut replacement = stack.clone();
    replacement.set_data_component(consumable(0.2));
    let restart = Arc::new(ReplaceUse {
        actor: actor.clone(),
        stack: replacement.clone(),
        pending: AtomicBool::new(true),
    });
    server
        .plugin_manager
        .register_blocking::<PacketSentEvent, _>(restart.clone(), EventPriority::Normal, None);
    for _ in 0..6 {
        actor.living_entity.tick(actor.as_ref(), &server);
    }
    let timer = &actor.living_entity.item_use_time;
    assert_eq!(timer.load(Ordering::Relaxed), 4);
    assert!(restart.pending.load(Ordering::Relaxed));
    actor.living_entity.tick(actor.as_ref(), &server);
    assert!(!restart.pending.load(Ordering::Relaxed));
    assert_eq!(timer.load(Ordering::Relaxed), 4);
    assert_eq!(actor.inventory().held_item().uid, stack.uid);
    assert!(actor.inventory().held_item().are_equal(&replacement));

    observer_client.close();
    observer_client.player.store(Arc::new(None));
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[allow(
    clippy::too_many_lines,
    reason = "The two use ticks share their target setup, source, and advancement listener."
)]
async fn configured_cart_kinetic_contacts_use_default_range_and_preserve_the_physical_source()
-> TestResult {
    use crate::{
        entity::player::Player, plugin::api::events::entity::entity_damage::EntityDamageEvent,
    };
    use pumpkin_data::{
        Advancement,
        attributes::Attributes,
        data_component_impl::{
            AttackRangeImpl, KineticConditionImpl, KineticWeaponImpl, MaxDamageImpl, WeaponImpl,
        },
        item::Item,
        item_stack::ItemStack,
    };
    use pumpkin_protocol::java::server::play::SUseItem;

    struct SelectOtherSlotOnContact {
        player: Arc<Player>,
        target_id: i32,
    }

    impl EventRwHandler<EntityDamageEvent> for SelectOtherSlotOnContact {
        fn handle<'a>(
            &'a self,
            _: &'a Arc<Server>,
            event: &'a mut EntityDamageEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                if event.entity_id == self.target_id && event.damage_type == DamageType::SPEAR {
                    self.player.inventory().set_selected_slot(1);
                }
            })
        }
    }

    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    world
        .level
        .loaded_chunks
        .insert(Vector2::new(0, 0), ChunkData::empty_sync(0, 0));
    let (player, _peer) = java_rider(&server, &world, "Notch")?;
    player.get_entity().set_pos(Vector3::new(8.5, 64.0, 6.5));
    player
        .living_entity
        .set_attribute_base(&Attributes::ENTITY_INTERACTION_RANGE, 2.5);
    let spawn_pig = |x, z| {
        let pig = PigEntity::new(Entity::new(
            world.clone(),
            Vector3::new(x, 65.0, z),
            &EntityType::PIG,
        ));
        world.spawn_entity(pig.clone());
        pig
    };
    let targets = [7.6, 8.0, 8.4, 8.8].map(|z| spawn_pig(8.5, z));
    let far_target = spawn_pig(8.5, 9.7);
    let side_target = spawn_pig(9.05, 8.5);
    let health = |pig: &PigEntity| pig.mob_entity.living_entity.health.load();
    let initial_health = health(far_target.as_ref());
    let advancement = Advancement::ADVENTURE_SPEAR_MANY_MOBS;
    let progress = || {
        player
            .advancements
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    };
    let advancement_started_done = {
        let mut initial_progress = progress();
        initial_progress.set_player(&player);
        initial_progress
            .progress
            .get_mut_or_start_progress(advancement);
        initial_progress.register_listeners(advancement);
        initial_progress
            .progress
            .get_mut_or_start_progress(advancement)
            .is_done()
    };
    assert!(!advancement_started_done);
    let has_advancement = || {
        progress()
            .progress
            .get_mut_or_start_progress(advancement)
            .is_done()
    };
    server
        .plugin_manager
        .register_blocking::<EntityDamageEvent, _>(
            Arc::new(SelectOtherSlotOnContact {
                player: player.clone(),
                target_id: far_target.get_entity().entity_id,
            }),
            EventPriority::Normal,
            None,
        );

    let mut cart = ItemStack::new(1, &Item::MINECART);
    cart.set_data_component(KineticWeaponImpl {
        contact_cooldown_ticks: 10,
        delay_ticks: 0,
        dismount_conditions: None,
        knockback_conditions: None,
        damage_conditions: Some(KineticConditionImpl {
            max_duration_ticks: 20,
            min_speed: 0.0,
            min_relative_speed: 0.0,
        }),
        forward_movement: 0.0,
        damage_multiplier: 1.0,
        sound: None,
        hit_sound: None,
    });
    cart.set_data_component(MaxDamageImpl { max_damage: 32 });
    cart.set_data_component(WeaponImpl {
        item_damage_per_attack: 1,
        disable_blocking_for_seconds: 0.0,
    });
    assert!(cart.get_data_component::<AttackRangeImpl>().is_none());
    let source_uid = cart.uid;
    let sword = ItemStack::new(1, &Item::IRON_SWORD);
    let sword_uid = sword.uid;
    let inventory = player.inventory();
    inventory.set_slot(0, cart);
    inventory.set_slot(1, sword);
    player.client.java().ok_or("Java player")?.handle_use_item(
        &player,
        &SUseItem {
            hand: 0.into(),
            sequence: 1.into(),
            yaw: 0.0,
            pitch: 0.0,
        },
        &server,
    );

    player.living_entity.tick(player.as_ref(), &server);
    assert!(targets.iter().all(|target| health(target) < initial_health));
    assert_eq!(inventory.get_slot(0).get_damage(), 4);
    assert_eq!(health(far_target.as_ref()), initial_health);
    assert_eq!(health(side_target.as_ref()), initial_health);
    assert!(!has_advancement());

    far_target
        .get_entity()
        .set_pos(Vector3::new(8.5, 65.0, 9.0));
    player.living_entity.tick(player.as_ref(), &server);
    assert!(health(far_target.as_ref()) < initial_health);
    assert_eq!(inventory.get_slot(0).uid, source_uid);
    assert_eq!(inventory.get_slot(0).get_damage(), 5);
    assert_eq!(inventory.held_item().uid, sword_uid);
    assert_eq!(inventory.held_item().get_damage(), 0);
    assert!(has_advancement());
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn java_cart_air_equipment_uses_packet_rotation_and_resets_attack() -> TestResult {
    use pumpkin_data::{
        data_component_impl::{EquipmentSlot, EquippableImpl},
        item::Item,
        item_stack::ItemStack,
    };
    use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};
    use pumpkin_protocol::java::server::play::SUseItem;

    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let (placer, _peer) = java_rider(&server, &world, "Notch")?;
    placer.get_entity().set_pos(Vector3::new(8.5, 65.0, 6.5));
    placer.get_entity().set_rotation(0.0, 0.0);
    world
        .level
        .loaded_chunks
        .insert(Vector2::new(0, 0), ChunkData::empty_sync(0, 0));
    let mut data = NbtCompound::new();
    data.put_string("slot", "head".to_owned());
    let mut cart = ItemStack::new(1, &Item::MINECART);
    cart.set_data_component(
        EquippableImpl::read_data(&NbtTag::Compound(data)).ok_or("equippable cart")?,
    );
    placer.inventory().set_held_item(cart);
    placer.last_attacked_ticks.store(20, Ordering::Relaxed);
    placer.client.java().ok_or("Java placer")?.handle_use_item(
        &placer,
        &SUseItem {
            hand: 0.into(),
            sequence: 1.into(),
            yaw: 450.0,
            pitch: -485.0,
        },
        &server,
    );
    assert_eq!(placer.rotation(), (90.0, -90.0));
    let inventory = placer.inventory();
    let head = inventory
        .equipment_slot_index(&EquipmentSlot::HEAD)
        .ok_or("head slot")?;
    assert_eq!(inventory.get_slot(head).get_item(), &Item::MINECART);
    assert!(inventory.held_item().is_empty());
    assert_eq!(placer.last_attacked_ticks.load(Ordering::Relaxed), 0);
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn configured_cart_completion_keeps_earned_items_after_world_transfer() -> TestResult {
    use crate::plugin::api::events::player::player_statistic_increment::PlayerStatisticIncrementEvent;
    use pumpkin_data::{
        data_component_impl::{ConsumableImpl, ConsumeAnimation, IdOr, UseRemainderImpl},
        item::Item,
        item_stack::ItemStack,
        statistic::StatisticCategory,
    };
    use pumpkin_protocol::java::server::play::SUseItem;
    use std::{borrow::Cow, sync::atomic::AtomicUsize};

    struct TransferDuringConsumption(Arc<World>);
    impl EventRwHandler<PlayerStatisticIncrementEvent> for TransferDuringConsumption {
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
    #[derive(Default)]
    struct DestinationEvents(AtomicUsize);
    impl EventRwHandler<GenericGameEvent> for DestinationEvents {
        fn handle<'a>(
            &'a self,
            _: &'a Arc<Server>,
            event: &'a mut GenericGameEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                if event.event_key == GameEvent::Eat.name()
                    || event.event_key == GameEvent::ItemInteractFinish.name()
                {
                    self.0.fetch_add(1, Ordering::Relaxed);
                }
            })
        }
    }

    let origin_folder = tempfile::tempdir()?;
    let destination_folder = tempfile::tempdir()?;
    let (server, origin) = vehicle_world(&origin_folder).await?;
    let (destination_server, destination) = vehicle_world(&destination_folder).await?;
    let (player, _peer) = java_rider(&server, &origin, "Notch")?;
    let events = Arc::new(DestinationEvents::default());
    server
        .plugin_manager
        .register_blocking::<PlayerStatisticIncrementEvent, _>(
            Arc::new(TransferDuringConsumption(destination.clone())),
            EventPriority::Normal,
            None,
        );
    destination_server
        .plugin_manager
        .register_blocking::<GenericGameEvent, _>(events.clone(), EventPriority::Normal, None);
    let mut cart = ItemStack::new(1, &Item::MINECART);
    cart.set_data_component(ConsumableImpl::new(
        0.05,
        ConsumeAnimation::Eat,
        IdOr::Id(Sound::EntityGenericEat),
        false,
        Cow::Borrowed(&[]),
    ));
    cart.set_data_component(UseRemainderImpl {
        remainder: Some(ItemStack::new(1, &Item::STICK)),
    });
    player.inventory().set_held_item(cart);
    player.client.java().ok_or("Java player")?.handle_use_item(
        &player,
        &SUseItem {
            hand: 0.into(),
            sequence: 1.into(),
            yaw: 0.0,
            pitch: 0.0,
        },
        &server,
    );
    assert_eq!(
        player.living_entity.item_use_time.load(Ordering::Relaxed),
        1
    );
    player.living_entity.tick(player.as_ref(), &server);
    assert!(Arc::ptr_eq(&player.world(), &destination));
    assert_eq!(
        player.get_stat(StatisticCategory::Used, i32::from(Item::MINECART.id)),
        1
    );
    assert_eq!(player.inventory().held_item().get_item(), &Item::STICK);
    assert_eq!(events.0.load(Ordering::Relaxed), 0);
    origin.level.shutdown().await;
    destination.level.shutdown().await;
    Ok(())
}
