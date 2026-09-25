use std::sync::{
    Arc, Mutex, Weak,
    atomic::{AtomicBool, Ordering},
};

use bytes::Bytes;
use pumpkin_config::{AdvancedConfiguration, BasicConfiguration, TelemetryConfig};
use pumpkin_data::packet::CURRENT_MC_VERSION;
use pumpkin_protocol::{
    ClientPacket, ConnectionState, RawPacket, ServerPacket,
    codec::var_int::VarInt,
    java::{
        client::{
            config::{CConfigKeepAlive, CFinishConfig, CKnownPacks},
            login::{CLoginSuccess, CSetCompression},
            play::{CKeepAlive, CStartConfiguration},
        },
        packet_decoder::TCPNetworkDecoder,
        packet_encoder::TCPNetworkEncoder,
        server::{
            config::{SAcknowledgeFinishConfig, SClientInformationConfig, SKnownPacks},
            handshake::SHandShake,
            login::{SLoginAcknowledged, SLoginStart},
            play::SConfigurationAcknowledged,
        },
    },
    packet::MultiVersionJavaPacket,
};
use tokio::{
    io::{BufReader, BufWriter},
    net::{
        TcpListener, TcpStream,
        tcp::{OwnedReadHalf, OwnedWriteHalf},
    },
    sync::Notify,
    time::{Duration, timeout},
};

use super::{JavaClient, pending::PendingConnection};
use crate::{
    data::VanillaData,
    entity::player::Player,
    net::{ClientPlatform, PacketHandlerResult, PacketRateLimiter},
    plugin::{
        BoxFuture, EventHandler, EventPriority,
        server::packet::{PacketReceivedEvent, PacketSentEvent, decode_packet, encode_packet},
    },
    server::Server,
};

const EDIT: i32 = 0x300;
const CANCEL: i32 = 0x301;
const REENTER: i32 = 0x302;
const FOLLOWUP: i32 = 0x401;
const SILENT: i32 = 0x402;

struct Listener {
    seen: Mutex<Vec<(bool, ConnectionState, i32)>>,
    cancel_compression: AtomicBool,
    cancel_config_ack: AtomicBool,
    cancelled_ack: Notify,
    player: Mutex<Weak<Player>>,
}

impl EventHandler<PacketReceivedEvent> for Listener {
    fn handle_blocking<'a>(
        &'a self,
        _: &'a Arc<Server>,
        event: &'a mut PacketReceivedEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            self.seen
                .lock()
                .unwrap()
                .push((false, event.state, event.packet_id));
            let replacement = match event.state {
                ConnectionState::HandShake => {
                    let mut packet =
                        SHandShake::read(&mut event.payload.as_ref(), &CURRENT_MC_VERSION).unwrap();
                    packet.server_address = "rewritten.local".into();
                    Some(serialized(&packet))
                }
                ConnectionState::Status if event.packet_id == 0 => {
                    event.user.send_packet(0x500, &[10], false).unwrap();
                    Some(encode_packet(1, &41i64.to_be_bytes()).unwrap())
                }
                ConnectionState::Login
                    if event.packet_id == SLoginStart::to_id(CURRENT_MC_VERSION) =>
                {
                    let mut packet =
                        SLoginStart::read(&mut event.payload.as_ref(), &CURRENT_MC_VERSION)
                            .unwrap();
                    packet.name = "ChangedPlayer".into();
                    Some(serialized(&packet))
                }
                ConnectionState::Config
                    if event.packet_id == SClientInformationConfig::to_id(CURRENT_MC_VERSION) =>
                {
                    let mut packet = SClientInformationConfig::read(
                        &mut event.payload.as_ref(),
                        &CURRENT_MC_VERSION,
                    )
                    .unwrap();
                    packet.locale = if packet.locale == "fr_fr" {
                        "es_es"
                    } else {
                        "de_de"
                    };
                    Some(serialized(&packet))
                }
                ConnectionState::Config
                    if event.packet_id == SAcknowledgeFinishConfig::to_id(CURRENT_MC_VERSION)
                        && self.cancel_config_ack.swap(false, Ordering::SeqCst) =>
                {
                    event.cancelled = true;
                    self.cancelled_ack.notify_one();
                    None
                }
                ConnectionState::Play if event.packet_id == EDIT => {
                    Some(encode_packet(0x400, &[21, 22]).unwrap())
                }
                ConnectionState::Play if event.packet_id == CANCEL => {
                    event.cancelled = true;
                    None
                }
                _ => None,
            };
            if let Some(replacement) = replacement {
                let packet = decode_packet(replacement).unwrap();
                event.packet_id = packet.id;
                event.payload = packet.payload;
            }
        })
    }
}

impl EventHandler<PacketSentEvent> for Listener {
    fn handle_blocking<'a>(
        &'a self,
        _: &'a Arc<Server>,
        event: &'a mut PacketSentEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            self.seen
                .lock()
                .unwrap()
                .push((true, event.state, event.packet_id));
            if event.state == ConnectionState::Status && event.packet_id == 1 {
                event.payload = Bytes::copy_from_slice(&42i64.to_be_bytes());
            } else if event.state == ConnectionState::Login
                && event.packet_id == CSetCompression::to_id(CURRENT_MC_VERSION)
            {
                event.cancelled = self.cancel_compression.swap(false, Ordering::SeqCst);
                event.payload = Bytes::from_static(&[1]);
            } else if event.state == ConnectionState::Play {
                match event.packet_id {
                    EDIT => {
                        event.packet_id = 0x400;
                        event.payload = Bytes::from(vec![9; 128]);
                    }
                    CANCEL => event.cancelled = true,
                    REENTER => {
                        let player = self.player.lock().unwrap().upgrade().unwrap();
                        player
                            .client
                            .java()
                            .unwrap()
                            .send_packet_now_data(encode_packet(FOLLOWUP, &[11]).unwrap())
                            .await;
                        event.user.send_packet(SILENT, &[12], true).unwrap();
                    }
                    _ => {}
                }
            }
        })
    }
}

fn serialized(packet: &impl ClientPacket) -> Bytes {
    JavaClient::serialize_packet_for_version(packet, CURRENT_MC_VERSION).unwrap()
}

struct Peer {
    reader: TCPNetworkDecoder<BufReader<OwnedReadHalf>>,
    writer: TCPNetworkEncoder<BufWriter<OwnedWriteHalf>>,
}

impl Peer {
    async fn send(&mut self, packet: &impl ClientPacket) {
        self.raw(serialized(packet)).await;
    }

    async fn raw(&mut self, data: Bytes) {
        self.writer.write_packet(data).await.unwrap();
        self.writer.flush().await.unwrap();
    }

    async fn receive(&mut self) -> RawPacket {
        timeout(Duration::from_secs(10), self.reader.get_raw_packet())
            .await
            .unwrap()
            .unwrap()
    }

    async fn handshake(&mut self, next_state: ConnectionState) {
        self.send(&SHandShake {
            protocol_version: VarInt(CURRENT_MC_VERSION.protocol_version()),
            server_address: "original.local".into(),
            server_port: 25565,
            next_state,
        })
        .await;
    }
}

async fn connection(server: &Arc<Server>) -> (PendingConnection, Peer) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let (peer, accepted) = tokio::join!(
        TcpStream::connect(listener.local_addr().unwrap()),
        listener.accept()
    );
    let (stream, address) = accepted.unwrap();
    let (read, write) = peer.unwrap().into_split();
    (
        PendingConnection::new(
            stream,
            address,
            1,
            PacketRateLimiter::new(false, 0.0, 0.0),
            Arc::downgrade(server),
        ),
        Peer {
            reader: TCPNetworkDecoder::new(BufReader::new(read)),
            writer: TCPNetworkEncoder::new(BufWriter::new(write)),
        },
    )
}

async fn inbound(player: &Player) -> RawPacket {
    timeout(Duration::from_secs(10), async {
        loop {
            if let Some(packet) = player.inbound_packets.pop() {
                return packet;
            }
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    })
    .await
    .unwrap()
}

#[expect(clippy::too_many_lines)]
async fn exercise_pipeline(server: Arc<Server>, listener: Arc<Listener>) {
    let (mut pending, mut peer) = connection(&server).await;
    let user = pending.user.clone();
    let status_server = server.clone();
    let status = tokio::spawn(async move { pending.handle_login_sequence(&status_server).await });
    peer.handshake(ConnectionState::Status).await;
    peer.raw(encode_packet(0, &[]).unwrap()).await;
    assert_eq!(peer.receive().await.id, 0x500);
    let reply = peer.receive().await;
    assert_eq!(
        (reply.id, reply.payload.as_ref()),
        (1, &42i64.to_be_bytes()[..])
    );
    assert!(matches!(status.await.unwrap(), PacketHandlerResult::Stop));
    assert_eq!(user.info().server_address, "rewritten.local");
    assert!(user.player().is_none());

    let (mut pending, mut peer) = connection(&server).await;
    let user = pending.user.clone();
    let login_server = server.clone();
    let login = tokio::spawn(async move { pending.handle_login_sequence(&login_server).await });
    peer.handshake(ConnectionState::Login).await;
    peer.send(&SLoginStart {
        name: "OriginalPlayer".into(),
        uuid: uuid::Uuid::new_v4(),
    })
    .await;
    assert_eq!(
        peer.receive().await.id,
        CLoginSuccess::to_id(CURRENT_MC_VERSION)
    );
    assert_eq!(user.decoder_state.load(), ConnectionState::Login);
    assert_eq!(user.encoder_state.load(), ConnectionState::Config);
    user.close();
    assert!(matches!(login.await.unwrap(), PacketHandlerResult::Stop));

    let (mut pending, mut peer) = connection(&server).await;
    let user = pending.user.clone();
    let login_server = server.clone();
    let login = tokio::spawn(async move {
        let result = pending.handle_login_sequence(&login_server).await;
        (pending, result)
    });
    peer.handshake(ConnectionState::Login).await;
    peer.send(&SLoginStart {
        name: "OriginalPlayer".into(),
        uuid: uuid::Uuid::new_v4(),
    })
    .await;
    let compression = peer.receive().await;
    assert_eq!(compression.id, CSetCompression::to_id(CURRENT_MC_VERSION));
    assert_eq!(compression.payload.as_ref(), &[1]);
    peer.reader.set_compression(1);
    peer.writer.set_compression((1, 4));
    assert_eq!(
        peer.receive().await.id,
        CLoginSuccess::to_id(CURRENT_MC_VERSION)
    );
    peer.raw(encode_packet(SLoginAcknowledged::to_id(CURRENT_MC_VERSION), &[]).unwrap())
        .await;
    while peer.receive().await.id != CKnownPacks::to_id(CURRENT_MC_VERSION) {}
    peer.send(&SClientInformationConfig {
        locale: "en_us",
        view_distance: 8,
        chat_mode: VarInt(0),
        chat_colors: true,
        skin_parts: 0x7f,
        main_hand: VarInt(1),
        text_filtering: false,
        server_listing: true,
    })
    .await;
    peer.send(&SKnownPacks {
        known_packs: vec![],
    })
    .await;
    while peer.receive().await.id != CFinishConfig::to_id(CURRENT_MC_VERSION) {}
    peer.raw(encode_packet(SAcknowledgeFinishConfig::to_id(CURRENT_MC_VERSION), &[]).unwrap())
        .await;
    listener.cancelled_ack.notified().await;
    assert_eq!(user.decoder_state.load(), ConnectionState::Config);
    peer.raw(encode_packet(SAcknowledgeFinishConfig::to_id(CURRENT_MC_VERSION), &[]).unwrap())
        .await;
    let (pending, result) = login.await.unwrap();
    let PacketHandlerResult::ReadyToPlay(profile, config) = result else {
        panic!("login did not finish");
    };
    assert_eq!(profile.name, "ChangedPlayer");
    assert_eq!(config.locale, "de_de");
    let mut client = JavaClient::from_pending(pending, profile.clone(), config.clone());
    assert!(Arc::ptr_eq(&user, &client.user));
    client.start_outgoing_packet_task();
    let platform = Arc::new(ClientPlatform::Java(client));
    let player = Arc::new(Player::new(
        platform.clone(),
        profile,
        config,
        &server.worlds.load()[0],
        pumpkin_util::GameMode::Survival,
    ));
    let client = platform.java().unwrap();
    client.set_player(player.clone());
    assert!(Arc::ptr_eq(&user.player().unwrap(), &player));
    *listener.player.lock().unwrap() = Arc::downgrade(&player);
    let reading_player = player.clone();
    let reading_server = server.clone();
    let inbound_loop = tokio::spawn(async move {
        reading_player
            .client
            .java()
            .unwrap()
            .progress_player_packets(&reading_player, &reading_server)
            .await;
    });

    peer.raw(encode_packet(EDIT, &[1]).unwrap()).await;
    peer.raw(encode_packet(CANCEL, &[2]).unwrap()).await;
    peer.raw(encode_packet(0x303, &[3]).unwrap()).await;
    let edited = inbound(&player).await;
    assert_eq!(edited.id, 0x400);
    assert_eq!(edited.payload.as_ref(), &[21, 22]);
    assert_eq!(inbound(&player).await.id, 0x303);
    assert!(player.inbound_packets.is_empty());

    client.try_enqueue_packet_data(encode_packet(0x310, &[1]).unwrap());
    client
        .send_packet_now_data(encode_packet(0x311, &[2]).unwrap())
        .await;
    assert_eq!(peer.receive().await.id, 0x310);
    assert_eq!(peer.receive().await.id, 0x311);
    client
        .send_packet_now_data(encode_packet(CANCEL, &[3]).unwrap())
        .await;
    assert!(!client.is_closed());
    client
        .send_packet_now_data(encode_packet(EDIT, &[4]).unwrap())
        .await;
    let edited = peer.receive().await;
    assert_eq!(edited.id, 0x400);
    assert_eq!(edited.payload.as_ref(), &[9; 128]);
    client
        .send_packet_now_data(encode_packet(REENTER, &[5]).unwrap())
        .await;
    assert_eq!(peer.receive().await.id, REENTER);
    assert_eq!(peer.receive().await.id, FOLLOWUP);
    assert_eq!(peer.receive().await.id, SILENT);
    client
        .send_packet_now_data(encode_packet(0x312, &[]).unwrap())
        .await;
    assert_eq!(peer.receive().await.id, 0x312);
    assert_eq!(client.pending_bytes.load(Ordering::SeqCst), 0);

    let reconfig_start = listener.seen.lock().unwrap().len();
    client.send_packet(&CStartConfiguration).await;
    assert_eq!(
        peer.receive().await.id,
        CStartConfiguration::to_id(CURRENT_MC_VERSION)
    );
    assert_eq!(user.encoder_state.load(), ConnectionState::Config);
    client.try_send_packet(&CKeepAlive::new(100));
    client
        .send_packet(&CConfigKeepAlive { keep_alive_id: 200 })
        .await;
    let keepalive = peer.receive().await;
    assert_eq!(keepalive.id, CConfigKeepAlive::to_id(CURRENT_MC_VERSION));
    assert_eq!(keepalive.payload.as_ref(), &200i64.to_be_bytes());
    client.send_packet(&CFinishConfig).await;
    assert_eq!(
        peer.receive().await.id,
        CFinishConfig::to_id(CURRENT_MC_VERSION)
    );
    assert_eq!(user.encoder_state.load(), ConnectionState::Play);
    assert_eq!(
        &listener.seen.lock().unwrap()[reconfig_start..],
        &[
            (
                true,
                ConnectionState::Play,
                CStartConfiguration::to_id(CURRENT_MC_VERSION)
            ),
            (
                true,
                ConnectionState::Config,
                CConfigKeepAlive::to_id(CURRENT_MC_VERSION)
            ),
            (
                true,
                ConnectionState::Config,
                CFinishConfig::to_id(CURRENT_MC_VERSION)
            ),
        ],
    );
    listener.cancel_config_ack.store(true, Ordering::SeqCst);
    peer.send(&SConfigurationAcknowledged).await;
    peer.send(&SClientInformationConfig {
        locale: "fr_fr",
        view_distance: 12,
        chat_mode: VarInt(0),
        chat_colors: true,
        skin_parts: 0x7f,
        main_hand: VarInt(1),
        text_filtering: false,
        server_listing: true,
    })
    .await;
    peer.raw(encode_packet(SAcknowledgeFinishConfig::to_id(CURRENT_MC_VERSION), &[]).unwrap())
        .await;
    listener.cancelled_ack.notified().await;
    assert_eq!(user.decoder_state.load(), ConnectionState::Config);
    assert_eq!(client.config.load().locale, "es_es");
    assert_eq!(client.config.load().view_distance.get(), 12);
    assert_eq!(user.info().config.unwrap().locale, "es_es");
    assert!(!client.is_closed());
    peer.raw(encode_packet(SAcknowledgeFinishConfig::to_id(CURRENT_MC_VERSION), &[]).unwrap())
        .await;
    peer.raw(encode_packet(0x304, &[]).unwrap()).await;
    assert_eq!(inbound(&player).await.id, 0x304);
    assert_eq!(user.decoder_state.load(), ConnectionState::Play);
    assert_eq!(player.config.load().locale, "es_es");
    assert!(!client.is_closed());
    client.close();
    inbound_loop.await.unwrap();
    client.await_tasks().await;
    client.player.store(Arc::new(None));
    drop(player);
    assert!(user.player().is_none());

    let (mut pending, mut peer) = connection(&server).await;
    let user = pending.user.clone();
    let body = encode_packet(0x320, &[7; 128]).unwrap();
    let mut frame = Vec::new();
    peer.writer.frame_packet(&body, &mut frame).unwrap();
    peer.writer.write_frame(&frame[..3]).await.unwrap();
    peer.writer.flush().await.unwrap();
    let reading = tokio::spawn(async move { pending.get_packet().await.unwrap() });
    tokio::time::sleep(Duration::from_millis(25)).await;
    user.send_packet(0x321, &[8], false).unwrap();
    assert_eq!(peer.receive().await.id, 0x321);
    peer.writer.write_frame(&frame[3..]).await.unwrap();
    peer.writer.flush().await.unwrap();
    let packet = reading.await.unwrap();
    assert_eq!(packet.id, 0x320);
    assert_eq!(packet.payload.as_ref(), &[7; 128]);

    let seen = listener.seen.lock().unwrap();
    for state in [
        ConnectionState::HandShake,
        ConnectionState::Status,
        ConnectionState::Login,
        ConnectionState::Config,
        ConnectionState::Play,
    ] {
        assert!(seen.iter().any(|&(sent, phase, _)| !sent && phase == state));
    }
    for id in [EDIT, CANCEL, REENTER, FOLLOWUP] {
        assert_eq!(
            seen.iter()
                .filter(|&&(sent, state, seen_id)| sent
                    && state == ConnectionState::Play
                    && seen_id == id)
                .count(),
            1
        );
    }
    assert!(!seen.iter().any(|&(sent, _, id)| sent && id == SILENT));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn packet_events_cover_java_connection_pipeline() {
    let world = tempfile::tempdir().unwrap();
    let basic = BasicConfiguration {
        default_level_name: world.path().join("world").to_string_lossy().into_owned(),
        allow_nether: false,
        allow_end: false,
        allow_chat_reports: false,
        use_favicon: false,
        ..Default::default()
    };
    let mut advanced = AdvancedConfiguration::default();
    advanced.networking.java.online_mode = false;
    advanced.networking.java.encryption = false;
    advanced.networking.java.compression.enabled = true;
    advanced.networking.java.keep_alive_time = 600;
    advanced.networking.bedrock.online_mode = false;
    let server = Server::new(
        basic,
        advanced,
        TelemetryConfig::default(),
        VanillaData {
            banned_ip_list: std::sync::RwLock::default(),
            banned_player_list: std::sync::RwLock::default(),
            operator_config: std::sync::RwLock::default(),
            user_cache: std::sync::RwLock::default(),
            whitelist_config: std::sync::RwLock::default(),
        },
    )
    .await;
    let listener = Arc::new(Listener {
        seen: Mutex::new(Vec::new()),
        cancel_compression: AtomicBool::new(true),
        cancel_config_ack: AtomicBool::new(true),
        cancelled_ack: Notify::new(),
        player: Mutex::new(Weak::new()),
    });
    server.plugin_manager.register::<PacketReceivedEvent, _>(
        listener.clone(),
        EventPriority::Normal,
        true,
    );
    server.plugin_manager.register::<PacketSentEvent, _>(
        listener.clone(),
        EventPriority::Normal,
        true,
    );
    let run = tokio::spawn(exercise_pipeline(server.clone(), listener));
    let result = timeout(Duration::from_secs(60), run).await;
    server.shutdown().await;
    result
        .expect("packet pipeline timed out")
        .expect("packet pipeline failed");
}
