use std::sync::{Arc, RwLock, atomic::Ordering};

use arc_swap::ArcSwap;
use pumpkin_config::{AdvancedConfiguration, BasicConfiguration, TelemetryConfig};
use pumpkin_data::{dimension::Dimension, entity::EntityType, item::Item, item_stack::ItemStack};
use pumpkin_protocol::ConnectionState;
use pumpkin_util::{GameMode, math::vector3::Vector3};
use tokio::net::{TcpListener, TcpStream};

use super::CatEntity;
use crate::{
    data::VanillaData,
    entity::{Entity, EntityBase, passive::tamable::TamableAnimal, player::Player},
    net::{
        ClientPlatform, GameProfile, PacketRateLimiter, PlayerConfig,
        java::{JavaClient, pending::PendingConnection},
    },
    server::Server,
    world::World,
};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

struct Fixture {
    _directory: tempfile::TempDir,
    _server: Arc<Server>,
    _peer: TcpStream,
    world: Arc<World>,
    player: Arc<Player>,
}

impl Fixture {
    async fn new() -> TestResult<Self> {
        let directory = tempfile::tempdir()?;
        let basic = BasicConfiguration {
            default_level_name: directory.path().display().to_string(),
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
        let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
        // These interactions need no chunks, ticking, or client network tasks.
        world.level.shutdown().await;
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let peer = TcpStream::connect(listener.local_addr()?).await?;
        let (socket, address) = listener.accept().await?;
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
            name: "CatOwner".to_owned(),
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
        Ok(Self {
            _directory: directory,
            _server: server,
            _peer: peer,
            world,
            player,
        })
    }

    fn owned_cat(&self, sitting: bool) -> Arc<CatEntity> {
        let cat = CatEntity::new(Entity::new(
            self.world.clone(),
            Vector3::new(0.0, 100.0, 0.0),
            &EntityType::CAT,
        ));
        cat.set_tame(true, Some(self.player.gameprofile.id));
        cat.set_sitting(sitting);
        cat.get_entity().set_age(0);
        let living = &cat.mob_entity.living_entity;
        living.set_health(living.get_max_health());
        cat
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn owned_cat_food_enters_love_before_sitting() -> TestResult {
    let fixture = Fixture::new().await?;
    for gamemode in [GameMode::Survival, GameMode::Creative] {
        fixture.player.gamemode.store(gamemode);
        for food in [&Item::COD, &Item::SALMON] {
            for sitting in [false, true] {
                let cat = fixture.owned_cat(sitting);
                let mut stack = ItemStack::new(2, food);
                assert!(EntityBase::interact(
                    cat.as_ref(),
                    &fixture.player,
                    &mut stack
                ));
                assert_eq!(
                    stack.item_count,
                    if gamemode == GameMode::Creative { 2 } else { 1 }
                );
                assert_eq!(cat.mob_entity.love_ticks.load(Ordering::Relaxed), 600);
                assert_eq!(
                    cat.mob_entity.breeder.load(),
                    Some(fixture.player.gameprofile.id)
                );
                assert_eq!(cat.is_sitting(), sitting);
                assert_eq!(cat.get_owner(), Some(fixture.player.gameprofile.id));
                assert!(cat.is_tame());
            }
        }
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn owned_leashed_cat_releases_leash_before_feeding() -> TestResult {
    let fixture = Fixture::new().await?;
    for age in [0, -2400] {
        for food in [&Item::COD, &Item::SALMON] {
            for sitting in [false, true] {
                let cat = fixture.owned_cat(sitting);
                let entity = cat.get_entity();
                entity.set_age(age);
                entity.leash_to(fixture.player.clone());
                assert!(entity.is_leashed());
                let mut stack = ItemStack::new(2, food);
                assert!(EntityBase::interact(
                    cat.as_ref(),
                    &fixture.player,
                    &mut stack
                ));
                assert_eq!(stack.item_count, 2);
                assert!(!entity.is_leashed());
                assert_eq!(cat.mob_entity.love_ticks.load(Ordering::Relaxed), 0);
                assert_eq!(cat.mob_entity.breeder.load(), None);
                assert_eq!(entity.age.load(Ordering::Relaxed), age);
                assert_eq!(cat.is_sitting(), sitting);
            }
        }
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn owned_cat_empty_hand_healing_and_dye_keep_priority() -> TestResult {
    let fixture = Fixture::new().await?;
    for gamemode in [GameMode::Survival, GameMode::Creative] {
        fixture.player.gamemode.store(gamemode);
        let cat = fixture.owned_cat(true);
        let mut empty = ItemStack::new(0, &Item::AIR);
        assert!(EntityBase::interact(
            cat.as_ref(),
            &fixture.player,
            &mut empty
        ));
        assert!(!cat.is_sitting());
        assert_eq!(cat.mob_entity.love_ticks.load(Ordering::Relaxed), 0);
        assert!(empty.is_empty());

        cat.set_sitting(true);
        let living = &cat.mob_entity.living_entity;
        let max_health = living.get_max_health();
        living.set_health(max_health - 4.0);
        let mut food = ItemStack::new(2, &Item::COD);
        assert!(EntityBase::interact(
            cat.as_ref(),
            &fixture.player,
            &mut food
        ));
        assert!((living.health.load() - (max_health - 2.0)).abs() < f32::EPSILON);
        assert_eq!(
            food.item_count,
            if gamemode == GameMode::Creative { 2 } else { 1 }
        );
        assert_eq!(cat.mob_entity.love_ticks.load(Ordering::Relaxed), 0);
        assert!(cat.is_sitting());

        let mut dye = ItemStack::new(2, &Item::BLUE_DYE);
        assert!(EntityBase::interact(
            cat.as_ref(),
            &fixture.player,
            &mut dye
        ));
        assert_eq!(cat.get_collar_color(), 11);
        assert_eq!(
            dye.item_count,
            if gamemode == GameMode::Creative { 2 } else { 1 }
        );
        assert_eq!(cat.mob_entity.love_ticks.load(Ordering::Relaxed), 0);
        assert!(cat.is_sitting());
    }
    Ok(())
}
