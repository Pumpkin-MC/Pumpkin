use std::sync::atomic::Ordering::Relaxed;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use arc_swap::ArcSwap;
use futures::FutureExt;
use pumpkin_config::{AdvancedConfiguration, BasicConfiguration, TelemetryConfig};
use pumpkin_data::{
    Block, entity::EntityType, item::Item, item_stack::ItemStack, meta_data_type::MetaDataType,
    packet::CURRENT_MC_VERSION, particle::Particle, tracked_data::abstract_arrow,
};
use pumpkin_protocol::java::client::play::{CParticle, CSetEntityMetadata, CSpawnEntity};
use pumpkin_protocol::java::server::play::{SPlayerAction, SUseItem, Status};
use pumpkin_protocol::{ConnectionState, packet::MultiVersionJavaPacket, ser::NetworkReadExt};
use pumpkin_util::GameMode;
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
use pumpkin_world::chunk::ChunkData;
use pumpkin_world::cylindrical_chunk_iterator::Cylindrical;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::time::timeout;

use super::{JavaClient, outgoing::OutgoingPacket, pending::PendingConnection};
use crate::data::VanillaData;
use crate::entity::{EntityBase, player::Player, projectile::arrow::ArrowEntity};
use crate::item::items::bow::BowItem;
use crate::net::{
    ClientPlatform, GameProfile, PacketRateLimiter, PlayerConfig, decrement_pending_bytes,
};
use crate::server::Server;
use crate::world::World;

async fn archer(
    server: &Arc<Server>,
    world: &Arc<World>,
) -> (Arc<Player>, UnboundedReceiver<OutgoingPacket>, TcpStream) {
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
        Arc::downgrade(server),
    );
    pending.connection_state.store(ConnectionState::Play);
    let profile = GameProfile {
        id: uuid::Uuid::new_v4(),
        name: "Archer".to_owned(),
        properties: ArcSwap::from_pointee(Vec::new()),
        profile_actions: None,
    };
    let mut client = JavaClient::from_pending(pending, profile.clone(), PlayerConfig::default());
    let packets = client.outgoing_packet_queue_recv.take().unwrap();
    let player = Arc::new(Player::new(
        Arc::new(ClientPlatform::Java(client)),
        profile,
        PlayerConfig::default(),
        world,
        GameMode::Survival,
    ));
    player.set_client_loaded(true);
    player.get_entity().set_pos(Vector3::new(8.0, 64.0, 4.0));
    player.get_entity().set_rotation(0.0, 0.0);
    let chunk_pos = player.get_entity().chunk_pos.load();
    player.watched_section.store(Cylindrical::new(
        chunk_pos,
        player.config.load().view_distance,
    ));
    player
        .chunk_sender
        .lock()
        .unwrap()
        .mark_sent_out_of_band(chunk_pos);
    world.add_player(&player).unwrap();
    (player, packets, peer)
}

#[derive(Default)]
struct ArrowPackets {
    spawns: usize,
    flags: Vec<u8>,
    in_ground: Vec<bool>,
    critical_particles: usize,
}

impl ArrowPackets {
    fn metadata(&mut self, mut bytes: &[u8]) {
        // Only the ordinary arrow's byte, boolean and integer entries are expected.
        for _ in 0..16 {
            let index = bytes.get_u8().unwrap();
            if index == 255 {
                assert!(bytes.is_empty());
                return;
            }
            let kind = bytes.get_var_int().unwrap().0;
            let value = if kind == MetaDataType::BYTE.id(CURRENT_MC_VERSION) {
                i32::from(bytes.get_u8().unwrap())
            } else if kind == MetaDataType::BOOLEAN.id(CURRENT_MC_VERSION) {
                i32::from(bytes.get_bool().unwrap())
            } else {
                assert_eq!(kind, MetaDataType::INT.id(CURRENT_MC_VERSION));
                bytes.get_var_int().unwrap().0
            };
            if index == abstract_arrow::ID_FLAGS.id.get(&CURRENT_MC_VERSION) {
                assert_eq!(kind, MetaDataType::BYTE.id(CURRENT_MC_VERSION));
                self.flags.push(value as u8);
            } else if index == abstract_arrow::IN_GROUND.id.get(&CURRENT_MC_VERSION) {
                assert_eq!(kind, MetaDataType::BOOLEAN.id(CURRENT_MC_VERSION));
                self.in_ground.push(value != 0);
            }
        }
        panic!("ordinary arrow metadata exceeded fixture bound");
    }
}

fn drain(
    player: &Player,
    packets: &mut UnboundedReceiver<OutgoingPacket>,
    arrow_id: i32,
) -> ArrowPackets {
    let mut observed = ArrowPackets::default();
    for _ in 0..256 {
        let Ok(packet) = packets.try_recv() else {
            return observed;
        };
        let OutgoingPacket::Data { data, .. } = packet else {
            continue;
        };
        decrement_pending_bytes(&player.client.java().unwrap().pending_bytes, data.len());
        let mut bytes = data.as_ref();
        let packet_id = bytes.get_var_int().unwrap().0;
        if packet_id == CSetEntityMetadata::to_id(CURRENT_MC_VERSION) {
            if bytes.get_var_int().unwrap().0 == arrow_id {
                observed.metadata(bytes);
            }
        } else if packet_id == CSpawnEntity::to_id(CURRENT_MC_VERSION) {
            if bytes.get_var_int().unwrap().0 == arrow_id {
                // Inspect the spawn prefix without round-tripping the packed velocity codec.
                let _uuid = bytes.get_uuid().unwrap();
                assert_eq!(
                    bytes.get_var_int().unwrap().0,
                    i32::from(EntityType::ARROW.id)
                );
                observed.spawns += 1;
            }
        } else if packet_id == CParticle::to_id(CURRENT_MC_VERSION) {
            // Read the actual 26.3 writer layout, not CParticle's legacy reader.
            assert_eq!(bytes.get_var_int().unwrap().0, Particle::Crit as i32);
            assert!(!bytes.get_bool().unwrap());
            assert!(!bytes.get_bool().unwrap());
            for _ in 0..3 {
                assert!(bytes.get_f64_be().unwrap().is_finite());
            }
            for _ in 0..6 {
                assert!(bytes.get_f32_be().unwrap().is_finite());
            }
            assert_eq!(bytes.get_var_int().unwrap().0, 1);
            assert_eq!(bytes.get_var_int().unwrap().0, 0);
            assert!(bytes.is_empty());
            observed.critical_particles += 1;
        }
    }
    panic!("arrow packet queue exceeded fixture bound");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[expect(
    clippy::too_many_lines,
    reason = "Real bow release, tracking, flight and impact regression"
)]
async fn bow_arrows_use_metadata_without_server_critical_particles() {
    let directory = tempfile::tempdir().unwrap();
    let basic = BasicConfiguration {
        default_level_name: directory.path().to_string_lossy().into_owned(),
        allow_nether: false,
        allow_end: false,
        allow_chat_reports: false,
        use_favicon: false,
        ..Default::default()
    };
    let mut advanced = AdvancedConfiguration::default();
    advanced.networking.bedrock.online_mode = false;
    let server = timeout(
        Duration::from_secs(60),
        Server::new(
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
        ),
    )
    .await
    .unwrap()
    .unwrap();
    let world = server.worlds.load()[0].clone();
    let chunk = ChunkData::empty_sync(0, 0);
    // A wide target keeps the normal, randomized bow launch inside one loaded chunk.
    for x in 6..=10 {
        for y in 61..=68 {
            chunk.set_block_absolute_y(x, y, 10, Block::STONE.default_state.id);
        }
    }
    world
        .level
        .loaded_chunks
        .insert(BlockPos::new(8, 64, 4).chunk_position(), chunk);

    let result = std::panic::AssertUnwindSafe(async {
        let (player, mut packets, _peer) = timeout(Duration::from_secs(5), archer(&server, &world))
            .await
            .unwrap();
        drain(&player, &mut packets, -1);
        let mut critical_packets_per_charge = Vec::new();
        for (draw_ticks, critical) in [(20, true), (10, false)] {
            player
                .inventory
                .set_held_item(ItemStack::new(1, &Item::BOW));
            player
                .inventory
                .set_slot(1, ItemStack::new(2, &Item::ARROW));
            let client = player.client.java().unwrap();
            client.handle_use_item(
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
                player.living_entity.item_use_time.load(Relaxed),
                BowItem::USE_DURATION
            );
            // Set the elapsed charge in the active-use fixture; production release chooses crit.
            player
                .living_entity
                .item_use_time
                .store(BowItem::USE_DURATION - draw_ticks, Relaxed);
            let before = world.entities.load().len();
            client.handle_player_action(
                &player,
                &SPlayerAction {
                    status: (Status::ReleaseItemInUse as i32).into(),
                    position: BlockPos::new(0, 0, 0),
                    face: 0,
                    sequence: 2.into(),
                },
                &server,
            );
            assert_eq!(world.entities.load().len(), before + 1);
            assert_eq!(player.inventory.get_slot(1).item_count, 1);
            assert!(player.living_entity.item_in_use.lock().unwrap().is_none());
            let spawned = world.entities.load().last().unwrap().clone();
            let arrow = spawned.cast_any().downcast_ref::<ArrowEntity>().unwrap();
            assert_eq!(arrow.is_critical.load(Relaxed), critical);
            assert_eq!(arrow.owner_id, Some(player.get_entity().entity_id));
            let arrow_id = arrow.entity.entity_id;
            let tracker = world.entity_tracker.get_tracked_entity(arrow_id).unwrap();
            assert!(tracker.seen_by.contains(&player.gameprofile.id));
            let spawn = drain(&player, &mut packets, arrow_id);
            assert_eq!(spawn.spawns, 1);
            assert!(!spawn.flags.is_empty());
            assert!(spawn.flags.iter().all(|flags| *flags == u8::from(critical)));
            assert!(spawn.in_ground.iter().all(|grounded| !grounded));
            let initial_velocity = arrow.entity.velocity.load();
            let initial_position = arrow.entity.pos.load();
            arrow.tick(arrow, &server);
            tracker.send_changes(&world);
            assert!(!arrow.in_ground.load(Relaxed));
            assert_eq!(arrow.is_critical.load(Relaxed), critical);
            assert!(arrow.entity.pos.load().z > initial_position.z);
            assert!(arrow.entity.velocity.load().z < initial_velocity.z);
            assert!(arrow.entity.velocity.load().y < initial_velocity.y);
            let flight = drain(&player, &mut packets, arrow_id);
            assert!(
                flight
                    .flags
                    .iter()
                    .all(|flags| *flags == u8::from(critical))
            );
            let mut critical_packets = spawn.critical_particles + flight.critical_particles;
            let mut impact = None;
            for _ in 0..12 {
                arrow.tick(arrow, &server);
                tracker.send_changes(&world);
                let observed = drain(&player, &mut packets, arrow_id);
                critical_packets += observed.critical_particles;
                if arrow.in_ground.load(Relaxed) {
                    impact = Some(observed);
                    break;
                }
                assert_eq!(arrow.is_critical.load(Relaxed), critical);
            }
            let impact = impact.expect("normal bow arrow must reach the stone target");
            assert_eq!(impact.in_ground, [true]);
            assert!(!arrow.is_critical.load(Relaxed));
            if critical {
                assert_eq!(impact.flags, [0]);
            }
            assert_eq!(arrow.entity.velocity.load(), Vector3::new(0.0, 0.0, 0.0));
            let resting_position = arrow.entity.pos.load();
            arrow.tick(arrow, &server);
            tracker.send_changes(&world);
            assert_eq!(arrow.entity.pos.load(), resting_position);
            let grounded = drain(&player, &mut packets, arrow_id);
            assert_eq!(grounded.critical_particles, 0);
            critical_packets_per_charge.push(critical_packets);
        }
        // Vanilla's common AbstractArrow call reaches the server Level.addParticle no-op.
        // Critical metadata remains the trail signal; rendering itself needs a client.
        assert_eq!(
            critical_packets_per_charge,
            [0, 0],
            "bow arrows must not broadcast server Crit particles"
        );
    })
    .catch_unwind()
    .await;
    timeout(Duration::from_secs(30), server.shutdown())
        .await
        .unwrap();
    if let Err(error) = result {
        std::panic::resume_unwind(error);
    }
}
