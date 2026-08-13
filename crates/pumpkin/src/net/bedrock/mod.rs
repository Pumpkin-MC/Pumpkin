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
            client_cache_miss_response::{CClientCacheMissResponse, CacheBlob},
            disconnect_player::CDisconnectPlayer,
            level_chunk::CLevelChunk,
        },
        packet_decoder::BedrockBatchDecoder,
        packet_encoder::BedrockBatchEncoder,
        server::{
            animate::SAnimate, block_pick_request::SBlockPickRequest,
            client_cache_blob_status::SClientCacheBlobStatus,
            client_cache_status::SClientCacheStatus, command_request::SCommandRequest,
            container_close::SContainerClose, emote::SEmote, emote_list::SEmoteList,
            interaction::SInteraction, inventory_transaction::SInventoryTransaction,
            loading_screen::SLoadingScreen, login::SLogin, mob_equipment::SMobEquipment,
            packet_violation_warning::SPacketViolationWarning, player_action::SPlayerAction,
            player_auth_input::SPlayerAuthInput, request_ability::SRequestAbility,
            request_chunk_radius::SRequestChunkRadius,
            request_network_settings::SRequestNetworkSettings,
            resource_pack_response::SResourcePackResponse, respawn::SRespawn,
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
    net::{DisconnectReason, PacketHandlerResult},
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
    pub blob_cache: Mutex<HashMap<u64, Vec<u8>>>,
    /// An notifier that is triggered when this client is closed.
    close_token: CancellationToken,
    last_seen: Arc<AtomicCell<std::time::Instant>>,
    incoming_game_packet_send: Sender<RawPacket>,
    incoming_game_packet_recv: Mutex<Option<Receiver<RawPacket>>>,
}

impl BedrockClient {
    #[must_use]
    pub fn new(
        session: Arc<NetherNetSession>,
        address: SocketAddr,
        be_clients: Arc<Mutex<HashMap<SocketAddr, Arc<Self>>>>,
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
            blob_cache: Mutex::new(HashMap::new()),
            close_token: CancellationToken::new(),
            last_seen: Arc::new(AtomicCell::new(std::time::Instant::now())),
            incoming_game_packet_send: incoming_send,
            incoming_game_packet_recv: Mutex::new(Some(incoming_recv)),
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

    pub async fn kick(&self, reason: DisconnectReason, message: String) {
        self.send_game_packet(&CDisconnectPlayer::new(reason as i32, message))
            .await;
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

        let mut serialize_tasks = Vec::with_capacity(valid_chunks.len());
        for chunk in valid_chunks {
            let block_actors = player.world().bedrock_chunk_block_actors(&chunk);
            serialize_tasks.push(tokio::task::spawn_blocking(move || {
                CLevelChunk::encode_chunk(&chunk, bedrock_dimension, cache_enabled, &block_actors)
            }));
        }

        let mut encoded_payloads = Vec::with_capacity(serialize_tasks.len());
        let mut new_blobs = Vec::new();
        for task in serialize_tasks {
            match task.await {
                Ok(Ok((payload, blobs))) => {
                    encoded_payloads.push(payload);
                    new_blobs.extend(blobs);
                }
                Ok(Err(e)) => error!("Failed to serialize Bedrock chunk: {:?}", e),
                Err(e) => error!("Join error in Bedrock chunk serialization: {:?}", e),
            }
        }

        if !new_blobs.is_empty() {
            let mut cache = self.blob_cache.lock().await;
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

    pub async fn enqueue_packet<P: BClientPacket>(&self, packet: &P) {
        let mut packet_buf = Vec::new();
        match self.write_game_packet(packet, &mut packet_buf).await {
            Ok(()) => {
                let payload = Bytes::from(packet_buf);
                let player = self.player.load_full();
                if let Some(player) = player.as_ref() {
                    let event = player
                        .fire_packet_sent_event_no_obj(P::PACKET_ID, payload)
                        .await;
                    if !event.cancelled {
                        self.enqueue_packet_data(event.payload).await;
                    }
                } else {
                    self.enqueue_packet_data(payload).await;
                }
            }
            Err(err) => error!("Failed to write game packet: {err}"),
        }
    }

    pub async fn enqueue_packet_internal<P: BClientPacket>(&self, packet: &P) {
        let mut packet_buf = Vec::new();
        match self.write_game_packet(packet, &mut packet_buf).await {
            Ok(()) => self.enqueue_packet_data(packet_buf.into()).await,
            Err(err) => error!("Failed to write game packet: {err}"),
        }
    }

    pub fn try_enqueue_packet<P: BClientPacket>(&self, packet: &P) {
        let mut packet_buf = Vec::new();
        let mut packet_payload = Vec::new();
        if let Err(err) = packet.write_packet(&mut packet_payload) {
            error!("Failed to write packet for try_enqueue_packet: {err}");
            return;
        }

        {
            let Ok(network_writer) = self.network_writer.try_read() else {
                debug!("Failed to lock network writer for try_enqueue_packet");
                return;
            };

            if let Err(err) = network_writer.write_game_packet(
                P::PACKET_ID as u16,
                SubClient::Main,
                SubClient::Main,
                &packet_payload,
                &mut packet_buf,
            ) {
                error!("Failed to write game packet for try_enqueue_packet: {err}");
                return;
            }
        }

        self.try_enqueue_packet_data(packet_buf.into());
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

    pub async fn send_game_packet<P: BClientPacket>(&self, packet: &P) {
        let mut packet_buf = Vec::new();
        match self.write_game_packet(packet, &mut packet_buf).await {
            Ok(()) => {
                let payload = Bytes::from(packet_buf);
                let player = self.player.load_full();
                let payload = if let Some(player) = player.as_ref() {
                    let event = player
                        .fire_packet_sent_event_no_obj(P::PACKET_ID, payload)
                        .await;
                    if event.cancelled {
                        return;
                    }
                    event.payload
                } else {
                    payload
                };
                let (tx, rx) = oneshot::channel();
                if let Err(err) = self
                    .outgoing_packet_priority_send
                    .send(OutgoingPacket::priority(payload, tx))
                    .await
                {
                    if !self.is_closed() {
                        error!("Failed to add priority packet to the outgoing packet queue: {err}");
                    }
                } else {
                    let _ = rx.await;
                }
            }
            Err(err) => error!("Failed to write game packet: {err}"),
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

    pub fn enqueue_spawn_packet(self: &Arc<Self>, entity: Arc<dyn crate::entity::EntityBase>) {
        let client = self.clone();
        self.spawn_task(async move {
            entity.send_bedrock_spawn_packet(&client).await;
        });
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
            self.handle_game_packet(server, game_packet).await?;
        }

        Ok(())
    }

    async fn handle_game_packet(
        &self,
        _server: &Arc<Server>,
        packet: RawPacket,
    ) -> Result<(), Error> {
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
                    self.handle_request_network_settings(packet, server).await;
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

    pub async fn progress_player_packets(
        self: &Arc<Self>,
        player: &Arc<Player>,
        server: &Arc<Server>,
    ) {
        while let Some(packet) = self.get_packet().await {
            let mut event = crate::plugin::server::packet::PacketReceivedEvent::new(
                player.clone(),
                packet.id,
                packet.payload.clone(),
            );
            server.plugin_manager.fire(server, &mut event).await;
            if event.cancelled {
                continue;
            }

            if let Err(err) = self.handle_play_packet(player, server, packet).await {
                error!("Failed to handle Bedrock play packet: {err}");
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    pub async fn handle_play_packet(
        &self,
        player: &Arc<Player>,
        server: &Arc<Server>,
        packet: RawPacket,
    ) -> Result<(), Error> {
        let payload = &packet.payload[..];
        let reader = &mut &payload[..];
        match packet.id {
            SClientCacheStatus::PACKET_ID => {
                let packet = SClientCacheStatus::read(reader)?;
                self.client_cache_supported
                    .store(packet.cache_supported, Ordering::Relaxed);
            }
            SClientCacheBlobStatus::PACKET_ID => {
                self.handle_client_cache_blob_status(SClientCacheBlobStatus::read(reader)?)
                    .await;
            }
            SResourcePackResponse::PACKET_ID => {
                self.handle_resource_pack_response(SResourcePackResponse::read(reader)?, server)
                    .await;
            }
            SPlayerAuthInput::PACKET_ID => {
                self.handle_player_auth_input(player, SPlayerAuthInput::read(reader)?, server)
                    .await;
            }
            SRequestChunkRadius::PACKET_ID => {
                self.handle_request_chunk_radius(player, SRequestChunkRadius::read(reader)?)
                    .await;
            }
            SInventoryTransaction::PACKET_ID => {
                self.handle_inventory_action(player, SInventoryTransaction::read(reader)?).await;
            }
            pumpkin_protocol::bedrock::server::item_stack_request::SItemStackRequest::PACKET_ID => {
                self.handle_item_stack_request(player, pumpkin_protocol::bedrock::server::item_stack_request::SItemStackRequest::read(reader)?).await;
            }
            SInteraction::PACKET_ID => {
                self.handle_interaction(player, SInteraction::read(reader)?)
                    .await;
            }
            SContainerClose::PACKET_ID => {
                self.handle_container_close(player, SContainerClose::read(reader)?)
                    .await;
            }
            SText::PACKET_ID => {
                self.handle_chat_message(server, player, SText::read_slice(reader)?)
                    .await;
            }
            SCommandRequest::PACKET_ID => {
                self.handle_chat_command(player, server, SCommandRequest::read_slice(reader)?)
                    .await;
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
                self.handle_player_action(player, server, SPlayerAction::read(reader)?)
                    .await;
            }
            SRespawn::PACKET_ID => {
                self.handle_respawn(player, SRespawn::read(reader)?).await;
            }
            SAnimate::PACKET_ID => {
                self.handle_animate(player, server, &SAnimate::read(reader)?).await;
            }
            SEmote::PACKET_ID => {
                self.handle_emote(player, server, SEmote::read_slice(reader)?).await;
            }
            SEmoteList::PACKET_ID => {
                self.handle_emote_list(player, server, &SEmoteList::read(reader)?);
            }
            pumpkin_protocol::bedrock::server::modal_form_response::SModalFormResponse::PACKET_ID => {
                self.handle_modal_form_response(
                    player,
                    server,
                    pumpkin_protocol::bedrock::server::modal_form_response::SModalFormResponse::read_slice(
                        reader,
                    )?,
                )
                .await;
            }
            SLoadingScreen::PACKET_ID => {
                // Ignore for now
            }
            SBlockPickRequest::PACKET_ID => {
                self.handle_block_pick_request(player, SBlockPickRequest::read(reader)?)
                    .await;
            }
            SRequestAbility::PACKET_ID => {
                self.handle_request_ability(player, SRequestAbility::read(reader)?)
                    .await;
            }
            SMobEquipment::PACKET_ID => {
                self.handle_mob_equipment(server, player, SMobEquipment::read(reader)?)
                    .await;
            }
            SPacketViolationWarning::PACKET_ID => {
                let warning = SPacketViolationWarning::read(reader)?;
                warn!(
                    violation_type = warning.violation_type.0,
                    severity = warning.severity.0,
                    packet_id = warning.packet_id.0,
                    context = %warning.context,
                    "Bedrock client rejected a server packet"
                );
            }
            _ => {
                warn!("Bedrock: Received Unknown Game packet: {}", packet.id);
            }
        }
        Ok(())
    }

    pub async fn handle_client_cache_blob_status(&self, packet: SClientCacheBlobStatus) {
        if packet.miss_hashes.is_empty() {
            return;
        }
        let cache = self.blob_cache.lock().await;
        let mut missing_blobs = Vec::with_capacity(packet.miss_hashes.len());
        for hash in packet.miss_hashes {
            if let Some(payload) = cache.get(&hash) {
                missing_blobs.push(CacheBlob {
                    hash,
                    payload: payload.clone(),
                });
            } else {
                warn!("Client requested missing blob {hash:#x} not found in server cache");
            }
        }
        if !missing_blobs.is_empty() {
            self.send_game_packet(&CClientCacheMissResponse {
                blobs: &missing_blobs,
            })
            .await;
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
                            warn!("Failed to decode packet from client: {err}");
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
