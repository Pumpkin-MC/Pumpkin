use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};

use arc_swap::ArcSwap;
use pumpkin_config::{AdvancedConfiguration, BasicConfiguration, TelemetryConfig};
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::dimension::Dimension;
use pumpkin_data::entity::{EntityPose, EntityType};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::packet::CURRENT_MC_VERSION;
use pumpkin_data::particle::Particle;
use pumpkin_data::sound::Sound;
use pumpkin_protocol::java::client::play::{CParticle, CSoundEffect};
use pumpkin_protocol::{ConnectionState, packet::MultiVersionJavaPacket, ser::NetworkReadExt};
use pumpkin_util::GameMode;
use pumpkin_util::math::{vector2::Vector2, vector3::Vector3};
use pumpkin_world::chunk::ChunkData;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::UnboundedReceiver;

use super::{JavaClient, outgoing::OutgoingPacket, pending::PendingConnection};
use crate::data::VanillaData;
use crate::entity::{Entity, EntityBase, passive::cow::CowEntity, player::Player};
use crate::net::{ClientPlatform, GameProfile, PacketRateLimiter, PlayerConfig};
use crate::plugin::EventHandler;
use crate::plugin::api::events::{
    EventPriority,
    entity::{
        entity_damage::EntityDamageEvent, entity_damage_by_entity::EntityDamageByEntityEvent,
    },
};
use crate::server::Server;
use crate::world::World;

#[derive(Default)]
struct ObserveDamage {
    cancel: AtomicBool,
    events: Mutex<Vec<(&'static str, i32)>>,
}

impl EventHandler<EntityDamageEvent> for ObserveDamage {
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut EntityDamageEvent,
    ) -> futures::future::BoxFuture<'a, ()> {
        Box::pin(async move {
            self.events
                .lock()
                .unwrap()
                .push(("damage", event.entity_id));
            event.cancelled = self.cancel.load(Ordering::Relaxed);
        })
    }
}

impl EventHandler<EntityDamageByEntityEvent> for ObserveDamage {
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut EntityDamageByEntityEvent,
    ) -> futures::future::BoxFuture<'a, ()> {
        Box::pin(async move {
            self.events
                .lock()
                .unwrap()
                .push(("by_entity", event.entity_id));
        })
    }
}

struct SweepFixture {
    _directory: tempfile::TempDir,
    server: Arc<Server>,
    world: Arc<World>,
    player: Arc<Player>,
    observer: Arc<ObserveDamage>,
    packets: UnboundedReceiver<OutgoingPacket>,
    _peer: TcpStream,
}

impl SweepFixture {
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
            name: "Sweeper".to_owned(),
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
        world.add_player(&player).unwrap();
        let sword = ItemStack::new(1, &Item::IRON_SWORD);
        player.inventory().set_held_item(sword.clone());
        player
            .living_entity
            .send_equipment_changes(&[(EquipmentSlot::MAIN_HAND, sword)]);
        let observer = Arc::new(ObserveDamage::default());
        server.plugin_manager.register::<EntityDamageEvent, _>(
            observer.clone(),
            EventPriority::Normal,
            true,
        );
        server
            .plugin_manager
            .register::<EntityDamageByEntityEvent, _>(
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

    fn cow(&self, pos: Vector3<f64>) -> Arc<dyn EntityBase> {
        let cow = CowEntity::new(Entity::new(self.world.clone(), pos, &EntityType::COW));
        cow.get_entity().yaw.store(-90.0);
        self.world.add_entity_silent(cow.clone());
        cow
    }

    fn effects(&mut self) -> (Vec<Vector3<f64>>, Vec<i32>) {
        let mut particles = Vec::new();
        let mut sounds = Vec::new();
        while let Ok(packet) = self.packets.try_recv() {
            let OutgoingPacket::Data { data, .. } = packet else {
                continue;
            };
            let mut bytes = data.as_ref();
            let id = bytes.get_var_int().unwrap().0;
            if id == CSoundEffect::to_id(CURRENT_MC_VERSION) {
                sounds.push(bytes.get_var_int().unwrap().0 - 1);
            } else if id == CParticle::to_id(CURRENT_MC_VERSION) {
                // Read the actual 26.3 wire layout, independently of CParticle::read.
                if bytes.get_var_int().unwrap().0 != Particle::SweepAttack as i32 {
                    continue;
                }
                assert!(sounds.contains(&(Sound::EntityPlayerAttackSweep as i32)));
                assert!(!bytes.get_bool().unwrap());
                assert!(!bytes.get_bool().unwrap());
                particles.push(Vector3::new(
                    bytes.get_f64_be().unwrap(),
                    bytes.get_f64_be().unwrap(),
                    bytes.get_f64_be().unwrap(),
                ));
                // Preserve offsets and per-axis speeds, count and randomization.
                for _ in 0..6 {
                    assert_eq!(bytes.get_f32_be().unwrap(), 0.0);
                }
                assert_eq!(bytes.get_var_int().unwrap().0, 0);
                assert_eq!(bytes.get_var_int().unwrap().0, 0);
                assert!(bytes.is_empty());
            }
        }
        (particles, sounds)
    }

    async fn close(self) {
        self.player.client.java().unwrap().close();
        self.world.players.store(Arc::new(Vec::new()));
        self.server.shutdown().await;
        self.world.level.world_portal.store(Arc::new(None));
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sweeping_attack_serializes_attacker_anchored_particle() {
    let mut fixture = SweepFixture::new().await;
    // Vanilla Player.doSweepAttack anchors to the attacker; crouching also changes getY(0.5).
    for (attacker, victim_pos, yaw, pose, expected) in [
        (
            Vector3::new(8.0, 100.0, 8.0),
            Vector3::new(8.0, 101.0, 10.0),
            0.0,
            EntityPose::Standing,
            Vector3::new(8.0, 100.9, 9.0),
        ),
        (
            Vector3::new(8.0, 104.0, 8.0),
            Vector3::new(6.0, 103.0, 8.0),
            90.0,
            EntityPose::Crouching,
            Vector3::new(7.0, 104.75, 8.0),
        ),
    ] {
        fixture.player.get_entity().set_pos(attacker);
        fixture.player.get_entity().set_pose(pose);
        fixture.player.get_entity().yaw.store(yaw);
        fixture
            .player
            .last_attacked_ticks
            .store(100, Ordering::Relaxed);
        let victim = fixture.cow(victim_pos);
        // Inside the victim's sweep area, but outside an attacker-centred search box.
        let nearby = fixture.cow(victim_pos + Vector3::new(0.5, 0.0, 0.0));
        let health = victim.get_living_entity().unwrap().health.load();
        let nearby_health = nearby.get_living_entity().unwrap().health.load();
        fixture.effects();
        fixture.observer.events.lock().unwrap().clear();

        fixture.player.attack(&victim);

        assert!(victim.get_living_entity().unwrap().health.load() < health);
        assert_eq!(
            nearby.get_living_entity().unwrap().health.load(),
            nearby_health - 1.0
        );
        assert_eq!(
            *fixture.observer.events.lock().unwrap(),
            vec![
                ("damage", victim.get_entity().entity_id),
                ("by_entity", victim.get_entity().entity_id),
                ("damage", nearby.get_entity().entity_id),
                ("by_entity", nearby.get_entity().entity_id),
            ]
        );
        let (particles, _) = fixture.effects();
        assert_eq!(particles.len(), 1);
        let actual = particles[0];
        assert!(
            (actual.x - expected.x).abs() < 1.0e-6
                && (actual.y - expected.y).abs() < 1.0e-6
                && (actual.z - expected.z).abs() < 1.0e-6,
            "sweep must originate at attacker: got {actual:?}, expected {expected:?}"
        );
        fixture.world.remove_entity(victim.as_ref());
        fixture.world.remove_entity(nearby.as_ref());
    }
    fixture.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cancelled_and_uncharged_attacks_do_not_emit_sweep_particles() {
    let mut fixture = SweepFixture::new().await;
    let victim = fixture.cow(Vector3::new(8.0, 100.0, 10.0));
    let health = victim.get_living_entity().unwrap().health.load();
    fixture.effects();
    fixture.observer.cancel.store(true, Ordering::Relaxed);
    fixture
        .player
        .last_attacked_ticks
        .store(100, Ordering::Relaxed);

    fixture.player.attack(&victim);

    assert_eq!(victim.get_living_entity().unwrap().health.load(), health);
    assert_eq!(
        *fixture.observer.events.lock().unwrap(),
        vec![("damage", victim.get_entity().entity_id)]
    );
    let (particles, sounds) = fixture.effects();
    assert!(particles.is_empty());
    assert!(sounds.contains(&(Sound::EntityPlayerAttackNodamage as i32)));
    assert!(!sounds.contains(&(Sound::EntityPlayerAttackSweep as i32)));

    fixture.observer.cancel.store(false, Ordering::Relaxed);
    fixture.observer.events.lock().unwrap().clear();
    // The cancelled attack reset the cooldown; this hit must remain a weak attack.
    fixture.player.attack(&victim);

    assert!(victim.get_living_entity().unwrap().health.load() < health);
    assert_eq!(
        *fixture.observer.events.lock().unwrap(),
        vec![
            ("damage", victim.get_entity().entity_id),
            ("by_entity", victim.get_entity().entity_id),
        ]
    );
    let (particles, sounds) = fixture.effects();
    assert!(particles.is_empty());
    assert!(sounds.contains(&(Sound::EntityPlayerAttackWeak as i32)));
    assert!(!sounds.contains(&(Sound::EntityPlayerAttackSweep as i32)));
    fixture.close().await;
}
