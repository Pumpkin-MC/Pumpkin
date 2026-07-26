use super::JavaClient;
use crate::entity::player::Player;
use crate::error::PumpkinError;
use crate::net::PacketHandlerResult;
use crate::plugin::player::player_custom_payload::PlayerCustomPayloadEvent;
use crate::server::Server;
use bytes::Bytes;
use pumpkin_data::translation;
use pumpkin_protocol::ConnectionState;
use pumpkin_protocol::RawPacket;
use pumpkin_protocol::ServerPacket;
use pumpkin_protocol::java::client::play::CAcknowledgeBlockChange;
use pumpkin_protocol::java::server::config::SAcknowledgeFinishConfig;
use pumpkin_protocol::java::server::config::SClientInformationConfig;
use pumpkin_protocol::java::server::config::SConfigCookieResponse;
use pumpkin_protocol::java::server::config::SConfigResourcePack;
use pumpkin_protocol::java::server::config::SKnownPacks;
use pumpkin_protocol::java::server::config::SPluginMessage;
use pumpkin_protocol::java::server::handshake::SHandShake;
use pumpkin_protocol::java::server::login::SEncryptionResponse;
use pumpkin_protocol::java::server::login::SLoginAcknowledged;
use pumpkin_protocol::java::server::login::SLoginCookieResponse;
use pumpkin_protocol::java::server::login::SLoginPluginResponse;
use pumpkin_protocol::java::server::login::SLoginStart;
use pumpkin_protocol::java::server::play::SAttack;
use pumpkin_protocol::java::server::play::SBundleItemSelected;
use pumpkin_protocol::java::server::play::SChangeGameMode;
use pumpkin_protocol::java::server::play::SChatCommand;
use pumpkin_protocol::java::server::play::SChatMessage;
use pumpkin_protocol::java::server::play::SChunkBatch;
use pumpkin_protocol::java::server::play::SClickSlot;
use pumpkin_protocol::java::server::play::SClientCommand;
use pumpkin_protocol::java::server::play::SClientInformationPlay;
use pumpkin_protocol::java::server::play::SClientTickEnd;
use pumpkin_protocol::java::server::play::SCloseContainer;
use pumpkin_protocol::java::server::play::SCommandSuggestion;
use pumpkin_protocol::java::server::play::SConfirmTeleport;
use pumpkin_protocol::java::server::play::SContainerButtonClick;
use pumpkin_protocol::java::server::play::SCookieResponse as SPCookieResponse;
use pumpkin_protocol::java::server::play::SCustomPayload;
use pumpkin_protocol::java::server::play::SInteract;
use pumpkin_protocol::java::server::play::SJigsawGenerate;
use pumpkin_protocol::java::server::play::SMoveVehicle;
use pumpkin_protocol::java::server::play::SPaddleBoat;
use pumpkin_protocol::java::server::play::SPickItemFromBlock;
use pumpkin_protocol::java::server::play::SPlaceRecipe;
use pumpkin_protocol::java::server::play::SPlayPingRequest;
use pumpkin_protocol::java::server::play::SPlayerAbilities;
use pumpkin_protocol::java::server::play::SPlayerAction;
use pumpkin_protocol::java::server::play::SPlayerCommand;
use pumpkin_protocol::java::server::play::SPlayerInput;
use pumpkin_protocol::java::server::play::SPlayerLoaded;
use pumpkin_protocol::java::server::play::SPlayerPosition;
use pumpkin_protocol::java::server::play::SPlayerPositionRotation;
use pumpkin_protocol::java::server::play::SPlayerRotation;
use pumpkin_protocol::java::server::play::SPlayerSession;
use pumpkin_protocol::java::server::play::SRecipeBookChangeSettings;
use pumpkin_protocol::java::server::play::SRecipeBookSeenRecipe;
use pumpkin_protocol::java::server::play::SRenameItem;
use pumpkin_protocol::java::server::play::SSeenAdvancement;
use pumpkin_protocol::java::server::play::SSelectTrade;
use pumpkin_protocol::java::server::play::SSetCommandBlock;
use pumpkin_protocol::java::server::play::SSetCreativeSlot;
use pumpkin_protocol::java::server::play::SSetHeldItem;
use pumpkin_protocol::java::server::play::SSetJigsawBlock;
use pumpkin_protocol::java::server::play::SSetPlayerGround;
use pumpkin_protocol::java::server::play::SSetTestBlock;
use pumpkin_protocol::java::server::play::SSwingArm;
use pumpkin_protocol::java::server::play::STeleportToEntity;
use pumpkin_protocol::java::server::play::STestInstanceBlockAction;
use pumpkin_protocol::java::server::play::SUpdateSign;
use pumpkin_protocol::java::server::play::SUseItem;
use pumpkin_protocol::java::server::play::SUseItemOn;
use pumpkin_protocol::java::server::status::SStatusPingRequest;
use pumpkin_protocol::java::server::status::SStatusRequest;
use pumpkin_protocol::packet::MultiVersionJavaPacket;
use pumpkin_protocol::ser::ReadingError;
use pumpkin_util::text::TextComponent;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;
use std::time::Instant;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use tracing::debug;
use tracing::error;
use tracing::warn;

impl JavaClient {
    /// Processes all packets received from the connected client in a loop.
    ///
    /// This function continuously dequeues packets from the client's packet queue and processes them.
    /// Processing involves calling the `handle_packet` function with the server instance and the packet itself.
    ///
    /// The loop exits when:
    ///
    /// - The connection is closed (checked before processing each packet).
    /// - An error occurs while processing a packet (client is kicked with an error message).
    ///
    /// # Arguments
    ///
    /// * `server`: A reference to the `Server` instance.
    pub async fn handle_login_sequence(&self, server: &Arc<Server>) -> PacketHandlerResult {
        while let Some(packet) = self.get_packet().await {
            match self.handle_packet(server, &packet).await {
                Ok(result) => {
                    if let Some(result) = result {
                        return result;
                    }
                }
                Err(error) => {
                    let text = format!("Error while reading incoming packet {error}");
                    debug!(
                        "Failed to read incoming packet with id {}: {}",
                        packet.id, error
                    );
                    self.kick(TextComponent::text(text)).await;
                }
            }
        }
        PacketHandlerResult::Stop
    }

    pub async fn progress_player_packets(&self, player: &Arc<Player>, server: &Arc<Server>) {
        let mut keep_alive_interval = tokio::time::interval(std::time::Duration::from_secs(15));
        let mut block_ack_interval = tokio::time::interval(Duration::from_millis(50));

        // Skip the immediate first tick for both timers.
        keep_alive_interval.tick().await;
        block_ack_interval.tick().await;

        loop {
            tokio::select! {
                // KEEP-ALIVE TIMER
                _ = keep_alive_interval.tick() => {
                    // If the client never responded to the LAST keep-alive, they timed out.
                    if self.wait_for_keep_alive.load(Ordering::Relaxed) {
                        self.kick(TextComponent::translate(translation::java::DISCONNECT_TIMEOUT, [])).await;
                        break;
                    }

                    // Generate a unique ID (current timestamp in ms)
                    let keep_alive_id = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as i64;

                    self.keep_alive_id.store(keep_alive_id);
                    self.wait_for_keep_alive.store(true, Ordering::Relaxed);
                    self.last_keep_alive_time.store(Instant::now());
                    let packet = pumpkin_protocol::java::client::play::CKeepAlive::new(keep_alive_id);
                    self.enqueue_packet(&packet).await;
                }

                // Vanilla sends ClientboundBlockChangedAckPacket once per server tick.
                // Delaying this until the keep-alive interval leaves client-side block
                // prediction active for up to 15 seconds, causing ghost blocks and stale
                // redstone rendering even after the authoritative update was queued.
                _ = block_ack_interval.tick() => {
                    let seq = self.packet_sequence.swap(-1, Ordering::Relaxed);
                    if seq != -1 {
                        self
                            .enqueue_packet(&CAcknowledgeBlockChange::new(seq.into()))
                            .await;
                    }
                }

                // INCOMING PACKETS
                packet_opt = self.get_packet() => {
                    let Some(packet) = packet_opt else {
                        break;
                    };

                    match self.handle_play_packet(player, server, &packet).await {
                        Ok(()) => {}
                        Err(e) => {
                            if e.is_kick() {
                                if let Some(kick_reason) = e.client_kick_reason() {
                                    self.kick(TextComponent::text(kick_reason)).await;
                                } else {
                                    self.kick(TextComponent::text(format!(
                                        "Error while handling incoming packet {e}"
                                    )))
                                    .await;
                                }
                            }
                            error!(
                                "Failed to handle play packet id {} (payload {} bytes): {}",
                                packet.id,
                                packet.payload.len(),
                                e
                            );
                        }
                    }
                }
            }
        }
    }
    /// Handles an incoming packet, routing it to the appropriate handler based on the current connection state.
    ///
    /// This function takes a `RawPacket` and routes it to the corresponding handler based on the current connection state.
    /// It supports the following connection states:
    ///
    /// - **Handshake:** Handles handshake packets.
    /// - **Status:** Handles status request and ping packets.
    /// - **Login/Transfer:** Handles login and transfer packets.
    /// - **Config:** Handles configuration packets.
    ///
    /// For the `Play` state, an error is logged as it indicates an invalid state for packet processing.
    ///
    /// # Arguments
    ///
    /// * `server`: A reference to the `Server` instance.
    /// * `packet`: A mutable reference to the `RawPacket` to be processed.
    ///
    /// # Returns
    ///
    /// A `Result` indicating whether the packet was read and handled successfully.
    ///
    /// # Errors
    ///
    /// Returns a `DeserializerError` if an error occurs during packet deserialization.
    pub async fn handle_packet(
        &self,
        server: &Arc<Server>,
        packet: &RawPacket,
    ) -> Result<Option<PacketHandlerResult>, ReadingError> {
        match self.connection_state.load() {
            ConnectionState::HandShake => self.handle_handshake_packet(packet).await,
            ConnectionState::Status => self.handle_status_packet(server, packet).await,
            // TODO: Check config if transfer is enabled
            ConnectionState::Login | ConnectionState::Transfer => {
                self.handle_login_packet(server, packet).await
            }
            ConnectionState::Config => self.handle_config_packet(server, packet).await,
            ConnectionState::Play => Ok(None),
        }
    }

    async fn handle_handshake_packet(
        &self,
        packet: &RawPacket,
    ) -> Result<Option<PacketHandlerResult>, ReadingError> {
        debug!("Handling handshake group");
        let mut payload = &packet.payload[..];
        match packet.id {
            0 => {
                self.handle_handshake(SHandShake::read(&mut payload, &self.version.load())?)
                    .await;
                Ok(None)
            }
            _ => Err(ReadingError::Message(format!(
                "Failed to handle packet id {} in Handshake State",
                packet.id
            ))),
        }
    }

    async fn handle_status_packet(
        &self,
        server: &Server,
        packet: &RawPacket,
    ) -> Result<Option<PacketHandlerResult>, ReadingError> {
        debug!("Handling status group");
        let mut payload = &packet.payload[..];
        let version = self.version.load();

        match packet.id {
            id if id == SStatusRequest::to_id(version) => {
                self.handle_status_request(server).await;
                Ok(None)
            }
            id if id == SStatusPingRequest::to_id(version) => {
                self.handle_ping_request(SStatusPingRequest::read(&mut payload, &version)?)
                    .await;
                Ok(None)
            }
            _ => Err(ReadingError::Message(format!(
                "Failed to handle java client packet id {} in Status State",
                packet.id
            ))),
        }
    }
    async fn handle_login_packet(
        &self,
        server: &Server,
        packet: &RawPacket,
    ) -> Result<Option<PacketHandlerResult>, ReadingError> {
        debug!("Handling login group for id");
        let mut payload = &packet.payload[..];
        let version = self.version.load();
        match packet.id {
            id if id == SLoginStart::to_id(version) => {
                self.handle_login_start(server, SLoginStart::read(&mut payload, &version)?)
                    .await;
            }
            id if id == SEncryptionResponse::to_id(version) => {
                self.handle_encryption_response(
                    server,
                    SEncryptionResponse::read(&mut payload, &version)?,
                )
                .await;
            }
            id if id == SLoginPluginResponse::to_id(version) => {
                self.handle_plugin_response(
                    server,
                    SLoginPluginResponse::read(&mut payload, &version)?,
                )
                .await;
            }
            id if id == SLoginAcknowledged::to_id(version) => {
                self.handle_login_acknowledged(server).await;
            }
            id if id == SLoginCookieResponse::to_id(version) => {
                self.handle_login_cookie_response(&SLoginCookieResponse::read(
                    &mut payload,
                    &version,
                )?);
            }
            _ => {
                error!(
                    "Failed to handle java client packet id {} in Login State",
                    packet.id
                );
            }
        }
        Ok(None)
    }

    async fn handle_config_packet(
        &self,
        server: &Arc<Server>,
        packet: &RawPacket,
    ) -> Result<Option<PacketHandlerResult>, ReadingError> {
        debug!("Handling config group for id {}", packet.id);
        let mut payload = &packet.payload[..];
        let version = self.version.load();

        match packet.id {
            id if id == SClientInformationConfig::to_id(version) => {
                self.handle_client_information_config(SClientInformationConfig::read(
                    &mut payload,
                    &version,
                )?)
                .await;
            }
            id if id == SPluginMessage::to_id(version) => {
                self.handle_plugin_message(SPluginMessage::read(&mut payload, &version)?)
                    .await;
            }
            id if id
                == pumpkin_protocol::java::server::config::SCustomClickAction::to_id(version) =>
            {
                let _packet = pumpkin_protocol::java::server::config::SCustomClickAction::read(
                    &mut payload,
                    &version,
                )?;
                warn!("CustomClickAction in config state not yet supported");
            }
            id if id == SAcknowledgeFinishConfig::to_id(version) => {
                return Ok(Some(self.handle_config_acknowledged(server).await));
            }
            id if id == SKnownPacks::to_id(version) => {
                if let Some(i) = self
                    .handle_known_packs(SKnownPacks::read(&mut payload, &version)?, server)
                    .await
                {
                    return Ok(Some(i));
                }
            }
            id if id == pumpkin_protocol::java::server::config::SKeepAlive::to_id(version) => {
                self.handle_config_keep_alive(
                    pumpkin_protocol::java::server::config::SKeepAlive::read(
                        &mut payload,
                        &version,
                    )?,
                )
                .await;
            }
            id if id == SConfigCookieResponse::to_id(version) => {
                self.handle_config_cookie_response(&SConfigCookieResponse::read(
                    &mut payload,
                    &version,
                )?);
            }
            id if id == SConfigResourcePack::to_id(version) => {
                self.handle_resource_pack_response(
                    server,
                    SConfigResourcePack::read(&mut payload, &version)?,
                )
                .await;
            }
            _ => {
                error!(
                    "Failed to handle java client packet id {} in Config State",
                    packet.id
                );
            }
        }
        Ok(None)
    }

    #[expect(clippy::too_many_lines)]
    pub async fn handle_play_packet(
        &self,
        player: &Arc<Player>,
        server: &Arc<Server>,
        packet: &RawPacket,
    ) -> Result<(), Box<dyn PumpkinError>> {
        let mut payload = &packet.payload[..];
        let version = self.version.load();

        let mut event = crate::plugin::server::packet::PacketReceivedEvent::new(
            player.clone(),
            packet.id,
            packet.payload.clone(),
        );
        event = server.plugin_manager.fire(event).await;
        if event.cancelled {
            return Ok(());
        }

        match packet.id {
            id if id == SConfirmTeleport::to_id(version) => {
                self.handle_confirm_teleport(
                    player,
                    SConfirmTeleport::read(&mut payload, &version)?,
                )
                .await;
            }
            id if id == SChangeGameMode::to_id(version) => {
                self.handle_change_game_mode(
                    player,
                    SChangeGameMode::read(&mut payload, &version)?,
                )
                .await;
            }
            id if id == SChatCommand::to_id(version) => {
                self.handle_chat_command(
                    player,
                    server,
                    &(SChatCommand::read(&mut payload, &version)?),
                )
                .await;
            }
            id if id == SChatMessage::to_id(version) => {
                self.handle_chat_message(
                    server,
                    player,
                    SChatMessage::read(&mut payload, &version)?,
                )
                .await;
            }
            id if id == SClientInformationPlay::to_id(version) => {
                self.handle_client_information(
                    player,
                    SClientInformationPlay::read(&mut payload, &version)?,
                )
                .await;
            }
            id if id == SClientCommand::to_id(version) => {
                self.handle_client_status(player, SClientCommand::read(&mut payload, &version)?)
                    .await;
            }
            id if id == SPlayerInput::to_id(version) => {
                self.handle_player_input(
                    player,
                    SPlayerInput::read(&mut payload, &version)?,
                    server,
                )
                .await;
            }
            id if id == SMoveVehicle::to_id(version) => {
                self.handle_move_vehicle(player, SMoveVehicle::read(&mut payload, &version)?)
                    .await;
            }
            id if id == SPaddleBoat::to_id(version) => {
                self.handle_paddle_boat(player, SPaddleBoat::read(&mut payload, &version)?)
                    .await;
            }
            id if id == SInteract::to_id(version) => {
                self.handle_interact(player, SInteract::read(&mut payload, &version)?, server)
                    .await;
            }
            id if id == SBundleItemSelected::to_id(version) => {
                self.handle_bundle_item_selected(
                    player,
                    SBundleItemSelected::read(&mut payload, &version)?,
                )
                .await;
            }
            id if id == SAttack::to_id(version) => {
                self.handle_attack(player, SAttack::read(&mut payload, &version)?, server)
                    .await;
            }
            id if id == STeleportToEntity::to_id(version) => {
                self.handle_teleport_to_entity(
                    player,
                    STeleportToEntity::read(&mut payload, &version)?,
                    server,
                )
                .await;
            }
            id if id == pumpkin_protocol::java::server::play::SKeepAlive::to_id(version) => {
                self.handle_keep_alive(
                    player,
                    pumpkin_protocol::java::server::play::SKeepAlive::read(&mut payload, &version)?,
                )
                .await;
            }
            id if id == SClientTickEnd::to_id(version) => {
                // TODO
            }
            id if id == STestInstanceBlockAction::to_id(version) => {
                self.handle_test_instance_block_action(
                    player,
                    &STestInstanceBlockAction::read(&mut payload, &version)?,
                );
            }
            id if id == SSetTestBlock::to_id(version) => {
                self.handle_set_test_block(player, &SSetTestBlock::read(&mut payload, &version)?);
            }
            id if id == SPlayerPosition::to_id(version) => {
                self.handle_position(
                    player,
                    server,
                    SPlayerPosition::read(&mut payload, &version)?,
                )
                .await;
            }
            id if id == SPlayerPositionRotation::to_id(version) => {
                self.handle_position_rotation(
                    player,
                    server,
                    SPlayerPositionRotation::read(&mut payload, &version)?,
                )
                .await;
            }
            id if id == SPlayerRotation::to_id(version) => {
                self.handle_rotation(player, SPlayerRotation::read(&mut payload, &version)?)
                    .await;
            }
            id if id == SSetPlayerGround::to_id(version) => {
                let ground = SSetPlayerGround::read(&mut payload, &version)?;
                self.handle_player_ground(player, &ground).await;
            }
            id if id == SPickItemFromBlock::to_id(version) => {
                self.handle_pick_item_from_block(
                    player,
                    SPickItemFromBlock::read(&mut payload, &version)?,
                )
                .await;
            }
            id if id
                == pumpkin_protocol::java::server::play::SPickItemFromEntity::to_id(version) =>
            {
                self.handle_pick_item_from_entity(
                    player,
                    pumpkin_protocol::java::server::play::SPickItemFromEntity::read(
                        &mut payload,
                        &version,
                    )?,
                )
                .await;
            }
            id if id == SPlayerAbilities::to_id(version) => {
                self.handle_player_abilities(
                    player,
                    SPlayerAbilities::read(&mut payload, &version)?,
                    server,
                )
                .await;
            }
            id if id == SPlayerAction::to_id(version) => {
                self.handle_player_action(
                    player,
                    SPlayerAction::read(&mut payload, &version)?,
                    server,
                )
                .await;
            }
            id if id == SSetCommandBlock::to_id(version) => {
                self.handle_set_command_block(
                    player,
                    SSetCommandBlock::read(&mut payload, &version)?,
                )
                .await;
            }
            id if id == SSetJigsawBlock::to_id(version) => {
                self.handle_set_jigsaw_block(
                    player,
                    SSetJigsawBlock::read(&mut payload, &version)?,
                )
                .await;
            }
            id if id == SJigsawGenerate::to_id(version) => {
                self.handle_jigsaw_generate(player, SJigsawGenerate::read(&mut payload, &version)?)
                    .await;
            }
            id if id == SPlayerCommand::to_id(version) => {
                self.handle_player_command(
                    player,
                    SPlayerCommand::read(&mut payload, &version)?,
                    server,
                )
                .await;
            }
            id if id == SPlayerLoaded::to_id(version) => {
                Self::handle_player_loaded(player);
            }
            id if id == SPlayPingRequest::to_id(version) => {
                self.handle_play_ping_request(SPlayPingRequest::read(&mut payload, &version)?)
                    .await;
            }
            id if id == SClickSlot::to_id(version) => {
                player
                    .on_slot_click(SClickSlot::read(&mut payload, &version)?, server)
                    .await;
            }
            id if id == SContainerButtonClick::to_id(version) => {
                player
                    .on_container_button_click(SContainerButtonClick::read(&mut payload, &version)?)
                    .await;
            }
            id if id == SSetHeldItem::to_id(version) => {
                self.handle_set_held_item(player, SSetHeldItem::read(&mut payload, &version)?)
                    .await;
            }
            id if id == SSetCreativeSlot::to_id(version) => {
                self.handle_set_creative_slot(
                    player,
                    SSetCreativeSlot::read(&mut payload, &version)?,
                )
                .await?;
            }
            id if id == SSwingArm::to_id(version) => {
                self.handle_swing_arm(player, SSwingArm::read(&mut payload, &version)?)
                    .await;
            }
            id if id == SUpdateSign::to_id(version) => {
                self.handle_sign_update(player, SUpdateSign::read(&mut payload, &version)?)
                    .await;
            }
            id if id == SUseItemOn::to_id(version) => {
                self.handle_use_item_on(player, SUseItemOn::read(&mut payload, &version)?, server)
                    .await?;
            }
            id if id == SUseItem::to_id(version) => {
                self.handle_use_item(player, &SUseItem::read(&mut payload, &version)?, server)
                    .await;
            }
            id if id == SCommandSuggestion::to_id(version) => {
                self.handle_command_suggestion(
                    player,
                    SCommandSuggestion::read(&mut payload, &version)?,
                    server,
                )
                .await;
            }
            id if id == SPCookieResponse::to_id(version) => {
                self.handle_cookie_response(&SPCookieResponse::read(&mut payload, &version)?);
            }
            id if id == SCloseContainer::to_id(version) => {
                self.handle_close_container(
                    player,
                    server,
                    SCloseContainer::read(&mut payload, &version)?,
                )
                .await;
            }
            id if id == SChunkBatch::to_id(version) => {
                self.handle_chunk_batch(player, SChunkBatch::read(&mut payload, &version)?)
                    .await;
            }
            id if id == SPlayerSession::to_id(version) => {
                self.handle_chat_session_update(
                    player,
                    server,
                    SPlayerSession::read(&mut payload, &version)?,
                )
                .await;
            }
            id if id == SCustomPayload::to_id(version) => {
                let payload = SCustomPayload::read(&mut payload, &version)?;
                let event = PlayerCustomPayloadEvent::new(
                    player.clone(),
                    payload.channel.to_string(),
                    Bytes::copy_from_slice(payload.data),
                );
                server.plugin_manager.fire(event).await;
            }
            id if id == SRecipeBookChangeSettings::to_id(version) => {
                self.handle_recipe_book_change_settings(
                    player,
                    SRecipeBookChangeSettings::read(&mut payload, &version)?,
                )
                .await;
            }
            id if id == SRecipeBookSeenRecipe::to_id(version) => {
                self.handle_recipe_book_seen_recipe(
                    player,
                    SRecipeBookSeenRecipe::read(&mut payload, &version)?,
                )
                .await;
            }
            id if id == SRenameItem::to_id(version) => {
                player
                    .on_rename_item(SRenameItem::read(&mut payload, &version)?)
                    .await;
            }
            id if id == SPlaceRecipe::to_id(version) => {
                let packet = SPlaceRecipe::read(&mut payload, &version)?;
                self.handle_place_recipe(server, player, packet).await;
            }
            id if id
                == pumpkin_protocol::java::server::play::SCustomClickAction::to_id(version) =>
            {
                let packet = pumpkin_protocol::java::server::play::SCustomClickAction::read(
                    &mut payload,
                    &version,
                )?;
                let event = crate::plugin::api::events::player::custom_click_action::CustomClickActionEvent::new(
                    player.clone(),
                    packet.action_id.to_string(),
                    packet.payload.map(Bytes::copy_from_slice),
                );
                server.plugin_manager.fire(event).await;
            }
            id if id == SSelectTrade::to_id(version) => {
                self.handle_select_trade(player, SSelectTrade::read(&mut payload, &version)?)
                    .await;
            }
            id if id == SSeenAdvancement::to_id(version) => {
                self.handle_seen_advancement(
                    player,
                    SSeenAdvancement::read(&mut payload, &version)?,
                )
                .await;
            }
            _ => {
                warn!("Failed to handle player packet id {}", packet.id);
            }
        }
        Ok(())
    }
}
