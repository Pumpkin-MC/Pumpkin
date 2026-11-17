use std::{net::SocketAddr, time::Duration};

use bytes::Bytes;
use pumpkin_core::net::java::{JavaReadHalf, JavaWriteHalf};
use pumpkin_data::packet::CURRENT_MC_VERSION;
use pumpkin_protocol::{
    ClientPacket, ConnectionState, RawPacket, ServerPacket,
    codec::var_int::VarInt,
    java::{
        client::{
            config::{CFinishConfig, CKnownPacks},
            login::CLoginSuccess,
            play::{CLogin, CPlayerPosition},
        },
        packet_decoder::TCPNetworkDecoder,
        packet_encoder::TCPNetworkEncoder,
        server::{
            config::{SAcknowledgeFinishConfig, SClientInformationConfig, SKnownPacks},
            handshake::SHandShake,
            login::{SLoginAcknowledged, SLoginStart},
            play::{
                Action, SChatCommand, SConfirmTeleport, SPlayerAction, SPlayerCommand,
                SPlayerLoaded, SPlayerPosition, SSetHeldItem, SUseItemOn, Status,
            },
        },
    },
    packet::MultiVersionJavaPacket,
    ser::NetworkWriteExt,
};
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
use tokio::{
    io::{AsyncWrite, BufReader, BufWriter},
    net::TcpStream,
    time::timeout,
};
use uuid::Uuid;

pub const PACKET_TIMEOUT: Duration = Duration::from_secs(60);
pub const MAX_POLL_TICKS: usize = 500;

pub struct HeadlessBot {
    pub name: String,
    pub reader: TCPNetworkDecoder<BufReader<JavaReadHalf>>,
    pub writer: TCPNetworkEncoder<BufWriter<JavaWriteHalf>>,
    pub position: Vector3<f64>,
    pub yaw: f32,
    pub pitch: f32,
}

impl HeadlessBot {
    pub async fn send_packet<P: ClientPacket + MultiVersionJavaPacket>(
        &mut self,
        packet: &P,
    ) -> Result<(), String> {
        send_framed_packet(&mut self.writer, packet).await
    }

    pub async fn next_packet(&mut self) -> Result<RawPacket, String> {
        match timeout(PACKET_TIMEOUT, self.reader.get_raw_packet()).await {
            Ok(Ok(packet)) => Ok(packet),
            Ok(Err(err)) => Err(format!("Bot '{}' packet decode error: {err}", self.name)),
            Err(_) => Err(format!(
                "Bot '{}' timed out waiting for packet from server",
                self.name
            )),
        }
    }

    pub async fn connect_and_join(addr: SocketAddr, name: &str) -> Result<Self, String> {
        let stream = TcpStream::connect(addr)
            .await
            .map_err(|err| format!("Bot '{name}' failed to connect to {addr}: {err}"))?;
        let _ = stream.set_nodelay(true);
        let (read_half, write_half) = stream.into_split();
        Self::join(Box::new(read_half), Box::new(write_half), addr.port(), name).await
    }

    pub async fn join(
        read_half: JavaReadHalf,
        write_half: JavaWriteHalf,
        port: u16,
        name: &str,
    ) -> Result<Self, String> {
        let mut bot = Self {
            name: name.to_string(),
            reader: TCPNetworkDecoder::new(BufReader::new(read_half)),
            writer: TCPNetworkEncoder::new(BufWriter::new(write_half)),
            position: Vector3::new(0.0, 0.0, 0.0),
            yaw: 0.0,
            pitch: 0.0,
        };

        // 1. Handshake -> Login
        bot.send_packet(&SHandShake {
            protocol_version: VarInt(CURRENT_MC_VERSION.protocol_version()),
            server_address: "127.0.0.1".into(),
            server_port: port,
            next_state: ConnectionState::Login,
        })
        .await?;

        bot.send_packet(&SLoginStart {
            name: name.into(),
            uuid: Uuid::new_v4(),
        })
        .await?;

        let login_success_id = CLoginSuccess::to_id(CURRENT_MC_VERSION);
        loop {
            let pkt = bot.next_packet().await?;
            if pkt.id == login_success_id {
                break;
            }
        }

        bot.send_packet(&SLoginAcknowledged).await?;

        // 2. Configuration state
        bot.send_packet(&SClientInformationConfig {
            locale: "en_us",
            view_distance: 4,
            chat_mode: VarInt(0),
            chat_colors: true,
            skin_parts: 0x7F,
            main_hand: VarInt(1),
            text_filtering: false,
            server_listing: true,
        })
        .await?;

        let known_packs_id = CKnownPacks::to_id(CURRENT_MC_VERSION);
        let finish_config_id = CFinishConfig::to_id(CURRENT_MC_VERSION);
        loop {
            let pkt = bot.next_packet().await?;
            if pkt.id == known_packs_id {
                bot.send_packet(&SKnownPacks {
                    known_packs: Vec::new(),
                })
                .await?;
            } else if pkt.id == finish_config_id {
                bot.send_packet(&SAcknowledgeFinishConfig).await?;
                break;
            }
        }

        // 3. Play state: wait for CLogin and initial CPlayerPosition synchronize packet
        let play_login_id = CLogin::to_id(CURRENT_MC_VERSION);
        let player_pos_id = CPlayerPosition::to_id(CURRENT_MC_VERSION);
        let mut saw_play_login = false;
        let mut saw_initial_pos = false;

        while !(saw_play_login && saw_initial_pos) {
            let pkt = bot.next_packet().await?;
            if pkt.id == play_login_id {
                saw_play_login = true;
            } else if pkt.id == player_pos_id {
                let mut slice = &pkt.payload[..];
                let pos_pkt = CPlayerPosition::read(&mut slice, &CURRENT_MC_VERSION)
                    .map_err(|err| format!("Failed to read CPlayerPosition: {err}"))?;
                bot.position = pos_pkt.position;
                bot.yaw = pos_pkt.yaw;
                bot.pitch = pos_pkt.pitch;

                bot.send_packet(&SConfirmTeleport {
                    teleport_id: pos_pkt.teleport_id,
                    position: pos_pkt.position,
                    yaw: pos_pkt.yaw,
                    pitch: pos_pkt.pitch,
                })
                .await?;
                bot.send_packet(&SPlayerLoaded).await?;
                saw_initial_pos = true;
            }
        }

        Ok(bot)
    }

    pub async fn move_to(&mut self, position: Vector3<f64>, on_ground: bool) -> Result<(), String> {
        self.position = position;
        self.send_packet(&SPlayerPosition {
            position,
            collision: u8::from(on_ground),
        })
        .await
    }

    pub async fn send_command(&mut self, command: &str) -> Result<(), String> {
        self.send_packet(&SChatCommand { command }).await
    }

    pub async fn set_held_item(&mut self, slot: i16) -> Result<(), String> {
        self.send_packet(&SSetHeldItem { slot }).await
    }

    pub async fn send_player_command(
        &mut self,
        entity_id: i32,
        action: Action,
    ) -> Result<(), String> {
        self.send_packet(&SPlayerCommand {
            entity_id: VarInt(entity_id),
            action,
            jump_boost: VarInt(0),
        })
        .await
    }

    pub async fn use_block(&mut self, position: BlockPos, sequence: i32) -> Result<(), String> {
        self.send_packet(&SUseItemOn {
            hand: VarInt(0),
            position,
            face: VarInt(1),
            cursor_pos: Vector3::new(0.5, 0.5, 0.5),
            inside_block: false,
            is_against_world_border: false,
            sequence: VarInt(sequence),
        })
        .await
    }

    pub async fn start_digging(&mut self, position: BlockPos, sequence: i32) -> Result<(), String> {
        self.send_packet(&SPlayerAction {
            status: VarInt(Status::StartedDigging as i32),
            position,
            face: 1,
            sequence: VarInt(sequence),
        })
        .await
    }
}

pub async fn send_framed_packet<P: ClientPacket + MultiVersionJavaPacket, W: AsyncWrite + Unpin>(
    writer: &mut TCPNetworkEncoder<BufWriter<W>>,
    packet: &P,
) -> Result<(), String> {
    let packet_id = P::to_id(CURRENT_MC_VERSION);
    let mut buf = Vec::new();
    buf.write_var_int(&VarInt(packet_id))
        .map_err(|err| format!("Failed to write packet id {packet_id}: {err:?}"))?;
    packet
        .write_packet_data(&mut buf, &CURRENT_MC_VERSION)
        .map_err(|err| format!("Failed to serialize packet {packet_id}: {err:?}"))?;
    writer
        .write_packet(Bytes::from(buf))
        .await
        .map_err(|err| format!("Failed to write packet to socket: {err}"))?;
    writer
        .flush()
        .await
        .map_err(|err| format!("Failed to flush socket: {err}"))
}
