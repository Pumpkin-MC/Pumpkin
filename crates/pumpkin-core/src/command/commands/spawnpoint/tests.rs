use std::{
    io::Cursor,
    sync::{Arc, RwLock},
    time::Duration,
};

use arc_swap::ArcSwap;
use pumpkin_config::{AdvancedConfiguration, BasicConfiguration, TelemetryConfig};
use pumpkin_data::{dimension::Dimension, packet::CURRENT_MC_VERSION};
use pumpkin_nbt::{Nbt, compound::NbtCompound, deserializer::NbtReadHelperJava, tag::NbtTag};
use pumpkin_protocol::{
    ConnectionState,
    java::{client::play::CSystemChatMessage, packet_decoder::TCPNetworkDecoder},
    packet::MultiVersionJavaPacket,
    ser::NetworkReadExt,
};
use pumpkin_util::{
    GameMode,
    math::vector3::Vector3,
    translation::{Locale, get_translation_text},
};
use tokio::{
    io::BufReader,
    net::{TcpListener, TcpStream},
};
use uuid::Uuid;

use super::*;
use crate::{
    command::CommandSender,
    data::VanillaData,
    entity::player::Player,
    net::{
        ClientPlatform, GameProfile, PacketRateLimiter, PlayerConfig,
        java::{JavaClient, pending::PendingConnection},
    },
    server::Server,
};

async fn connected_player(
    server: &Arc<Server>,
    name: &str,
) -> (Arc<Player>, TCPNetworkDecoder<BufReader<TcpStream>>) {
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
        Arc::downgrade(server),
    );
    pending.connection_state.store(ConnectionState::Play);
    let profile = GameProfile {
        id: Uuid::new_v4(),
        name: name.to_string(),
        properties: ArcSwap::from_pointee(Vec::new()),
        profile_actions: None,
    };
    let mut client = JavaClient::from_pending(pending, profile.clone(), PlayerConfig::default());
    client.start_outgoing_packet_task();
    let player = Arc::new(Player::new(
        Arc::new(ClientPlatform::Java(client)),
        profile,
        PlayerConfig::default(),
        &server.get_world_from_dimension(&Dimension::OVERWORLD),
        GameMode::Creative,
    ));
    (player, TCPNetworkDecoder::new(BufReader::new(peer)))
}

async fn read_feedback(reader: &mut TCPNetworkDecoder<BufReader<TcpStream>>) -> NbtCompound {
    // Setting the respawn point sends a spawn-position packet before the feedback.
    let mut feedback = None;
    for _ in 0..2 {
        let packet = tokio::time::timeout(Duration::from_secs(5), reader.get_raw_packet())
            .await
            .unwrap()
            .unwrap();
        if packet.id == CSystemChatMessage::to_id(CURRENT_MC_VERSION) {
            let mut payload = Cursor::new(packet.payload.as_ref());
            let nbt = Nbt::read_unnamed(&mut NbtReadHelperJava::new(&mut payload)).unwrap();
            assert!(!payload.get_bool().unwrap());
            feedback = Some(nbt.root_tag);
        }
    }
    feedback.expect("spawnpoint must send command feedback")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn feedback_for_self_and_multiple_targets() {
    let directory = tempfile::tempdir().unwrap();
    let basic = BasicConfiguration {
        default_level_name: directory.path().to_str().unwrap().to_string(),
        allow_nether: false,
        allow_end: false,
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
    let (sender, mut reader) = connected_player(&server, "Alex").await;
    let (other, _other_reader) = connected_player(&server, "Steve").await;
    world
        .players
        .store(Arc::new(vec![sender.clone(), other.clone()]));
    sender.permission_lvl.store(PermissionLvl::Four);
    sender
        .living_entity
        .entity
        .pos
        .store(Vector3::new(12.75, 64.5, -8.25));
    let source = CommandSender::Player(sender.clone()).into_source(&server);
    let mut feedback = Vec::new();
    for (input, count) in [("spawnpoint", 1), ("spawnpoint @a 10 70 -20 135 -30", 2)] {
        assert_eq!(
            server
                .command_dispatcher
                .load()
                .execute_input(input, &source)
                .unwrap(),
            count
        );
        feedback.push(read_feedback(&mut reader).await);
    }

    for player in [&sender, &other] {
        let client = player.client.java().unwrap();
        client.close();
        client.await_tasks().await;
    }
    world.players.store(Arc::new(Vec::new()));
    server.shutdown().await;
    world.level.world_portal.store(Arc::new(None));

    let rendered: Vec<_> = feedback
        .iter()
        .map(|component| {
            let key = component.get_string("translate").unwrap();
            let arguments = component
                .get_list("with")
                .unwrap()
                .iter()
                .map(|tag| TextComponent::text(tag.extract_string().unwrap().to_string()).0)
                .collect();
            get_translation_text(format!("minecraft:{key}"), Locale::EnUs, arguments)
        })
        .collect();
    assert_eq!(
        rendered,
        [
            "Set spawn point to 12, 64, -9 [0, 0] in minecraft:overworld for Alex",
            "Set spawn point to 10, 70, -20 [135, -30] in minecraft:overworld for 2 players",
        ]
    );
    for (component, key, arguments) in [
        (
            &feedback[0],
            "commands.spawnpoint.success.single.new",
            ["12", "64", "-9", "0", "0", "minecraft:overworld", "Alex"],
        ),
        (
            &feedback[1],
            "commands.spawnpoint.success.multiple.new",
            ["10", "70", "-20", "135", "-30", "minecraft:overworld", "2"],
        ),
    ] {
        assert_eq!(component.get_string("translate"), Some(key));
        assert_eq!(
            component.get_list("with").unwrap(),
            arguments.map(|text| NbtTag::String(text.into()))
        );
    }
}
