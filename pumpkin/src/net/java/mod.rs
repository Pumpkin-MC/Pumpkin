use crate::entity::player::Player;
use crate::net::EncryptionError;
use crate::net::GameProfile;
use crate::net::PlayerConfig;
use crate::plugin::api::events::world::chunk_send::ChunkSend;
use bytes::Bytes;
use crossbeam::atomic::AtomicCell;
use pumpkin_config::networking::compression::CompressionInfo;
use pumpkin_data::packet::CURRENT_MC_VERSION;
use pumpkin_protocol::ClientPacket;
use pumpkin_protocol::ConnectionState;
use pumpkin_protocol::PacketDecodeError;
use pumpkin_protocol::RawPacket;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::config::CConfigDisconnect;
use pumpkin_protocol::java::client::login::CLoginDisconnect;
use pumpkin_protocol::java::client::play::CChunkBatchEnd;
use pumpkin_protocol::java::client::play::CChunkBatchStart;
use pumpkin_protocol::java::client::play::CChunkData;
use pumpkin_protocol::java::client::play::CPlayDisconnect;
use pumpkin_protocol::java::packet_decoder::TCPNetworkDecoder;
use pumpkin_protocol::java::packet_encoder::TCPNetworkEncoder;
use pumpkin_protocol::packet::MultiVersionJavaPacket;
use pumpkin_protocol::ser::NetworkWriteExt;
use pumpkin_protocol::ser::WritingError;
use pumpkin_util::text::TextComponent;
use pumpkin_util::version::JavaMinecraftVersion;
use pumpkin_world::level::SyncChunk;
use std::io::Write;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicI32;
use std::time::Instant;
use tokio::io::BufReader;
use tokio::io::BufWriter;
use tokio::net::TcpStream;
use tokio::net::tcp::OwnedReadHalf;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::Mutex;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use tracing::debug;
use tracing::error;
use tracing::warn;

pub mod config;
mod dispatch;
pub mod handshake;
pub mod login;
pub mod play;
pub mod recipe_helper;
pub mod status;

pub struct JavaClient {
    pub id: u64,
    pub version: AtomicCell<JavaMinecraftVersion>,
    /// The client's game profile information.
    pub gameprofile: Mutex<Option<GameProfile>>,
    /// The nonce sent with the pending encryption request.
    pub verify_token: Mutex<Option<[u8; 4]>>,
    /// The client's configuration settings, Optional
    pub config: Mutex<Option<PlayerConfig>>,
    /// The Address used to connect to the Server, Send in the Handshake
    pub server_address: Mutex<Box<str>>,
    /// The current connection state of the client (e.g., Handshaking, Status, Play).
    pub connection_state: AtomicCell<ConnectionState>,
    /// The client's IP address.
    pub address: Mutex<SocketAddr>,
    /// The client's brand or modpack information, Optional.
    pub brand: Mutex<Option<String>>,
    pub player: Mutex<Option<Arc<Player>>>,
    /// A collection of tasks associated with this client. The tasks await completion when removing the client.
    tasks: TaskTracker,
    /// An notifier that is triggered when this client is closed.
    close_token: CancellationToken,
    /// FIFO queue of serialized packets to send to the network.
    outgoing_packet_queue_send: Sender<OutgoingPacket>,
    /// FIFO queue of serialized packets to send to the network.
    outgoing_packet_queue_recv: Option<Receiver<OutgoingPacket>>,
    /// The packet encoder for outgoing packets.
    network_writer: Arc<Mutex<TCPNetworkEncoder<BufWriter<OwnedWriteHalf>>>>,
    /// The packet decoder for incoming packets.
    network_reader: Mutex<TCPNetworkDecoder<BufReader<OwnedReadHalf>>>,
    /// Keep Alive:
    ///
    /// Whether we are waiting for a response after sending a keep alive packet.
    pub wait_for_keep_alive: AtomicBool,
    /// The keep alive packet payload we send. The client should respond with the same id.
    pub keep_alive_id: AtomicCell<i64>,
    /// The last time we sent a keep alive packet.
    pub last_keep_alive_time: AtomicCell<Instant>,

    pub packet_sequence: AtomicI32,
}

struct OutgoingPacket {
    data: Bytes,
    completion: Option<oneshot::Sender<()>>,
}

impl OutgoingPacket {
    const fn normal(data: Bytes) -> Self {
        Self {
            data,
            completion: None,
        }
    }

    const fn with_completion(data: Bytes, completion: oneshot::Sender<()>) -> Self {
        Self {
            data,
            completion: Some(completion),
        }
    }
}

impl JavaClient {
    #[must_use]
    pub fn new(tcp_stream: TcpStream, address: SocketAddr, id: u64) -> Self {
        let (read, write) = tcp_stream.into_split();
        let (send, recv) = tokio::sync::mpsc::channel(4096);
        Self {
            id,
            gameprofile: Mutex::new(None),
            verify_token: Mutex::new(None),
            config: Mutex::new(None),
            server_address: Mutex::new("".into()),
            address: Mutex::new(address),
            connection_state: AtomicCell::new(ConnectionState::HandShake),
            close_token: CancellationToken::new(),
            tasks: TaskTracker::new(),
            outgoing_packet_queue_send: send,
            outgoing_packet_queue_recv: Some(recv),
            version: AtomicCell::new(CURRENT_MC_VERSION),
            network_writer: Arc::new(Mutex::new(TCPNetworkEncoder::new(BufWriter::new(write)))),
            network_reader: Mutex::new(TCPNetworkDecoder::new(BufReader::new(read))),
            brand: Mutex::new(None),
            player: Mutex::new(None),
            wait_for_keep_alive: AtomicBool::new(false),
            keep_alive_id: AtomicCell::new(0),
            last_keep_alive_time: AtomicCell::new(std::time::Instant::now()),
            packet_sequence: AtomicI32::new(-1),
        }
    }
    pub async fn set_encryption(
        &self,
        shared_secret: &[u8], // decrypted
    ) -> Result<(), EncryptionError> {
        let crypt_key: [u8; 16] = shared_secret
            .try_into()
            .map_err(|_| EncryptionError::SharedWrongLength)?;
        self.network_reader
            .lock()
            .await
            .set_encryption(&crypt_key)
            .map_err(|_| EncryptionError::AlreadyEncrypted)?;
        self.network_writer
            .lock()
            .await
            .set_encryption(&crypt_key)
            .map_err(|_| EncryptionError::AlreadyEncrypted)?;
        Ok(())
    }

    pub async fn set_compression(&self, compression: CompressionInfo) {
        if compression.level > 9 {
            error!("Invalid compression level! Clients will not be able to read this!");
        }

        self.network_reader
            .lock()
            .await
            .set_compression(compression.threshold as usize);

        self.network_writer
            .lock()
            .await
            .set_compression((compression.threshold as usize, compression.level));
    }
    pub async fn await_tasks(&self) {
        self.tasks.close();
        self.tasks.wait().await;
    }

    /// Spawns a task associated with this client. All tasks spawned with this method are awaited
    /// when the client. This means tasks should complete in a reasonable amount of time or select
    /// on `Self::await_close_interrupt` to cancel the task when the client is closed
    ///
    /// Returns an `Option<JoinHandle<F::Output>>`. If the client is closed, this returns `None`.
    pub fn spawn_task<F>(&self, task: F) -> Option<JoinHandle<F::Output>>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        if self.close_token.is_cancelled() {
            None
        } else {
            Some(self.tasks.spawn(task))
        }
    }

    pub async fn send_chunks(&self, chunks: &[SyncChunk]) {
        let player = self.player.lock().await.clone();
        let Some(player) = player.as_ref() else {
            debug!(
                "send_chunks: player not set yet, dropping {} chunks",
                chunks.len()
            );
            return;
        };
        let Some(server) = player.world().server.upgrade() else {
            return;
        };

        self.send_packet_now(&CChunkBatchStart).await;
        for chunk in chunks {
            let event = ChunkSend::new(player.world(), chunk.clone());
            let event = server.plugin_manager.fire(event).await;
            if event.cancelled {
                continue;
            }

            let mut buf = Vec::new();
            let version = self.version.load();
            if let Err(err) = buf.write_var_int(&VarInt(CChunkData::to_id(version))) {
                error!("Failed to write chunk data id: {err:?}");
                continue;
            }
            if let Err(err) = CChunkData(chunk).write_packet_data(&mut buf, &version) {
                error!("Failed to write chunk data: {err:?}");
                continue;
            }
            self.send_packet_now_data(buf.into()).await;
        }
        self.send_packet_now(&CChunkBatchEnd::new(chunks.len() as u16))
            .await;
    }

    pub async fn enqueue_packet<P: ClientPacket>(&self, packet: &P) {
        let mut buf = Vec::new();
        let writer = &mut buf;
        if let Err(err) = self.write_packet(packet, writer) {
            error!("Failed to write packet: {err:?}");
            return;
        }
        let payload = Bytes::from(buf);

        let player = self.player.lock().await.clone();
        let cancelled = if let Some(player) = player.as_ref() {
            player
                .fire_packet_sent_no_obj(P::to_id(self.version.load()), payload.clone())
                .await
        } else {
            false
        };

        if !cancelled {
            self.enqueue_packet_data(payload).await;
        }
    }

    pub fn try_enqueue_packet<P: ClientPacket>(&self, packet: &P) {
        let mut buf = Vec::new();
        let writer = &mut buf;
        if let Err(err) = self.write_packet(packet, writer) {
            error!("Failed to write packet: {err:?}");
            return;
        }
        self.try_enqueue_packet_data(buf.into());
    }

    /// Queues a clientbound packet to be sent to the connected client. Queued chunks are sent
    /// in-order to the client
    ///
    /// # Arguments
    ///
    /// * `packet`: A reference to a packet object implementing the `ClientPacket` trait.
    pub async fn enqueue_packet_data(&self, packet_data: Bytes) {
        if let Err(err) = self
            .outgoing_packet_queue_send
            .send(OutgoingPacket::normal(packet_data))
            .await
        {
            // This is expected to fail if we are closed
            if !self.close_token.is_cancelled() {
                error!(
                    "Failed to add packet to the outgoing packet queue for client {}: {}",
                    self.id, err
                );
            }
        }
    }

    pub fn try_enqueue_packet_data(&self, packet_data: Bytes) {
        if let Err(err) = self
            .outgoing_packet_queue_send
            .try_send(OutgoingPacket::normal(packet_data))
        {
            match err {
                tokio::sync::mpsc::error::TrySendError::Full(_) => {
                    debug!(
                        "Failed to add packet to the outgoing packet queue for client {}: channel full",
                        self.id
                    );
                }
                tokio::sync::mpsc::error::TrySendError::Closed(_) => {
                    if !self.close_token.is_cancelled() {
                        error!(
                            "Failed to add packet to the outgoing packet queue for client {}: channel closed",
                            self.id
                        );
                    }
                }
            }
        }
    }

    pub async fn await_close_interrupt(&self) {
        self.close_token.cancelled().await;
    }

    pub async fn get_packet(&self) -> Option<RawPacket> {
        let mut network_reader = self.network_reader.lock().await;
        tokio::select! {
            () = self.await_close_interrupt() => {
                debug!("Canceling player packet processing");
                None
            },
            packet_result = network_reader.get_raw_packet() => {
                match packet_result {
                    Ok(packet) => Some(packet),
                    Err(err) => {
                        if !matches!(err, PacketDecodeError::ConnectionClosed) {
                            warn!("Failed to decode packet from client {}: {}", self.id, err);
                            let text = format!("Error while reading incoming packet {err}");
                            self.kick(TextComponent::text(text)).await;
                        }
                        None
                    }
                }
            }
        }
    }

    pub async fn kick(&self, reason: TextComponent) {
        match self.connection_state.load() {
            ConnectionState::Login => {
                // TextComponent implements Serialize and writes in bytes instead of String, that's the reasib we only use content
                self.send_packet_now(&CLoginDisconnect::new(
                    serde_json::to_string(&reason.0).unwrap_or_else(|_| String::new()),
                ))
                .await;
            }
            ConnectionState::Config => {
                self.send_packet_now(&CConfigDisconnect::new(&reason.get_text()))
                    .await;
            }
            ConnectionState::Play => self.send_packet_now(&CPlayDisconnect::new(&reason)).await,
            _ => {}
        }
        debug!("Closing connection for {}", self.id);
        self.close();
    }

    pub async fn send_packet_now<P: ClientPacket>(&self, packet: &P) {
        let mut packet_buf = Vec::new();
        let writer = &mut packet_buf;
        if let Err(err) = self.write_packet(packet, writer) {
            error!("Failed to write packet: {err:?}");
            return;
        }
        let payload = Bytes::from(packet_buf);

        let player = self.player.lock().await.clone();
        let cancelled = if let Some(player) = player.as_ref() {
            player
                .fire_packet_sent_no_obj(P::to_id(self.version.load()), payload.clone())
                .await
        } else {
            false
        };

        if !cancelled {
            self.send_packet_now_data(payload).await;
        }
    }

    pub async fn send_packet_now_data(&self, packet: Bytes) {
        let (completion_tx, completion_rx) = oneshot::channel();

        if let Err(err) = self
            .outgoing_packet_queue_send
            .send(OutgoingPacket::with_completion(packet, completion_tx))
            .await
        {
            // It is expected that the packet will fail if we are closed
            if !self.close_token.is_cancelled() {
                warn!(
                    "Failed to add packet to the outgoing packet queue for client {}: {}",
                    self.id, err
                );
                // We now need to close the connection to the client since the stream is in an
                // unknown state
                self.close();
            }
            return;
        }

        if completion_rx.await.is_err() && !self.close_token.is_cancelled() {
            // The outgoing packet task dropped before confirming the write.
            self.close();
        }
    }

    pub fn write_packet_for_version<P: ClientPacket>(
        packet: &P,
        version: JavaMinecraftVersion,
        mut write: impl Write,
    ) -> Result<(), WritingError> {
        let version_number = P::to_id(version);
        if version_number == -1 {
            error!(
                "Packet ID for version {} is invalid ({} at latest)",
                version,
                P::to_id(CURRENT_MC_VERSION),
            );
        }
        write.write_var_int(&VarInt(version_number))?;
        packet.write_packet_data(write, &version)
    }

    pub fn serialize_packet_for_version<P: ClientPacket>(
        packet: &P,
        version: JavaMinecraftVersion,
    ) -> Result<Bytes, WritingError> {
        let mut packet_buf = Vec::new();

        Self::write_packet_for_version(packet, version, &mut packet_buf)?;
        Ok(packet_buf.into())
    }

    pub fn write_packet<P: ClientPacket>(
        &self,
        packet: &P,
        write: impl Write,
    ) -> Result<(), WritingError> {
        Self::write_packet_for_version(packet, self.version.load(), write)
    }
    pub fn start_outgoing_packet_task(&mut self) {
        const MAX_BATCH_SIZE: usize = 64;

        let mut packet_receiver = self
            .outgoing_packet_queue_recv
            .take()
            .expect("This was set in the new fn");
        let close_token = self.close_token.clone();
        let writer = self.network_writer.clone();
        let id = self.id;
        self.spawn_task(async move {
            while !close_token.is_cancelled() {
                let recv_result = tokio::select! {
                    () = close_token.cancelled() => None,
                    res = packet_receiver.recv() => res,
                };

                let Some(packet_data) = recv_result else {
                    break;
                };

                let mut packet_batch = Vec::with_capacity(MAX_BATCH_SIZE);
                packet_batch.push(packet_data);

                while packet_batch.len() < MAX_BATCH_SIZE {
                    match packet_receiver.try_recv() {
                        Ok(packet_data) => packet_batch.push(packet_data),
                        Err(TryRecvError::Disconnected | TryRecvError::Empty) => break,
                    }
                }

                let send_failed = {
                    let mut writer = writer.lock().await;
                    let mut failed = false;
                    for packet in &packet_batch {
                        if let Err(err) = writer.write_packet(packet.data.clone()).await {
                            failed = true;
                            // It is expected that the packet will fail if we are closed
                            if !close_token.is_cancelled() {
                                warn!("Failed to send packet to client {id}: {err}");
                            }
                            break;
                        }
                    }

                    if !failed && let Err(err) = writer.flush().await {
                        failed = true;
                        if !close_token.is_cancelled() {
                            warn!("Failed to flush packet batch for client {id}: {err}");
                        }
                    }
                    failed
                };

                if send_failed {
                    // We now need to close the connection to the client since the stream is in an unknown state.
                    close_token.cancel();
                    break;
                }

                for packet in packet_batch {
                    if let Some(completion) = packet.completion {
                        let _ = completion.send(());
                    }
                }
            }
        });
    }

    /// Closes the connection to the client.
    ///
    /// This function marks the connection as closed using an atomic flag. It's generally preferable
    /// to use the `kick` function if you want to send a specific message to the client explaining the reason for the closure.
    /// However, use `close` in scenarios where sending a message is not critical or might not be possible (e.g., sudden connection drop).
    ///
    /// # Notes
    ///
    /// This function does not attempt to send any disconnect packets to the client.
    pub fn close(&self) {
        self.close_token.cancel();
    }

    pub fn is_closed(&self) -> bool {
        self.close_token.is_cancelled()
    }
}
