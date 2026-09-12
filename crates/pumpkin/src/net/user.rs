use std::{
    io,
    net::SocketAddr,
    sync::atomic::{AtomicUsize, Ordering},
    sync::{Arc, Mutex, RwLock, Weak},
};

use bytes::Bytes;
use crossbeam::atomic::AtomicCell;
use pumpkin_protocol::{ConnectionState, MAX_PACKET_DATA_SIZE};
use pumpkin_util::version::{BedrockMinecraftVersion, JavaMinecraftVersion};
use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use super::{
    ClientPlatform, GameProfile, MAX_PENDING_BYTES, PlayerConfig, decrement_pending_bytes,
};
use crate::{entity::player::Player, server::Server};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edition {
    Java,
    Bedrock,
}

#[derive(Clone)]
pub struct UserInfo {
    pub address: SocketAddr,
    pub server_address: String,
    pub server_port: Option<u16>,
    pub profile: Option<GameProfile>,
    pub config: Option<PlayerConfig>,
    pub brand: Option<String>,
}

pub(crate) struct OutgoingPacket {
    pub data: Bytes,
    pub state: Option<ConnectionState>,
    pub silent: bool,
    pub completion: Option<oneshot::Sender<()>>,
}

impl OutgoingPacket {
    pub const fn normal(data: Bytes) -> Self {
        Self {
            data,
            state: None,
            silent: false,
            completion: None,
        }
    }
}

/// A connection identity retained before and after the player is created.
pub struct User {
    pub id: Uuid,
    pub edition: Edition,
    pub decoder_state: Arc<AtomicCell<ConnectionState>>,
    pub encoder_state: AtomicCell<ConnectionState>,
    pub java_version: Arc<AtomicCell<JavaMinecraftVersion>>,
    pub bedrock_version: Arc<AtomicCell<BedrockMinecraftVersion>>,
    pub protocol_version: AtomicCell<Option<i32>>,
    pub(crate) server: Weak<Server>,
    pub(crate) close_token: CancellationToken,
    pub(crate) pending_bytes: Arc<AtomicUsize>,
    pub(crate) outgoing: mpsc::UnboundedSender<OutgoingPacket>,
    receiver: Mutex<Option<mpsc::UnboundedReceiver<OutgoingPacket>>>,
    info: RwLock<UserInfo>,
    player: RwLock<Weak<Player>>,
    expected_states: Mutex<std::collections::VecDeque<ConnectionState>>,
}

impl User {
    #[must_use]
    pub fn new(address: SocketAddr, edition: Edition, server: Weak<Server>) -> Arc<Self> {
        let (outgoing, receiver) = mpsc::unbounded_channel();
        Arc::new(Self {
            id: Uuid::new_v4(),
            edition,
            decoder_state: Arc::new(AtomicCell::new(ConnectionState::HandShake)),
            encoder_state: AtomicCell::new(ConnectionState::HandShake),
            java_version: Arc::new(AtomicCell::new(JavaMinecraftVersion::Unknown)),
            bedrock_version: Arc::new(AtomicCell::new(BedrockMinecraftVersion::Unknown)),
            protocol_version: AtomicCell::new(None),
            server,
            close_token: CancellationToken::new(),
            pending_bytes: Arc::new(AtomicUsize::new(0)),
            outgoing,
            receiver: Mutex::new(Some(receiver)),
            info: RwLock::new(UserInfo {
                address,
                server_address: String::new(),
                server_port: None,
                profile: None,
                config: None,
                brand: None,
            }),
            player: RwLock::new(Weak::new()),
            expected_states: Mutex::new(std::collections::VecDeque::new()),
        })
    }

    pub fn player(&self) -> Option<Arc<Player>> {
        self.player
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .upgrade()
    }

    pub(crate) fn set_player(&self, player: &Arc<Player>) {
        *self
            .player
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Arc::downgrade(player);
    }

    pub(crate) fn update_info(&self, update: impl FnOnce(&mut UserInfo)) {
        update(
            &mut self
                .info
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
    }

    pub fn info(&self) -> UserInfo {
        let mut info = self
            .info
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        if let Some(player) = self.player() {
            info.profile = Some(player.gameprofile.clone());
            info.config = Some((**player.config.load()).clone());
            if let ClientPlatform::Java(client) = player.client.as_ref() {
                info.address = client.address;
                info.brand.clone_from(&**client.brand.load());
                if self.decoder_state.load() == ConnectionState::Config {
                    info.config = Some((**client.config.load()).clone());
                }
            }
        }
        info
    }

    pub fn is_closed(&self) -> bool {
        self.close_token.is_cancelled()
    }

    pub(crate) fn expect_state(&self, state: ConnectionState) {
        self.expected_states
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push_back(state);
    }

    pub(crate) fn acknowledge_state(&self, state: ConnectionState) -> bool {
        let mut expected = self
            .expected_states
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if expected.front() != Some(&state) {
            return false;
        }
        expected.pop_front();
        true
    }

    pub fn close(&self) {
        self.close_token.cancel();
    }

    pub(crate) fn take_outgoing(&self) -> Option<mpsc::UnboundedReceiver<OutgoingPacket>> {
        self.receiver
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
    }

    pub(crate) fn enqueue(&self, packet: OutgoingPacket) -> io::Result<()> {
        if self.is_closed() {
            return Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "connection closed",
            ));
        }
        let len = packet.data.len();
        if len > MAX_PACKET_DATA_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "packet too large",
            ));
        }
        let previous = self.pending_bytes.fetch_add(len, Ordering::AcqRel);
        if previous.saturating_add(len) > MAX_PENDING_BYTES {
            decrement_pending_bytes(&self.pending_bytes, len);
            self.close();
            return Err(io::Error::other("outbound packet buffer overflow"));
        }
        self.outgoing.send(packet).map_err(|_| {
            decrement_pending_bytes(&self.pending_bytes, len);
            io::Error::new(io::ErrorKind::BrokenPipe, "packet queue closed")
        })
    }

    /// Queues a packet body; silent sends bypass packet listeners.
    pub fn send_packet(&self, packet_id: i32, payload: &[u8], silent: bool) -> io::Result<()> {
        let data = match self.edition {
            Edition::Java => crate::plugin::server::packet::encode_packet(packet_id, payload)?,
            Edition::Bedrock => {
                if !(0..=0x3ff).contains(&packet_id) || payload.len() > MAX_PACKET_DATA_SIZE {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "invalid Bedrock packet ID or size",
                    ));
                }
                let mut data = Vec::new();
                pumpkin_protocol::bedrock::packet_encoder::BedrockBatchEncoder::new()
                    .write_game_packet(
                        packet_id as u16,
                        pumpkin_protocol::bedrock::SubClient::Main,
                        pumpkin_protocol::bedrock::SubClient::Main,
                        payload,
                        &mut data,
                    )?;
                data.into()
            }
        };
        self.enqueue(OutgoingPacket {
            data,
            state: None,
            silent,
            completion: None,
        })
    }
}
