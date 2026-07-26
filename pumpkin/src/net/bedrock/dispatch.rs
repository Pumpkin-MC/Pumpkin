use super::BedrockClient;
use crate::entity::player::Player;
use crate::net::DisconnectReason;
use crate::net::PacketHandlerResult;
use crate::server::Server;
use pumpkin_protocol::RawPacket;
use pumpkin_protocol::bedrock::client::play_status::CPlayStatus;
use pumpkin_protocol::bedrock::client::resource_packs_info::CResourcePacksInfo;
use pumpkin_protocol::bedrock::client::resource_packs_info::ResourcePackEntry;
use pumpkin_protocol::bedrock::server::animate::SAnimate;
use pumpkin_protocol::bedrock::server::block_pick_request::SBlockPickRequest;
use pumpkin_protocol::bedrock::server::client_cache_status::SClientCacheStatus;
use pumpkin_protocol::bedrock::server::client_to_server_handshake::SClientToServerHandshake;
use pumpkin_protocol::bedrock::server::command_request::SCommandRequest;
use pumpkin_protocol::bedrock::server::container_close::SContainerClose;
use pumpkin_protocol::bedrock::server::emote::SEmote;
use pumpkin_protocol::bedrock::server::interaction::SInteraction;
use pumpkin_protocol::bedrock::server::inventory_transaction::SInventoryTransaction;
use pumpkin_protocol::bedrock::server::loading_screen::SLoadingScreen;
use pumpkin_protocol::bedrock::server::login::SLogin;
use pumpkin_protocol::bedrock::server::mob_equipment::SMobEquipment;
use pumpkin_protocol::bedrock::server::player_action::SPlayerAction;
use pumpkin_protocol::bedrock::server::player_auth_input::SPlayerAuthInput;
use pumpkin_protocol::bedrock::server::request_ability::SRequestAbility;
use pumpkin_protocol::bedrock::server::request_chunk_radius::SRequestChunkRadius;
use pumpkin_protocol::bedrock::server::request_network_settings::SRequestNetworkSettings;
use pumpkin_protocol::bedrock::server::resource_pack_response::SResourcePackResponse;
use pumpkin_protocol::bedrock::server::set_local_player_as_initialized::SSetLocalPlayerAsInitialized;
use pumpkin_protocol::bedrock::server::set_player_inventory_options::SSetPlayerInventoryOptions;
use pumpkin_protocol::bedrock::server::text::SText;
use pumpkin_protocol::packet::Packet;
use pumpkin_protocol::serial::{PacketRead, PacketReadSlice};
use std::io::Cursor;
use std::io::Error;
use std::sync::Arc;
use tracing::debug;
use tracing::error;
use tracing::warn;

impl BedrockClient {
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
                        Ok(Some(result)) => return result,
                        Ok(None) => {} // encryption enabled, wait for handshake
                        Err(err) => {
                            self.kick(DisconnectReason::Unknown, err.to_string()).await;
                            return PacketHandlerResult::Stop;
                        }
                    }
                }
                SClientToServerHandshake::PACKET_ID => {
                    let _packet = match SClientToServerHandshake::read(payload) {
                        Ok(p) => p,
                        Err(err) => {
                            error!("Failed to read SClientToServerHandshake: {err}");
                            continue;
                        }
                    };
                    let pending = self.pending_profile.lock().await.take();
                    if let Some((profile, new_config)) = pending {
                        self.enqueue_packet_internal(&CPlayStatus::LoginSuccess)
                            .await;
                        let br_config = &server.advanced_config.resource_pack.bedrock;

                        let mut entries = Vec::new();
                        if br_config.enabled {
                            for pack in &br_config.packs {
                                entries.push(ResourcePackEntry {
                                    uuid: pack.uuid,
                                    version: pack.version.clone(),
                                    size: pack.size,
                                    download_url: pack.download_url.clone(),
                                    content_key: pack.content_key.clone(),
                                    sub_pack_name: pack.sub_pack_name.clone(),
                                    content_id: pack.content_id.clone(),
                                    has_scripts: pack.has_scripts,
                                    addon_pack: pack.addon_pack,
                                    rtx_enabled: pack.rtx_enabled,
                                });
                            }
                        }

                        let packs_info = CResourcePacksInfo {
                            resource_pack_required: br_config.force,
                            has_addon_packs: false,
                            has_scripts: false,
                            is_vibrant_visuals_force_disabled: false,
                            world_template_id: uuid::Uuid::nil(),
                            world_template_version: String::new(),
                            resource_packs: entries,
                        };
                        self.enqueue_packet_internal(&packs_info).await;
                        return PacketHandlerResult::ReadyToPlay(profile, new_config);
                    }
                    error!("Received ClientToServerHandshake but no pending profile was found.");
                    self.kick(DisconnectReason::BadPacket, "Handshake error".into())
                        .await;
                    return PacketHandlerResult::Stop;
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
            event = server.plugin_manager.fire(event).await;
            if event.cancelled {
                continue;
            }

            if let Err(err) = self.handle_play_packet(player, server, packet).await {
                error!("Failed to handle Bedrock play packet: {err}");
            }
        }
    }

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
                // TODO
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
            SAnimate::PACKET_ID => {
                self.handle_animate(player, server, &SAnimate::read(reader)?).await;
            }
            SEmote::PACKET_ID => {
                self.handle_emote(player, server, SEmote::read_slice(reader)?).await;
            }
            // SEmoteList::PACKET_ID => {
            //     self.handle_emote_list(player, server, SEmoteList::read(reader)?);
            // }
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
                self.handle_mob_equipment(player, SMobEquipment::read(reader)?)
                    .await;
            }
            _ => {
                warn!("Bedrock: Received Unknown Game packet: {}", packet.id);
            }
        }
        Ok(())
    }
}
