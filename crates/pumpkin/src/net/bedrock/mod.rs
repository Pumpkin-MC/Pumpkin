pub mod nethernet;
pub mod play;
pub mod status;
use crossbeam::atomic::AtomicCell;
use std::{
    collections::HashMap,
    io::{Cursor, Error, Write},
    net::SocketAddr,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU32, Ordering},
    },
};

use tracing::{debug, error, warn};

use bytes::Bytes;
use pumpkin_config::networking::compression::CompressionInfo;
use pumpkin_protocol::{
    BClientPacket, PacketDecodeError, RawPacket,
    bedrock::{
        BEDROCK_GAME_PACKET, SubClient,
        client::{
            client_cache_miss_response::{CClientCacheMissResponse, MissingBlobData},
            disconnect::CDisconnect,
            level_chunk::CLevelChunk,
        },
        packet_decoder::BedrockBatchDecoder,
        packet_encoder::BedrockBatchEncoder,
        server::{
            actor_event::SActorEvent, animate::SAnimate, block_pick_request::SBlockPickRequest,
            client_cache_blob_status::SClientCacheBlobStatus,
            client_cache_status::SClientCacheStatus, command_request::SCommandRequest,
            container_close::SContainerClose, emote::SEmote, emote_list::SEmoteList,
            interact::SInteract, inventory_transaction::SInventoryTransaction,
            loading_screen::SLoadingScreen, login::SLogin, mob_equipment::SMobEquipment,
            packet_violation_warning::SPacketViolationWarning, player_action::SPlayerAction,
            player_auth_input::SPlayerAuthInput, request_ability::SRequestAbility,
            request_chunk_radius::SRequestChunkRadius,
            request_network_settings::SRequestNetworkSettings,
            resource_pack_client_response::SResourcePackClientResponse, respawn::SRespawn,
            set_local_player_as_initialized::SSetLocalPlayerAsInitialized,
            set_player_inventory_options::SSetPlayerInventoryOptions, text::SText,
        },
    },
    packet::Packet,
    serial::{PacketRead, PacketReadSlice},
};
use tokio::{
    sync::mpsc::{Receiver, Sender},
    sync::{Mutex, RwLock, oneshot},
    task::JoinHandle,
};

use tokio_util::{sync::CancellationToken, task::TaskTracker};

pub mod login;
use self::nethernet::NetherNetSession;
use crate::{
    entity::player::Player,
    net::{DisconnectReason, PacketHandlerResult, PacketRateLimiter},
    plugin::api::events::world::chunk_send::ChunkSend,
    server::Server,
};
use arc_swap::ArcSwap;
use pumpkin_protocol::bedrock::server::login::ClientData;
use pumpkin_util::version::BedrockMinecraftVersion;
use pumpkin_world::level::SyncChunk;

pub struct OutgoingPacket {
    pub data: Bytes,
    pub completion: Option<oneshot::Sender<()>>,
}

impl OutgoingPacket {
    pub const fn normal(data: Bytes) -> Self {
        Self {
            data,
            completion: None,
        }
    }

    pub const fn priority(data: Bytes, completion: oneshot::Sender<()>) -> Self {
        Self {
            data,
            completion: Some(completion),
        }
    }
}

pub struct BedrockClient {
    session: Arc<NetherNetSession>,
    /// The client's IP address.
    pub address: SocketAddr,
    pub player: ArcSwap<Option<Arc<Player>>>,
    pub version: AtomicCell<BedrockMinecraftVersion>,
    pub client_data: ArcSwap<Option<Arc<ClientData>>>,
    /// All Bedrock clients
    /// This list is used to remove the client if the connection gets closed
    pub be_clients: Arc<Mutex<HashMap<SocketAddr, Arc<Self>>>>,

    tasks: TaskTracker,
    rt_handle: tokio::runtime::Handle,
    outgoing_packet_queue_send: Sender<OutgoingPacket>,
    /// A queue of serialized packets to send to the network
    outgoing_packet_queue_recv: Mutex<Option<Receiver<OutgoingPacket>>>,

    outgoing_packet_priority_send: Sender<OutgoingPacket>,
    outgoing_packet_priority_recv: Mutex<Option<Receiver<OutgoingPacket>>>,

    /// The packet encoder for outgoing packets.
    network_writer: Arc<RwLock<BedrockBatchEncoder>>,
    /// The packet decoder for incoming packets.
    network_reader: Mutex<BedrockBatchDecoder>,

    /// The next form ID to use for custom forms.
    pub next_form_id: AtomicU32,
    pub inventory_opened: AtomicBool,
    pub client_cache_supported: AtomicBool,
    pub blob_cache: std::sync::Mutex<HashMap<u64, Vec<u8>>>,
    /// An notifier that is triggered when this client is closed.
    close_token: CancellationToken,
    last_seen: Arc<AtomicCell<std::time::Instant>>,
    incoming_game_packet_send: Sender<RawPacket>,
    incoming_game_packet_recv: Mutex<Option<Receiver<RawPacket>>>,
    /// Packet rate limiter for incoming client packets.
    pub packet_limiter: PacketRateLimiter,
}

impl BedrockClient {
    #[must_use]
    pub fn new(
        session: Arc<NetherNetSession>,
        address: SocketAddr,
        be_clients: Arc<Mutex<HashMap<SocketAddr, Arc<Self>>>>,
        packet_limiter: PacketRateLimiter,
    ) -> Self {
        let (send, recv) = tokio::sync::mpsc::channel(4096);
        let (priority_send, priority_recv) = tokio::sync::mpsc::channel(4096);
        let (incoming_send, incoming_recv) = tokio::sync::mpsc::channel(4096);
        let rt_handle = tokio::runtime::Handle::current();
        Self {
            session,
            player: ArcSwap::new(Arc::new(None)),
            address,
            version: AtomicCell::new(BedrockMinecraftVersion::Unknown),
            client_data: ArcSwap::new(Arc::new(None)),
            be_clients,
            network_writer: Arc::new(RwLock::new(BedrockBatchEncoder::new())),
            network_reader: Mutex::new(BedrockBatchDecoder::new()),
            tasks: TaskTracker::new(),
            rt_handle,
            outgoing_packet_queue_send: send,
            outgoing_packet_queue_recv: Mutex::new(Some(recv)),
            outgoing_packet_priority_send: priority_send,
            outgoing_packet_priority_recv: Mutex::new(Some(priority_recv)),
            next_form_id: AtomicU32::new(0),
            inventory_opened: AtomicBool::new(false),
            client_cache_supported: AtomicBool::new(false),
            blob_cache: std::sync::Mutex::new(HashMap::new()),
            close_token: CancellationToken::new(),
            last_seen: Arc::new(AtomicCell::new(std::time::Instant::now())),
            incoming_game_packet_send: incoming_send,
            incoming_game_packet_recv: Mutex::new(Some(incoming_recv)),
            packet_limiter,
        }
    }

    pub async fn get_packet(&self) -> Option<RawPacket> {
        let mut guard = self.incoming_game_packet_recv.lock().await;
        let recv = guard.as_mut()?;
        tokio::select! {
            () = self.await_close_interrupt() => None,
            packet = recv.recv() => packet,
        }
    }

    pub fn start_outgoing_packet_task(self: &Arc<Self>) {
        let client = self.clone();
        self.spawn_task(async move {
            let Some(mut packet_receiver) = client.outgoing_packet_queue_recv.lock().await.take()
            else {
                return;
            };
            let Some(mut priority_packet_receiver) =
                client.outgoing_packet_priority_recv.lock().await.take()
            else {
                return;
            };
            let mut interval = tokio::time::interval(std::time::Duration::from_millis(100));

            while !client.close_token.is_cancelled() {
                let packet = tokio::select! {
                    biased;
                    () = client.close_token.cancelled() => break,
                    res = priority_packet_receiver.recv() => match res {
                        Some(p) => p,
                        None => break,
                    },
                    _ = interval.tick() => {
                        if !client.tick_connection().await {
                            break;
                        }
                        continue;
                    }
                    res = packet_receiver.recv() => match res {
                        Some(p) => p,
                        None => break,
                    },
                };

                let data = packet.data.strip_prefix(&[BEDROCK_GAME_PACKET]);
                let Some(data) = data else {
                    warn!("Refusing to send a non-game packet over NetherNet");
                    continue;
                };
                if let Err(error) = client.session.send(Bytes::copy_from_slice(data)).await {
                    warn!(
                        "Failed to send NetherNet packet to {}: {error}",
                        client.address
                    );
                    client.close().await;
                }

                if let Some(completion) = packet.completion {
                    let _ = completion.send(());
                }
            }
        });
    }

    async fn tick_connection(&self) -> bool {
        if self.last_seen.load().elapsed() > std::time::Duration::from_secs(10) {
            debug!("Bedrock client {} timed out", self.address);
            self.close().await;
            return false;
        }
        true
    }

    pub async fn process_nethernet_packet(self: &Arc<Self>, server: &Arc<Server>, packet: Bytes) {
        self.last_seen.store(std::time::Instant::now());
        let mut batch = Vec::with_capacity(packet.len() + 1);
        batch.push(BEDROCK_GAME_PACKET);
        batch.extend_from_slice(&packet);
        if let Err(error) = self.process_batch(server, batch).await {
            error!(
                "Failed to handle NetherNet payload for {}: {error}",
                self.address
            );
            self.kick(DisconnectReason::BadPacket, error.to_string())
                .await;
        }
    }

    pub fn nethernet_public_key(&self) -> Option<&pumpkin_util::p384::PublicKey> {
        self.session.client_public_key()
    }

    pub async fn set_compression(&self, compression: CompressionInfo) {
        self.network_reader
            .lock()
            .await
            .set_compression(compression.threshold as usize);

        self.network_writer
            .write()
            .await
            .set_compression((compression.threshold as usize, compression.level));
    }

    pub fn try_kick(&self, reason: DisconnectReason, message: String) {
        let packet = CDisconnect::new(reason as i32, message);
        if let Ok(data) = self.serialize_packet(&packet) {
            self.try_enqueue_packet(data);
        }
        if !self.close_token.is_cancelled() {
            self.close_token.cancel();
        }
    }

    pub async fn kick(&self, reason: DisconnectReason, message: String) {
        self.send_packet(&CDisconnect::new(reason as i32, message))
            .await;
        self.close().await;
    }

    pub async fn kick_explicit(
        &self,
        reason: DisconnectReason,
        message: String,
        skip_message: bool,
        filtered_message: String,
        send_packet: bool,
    ) {
        if send_packet {
            self.send_packet(&CDisconnect {
                reason: pumpkin_protocol::codec::var_int::VarInt(reason as i32),
                skip_message,
                message,
                filtered_message,
            })
            .await;
        }
        self.close().await;
    }

    pub async fn send_chunks(&self, chunks: &[SyncChunk]) {
        let player = self.player.load_full();
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

        let mut valid_chunks = Vec::with_capacity(chunks.len());
        for chunk in chunks {
            let mut event = ChunkSend::new(player.world(), chunk.clone());
            server.plugin_manager.fire(&server, &mut event).await;
            if !event.cancelled {
                valid_chunks.push(chunk.clone());
            }
        }

        if valid_chunks.is_empty() {
            return;
        }

        let bedrock_dimension =
            if player.world().dimension == pumpkin_data::dimension::Dimension::THE_NETHER {
                1
            } else if player.world().dimension == pumpkin_data::dimension::Dimension::THE_END {
                2
            } else {
                0
            };

        let cache_enabled = server.advanced_config.networking.bedrock.chunk_caching
            && self.client_cache_supported.load(Ordering::Relaxed);

        let world = player.world();
        let (tx, rx) = tokio::sync::oneshot::channel();
        rayon::spawn(move || {
            let mut encoded_payloads = Vec::with_capacity(valid_chunks.len());
            let mut new_blobs = Vec::new();
            for chunk in valid_chunks {
                let block_actors = world.bedrock_chunk_block_actors(&chunk);
                match CLevelChunk::encode_chunk(
                    &chunk,
                    bedrock_dimension,
                    cache_enabled,
                    &block_actors,
                ) {
                    Ok((payload, blobs)) => {
                        encoded_payloads.push(payload);
                        new_blobs.extend(blobs);
                    }
                    Err(e) => error!("Failed to serialize Bedrock chunk: {:?}", e),
                }
            }
            let _ = tx.send((encoded_payloads, new_blobs));
        });

        let Ok((encoded_payloads, new_blobs)) = rx.await else {
            return;
        };

        if !new_blobs.is_empty() {
            let mut cache = self
                .blob_cache
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            for (hash, payload) in new_blobs {
                cache.insert(hash, payload);
            }
        }

        let mut packets_to_enqueue = Vec::with_capacity(encoded_payloads.len());
        {
            let encoder = self.network_writer.read().await;
            for payload in encoded_payloads {
                let mut packet_buf = Vec::new();
                match encoder.write_game_packet(
                    CLevelChunk::PACKET_ID as u16,
                    SubClient::Main,
                    SubClient::Main,
                    &payload,
                    &mut packet_buf,
                ) {
                    Ok(()) => packets_to_enqueue.push(packet_buf),
                    Err(err) => error!("Failed to write game packet wrapper: {err}"),
                }
            }
        }
        for packet_buf in packets_to_enqueue {
            self.enqueue_packet_data(packet_buf.into()).await;
        }
    }

    pub fn set_player(&self, player: Arc<Player>) {
        self.player.store(Arc::new(Some(player)));
    }

    pub async fn enqueue_packet(&self, packet_data: Bytes) {
        self.enqueue_packet_data(packet_data).await;
    }

    pub fn try_enqueue_packet(&self, packet_data: Bytes) {
        self.try_enqueue_packet_data(packet_data);
    }

    /// Queues a clientbound packet to be sent to the connected client. Queued chunks are sent
    /// in-order to the client
    ///
    /// # Arguments
    ///
    /// * `packet_data`: A `Bytes` payload representing the encoded packet.
    pub async fn enqueue_packet_data(&self, packet_data: Bytes) {
        if let Err(err) = self
            .outgoing_packet_queue_send
            .send(OutgoingPacket::normal(packet_data))
            .await
        {
            // This is expected to fail if we are closed
            if !self.is_closed() {
                error!("Failed to add packet to the outgoing packet queue for client: {err}");
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
                        "Failed to add packet to the outgoing packet queue for client: channel full"
                    );
                }
                tokio::sync::mpsc::error::TrySendError::Closed(_) => {
                    if !self.is_closed() {
                        error!(
                            "Failed to add packet to the outgoing packet queue for client: channel closed"
                        );
                    }
                }
            }
        }
    }

    pub fn write_raw_packet<P: BClientPacket>(
        packet: &P,
        mut writer: impl Write,
    ) -> Result<(), Error> {
        writer.write_all(&[P::PACKET_ID as u8])?;
        packet.write_packet(writer)
    }

    pub async fn write_game_packet<P: BClientPacket>(
        &self,
        packet: &P,
        write: impl Write,
    ) -> Result<(), Error> {
        let mut packet_payload = Vec::new();
        packet.write_packet(&mut packet_payload)?;

        let encoder = self.network_writer.read().await;
        encoder.write_game_packet(
            P::PACKET_ID as u16,
            SubClient::Main,
            SubClient::Main,
            &packet_payload,
            write,
        )
    }

    pub fn serialize_packet<P: BClientPacket>(&self, packet: &P) -> Result<Bytes, Error> {
        self.network_writer
            .try_read()
            .map_err(|_| Error::other("Bedrock packet encoder is busy"))?
            .serialize_packet(packet)
    }

    pub async fn send_packet<P: BClientPacket>(&self, packet: &P) {
        let mut data = Vec::new();
        match self.write_game_packet(packet, &mut data).await {
            Ok(()) => self.send_game_packet(data.into()).await,
            Err(err) => error!("Failed to serialize Bedrock packet: {err}"),
        }
    }

    pub async fn enqueue_client_packet<P: BClientPacket>(&self, packet: &P) {
        let mut data = Vec::new();
        match self.write_game_packet(packet, &mut data).await {
            Ok(()) => self.enqueue_packet(data.into()).await,
            Err(err) => error!("Failed to serialize Bedrock packet: {err}"),
        }
    }

    pub fn try_enqueue_client_packet<P: BClientPacket>(&self, packet: &P) {
        match self.serialize_packet(packet) {
            Ok(data) => self.try_enqueue_packet(data),
            Err(err) => error!("Failed to serialize Bedrock packet: {err}"),
        }
    }

    pub async fn send_game_packet(&self, packet_data: Bytes) {
        let (tx, rx) = oneshot::channel();
        if let Err(err) = self
            .outgoing_packet_priority_send
            .send(OutgoingPacket::priority(packet_data, tx))
            .await
        {
            if !self.is_closed() {
                error!("Failed to add priority packet to the outgoing packet queue: {err}");
            }
        } else {
            let _ = rx.await;
        }
    }

    pub async fn close(&self) {
        if self.close_token.is_cancelled() {
            return;
        }
        self.close_token.cancel();
        self.session.close().await;
        self.be_clients.lock().await.remove(&self.address);
    }

    pub async fn await_tasks(&self) {
        self.tasks.close();
        self.tasks.wait().await;
    }

    pub fn is_closed(&self) -> bool {
        self.close_token.is_cancelled() || self.session.is_closed()
    }

    pub fn enqueue_spawn_packet(&self, entity: &dyn crate::entity::EntityBase) {
        entity.send_bedrock_spawn_packet(self);
    }

    async fn process_batch(
        self: &Arc<Self>,
        server: &Arc<Server>,
        payload: Vec<u8>,
    ) -> Result<(), Error> {
        let decompressed_payload = self
            .get_packet_payload(payload)
            .await
            .ok_or_else(|| Error::other("Failed to decompress game packet batch"))?;
        let mut cursor = Cursor::new(decompressed_payload);

        while (cursor.position() as usize) < cursor.get_ref().len() {
            let game_packet = self
                .network_reader
                .lock()
                .await
                .get_game_packet(&mut cursor)
                .map_err(|e| Error::other(e.to_string()))?;

            if !self.packet_limiter.check_packet() {
                warn!(
                    "Bedrock client {} exceeded packet rate limit (rate: {}/s)",
                    self.address,
                    self.packet_limiter.max_rate()
                );
                self.kick(
                    DisconnectReason::Kicked,
                    server
                        .advanced_config
                        .networking
                        .bedrock
                        .packet_limiter
                        .kick_message
                        .clone(),
                )
                .await;
                return Err(Error::other("Packet rate limit exceeded"));
            }

            self.handle_game_packet(game_packet).await?;
        }

        Ok(())
    }

    async fn handle_game_packet(&self, packet: RawPacket) -> Result<(), Error> {
        if let Err(err) = self.incoming_game_packet_send.send(packet).await {
            debug!("Failed to send game packet to session task: {err}");
        }
        Ok(())
    }

    pub async fn handle_login_sequence(
        self: &Arc<Self>,
        server: &Arc<Server>,
    ) -> PacketHandlerResult {
        while let Some(packet) = self.get_packet().await {
            let payload = &mut Cursor::new(&packet.payload);
            match packet.id {
                SRequestNetworkSettings::PACKET_ID => {
                    let packet = match SRequestNetworkSettings::read(payload) {
                        Ok(p) => p,
                        Err(err) => {
                            error!("Failed to read SRequestNetworkSettings: {err}");
                            continue;
                        }
                    };
                    if !self.handle_request_network_settings(packet, server).await {
                        return PacketHandlerResult::Stop;
                    }
                }
                SLogin::PACKET_ID => {
                    let packet = match SLogin::read(payload) {
                        Ok(p) => p,
                        Err(err) => {
                            error!("Failed to read SLogin: {err}");
                            self.kick(DisconnectReason::BadPacket, err.to_string())
                                .await;
                            return PacketHandlerResult::Stop;
                        }
                    };
                    match self.handle_login(packet, server).await {
                        Ok(result) => return result,
                        Err(err) => {
                            self.kick(DisconnectReason::Unknown, err.to_string()).await;
                            return PacketHandlerResult::Stop;
                        }
                    }
                }
                _ => {
                    debug!(
                        "Received unexpected game packet {} during login sequence",
                        packet.id
                    );
                }
            }
        }
        PacketHandlerResult::Stop
    }

    pub async fn progress_player_packets(self: &Arc<Self>, player: &Arc<Player>) {
        while let Some(packet) = self.get_packet().await {
            player.inbound_packets.push(packet);
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn handle_play_packet(
        self: &Arc<Self>,
        player: &Arc<Player>,
        server: &Arc<Server>,
        packet: &RawPacket,
    ) -> Result<(), Error> {
        let payload = &packet.payload[..];
        let reader = &mut &payload[..];
        match packet.id {
            SClientCacheStatus::PACKET_ID => {
                let packet = SClientCacheStatus::read(reader)?;
                self.client_cache_supported
                    .store(packet.is_cache_supported, Ordering::Relaxed);
            }
            SClientCacheBlobStatus::PACKET_ID => {
                let packet = SClientCacheBlobStatus::read(reader)?;
                self.handle_client_cache_blob_status(packet);
            }
            SResourcePackClientResponse::PACKET_ID => {
                let packet = SResourcePackClientResponse::read(reader)?;
                let client = self.clone();
                let server_c = server.clone();
                server.spawn_task(async move {
                    client.handle_resource_pack_response(packet, &server_c).await;
                });
            }
            SPlayerAuthInput::PACKET_ID => {
                let packet = SPlayerAuthInput::read(reader)?;
                self.handle_player_auth_input(player, packet, server);
            }
            SRequestChunkRadius::PACKET_ID => {
                let packet = SRequestChunkRadius::read(reader)?;
                self.handle_request_chunk_radius(player, &packet);
            }
            SInventoryTransaction::PACKET_ID => {
                let packet = SInventoryTransaction::read(reader)?;
                self.handle_inventory_action(player, packet);
            }
            pumpkin_protocol::bedrock::server::item_stack_request::SItemStackRequest::PACKET_ID => {
                let packet = pumpkin_protocol::bedrock::server::item_stack_request::SItemStackRequest::read(reader)?;
                self.handle_item_stack_request(player, packet);
            }
            SInteract::PACKET_ID => {
                let packet = SInteract::read(reader)?;
                self.handle_interaction(&packet);
            }
            SContainerClose::PACKET_ID => {
                let packet = SContainerClose::read(reader)?;
                self.handle_container_close(player, &packet);
            }
            SText::PACKET_ID => {
                let text = SText::read(reader)?;
                let client = self.clone();
                let player_c = player.clone();
                let server_c = server.clone();
                player.spawn_task(async move {
                    client.handle_chat_message(&server_c, &player_c, text).await;
                });
            }
            SCommandRequest::PACKET_ID => {
                let req = SCommandRequest::read(reader)?;
                let client = self.clone();
                let player_c = player.clone();
                let server_c = server.clone();
                player.spawn_task(async move {
                    client.handle_chat_command(&player_c, &server_c, req).await;
                });
            }
            SSetLocalPlayerAsInitialized::PACKET_ID => {
                self.handle_set_local_player_as_initialized(
                    player,
                    &SSetLocalPlayerAsInitialized::read(reader)?,
                );
            }
            SSetPlayerInventoryOptions::PACKET_ID => {
                let _ = SSetPlayerInventoryOptions::read(reader)?;
                // Ignore for now
            }
            SPlayerAction::PACKET_ID => {
                let packet = SPlayerAction::read(reader)?;
                self.handle_player_action(player, server, packet);
            }
            SRespawn::PACKET_ID => {
                let packet = SRespawn::read(reader)?;
                self.handle_respawn(player, &packet);
            }
            SAnimate::PACKET_ID => {
                self.handle_animate(player, &SAnimate::read(reader)?);
            }
            SActorEvent::PACKET_ID => {
                self.handle_actor_event(player, &SActorEvent::read(reader)?);
            }
            SEmote::PACKET_ID => {
                self.handle_emote(player, SEmote::read_slice(reader)?);
            }
            SEmoteList::PACKET_ID => {
                self.handle_emote_list(player, &SEmoteList::read(reader)?);
            }
            pumpkin_protocol::bedrock::server::modal_form_response::SModalFormResponse::PACKET_ID => {
                let form_resp = pumpkin_protocol::bedrock::server::modal_form_response::SModalFormResponse::read(
                    reader,
                )?;
                self.handle_modal_form_response(player, server, form_resp);
            }
            SLoadingScreen::PACKET_ID => {
                // Ignore for now
            }
            SBlockPickRequest::PACKET_ID => {
                let packet = SBlockPickRequest::read(reader)?;
                self.handle_block_pick_request(player, &packet);
            }
            SRequestAbility::PACKET_ID => {
                self.handle_request_ability(player, &SRequestAbility::read(reader)?);
            }
            SMobEquipment::PACKET_ID => {
                let packet = SMobEquipment::read(reader)?;
                self.handle_mob_equipment(server, player, &packet);
            }
            SPacketViolationWarning::PACKET_ID => {
                let warning = SPacketViolationWarning::read(reader)?;
                warn!(
                    violation_type = warning.violation_type.0,
                    violation_severity = warning.violation_severity.0,
                    violation_packet_id = warning.violation_packet_id.0,
                    violation_context = %warning.violation_context,
                    "Bedrock client rejected a server packet"
                );
            }
            _ => {
                warn!("Bedrock: Received Unknown Game packet: {}", packet.id);
            }
        }
        Ok(())
    }

    pub fn handle_client_cache_blob_status(&self, packet: SClientCacheBlobStatus) {
        if packet.miss_hashes.is_empty() {
            return;
        }
        let missing_blobs = {
            let cache = self
                .blob_cache
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let mut missing_blobs = Vec::with_capacity(packet.miss_hashes.len());
            for hash in packet.miss_hashes {
                if let Some(payload) = cache.get(&hash) {
                    missing_blobs.push(MissingBlobData {
                        blob_id: hash,
                        blob_data: payload.clone(),
                    });
                } else {
                    warn!("Client requested missing blob {hash:#x} not found in server cache");
                }
            }
            missing_blobs
        };
        if !missing_blobs.is_empty() {
            self.try_enqueue_client_packet(&CClientCacheMissResponse { missing_blobs });
        }
    }

    pub async fn await_close_interrupt(&self) {
        self.close_token.cancelled().await;
    }

    pub async fn get_packet_payload(&self, packet: Vec<u8>) -> Option<Vec<u8>> {
        let mut network_reader = self.network_reader.lock().await;
        tokio::select! {
            () = self.await_close_interrupt() => {
                debug!("Canceling player packet processing");
                None
            },
            packet_result = network_reader.get_packet_payload(packet) => {
                match packet_result {
                    Ok(packet) => Some(packet),
                    Err(err) => {
                        if !matches!(err, PacketDecodeError::ConnectionClosed) {
                            debug!("Failed to decode packet from client: {err}");
                            let text = format!("Error while reading incoming packet {err}");
                            self.kick(DisconnectReason::BadPacket, text).await;
                        }
                        None
                    }
                }
            }
        }
    }

    pub fn spawn_task<F>(&self, task: F) -> Option<JoinHandle<F::Output>>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        if self.close_token.is_cancelled() {
            None
        } else {
            let _guard = self.rt_handle.enter();
            Some(self.tasks.spawn(task))
        }
    }
}
