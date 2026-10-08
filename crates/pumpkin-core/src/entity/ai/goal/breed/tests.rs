use std::sync::{
    Arc, Barrier, RwLock,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};

use arc_swap::ArcSwap;
use futures::future::BoxFuture;
use pumpkin_config::{AdvancedConfiguration, BasicConfiguration, TelemetryConfig};
use pumpkin_data::{
    advancement::Advancement, dimension::Dimension, entity::EntityType, item::Item,
    item_stack::ItemStack, statistic::CustomStatistic,
};
use pumpkin_protocol::ConnectionState;
use pumpkin_util::{GameMode, math::vector3::Vector3};
use tokio::net::{TcpListener, TcpStream};

use super::*;
use crate::{
    data::VanillaData,
    entity::{Entity, passive::cow::CowEntity, passive::pig::PigEntity, player::Player},
    net::{
        ClientPlatform, GameProfile, PacketRateLimiter, PlayerConfig,
        java::{JavaClient, pending::PendingConnection},
    },
    plugin::{
        EventHandler,
        api::{EventPriority, events::entity::entity_spawn::EntitySpawnEvent},
    },
    server::Server,
    world::World,
};

struct Fixture {
    server: Arc<Server>,
    player: Arc<Player>,
    world: Arc<World>,
    _peer: TcpStream,
    _directory: tempfile::TempDir,
}

impl Fixture {
    async fn new() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let basic = BasicConfiguration {
            default_level_name: directory.path().to_str().unwrap().to_string(),
            allow_nether: false,
            allow_end: false,
            allow_chat_reports: false,
            use_favicon: false,
            ..Default::default()
        };
        let mut advanced = AdvancedConfiguration::default();
        advanced.networking.bedrock.online_mode = false;
        let data = VanillaData {
            banned_ip_list: RwLock::default(),
            banned_player_list: RwLock::default(),
            operator_config: RwLock::default(),
            user_cache: RwLock::default(),
            whitelist_config: RwLock::default(),
        };
        let server = Server::new(basic, advanced, TelemetryConfig::default(), data, vec![])
            .await
            .unwrap();
        let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
        // These goal tests need entity tracking, but not chunk generation or a world tick loop.
        world.level.shutdown().await;
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let peer = TcpStream::connect(listener.local_addr().unwrap())
            .await
            .unwrap();
        let (stream, address) = listener.accept().await.unwrap();
        let pending = PendingConnection::new(
            stream,
            address,
            0,
            PacketRateLimiter::from_config(&server.advanced_config.networking.java.packet_limiter),
            Arc::downgrade(&server),
        );
        pending.connection_state.store(ConnectionState::Play);
        let profile = GameProfile {
            id: Uuid::new_v4(),
            name: "Breeder".to_string(),
            properties: ArcSwap::from_pointee(Vec::new()),
            profile_actions: None,
        };
        let mut client =
            JavaClient::from_pending(pending, profile.clone(), PlayerConfig::default());
        client.start_outgoing_packet_task();
        let player = Arc::new(Player::new(
            Arc::new(ClientPlatform::Java(client)),
            profile,
            PlayerConfig::default(),
            &world,
            GameMode::Survival,
        ));
        player.advancements.lock().unwrap().set_player(&player);
        world.players.store(Arc::new(vec![player.clone()]));
        Self {
            server,
            player,
            world,
            _peer: peer,
            _directory: directory,
        }
    }

    fn parents(&self, entity_type: &'static EntityType, count: usize) -> Vec<Arc<dyn Mob>> {
        self.world.entities.store(Arc::new(Vec::new()));
        self.player.set_custom_stat(CustomStatistic::AnimalsBred, 0);
        let mut parents = Vec::new();
        for index in 0..count {
            let entity = Entity::new(
                self.world.clone(),
                Vector3::new(index as f64, 64.0, 0.0),
                entity_type,
            );
            let parent: Arc<dyn Mob> = if entity_type == &EntityType::COW {
                CowEntity::new(entity)
            } else {
                PigEntity::new(entity)
            };
            self.world.add_entity_silent(parent.clone());
            let food = if entity_type == &EntityType::COW {
                &Item::WHEAT
            } else {
                &Item::CARROT
            };
            let mut stack = ItemStack::new(1, food);
            assert!(parent.mob_interact(&self.player, &mut stack));
            assert!(parent.is_in_love());
            parents.push(parent);
        }
        parents
    }

    fn assert_completion(&self, parents: &[Arc<dyn Mob>], babies: usize) {
        let entities = self.world.entities.load();
        let offspring: Vec<_> = entities
            .iter()
            .filter(|entity| {
                entity.get_entity().entity_type == parents[0].get_entity().entity_type
                    && entity.get_entity().age.load(Ordering::Relaxed) < 0
            })
            .collect();
        assert_eq!(offspring.len(), babies, "offspring count");
        for baby in offspring {
            assert_eq!(baby.get_entity().age.load(Ordering::Relaxed), -24000);
        }
        assert_eq!(self.player.get_custom_stat(CustomStatistic::AnimalsBred), 1);
        assert_eq!(parents.iter().filter(|p| !p.is_in_love()).count(), 2);
        assert_eq!(
            parents
                .iter()
                .filter(|p| {
                    p.get_mob_entity().breeding_cooldown.load(Ordering::Relaxed) == 6000
                })
                .count(),
            2
        );
        assert!(
            entities
                .iter()
                .any(|entity| entity.get_entity().entity_type == &EntityType::EXPERIENCE_ORB)
        );
        let mut advancements = self.player.advancements.lock().unwrap();
        assert!(
            advancements
                .progress
                .get_mut_or_start_progress(Advancement::HUSBANDRY_BREED_AN_ANIMAL)
                .is_done()
        );
    }
}

fn ready_goal(parent: &dyn Mob, mate: &Arc<dyn Mob>) -> BreedGoal {
    let mut goal = BreedGoal::new(1.0);
    assert!(goal.can_start(parent));
    assert_eq!(
        goal.mate.as_ref().unwrap().get_entity().entity_uuid,
        mate.get_entity().entity_uuid
    );
    goal.start(parent);
    for _ in 0..goal.get_tick_count(60) - 1 {
        assert!(goal.should_continue(parent));
        goal.tick(parent);
    }
    *goal
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn stale_continuations_complete_once_for_cows_and_pigs() {
    let fixture = Fixture::new().await;
    for entity_type in [&EntityType::COW, &EntityType::PIG] {
        let parents = fixture.parents(entity_type, 2);
        let mut first = ready_goal(parents[0].as_ref(), &parents[1]);
        let mut second = ready_goal(parents[1].as_ref(), &parents[0]);
        // Both selectors decide to continue before either parent's completion tick runs.
        assert!(first.should_continue(parents[0].as_ref()));
        assert!(second.should_continue(parents[1].as_ref()));
        first.tick(parents[0].as_ref());
        second.tick(parents[1].as_ref());
        fixture.assert_completion(&parents, 1);
        first.tick(parents[0].as_ref());
        fixture.assert_completion(&parents, 1);
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn concurrent_pair_and_shared_parent_complete_once() {
    let fixture = Fixture::new().await;
    for entity_type in [&EntityType::COW, &EntityType::PIG] {
        for count in [2, 3] {
            let parents = fixture.parents(entity_type, count);
            let mut first = ready_goal(parents[0].as_ref(), &parents[1]);
            let last = count - 1;
            let other_mate = usize::from(count != 2);
            let mut second = ready_goal(parents[last].as_ref(), &parents[other_mate]);
            let barrier = Barrier::new(2);
            std::thread::scope(|scope| {
                let first_thread = scope.spawn(|| {
                    assert!(first.should_continue(parents[0].as_ref()));
                    barrier.wait();
                    first.tick(parents[0].as_ref());
                });
                let second_thread = scope.spawn(|| {
                    assert!(second.should_continue(parents[last].as_ref()));
                    barrier.wait();
                    second.tick(parents[last].as_ref());
                });
                first_thread.join().unwrap();
                second_thread.join().unwrap();
            });
            fixture.assert_completion(&parents, 1);
            if count == 3 {
                assert_eq!(
                    parents
                        .iter()
                        .filter(|p| p.is_in_love() && p.is_breeding_ready())
                        .count(),
                    1
                );
            }
        }
    }
}

struct SpawnObserver {
    cancel: AtomicBool,
    babies: AtomicUsize,
}

impl EventHandler<EntitySpawnEvent> for SpawnObserver {
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut EntitySpawnEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            if event.entity_type == EntityType::COW.id.to_string()
                || event.entity_type == EntityType::PIG.id.to_string()
            {
                let first_callback = self.babies.fetch_add(1, Ordering::Relaxed) == 0;
                let parents: Vec<_> = event
                    .world
                    .entities
                    .load()
                    .iter()
                    .filter(|e| e.get_entity().entity_type.id.to_string() == event.entity_type)
                    .cloned()
                    .collect();
                assert_eq!(parents.len(), 2);
                assert!(
                    parents
                        .iter()
                        .all(|p| !p.is_in_love() && !p.is_breeding_ready())
                );
                if first_callback {
                    // A plugin may re-enter completion: the claim must already be consumed,
                    // and its lock must have been released before firing this event.
                    if let Some(parent) = parents[0].cast_any().downcast_ref::<CowEntity>() {
                        BreedGoal::breed(parent, parents[1].as_ref());
                    } else {
                        let parent = parents[0].cast_any().downcast_ref::<PigEntity>().unwrap();
                        BreedGoal::breed(parent, parents[1].as_ref());
                    }
                }
                event.cancelled = self.cancel.load(Ordering::Relaxed);
            }
        })
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn one_goal_completes_and_spawn_cancellation_is_preserved() {
    let fixture = Fixture::new().await;
    let observer = Arc::new(SpawnObserver {
        cancel: AtomicBool::new(false),
        babies: AtomicUsize::new(0),
    });
    fixture
        .server
        .plugin_manager
        .register(observer.clone(), EventPriority::Normal, true);
    for entity_type in [&EntityType::COW, &EntityType::PIG] {
        for cancel in [false, true] {
            observer.cancel.store(cancel, Ordering::Relaxed);
            observer.babies.store(0, Ordering::Relaxed);
            let parents = fixture.parents(entity_type, 2);
            // Only the higher-ID parent's goal runs; no fixed partner may be required to win.
            let mut goal = ready_goal(parents[1].as_ref(), &parents[0]);
            assert!(goal.should_continue(parents[1].as_ref()));
            goal.tick(parents[1].as_ref());
            fixture.assert_completion(&parents, usize::from(!cancel));
            assert_eq!(observer.babies.load(Ordering::Relaxed), 1);
            goal.tick(parents[1].as_ref());
            assert_eq!(
                observer.babies.load(Ordering::Relaxed),
                1,
                "no retry after cancellation"
            );
        }
    }
}
