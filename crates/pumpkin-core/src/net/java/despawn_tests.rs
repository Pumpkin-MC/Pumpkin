use std::sync::atomic::Ordering;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use arc_swap::ArcSwap;
use pumpkin_config::{AdvancedConfiguration, BasicConfiguration, TelemetryConfig};
use pumpkin_data::{dimension::Dimension, entity::EntityType, packet::CURRENT_MC_VERSION};
use pumpkin_protocol::ConnectionState;
use pumpkin_util::{Difficulty, GameMode, math::vector3::Vector3, text::TextComponent};

use super::JavaClient;
use crate::{
    data::VanillaData,
    entity::{
        Entity, EntityBase,
        mob::{Mob, creeper::CreeperEntity, zombie::zombie_villager::ZombieVillagerEntity},
        player::Player,
    },
    net::{ClientPlatform, GameProfile, PacketRateLimiter, PlayerConfig},
    server::Server,
    world::World,
};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

struct DespawnFixture {
    _directory: tempfile::TempDir,
    server: Arc<Server>,
    world: Arc<World>,
}

impl DespawnFixture {
    async fn new() -> TestResult<Self> {
        let directory = tempfile::tempdir()?;
        let basic = BasicConfiguration {
            default_level_name: directory
                .path()
                .to_str()
                .ok_or("non-UTF8 test directory")?
                .to_owned(),
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
        .await?;
        server.set_difficulty(Difficulty::Normal, true);
        let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
        Ok(Self {
            _directory: directory,
            server,
            world,
        })
    }

    fn player(&self, distance: f64, gamemode: GameMode) -> Arc<Player> {
        let profile = GameProfile {
            id: uuid::Uuid::new_v4(),
            name: "DespawnObserver".to_owned(),
            properties: ArcSwap::from_pointee(Vec::new()),
            profile_actions: None,
        };
        let (send, recv) = tokio::sync::mpsc::unbounded_channel();
        // Keep the real client queue, without sockets or a reader/writer task.
        let client = JavaClient {
            id: 1,
            version: CURRENT_MC_VERSION.into(),
            gameprofile: profile.clone(),
            config: ArcSwap::from_pointee(PlayerConfig::default()),
            server_address: String::new(),
            connection_state: ConnectionState::Play.into(),
            address: ([127, 0, 0, 1], 0).into(),
            brand: ArcSwap::from_pointee(None),
            player: ArcSwap::from_pointee(None),
            tasks: tokio_util::task::TaskTracker::new(),
            rt_handle: tokio::runtime::Handle::current(),
            close_token: tokio_util::sync::CancellationToken::new(),
            outgoing_packet_queue_send: send,
            outgoing_packet_queue_recv: Some(recv),
            pending_bytes: Arc::default(),
            network_writer: std::sync::Mutex::new(None),
            network_reader: std::sync::Mutex::new(None),
            wait_for_keep_alive: false.into(),
            received_movement_this_tick: false.into(),
            keep_alive_id: 0.into(),
            last_keep_alive_time: Instant::now().into(),
            last_packet_time: Instant::now().into(),
            pending_keep_alives: std::sync::Mutex::default(),
            packet_sequence: (-1).into(),
            packet_limiter: PacketRateLimiter::new(false, 0.0, 0.0),
            suspend_flushing: Arc::default(),
        };
        let player = Arc::new(Player::new(
            Arc::new(ClientPlatform::Java(client)),
            profile,
            PlayerConfig::default(),
            &self.world,
            gamemode,
        ));
        player
            .get_entity()
            .set_pos(Vector3::new(distance, 100.0, 0.0));
        player
    }

    fn creeper(&self) -> Arc<CreeperEntity> {
        CreeperEntity::new(Entity::new(
            self.world.clone(),
            Vector3::new(0.0, 100.0, 0.0),
            &EntityType::CREEPER,
        ))
    }

    fn observe_despawn<M: Mob + 'static>(
        &self,
        mob: &Arc<M>,
        players: Vec<Arc<Player>>,
    ) -> (bool, bool) {
        // Each fresh mob sees exactly this player snapshot; no world ticks or chunks are needed.
        self.world.players.store(Arc::new(players));
        self.world.add_entity_silent(mob.clone());
        let entity = mob.get_entity();
        assert!(!entity.is_removed());
        assert!(self.world.get_entity_by_id(entity.entity_id).is_some());

        mob.get_mob_entity().check_despawn(mob.as_ref());

        let outcome = (
            entity.is_removed(),
            self.world.get_entity_by_id(entity.entity_id).is_some(),
        );
        entity.remove();
        // Break the creeper ignition goal's owning Arc after recording the result.
        drop(
            mob.get_mob_entity()
                .goals_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clear(),
        );
        self.world.players.store(Arc::new(Vec::new()));
        outcome
    }

    async fn close(&self) {
        self.server.shutdown().await;
        self.world.level.world_portal.store(Arc::new(None));
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn distance_despawn_requires_an_eligible_player() -> TestResult {
    tokio::time::timeout(Duration::from_secs(60), despawn_cases()).await?
}

async fn despawn_cases() -> TestResult {
    let fixture = DespawnFixture::new().await?;
    // Stay outside the random 32..128 block branch in every case.
    let near = fixture.player(16.0, GameMode::Survival);
    let far = fixture.player(129.0, GameMode::Survival);
    let near_spectator = fixture.player(16.0, GameMode::Spectator);
    let far_spectator = fixture.player(129.0, GameMode::Spectator);
    let mut observed = Vec::new();
    for (name, players) in [
        ("no players", vec![]),
        ("spectator only", vec![far_spectator]),
        ("near eligible", vec![near.clone()]),
        ("far eligible", vec![far.clone()]),
        ("far then near", vec![far.clone(), near.clone()]),
        ("near then far", vec![near, far.clone()]),
        (
            "near spectator, far eligible",
            vec![near_spectator, far.clone()],
        ),
    ] {
        observed.push((name, fixture.observe_despawn(&fixture.creeper(), players)));
    }

    let persistent = fixture.creeper();
    persistent
        .mob_entity
        .persistence_required
        .store(true, Ordering::Relaxed);
    let named = fixture.creeper();
    named
        .get_entity()
        .set_custom_name(TextComponent::text("Keep me"));
    for (name, creeper) in [("persistent", persistent), ("custom named", named)] {
        observed.push((name, fixture.observe_despawn(&creeper, vec![far.clone()])));
    }

    for (name, converting) in [("converting veto", true), ("nonconverting allows", false)] {
        let zombie = ZombieVillagerEntity::new(Entity::new(
            fixture.world.clone(),
            Vector3::new(0.0, 100.0, 0.0),
            &EntityType::ZOMBIE_VILLAGER,
        ));
        if converting {
            zombie.start_converting(None, 3600);
        }
        assert!(
            !zombie
                .get_mob_entity()
                .persistence_required
                .load(Ordering::Relaxed),
            "conversion must exercise the veto, not persistence"
        );
        observed.push((name, fixture.observe_despawn(&zombie, vec![far.clone()])));
    }
    fixture.close().await;

    // Compare after all eleven direct checks and cleanup, including on the failing base.
    assert_eq!(
        observed,
        [
            ("no players", (false, true)),
            ("spectator only", (false, true)),
            ("near eligible", (false, true)),
            ("far eligible", (true, false)),
            ("far then near", (false, true)),
            ("near then far", (false, true)),
            ("near spectator, far eligible", (true, false)),
            ("persistent", (false, true)),
            ("custom named", (false, true)),
            ("converting veto", (false, true)),
            ("nonconverting allows", (true, false)),
        ],
        "(removed, present in world) after one distance-despawn check"
    );
    Ok(())
}
