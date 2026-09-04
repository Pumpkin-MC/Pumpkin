//! Exercise Java sprint commands, movement, fluid contact, and pose changes together.

use std::num::NonZeroU8;
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
use std::sync::atomic::Ordering;
use std::sync::{Arc, RwLock};

use arc_swap::ArcSwap;
use pumpkin_config::{AdvancedConfiguration, BasicConfiguration};
use pumpkin_data::{Block, dimension::Dimension, entity::EntityPose};
use pumpkin_protocol::java::server::play::{
    Action, SPlayerCommand, SPlayerInput, SPlayerPosition, SPlayerPositionRotation,
};
use pumpkin_util::GameMode;
use pumpkin_util::math::{position::BlockPos, vector2::Vector2, vector3::Vector3};
use pumpkin_world::{chunk::ChunkData, cylindrical_chunk_iterator::Cylindrical};
use tempfile::TempDir;
use tokio::net::{TcpListener, TcpStream};
use uuid::Uuid;

use super::Player;
use crate::data::VanillaData;
use crate::entity::EntityBase;
use crate::net::java::{JavaClient, pending::PendingConnection};
use crate::net::{ClientPlatform, GameProfile, PacketRateLimiter, PlayerConfig};
use crate::server::Server;
use crate::world::World;

struct SwimmingFixture {
    server: Arc<Server>,
    world: Arc<World>,
    player: Arc<Player>,
    _peer: TcpStream,
    _directory: TempDir,
}

impl SwimmingFixture {
    async fn new() -> Self {
        let directory = tempfile::tempdir().expect("temporary swimming test world");
        let basic = BasicConfiguration {
            seed: pumpkin_util::world_seed::Seed(0),
            default_level_name: directory
                .path()
                .join("world")
                .to_string_lossy()
                .into_owned(),
            allow_nether: false,
            allow_end: false,
            allow_chat_reports: false,
            use_favicon: false,
            ..BasicConfiguration::default()
        };
        let mut advanced = AdvancedConfiguration::default();
        advanced.networking.java.online_mode = false;
        advanced.networking.java.authentication.enabled = false;
        advanced.networking.bedrock.online_mode = false;
        advanced.networking.bedrock.authentication.enabled = false;
        advanced.networking.java.view_distance = NonZeroU8::new(2).unwrap();
        advanced.networking.java.simulation_distance = NonZeroU8::new(2).unwrap();
        advanced.world.autosave_ticks = 0;
        advanced.player_data.save_player_data = false;
        advanced.advancement.save_advancements = false;
        // Do not load or modify the user's bans, operators, whitelist, or save files.
        let data = VanillaData {
            banned_ip_list: RwLock::default(),
            banned_player_list: RwLock::default(),
            operator_config: RwLock::default(),
            user_cache: RwLock::default(),
            whitelist_config: RwLock::default(),
        };
        let server = Server::new(basic, advanced, data).await;
        let world = server.get_world_from_dimension(&Dimension::OVERWORLD);

        // Only create an unstarted Java connection on loopback. There is no login,
        // authentication, server ticker, or socket reader/writer in these tests.
        let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
            .await
            .unwrap();
        let address = listener.local_addr().unwrap();
        let (peer, accepted) = tokio::join!(TcpStream::connect(address), listener.accept());
        let (stream, address) = accepted.unwrap();
        let profile = GameProfile {
            id: Uuid::new_v4(),
            name: "swimming_test".to_owned(),
            properties: ArcSwap::from_pointee(Vec::new()),
            profile_actions: None,
        };
        let config = PlayerConfig {
            view_distance: NonZeroU8::new(2).unwrap(),
            ..PlayerConfig::default()
        };
        let pending =
            PendingConnection::new(stream, address, 0, PacketRateLimiter::new(false, 0.0, 0.0));
        let client = JavaClient::from_pending(pending, profile.clone(), config.clone());
        client
            .connection_state
            .store(pumpkin_protocol::ConnectionState::Play);
        let player = Arc::new(Player::new(
            Arc::new(ClientPlatform::Java(client)),
            profile,
            config,
            &world,
            GameMode::Creative,
        ));
        player.set_client_loaded(true);
        player.client.java().unwrap().set_player(player.clone());
        world.players.store(Arc::new(vec![player.clone()]));
        // Movement stays in this already loaded chunk, without scheduling terrain generation.
        player.watched_section.store(Cylindrical::new(
            Vector2::new(0, 0),
            NonZeroU8::new(2).unwrap(),
        ));
        world
            .level
            .loaded_chunks
            .insert(Vector2::new(0, 0), ChunkData::empty_sync(0, 0));

        Self {
            server,
            world,
            player,
            _peer: peer.unwrap(),
            _directory: directory,
        }
    }

    fn fill_water(&self, bottom: i32, top: i32) {
        let chunk = self
            .world
            .level
            .loaded_chunks
            .get(&Vector2::new(0, 0))
            .unwrap();
        for x in 3..=9 {
            for z in 3..=9 {
                for y in bottom..=top {
                    chunk.set_block_absolute_y(x, y, z, Block::WATER.default_state.id);
                }
            }
        }
    }

    fn command(&self, action: Action) {
        self.player.client.java().unwrap().handle_player_command(
            &self.player,
            &SPlayerCommand {
                entity_id: self.player.entity_id().into(),
                action,
                jump_boost: 0.into(),
            },
            &self.server,
        );
    }

    fn forward_input(&self) {
        self.player.client.java().unwrap().handle_player_input(
            &self.player,
            &SPlayerInput {
                input: SPlayerInput::FORWARD,
            },
            &self.server,
        );
    }

    fn move_to(&self, position: Vector3<f64>, with_rotation: bool) {
        let client = self.player.client.java().unwrap();
        if with_rotation {
            client.handle_position_rotation(
                &self.player,
                &self.server,
                &SPlayerPositionRotation {
                    position,
                    yaw: 0.0,
                    pitch: 0.0,
                    collision: 0,
                },
            );
        } else {
            client.handle_position(
                &self.player,
                &self.server,
                &SPlayerPosition {
                    position,
                    collision: 0,
                },
            );
        }
        assert_eq!(
            self.player.position(),
            position,
            "movement packet was accepted"
        );
    }

    fn tick_swimming(&self) {
        // These are the production tick operations in their normal order. Avoid ticking
        // unrelated inventory, terrain streaming, and hunger systems in this focused fixture.
        self.player
            .get_entity()
            .tick(self.player.as_ref(), &self.server);
        self.player.update_player_pose();
    }

    fn assert_swimming(&self, swimming: bool) {
        assert_eq!(
            self.player.is_swimming(),
            swimming,
            "swimming state at {:?}",
            self.player.position()
        );
        let expected_pose = if swimming {
            EntityPose::Swimming
        } else {
            EntityPose::Standing
        };
        assert!(
            self.player.get_entity().pose.load() == expected_pose,
            "player pose must match swimming={swimming}"
        );
    }

    async fn finish(self, outcome: std::thread::Result<()>) {
        let client = self.player.client.java().unwrap();
        client.close();
        client.await_tasks().await;
        client.player.store(Arc::new(None));
        self.world.players.store(Arc::new(Vec::new()));
        // Join chunk workers before the temporary directory is removed, including on a
        // regression assertion failure, so a failing test cannot leave background writes.
        self.server.shutdown().await;
        if let Err(panic) = outcome {
            resume_unwind(panic);
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn java_double_tap_sprint_swimming_stays_stable_across_movement_and_ticks() {
    let fixture = SwimmingFixture::new().await;
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        fixture.fill_water(63, 66);
        fixture.move_to(Vector3::new(5.5, 64.25, 5.5), false);
        fixture.tick_swimming();
        fixture.forward_input();
        fixture.command(Action::StartSprinting);
        fixture.assert_swimming(true);

        // Double-tapping W sends a sprint command without holding the sprint key.
        // Test both movement handlers and water contact after the body shrinks to 0.6 high.
        for (step, y) in [64.25, 64.25, 64.5, 64.9, 65.05, 64.25]
            .into_iter()
            .enumerate()
        {
            fixture.forward_input();
            fixture.move_to(Vector3::new(5.5 + step as f64 * 0.1, y, 5.5), step % 2 == 1);
            for _ in 0..3 {
                fixture.tick_swimming();
                assert!(
                    fixture.player.get_entity().is_in_water(),
                    "water contact at Y={y}"
                );
                assert!(fixture.player.get_entity().is_sprinting());
                assert_eq!(
                    fixture.player.last_input.load(Ordering::Relaxed) & SPlayerInput::SPRINT,
                    0
                );
                fixture.assert_swimming(true);
            }
        }

        fixture.command(Action::StopSprinting);
        fixture.tick_swimming();
        assert!(!fixture.player.get_entity().is_sprinting());
        fixture.assert_swimming(false);

        fixture.command(Action::StartSprinting);
        fixture.tick_swimming();
        fixture.assert_swimming(true);
        fixture.move_to(Vector3::new(12.5, 64.25, 5.5), false);
        fixture.tick_swimming();
        assert!(!fixture.player.get_entity().is_in_water());
        fixture.assert_swimming(false);

        fixture.move_to(Vector3::new(5.5, 64.25, 5.5), true);
        fixture.tick_swimming();
        fixture.assert_swimming(true);
        fixture.player.abilities.lock().unwrap().flying = true;
        fixture.tick_swimming();
        fixture.assert_swimming(false);
    }));
    fixture.finish(outcome).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn java_surface_swimming_continues_but_cannot_start_with_dry_eyes() {
    let fixture = SwimmingFixture::new().await;
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        fixture.fill_water(63, 64);
        // Standing eyes are above the water: sprinting alone must not start swimming.
        fixture.move_to(Vector3::new(5.5, 64.75, 5.5), false);
        fixture.tick_swimming();
        fixture.forward_input();
        fixture.command(Action::StartSprinting);
        fixture.tick_swimming();
        assert!(!fixture.player.get_entity().is_submerged_in_water());
        fixture.assert_swimming(false);

        // Dive, then rise until swimming eyes are above the surface. Existing swimming
        // continues while the body still touches water (vanilla entry/continuation hysteresis).
        fixture.move_to(Vector3::new(5.5, 63.25, 5.5), true);
        fixture.tick_swimming();
        fixture.assert_swimming(true);
        fixture.move_to(Vector3::new(5.5, 64.75, 5.5), false);
        for _ in 0..4 {
            fixture.tick_swimming();
            assert!(fixture.player.get_entity().is_in_water());
            assert!(!fixture.player.get_entity().is_submerged_in_water());
            fixture.assert_swimming(true);
        }

        fixture.command(Action::StopSprinting);
        fixture.tick_swimming();
        fixture.assert_swimming(false);
    }));
    fixture.finish(outcome).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn java_water_eye_detection_and_breathing_agree_at_real_fluid_surfaces() {
    let fixture = SwimmingFixture::new().await;
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        fixture.player.gamemode.store(GameMode::Survival);
        let set_block = |y, state| {
            fixture
                .world
                .level
                .set_block_state(&BlockPos::new(5, y, 5), state);
        };
        let check_eye = |eye_y, expected_water, description| {
            let entity = fixture.player.get_entity();
            entity.set_pos(Vector3::new(5.5, eye_y - entity.get_eye_height(), 5.5));
            entity.tick(fixture.player.as_ref(), &fixture.server);
            assert_eq!(
                entity.is_submerged_in_water(),
                expected_water,
                "{description}"
            );
            fixture
                .player
                .breath_manager
                .air_supply
                .store(100, Ordering::Relaxed);
            fixture.player.breath_manager.tick(&fixture.player);
            assert_eq!(
                fixture
                    .player
                    .breath_manager
                    .air_supply
                    .load(Ordering::Relaxed),
                if expected_water { 99 } else { 104 },
                "breathing must use the same eye-water result: {description}"
            );
        };

        // A source's own surface is 8/9 of a block. Check both sides closely enough
        // that rounding the eye down by 1/9 or treating sources as full blocks fails.
        let source_surface = 64.0 + f64::from(8.0f32 / 9.0);
        set_block(64, Block::WATER.default_state.id);
        check_eye(
            source_surface - 0.000_01,
            true,
            "eyes just below source surface",
        );
        check_eye(
            source_surface + 0.000_01,
            false,
            "eyes just above source surface",
        );
        set_block(65, Block::WATER.default_state.id);
        check_eye(
            64.999_99,
            true,
            "water above makes the lower source full height",
        );
        set_block(64, Block::AIR.default_state.id);
        check_eye(64.95, false, "dry air below water is not itself water");

        // Kelp contains source water despite not being a water block or having a
        // waterlogged property. Its own and column heights must work for breathing too.
        set_block(64, Block::KELP.default_state.id);
        set_block(65, Block::AIR.default_state.id);
        check_eye(64.75, true, "eyes inside kelp source water");
        check_eye(64.95, false, "eyes above kelp's own water surface");
        set_block(65, Block::KELP_PLANT.default_state.id);
        check_eye(64.95, true, "kelp column fills the lower fluid cell");

        let mut slab = pumpkin_data::block_properties::OakSlabProperties::default(&Block::OAK_SLAB);
        slab.waterlogged = true;
        set_block(64, slab.to_state_id(&Block::OAK_SLAB));
        set_block(65, Block::AIR.default_state.id);
        check_eye(64.75, true, "eyes in a waterlogged slab's water");
        check_eye(64.95, false, "eyes above a waterlogged slab's water");
    }));
    fixture.finish(outcome).await;
}
