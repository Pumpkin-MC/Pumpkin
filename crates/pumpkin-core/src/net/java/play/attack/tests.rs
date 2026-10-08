use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};

use arc_swap::ArcSwap;
use pumpkin_config::{AdvancedConfiguration, BasicConfiguration, TelemetryConfig};
use pumpkin_data::packet::CURRENT_MC_VERSION;
use pumpkin_data::{damage::DamageType, dimension::Dimension, entity::EntityType};
use pumpkin_protocol::java::client::play::CPlayDisconnect;
use pumpkin_protocol::java::server::play::{ActionType, SAttack, SInteract};
use pumpkin_protocol::{
    ClientPacket, ConnectionState, RawPacket, packet::MultiVersionJavaPacket, ser::NetworkReadExt,
};
use pumpkin_util::{GameMode, math::vector2::Vector2, math::vector3::Vector3};
use pumpkin_world::chunk::ChunkData;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::UnboundedReceiver;

use crate::data::VanillaData;
use crate::entity::{Entity, EntityBase, passive::cow::CowEntity, player::Player};
use crate::net::java::{JavaClient, outgoing::OutgoingPacket, pending::PendingConnection};
use crate::net::{ClientPlatform, GameProfile, PacketRateLimiter, PlayerConfig};
use crate::plugin::EventHandler;
use crate::plugin::api::events::{
    EventPriority,
    entity::entity_damage::EntityDamageEvent,
    player::{
        player_interact_entity_event::PlayerInteractEntityEvent,
        player_interact_unknown_entity_event::PlayerInteractUnknownEntityEvent,
    },
};
use crate::server::Server;
use crate::world::World;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Observed {
    Unknown(i32, i32, ActionType, bool),
    Interact(i32, i32, ActionType, Option<Vector3<f64>>, bool),
    Damage(i32),
}

#[derive(Default)]
struct ObserveEvents {
    cancel: AtomicBool,
    events: Mutex<Vec<Observed>>,
}

impl EventHandler<PlayerInteractUnknownEntityEvent> for ObserveEvents {
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut PlayerInteractUnknownEntityEvent,
    ) -> futures::future::BoxFuture<'a, ()> {
        Box::pin(async move {
            self.events.lock().unwrap().push(Observed::Unknown(
                event.player.entity_id(),
                event.entity_id,
                event.action,
                event.player.get_entity().is_sneaking(),
            ));
            event.cancelled = self.cancel.load(Ordering::Relaxed);
        })
    }
}

impl EventHandler<PlayerInteractEntityEvent> for ObserveEvents {
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut PlayerInteractEntityEvent,
    ) -> futures::future::BoxFuture<'a, ()> {
        Box::pin(async move {
            self.events.lock().unwrap().push(Observed::Interact(
                event.player.entity_id(),
                event.target.get_entity().entity_id,
                event.action,
                event.target_position,
                event.sneaking,
            ));
            event.cancelled = self.cancel.load(Ordering::Relaxed);
        })
    }
}

impl EventHandler<EntityDamageEvent> for ObserveEvents {
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut EntityDamageEvent,
    ) -> futures::future::BoxFuture<'a, ()> {
        Box::pin(async move {
            self.events
                .lock()
                .unwrap()
                .push(Observed::Damage(event.entity_id));
        })
    }
}

struct AttackFixture {
    _directory: tempfile::TempDir,
    server: Arc<Server>,
    world: Arc<World>,
    player: Arc<Player>,
    observer: Arc<ObserveEvents>,
    packets: UnboundedReceiver<OutgoingPacket>,
    _peer: TcpStream,
}

impl AttackFixture {
    async fn new() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let basic = BasicConfiguration {
            default_level_name: directory.path().to_str().unwrap().to_owned(),
            allow_nether: false,
            allow_end: false,
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
        world
            .level
            .loaded_chunks
            .insert(Vector2::new(0, 0), ChunkData::empty_sync(0, 0));
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
            Arc::downgrade(&server),
        );
        pending.connection_state.store(ConnectionState::Play);
        let profile = GameProfile {
            id: uuid::Uuid::new_v4(),
            name: "Attacker".to_owned(),
            properties: ArcSwap::from_pointee(Vec::new()),
            profile_actions: None,
        };
        let mut client =
            JavaClient::from_pending(pending, profile.clone(), PlayerConfig::default());
        let packets = client.outgoing_packet_queue_recv.take().unwrap();
        let player = Arc::new(Player::new(
            Arc::new(ClientPlatform::Java(client)),
            profile,
            PlayerConfig::default(),
            &world,
            GameMode::Survival,
        ));
        player.set_client_loaded(true);
        player.get_entity().set_pos(Vector3::new(8.0, 100.0, 8.0));
        player.get_entity().on_ground.store(true, Ordering::Relaxed);
        player.last_attacked_ticks.store(100, Ordering::Relaxed);
        world.add_player(&player).unwrap();
        let observer = Arc::new(ObserveEvents::default());
        server.plugin_manager.register::<EntityDamageEvent, _>(
            observer.clone(),
            EventPriority::Normal,
            true,
        );
        server
            .plugin_manager
            .register::<PlayerInteractEntityEvent, _>(
                observer.clone(),
                EventPriority::Normal,
                true,
            );
        server
            .plugin_manager
            .register::<PlayerInteractUnknownEntityEvent, _>(
                observer.clone(),
                EventPriority::Normal,
                true,
            );
        Self {
            _directory: directory,
            server,
            world,
            player,
            observer,
            packets,
            _peer: peer,
        }
    }

    fn cow(&self) -> Arc<CowEntity> {
        let cow = CowEntity::new(Entity::new(
            self.world.clone(),
            Vector3::new(8.0, 100.0, 10.0),
            &EntityType::COW,
        ));
        self.world.add_entity_silent(cow.clone());
        cow
    }

    fn queue_attack(&self, cow: &CowEntity) {
        assert!(cow.get_living_entity().unwrap().health.load() > 0.0);
        assert!(
            self.world
                .get_entity_by_id(cow.get_entity().entity_id)
                .is_some()
        );
        let attack = SAttack {
            entity_id: cow.get_entity().entity_id.into(),
        };
        let mut payload = Vec::new();
        attack
            .write_packet_data(&mut payload, &CURRENT_MC_VERSION)
            .unwrap();
        self.player.inbound_packets.push(RawPacket {
            id: SAttack::to_id(CURRENT_MC_VERSION),
            payload: payload.into(),
        });
    }

    fn finish_death(&self, cow: &CowEntity) {
        let living = cow.get_living_entity().unwrap();
        assert!(cow.damage(cow, living.health.load(), DamageType::FALL));
        assert!(living.dead.load(Ordering::Relaxed));
        // Drive the real mob/living lifecycle through the normal death animation.
        for _ in 0..20 {
            cow.tick(cow, &self.server);
        }
        assert!(cow.get_entity().removed.load(Ordering::Relaxed));
        assert!(
            self.world
                .get_entity_by_id(cow.get_entity().entity_id)
                .is_none()
        );
        self.observer.events.lock().unwrap().clear();
    }

    fn has_disconnect(&mut self) -> bool {
        let mut disconnected = false;
        while let Ok(packet) = self.packets.try_recv() {
            if let OutgoingPacket::Data { data, .. } = packet {
                let mut bytes = data.as_ref();
                disconnected |=
                    bytes.get_var_int().unwrap().0 == CPlayDisconnect::to_id(CURRENT_MC_VERSION);
            }
        }
        disconnected
    }

    async fn close(self) {
        self.player.client.java().unwrap().close();
        self.world.players.store(Arc::new(Vec::new()));
        self.server.shutdown().await;
        self.world.level.world_portal.store(Arc::new(None));
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn queued_attack_survives_mob_death_before_handling() {
    let mut fixture = AttackFixture::new().await;
    let cow = fixture.cow();
    fixture.queue_attack(&cow);
    fixture.finish_death(&cow);
    fixture.player.process_inbound_packets();

    let closed = fixture.player.client.closed();
    let disconnected = fixture.has_disconnect();
    assert!(fixture.observer.events.lock().unwrap().is_empty());
    assert_eq!(
        fixture.player.last_attacked_ticks.load(Ordering::Relaxed),
        100
    );
    assert_eq!(cow.get_living_entity().unwrap().health.load(), 0.0);

    let next = fixture.cow();
    let health = next.get_living_entity().unwrap().health.load();
    fixture.queue_attack(&next);
    fixture.player.process_inbound_packets();
    let next_health = next.get_living_entity().unwrap().health.load();
    let events = std::mem::take(&mut *fixture.observer.events.lock().unwrap());
    let closed_after_control = fixture.player.client.closed();
    let disconnect_after_control = fixture.has_disconnect();
    fixture.close().await;

    assert!(
        !closed,
        "an attack queued for a living mob must not close after its removal"
    );
    assert!(
        !disconnected,
        "stale mob target must not emit a disconnect packet"
    );
    assert!(
        next_health < health,
        "the next valid queued attack must still deal damage"
    );
    assert_eq!(events, vec![Observed::Damage(next.get_entity().entity_id)]);
    assert!(!closed_after_control && !disconnect_after_control);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn legacy_stale_mob_attack_keeps_interaction_events_and_cancellation() {
    let mut fixture = AttackFixture::new().await;
    let cow = fixture.cow();
    // The retained legacy handler accepts Attack; the 26.3 wire uses SAttack instead.
    let mut attack = SInteract {
        entity_id: cow.get_entity().entity_id.into(),
        r#type: (ActionType::Attack as i32).into(),
        target_position: None,
        hand: None,
        sneaking: true,
    };
    fixture.finish_death(&cow);
    let player_id = fixture.player.entity_id();
    let unknown = Observed::Unknown(
        player_id,
        cow.get_entity().entity_id,
        ActionType::Attack,
        true,
    );
    for cancelled in [true, false] {
        fixture.observer.cancel.store(cancelled, Ordering::Relaxed);
        fixture.player.client.java().unwrap().handle_interact(
            &fixture.player,
            &attack,
            &fixture.server,
        );
        assert_eq!(*fixture.observer.events.lock().unwrap(), vec![unknown]);
        fixture.observer.events.lock().unwrap().clear();
    }
    let closed = fixture.player.client.closed();
    let disconnected = fixture.has_disconnect();
    assert_eq!(
        fixture.player.last_attacked_ticks.load(Ordering::Relaxed),
        100
    );

    let next = fixture.cow();
    attack.entity_id = next.get_entity().entity_id.into();
    let health = next.get_living_entity().unwrap().health.load();
    fixture.observer.cancel.store(true, Ordering::Relaxed);
    fixture.player.client.java().unwrap().handle_interact(
        &fixture.player,
        &attack,
        &fixture.server,
    );
    assert_eq!(next.get_living_entity().unwrap().health.load(), health);
    let interaction = Observed::Interact(
        player_id,
        next.get_entity().entity_id,
        ActionType::Attack,
        None,
        true,
    );
    assert_eq!(*fixture.observer.events.lock().unwrap(), vec![interaction]);
    fixture.observer.events.lock().unwrap().clear();
    fixture.observer.cancel.store(false, Ordering::Relaxed);
    fixture.player.client.java().unwrap().handle_interact(
        &fixture.player,
        &attack,
        &fixture.server,
    );
    assert!(next.get_living_entity().unwrap().health.load() < health);
    assert_eq!(
        *fixture.observer.events.lock().unwrap(),
        vec![interaction, Observed::Damage(next.get_entity().entity_id)]
    );
    fixture.close().await;

    assert!(
        !closed && !disconnected,
        "legacy stale mob attack must remain a no-op after its event"
    );
}
