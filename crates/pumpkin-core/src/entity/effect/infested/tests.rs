use std::sync::atomic::Ordering::Relaxed;
use std::sync::{Arc, Mutex, RwLock, Weak};

use arc_swap::ArcSwap;
use crossbeam::atomic::AtomicCell;
use pumpkin_config::{AdvancedConfiguration, BasicConfiguration, TelemetryConfig};
use pumpkin_data::Block;
use pumpkin_data::dimension::Dimension;
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::potion::{Effect, Potion};
use pumpkin_protocol::ConnectionState;
use pumpkin_util::GameMode;
use pumpkin_util::Hand;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::chunk::ChunkData;
use tokio::net::{TcpListener, TcpStream};

use super::*;
use crate::data::VanillaData;
use crate::entity::{EntityBase, player::Player};
use crate::net::java::{JavaClient, pending::PendingConnection};
use crate::net::{ClientPlatform, GameProfile, PacketRateLimiter, PlayerConfig};
use crate::plugin::EventHandler;
use crate::plugin::api::events::{
    EventPriority, entity::entity_damage::EntityDamageEvent,
    entity::entity_damage_by_block::EntityDamageByBlockEvent,
    entity::entity_damage_by_entity::EntityDamageByEntityEvent,
    entity::entity_death::EntityDeathEvent,
};
use crate::server::Server;
use crate::world::World;

#[derive(Debug, PartialEq)]
enum Observed {
    Damage(f32, DamageType),
    ByEntity(f32, i32, String),
    ByBlock(f32, Option<BlockPos>, String),
    Death,
    Hurt {
        amplifier: u8,
        damage_type: DamageType,
        amount: f32,
        health: f32,
        absorption: f32,
        dead: bool,
        infested: bool,
    },
}

#[derive(Clone, Copy, PartialEq)]
enum Policy {
    Normal,
    CancelDamage,
    CancelEntity,
    CancelBlock,
    Modify,
}

struct Observer {
    entity_id: i32,
    policy: AtomicCell<Policy>,
    events: Mutex<Vec<Observed>>,
}

// Only the registered fixture UUID is observed; the actual Infested handler still runs.
static OBSERVERS: Mutex<Vec<(Uuid, Weak<Observer>)>> = Mutex::new(Vec::new());

pub(super) fn observe_hurt(
    living: &LivingEntity,
    amplifier: u8,
    damage_type: &DamageType,
    amount: f32,
) {
    let observer = OBSERVERS
        .lock()
        .unwrap()
        .iter()
        .find(|(uuid, _)| *uuid == living.entity.entity_uuid)
        .and_then(|(_, observer)| observer.upgrade());
    let Some(observer) = observer else {
        return;
    };
    let infested = living
        .active_effects
        .try_lock()
        .expect("hurt callbacks must not hold the effects mutex")
        .contains_key(&StatusEffect::INFESTED);
    observer.events.lock().unwrap().push(Observed::Hurt {
        amplifier,
        damage_type: *damage_type,
        amount,
        health: living.health.load(),
        absorption: living.absorption.load(),
        dead: living.dead.load(Relaxed),
        infested,
    });
}

impl Observer {
    fn take(&self) -> Vec<Observed> {
        std::mem::take(&mut *self.events.lock().unwrap())
    }
}

impl EventHandler<EntityDamageEvent> for Observer {
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut EntityDamageEvent,
    ) -> futures::future::BoxFuture<'a, ()> {
        Box::pin(async move {
            if event.entity_id == self.entity_id {
                self.events
                    .lock()
                    .unwrap()
                    .push(Observed::Damage(event.damage, event.damage_type));
                event.cancelled = self.policy.load() == Policy::CancelDamage;
                if self.policy.load() == Policy::Modify {
                    event.damage = 4.0;
                }
            }
        })
    }
}

impl EventHandler<EntityDamageByEntityEvent> for Observer {
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut EntityDamageByEntityEvent,
    ) -> futures::future::BoxFuture<'a, ()> {
        Box::pin(async move {
            if event.entity_id == self.entity_id {
                self.events.lock().unwrap().push(Observed::ByEntity(
                    event.damage,
                    event.damager_id,
                    event.cause.clone(),
                ));
                event.cancelled = self.policy.load() == Policy::CancelEntity;
                if self.policy.load() == Policy::Modify {
                    event.damage = 3.0;
                }
            }
        })
    }
}

impl EventHandler<EntityDamageByBlockEvent> for Observer {
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut EntityDamageByBlockEvent,
    ) -> futures::future::BoxFuture<'a, ()> {
        Box::pin(async move {
            if event.entity_id == self.entity_id {
                self.events.lock().unwrap().push(Observed::ByBlock(
                    event.damage,
                    event.damager_pos,
                    event.cause.clone(),
                ));
                event.cancelled = self.policy.load() == Policy::CancelBlock;
            }
        })
    }
}

impl EventHandler<EntityDeathEvent> for Observer {
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut EntityDeathEvent,
    ) -> futures::future::BoxFuture<'a, ()> {
        Box::pin(async move {
            if event.entity_id == self.entity_id {
                self.events.lock().unwrap().push(Observed::Death);
            }
        })
    }
}

struct Fixture {
    _directory: tempfile::TempDir,
    server: Arc<Server>,
    world: Arc<World>,
    player: Arc<Player>,
    attacker: Arc<dyn EntityBase>,
    observer: Arc<Observer>,
    _peer: TcpStream,
}

impl Drop for Fixture {
    fn drop(&mut self) {
        OBSERVERS
            .lock()
            .unwrap()
            .retain(|(uuid, _)| *uuid != self.player.get_entity().entity_uuid);
    }
}

impl Fixture {
    #[expect(
        clippy::too_many_lines,
        reason = "Keep the real server/player fixture together"
    )]
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
        let chunk = ChunkData::empty_sync(0, 0);
        chunk.set_block_absolute_y(8, 99, 8, Block::STONE.default_state.id);
        world
            .level
            .loaded_chunks
            .insert(BlockPos::new(8, 100, 8).chunk_position(), chunk);

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
            id: Uuid::new_v4(),
            name: "Infested".to_owned(),
            properties: ArcSwap::from_pointee(Vec::new()),
            profile_actions: None,
        };
        let client = JavaClient::from_pending(pending, profile.clone(), PlayerConfig::default());
        let player = Arc::new(Player::new(
            Arc::new(ClientPlatform::Java(client)),
            profile,
            PlayerConfig::default(),
            &world,
            GameMode::Survival,
        ));
        player.set_client_loaded(true);
        player.get_entity().set_pos(Vector3::new(8.5, 100.0, 8.5));
        world.add_player(&player).unwrap();
        let attacker = from_type(
            &EntityType::ZOMBIE,
            Vector3::new(8.5, 100.0, 10.5),
            &world,
            Uuid::new_v4(),
        );
        assert!(world.spawn_entity(attacker.clone()));
        let observer = Arc::new(Observer {
            entity_id: player.entity_id(),
            policy: AtomicCell::new(Policy::Normal),
            events: Mutex::new(Vec::new()),
        });
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
        server
            .plugin_manager
            .register::<EntityDamageByBlockEvent, _>(observer.clone(), EventPriority::Normal, true);
        server.plugin_manager.register::<EntityDeathEvent, _>(
            observer.clone(),
            EventPriority::Normal,
            true,
        );
        OBSERVERS
            .lock()
            .unwrap()
            .push((player.get_entity().entity_uuid, Arc::downgrade(&observer)));
        player.add_effect(Effect {
            effect_type: &StatusEffect::INFESTED,
            duration: 600,
            amplifier: 2,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        });
        Self {
            _directory: directory,
            server,
            world,
            player,
            attacker,
            observer,
            _peer: peer,
        }
    }

    fn attack(&self, amount: f32) -> bool {
        self.player.damage_with_context(
            &*self.player,
            amount,
            DamageType::MOB_ATTACK,
            Some(self.attacker.get_entity().pos.load()),
            Some(&*self.attacker),
            Some(&*self.attacker),
        )
    }

    fn attack_events(&self, input: f32, secondary: f32) -> Vec<Observed> {
        vec![
            Observed::Damage(input, DamageType::MOB_ATTACK),
            Observed::ByEntity(
                secondary,
                self.attacker.get_entity().entity_id,
                format!("{:?}", DamageType::MOB_ATTACK),
            ),
        ]
    }

    fn check_rejections(&self) {
        let living = &self.player.living_entity;
        assert!(self.player.set_gamemode(GameMode::Creative));
        assert!(!self.attack(1.0));
        assert!(self.observer.take().is_empty());
        assert!(self.player.set_gamemode(GameMode::Survival));

        self.observer.policy.store(Policy::CancelDamage);
        assert!(!self.attack(1.0));
        assert_eq!(
            self.observer.take(),
            [Observed::Damage(1.0, DamageType::MOB_ATTACK)]
        );
        self.observer.policy.store(Policy::CancelEntity);
        assert!(!self.attack(1.0));
        assert_eq!(self.observer.take(), self.attack_events(1.0, 1.0));

        self.observer.policy.store(Policy::CancelBlock);
        let cactus_pos = BlockPos::new(8, 100, 9);
        assert!(!self.player.damage_with_context(
            &*self.player,
            1.0,
            DamageType::CACTUS,
            Some(cactus_pos.to_centered_f64()),
            None,
            None,
        ));
        assert_eq!(
            self.observer.take(),
            [
                Observed::Damage(1.0, DamageType::CACTUS),
                Observed::ByBlock(1.0, Some(cactus_pos), format!("{:?}", DamageType::CACTUS)),
            ]
        );
        self.observer.policy.store(Policy::Normal);

        living.add_effect(Potion::FIRE_RESISTANCE.effects[0].clone());
        assert!(!self.player.damage(&*self.player, 1.0, DamageType::ON_FIRE));
        assert_eq!(
            self.observer.take(),
            [Observed::Damage(1.0, DamageType::ON_FIRE)]
        );
        living.remove_effect(&StatusEffect::FIRE_RESISTANCE);

        let shield = ItemStack::new(1, &Item::SHIELD);
        let use_time = shield.get_max_use_time() - 5;
        self.player
            .inventory()
            .set_stack_in_hand(Hand::Right, shield.clone());
        living.set_active_hand(Hand::Right, shield, use_time);
        self.player.get_entity().yaw.store(0.0);
        assert!(living.is_blocking());
        assert!(!self.attack(1.0));
        assert_eq!(self.observer.take(), self.attack_events(1.0, 1.0));
        living.clear_active_hand();
        self.player
            .inventory()
            .set_stack_in_hand(Hand::Right, ItemStack::EMPTY.clone());
        assert_eq!(living.health.load(), 20.0);
    }

    fn check_accepted_and_lethal(&self) {
        let living = &self.player.living_entity;
        living.add_effect(Effect {
            effect_type: &StatusEffect::ABSORPTION,
            duration: 600,
            amplifier: 0,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        });
        assert_eq!(living.absorption.load(), 4.0);
        self.observer.policy.store(Policy::Modify);
        assert!(self.attack(8.0));
        let mut expected = self.attack_events(8.0, 4.0);
        expected.push(Observed::Hurt {
            amplifier: 2,
            damage_type: DamageType::MOB_ATTACK,
            amount: 3.0,
            health: 20.0,
            absorption: 1.0,
            dead: false,
            infested: true,
        });
        assert_eq!(
            self.observer.take(),
            expected,
            "accepted hit must call the real Infested handler after absorption"
        );

        self.observer.policy.store(Policy::Normal);
        assert!(!self.attack(2.0));
        assert_eq!(self.observer.take(), self.attack_events(2.0, 2.0));
        assert!(self.attack(5.0));
        let mut expected = self.attack_events(5.0, 5.0);
        expected.push(Observed::Hurt {
            amplifier: 2,
            damage_type: DamageType::MOB_ATTACK,
            amount: 5.0,
            health: 19.0,
            absorption: 0.0,
            dead: false,
            infested: true,
        });
        assert_eq!(
            self.observer.take(),
            expected,
            "callback receives the hit amount, not the cooldown delta or health loss"
        );

        assert!(self.attack(30.0));
        let mut expected = self.attack_events(30.0, 30.0);
        expected.push(Observed::Death);
        // Pumpkin clears effects in on_death, earlier than vanilla's later removal.
        expected.push(Observed::Hurt {
            amplifier: 2,
            damage_type: DamageType::MOB_ATTACK,
            amount: 30.0,
            health: 0.0,
            absorption: 0.0,
            dead: true,
            infested: false,
        });
        assert_eq!(
            self.observer.take(),
            expected,
            "lethal hit must not lose the callback during death clearing"
        );
        assert!(living.active_effects.lock().unwrap().is_empty());
        assert!(!self.attack(1.0));
        assert!(self.observer.take().is_empty());
    }

    async fn close(self) {
        self.player.client.java().unwrap().close();
        self.world.players.store(Arc::new(Vec::new()));
        self.server.shutdown().await;
        self.world.level.world_portal.store(Arc::new(None));
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn infested_hurt_dispatch_preserves_damage_lifecycle() {
    let fixture = Fixture::new().await;
    // Shut down the real world even when the behavioral RED assertion fails.
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        fixture.check_rejections();
        fixture.check_accepted_and_lethal();
    }));
    fixture.close().await;
    if let Err(panic) = result {
        std::panic::resume_unwind(panic);
    }
}
