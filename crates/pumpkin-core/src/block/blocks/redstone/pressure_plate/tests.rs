use std::sync::RwLock;

use arc_swap::ArcSwap;
use pumpkin_config::{AdvancedConfiguration, BasicConfiguration, TelemetryConfig};
use pumpkin_data::dimension::Dimension;
use pumpkin_protocol::ConnectionState;
use pumpkin_util::GameMode;
use pumpkin_util::math::{boundingbox::BoundingBox, position::BlockPos, vector3::Vector3};
use pumpkin_world::chunk::ChunkData;
use tokio::net::{TcpListener, TcpStream};

use crate::data::VanillaData;
use crate::entity::{EntityBase, player::Player};
use crate::net::java::{JavaClient, pending::PendingConnection};
use crate::net::{ClientPlatform, GameProfile, PacketRateLimiter, PlayerConfig};
use crate::server::Server;

use super::*;

#[test]
fn weighted_plates_scale_with_entity_count() {
    use super::weighted::signal_strength;

    assert_eq!(signal_strength(0, 15), 0);
    assert_eq!(signal_strength(1, 15), 1);
    assert_eq!(signal_strength(7, 15), 7);
    assert_eq!(signal_strength(20, 15), 15);
    assert_eq!(signal_strength(1, 150), 1);
    assert_eq!(signal_strength(75, 150), 8);
}

#[test]
fn detection_box_is_offset_to_block_position() {
    let bounding_box = detection_box_at(&BlockPos::new(10, 64, -5));

    assert_eq!(bounding_box.min, Vector3::new(10.0625, 64.0, -4.9375));
    assert_eq!(bounding_box.max, Vector3::new(10.9375, 64.25, -4.0625));
}

#[test]
fn entity_at_plate_center_intersects_detection_box() {
    let detection_box = detection_box_at(&BlockPos::new(0, 0, 0));
    let entity_box = BoundingBox::new_array([0.4, 0.0, 0.4], [0.6, 1.8, 0.6]);

    assert!(detection_box.intersects(&entity_box));
}

#[test]
fn entity_outside_horizontal_bounds_does_not_intersect_detection_box() {
    let detection_box = detection_box_at(&BlockPos::new(0, 0, 0));
    let entity_box = BoundingBox::new_array([0.95, 0.0, 0.4], [1.0, 0.2, 0.6]);

    assert!(!detection_box.intersects(&entity_box));
}

#[test]
fn entity_above_detection_height_does_not_intersect_detection_box() {
    let detection_box = detection_box_at(&BlockPos::new(0, 0, 0));
    let entity_box = BoundingBox::new_array([0.4, 0.3, 0.4], [0.6, 0.6, 0.6]);

    assert!(!detection_box.intersects(&entity_box));
}

#[test]
fn entity_partially_overlapping_detection_box_intersects() {
    let detection_box = detection_box_at(&BlockPos::new(0, 0, 0));
    let entity_box = BoundingBox::new_array([0.9, 0.2, 0.4], [0.95, 0.3, 0.6]);

    assert!(detection_box.intersects(&entity_box));
}

#[test]
fn entity_touching_detection_box_boundary_does_not_intersect() {
    let detection_box = detection_box_at(&BlockPos::new(0, 0, 0));
    let entity_box = BoundingBox::new_array([0.9375, 0.0, 0.4], [1.0, 0.2, 0.6]);

    assert!(!detection_box.intersects(&entity_box));
}

async fn plate_world() -> (tempfile::TempDir, Arc<Server>, Arc<World>, Arc<ChunkData>) {
    let temp = tempfile::tempdir().unwrap();
    let basic = BasicConfiguration {
        default_level_name: temp.path().to_str().unwrap().to_owned(),
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
    .await
    .unwrap();
    let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
    world.level_info.rcu(|info| {
        let mut info = (**info).clone();
        info.game_rules.spawn_mobs = false;
        info.game_rules.random_tick_speed = 0;
        info.game_rules.spectators_generate_chunks = false;
        info
    });
    let chunk = ChunkData::empty_sync(0, 0);
    world
        .level
        .loaded_chunks
        .insert(BlockPos::new(8, 100, 8).chunk_position(), chunk.clone());
    (temp, server, world, chunk)
}

async fn plate_player(
    server: &Arc<Server>,
    world: &Arc<World>,
    id: u64,
) -> (Arc<Player>, TcpStream) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let peer = TcpStream::connect(listener.local_addr().unwrap())
        .await
        .unwrap();
    let (socket, address) = listener.accept().await.unwrap();
    let pending = PendingConnection::new(
        socket,
        address,
        id,
        PacketRateLimiter::new(false, 0.0, 0.0),
        Arc::downgrade(server),
    );
    pending.connection_state.store(ConnectionState::Play);
    let profile = GameProfile {
        id: uuid::Uuid::new_v4(),
        name: format!("Plate{id}"),
        properties: ArcSwap::from_pointee(Vec::new()),
        profile_actions: None,
    };
    let client = JavaClient::from_pending(pending, profile.clone(), PlayerConfig::default());
    let player = Arc::new(Player::new(
        Arc::new(ClientPlatform::Java(client)),
        profile,
        PlayerConfig::default(),
        world,
        GameMode::Survival,
    ));
    player.set_client_loaded(true);
    player.get_entity().set_pos(Vector3::new(8.5, 100.0, 12.5));
    world.add_player(&player).unwrap();
    (player, peer)
}

fn plate_power(world: &World, pos: &BlockPos) -> u8 {
    let (block, state) = world.get_block_and_state(pos);
    world
        .block_registry
        .get_weak_redstone_power(block, world, pos, state, BlockDirection::Up)
}

fn tick_plate(world: &Arc<World>, server: &Arc<Server>, delay: u8) {
    world.update_active_chunks();
    // Include the scheduler's current slot as well as the plate's delay.
    for _ in 0..=delay {
        world.tick_chunks(server);
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn pressure_plates_ignore_spectators_on_scheduled_ticks() {
    let (_temp, server, world, chunk) = plate_world().await;
    // Keep the chunk active after every occupant becomes a spectator.
    let (_bystander, _peer) = plate_player(&server, &world, 0).await;
    let mut occupants = Vec::new();
    for id in 1..=11 {
        occupants.push(plate_player(&server, &world, id).await);
    }
    let mut observed = Vec::new();
    for (block, x, full_power, delay) in [
        (&Block::OAK_PRESSURE_PLATE, 2, 15, 20),
        (&Block::STONE_PRESSURE_PLATE, 5, 15, 20),
        (&Block::LIGHT_WEIGHTED_PRESSURE_PLATE, 8, 11, 10),
        (&Block::HEAVY_WEIGHTED_PRESSURE_PLATE, 11, 2, 10),
    ] {
        let pos = BlockPos::new(x, 100, 8);
        chunk.set_block_absolute_y(x as usize, 99, 8, Block::STONE.default_state.id);
        chunk.set_block_absolute_y(x as usize, 100, 8, block.default_state.id);
        for (player, _) in &occupants {
            player.set_gamemode(GameMode::Survival);
            player
                .get_entity()
                .set_pos(Vector3::new(f64::from(x) + 0.5, 100.0, 8.5));
        }
        let player = &occupants[0].0;
        player.get_entity().tick_block_collisions(player.as_ref());
        assert_eq!(plate_power(&world, &pos), full_power);
        assert!(world.is_block_tick_scheduled(&pos, block));

        // Ten spectators must not inflate either weighted plate's output.
        for (player, _) in &occupants[1..] {
            assert!(player.set_gamemode(GameMode::Spectator));
        }
        tick_plate(&world, &server, delay);
        let survival_power = plate_power(&world, &pos);
        assert!(world.is_block_tick_scheduled(&pos, block));
        let mut controls = Vec::new();
        for mode in [GameMode::Adventure, GameMode::Creative] {
            assert!(player.set_gamemode(mode));
            tick_plate(&world, &server, delay);
            controls.push(plate_power(&world, &pos));
        }

        assert!(player.set_gamemode(GameMode::Survival));
        assert!(player.set_gamemode(GameMode::Spectator));
        assert_eq!(plate_power(&world, &pos), survival_power);
        tick_plate(&world, &server, delay);
        assert!(
            world
                .active_chunks
                .read()
                .unwrap()
                .contains(&pos.chunk_position())
        );
        // The general world query still includes spectators; only plates filter them.
        assert_eq!(world.get_players_at_box(&detection_box_at(&pos)).len(), 11);
        let released_power = plate_power(&world, &pos);
        let scheduled = world.is_block_tick_scheduled(&pos, block);
        player.get_entity().tick_block_collisions(player.as_ref());
        assert_eq!(plate_power(&world, &pos), released_power);
        observed.push((survival_power, controls, released_power, scheduled));
    }
    assert_eq!(
        observed,
        [
            (15, vec![15, 15], 0, false),
            (15, vec![15, 15], 0, false),
            (1, vec![1, 1], 0, false),
            (1, vec![1, 1], 0, false),
        ]
    );
}
