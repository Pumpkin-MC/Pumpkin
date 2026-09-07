use std::sync::atomic::Ordering;

#[allow(clippy::wildcard_imports)]
use super::*;

impl BedrockClient {
    pub async fn handle_resource_pack_response(
        &self,
        packet: SResourcePackClientResponse,
        server: &Arc<Server>,
    ) {
        if self.resource_pack_completed.load(Ordering::Acquire) {
            return;
        }

        match packet.response {
            SResourcePackClientResponse::STATUS_REFUSED => {
                debug!("Bedrock: SResourcePackResponse::STATUS_REFUSED");
                self.kick(
                    DisconnectReason::ResourcePackProblem,
                    "You must accept resource packs to join this server.".into(),
                )
                .await;
            }
            SResourcePackClientResponse::STATUS_SEND_PACKS => {
                debug!("Bedrock: SResourcePackResponse::STATUS_SEND_PACKS");
                // TODO: send packs
            }
            SResourcePackClientResponse::STATUS_HAVE_ALL_PACKS => {
                debug!("Bedrock: SResourcePackResponse::STATUS_HAVE_ALL_PACKS");
                let br_config = &server.advanced_config.resource_pack.bedrock;
                // Convert your config packs into protocol stack entries
                let resource_packs = if br_config.enabled {
                    br_config
                        .packs
                        .iter()
                        .map(|pack| PackInstanceId {
                            pack_id: pack.uuid.to_string(),
                            version: pack.version.clone(),
                            sub_pack_name: String::new(),
                        })
                        .collect()
                } else {
                    Vec::new()
                };

                self.enqueue_client_packet(&CResourcePackStackPacket {
                    texture_pack_required: br_config.force,
                    texture_pack_list: resource_packs,
                    base_game_version: CURRENT_BEDROCK_MC_VERSION.to_string(),
                    experiments: Experiments {
                        toggles: Vec::new(),
                        experiments_ever_toggled: false,
                    },
                    include_editor_packs: false,
                })
                .await;
            }
            SResourcePackClientResponse::STATUS_COMPLETED => {
                debug!("Bedrock: SResourcePackResponse::STATUS_COMPLETED");
                let player = self.player.load_full();
                if let Some(player) = player.as_ref() {
                    // Multiple packet-handler tasks may reach completion together.
                    // Claim before any spawn await, not when initial chunks arrive.
                    if self
                        .resource_pack_completed
                        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
                        .is_err()
                    {
                        return;
                    }
                    player
                        .world()
                        .spawn_bedrock_player(&server.basic_config, player.clone(), server)
                        .await;
                } else {
                    tracing::error!(
                        "Got SResourcePackResponse::STATUS_COMPLETED before authentication was completed."
                    );
                    self.kick(DisconnectReason::Disconnected, String::new())
                        .await;
                }
            }
            _ => {
                tracing::error!("Bedrock: SResourcePackResponse bad response type");
                self.kick(DisconnectReason::Disconnected, String::new())
                    .await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::future::Future;
    use std::io::Cursor;
    use std::sync::{Arc, atomic::Ordering};
    use std::time::Duration;

    use pumpkin_protocol::bedrock::client::{CDisconnect, CStartGame};
    use pumpkin_protocol::bedrock::packet_decoder::BedrockBatchDecoder;
    use pumpkin_protocol::bedrock::server::resource_pack_client_response::SResourcePackClientResponse;
    use pumpkin_protocol::{Packet, RawPacket};

    use crate::net::bedrock::{BedrockClient, OutgoingPacket};
    use crate::net::decrement_pending_bytes;
    use crate::test_support::TestServer;

    fn response(status: u8) -> SResourcePackClientResponse {
        SResourcePackClientResponse {
            response: status,
            download_size: 0,
            pack_ids: Vec::new(),
        }
    }

    async fn first_priority_packet(
        client: &BedrockClient,
        handler: &mut (impl Future<Output = ()> + Unpin),
    ) -> OutgoingPacket {
        let mut receiver = client.outgoing_packet_priority_recv.lock().await;
        let receiver = receiver.as_mut().expect("test owns the network queue");
        let packet = tokio::time::timeout(Duration::from_secs(5), async {
            tokio::select! {
                () = handler => panic!("handler must wait for its priority packet completion"),
                packet = receiver.recv() => packet.expect("priority packet"),
            }
        })
        .await
        .expect("handler should enqueue its first packet");
        decrement_pending_bytes(&client.pending_bytes, packet.data.len());
        assert!(packet.completion.is_some());
        packet
    }

    async fn decode_packet(packet: &OutgoingPacket) -> RawPacket {
        let mut decoder = BedrockBatchDecoder::new();
        let payload = decoder
            .get_packet_payload(packet.data.to_vec())
            .await
            .unwrap();
        decoder.get_game_packet(&mut Cursor::new(payload)).unwrap()
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn duplicate_resource_pack_completion_cannot_spawn_while_start_game_is_pending() {
        let fixture = TestServer::new().await;
        let player = fixture.new_bedrock_player().await;
        // Keep this focused on the handshake rather than generating a spawn chunk.
        player.has_played_before.store(true, Ordering::Relaxed);
        player.bedrock_spawned.store(false, Ordering::Relaxed);
        let client = player.client.bedrock().unwrap();
        let mut first = Box::pin(client.handle_resource_pack_response(
            response(SResourcePackClientResponse::STATUS_COMPLETED),
            &fixture.server,
        ));
        let start_game = first_priority_packet(client, &mut first).await;
        assert_eq!(decode_packet(&start_game).await.id, CStartGame::PACKET_ID);
        assert!(client.resource_pack_completed.load(Ordering::Acquire));
        assert!(!player.bedrock_spawned.load(Ordering::Relaxed));

        // The first task is still suspended: its completion sender is held here.
        // Neither a concurrent completion nor stale negotiation packets may restart it.
        for status in [
            SResourcePackClientResponse::STATUS_COMPLETED,
            SResourcePackClientResponse::STATUS_HAVE_ALL_PACKS,
            SResourcePackClientResponse::STATUS_SEND_PACKS,
            SResourcePackClientResponse::STATUS_REFUSED,
            0,
        ] {
            tokio::time::timeout(
                Duration::from_secs(1),
                client.handle_resource_pack_response(response(status), &fixture.server),
            )
            .await
            .expect("responses after completion must not wait for another spawn");
        }
        assert!(!client.is_closed());
        assert!(client.drain_outgoing_packets_for_test().await.is_empty());
        assert!(
            client
                .outgoing_packet_priority_recv
                .lock()
                .await
                .as_mut()
                .unwrap()
                .try_recv()
                .is_err()
        );
        assert_eq!(client.pending_bytes.load(Ordering::Acquire), 0);

        // Cancel the intentionally blocked spawn before shutting down the fixture.
        drop(first);
        drop(start_game);
        drop(player);
        fixture.shutdown().await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn resource_pack_completion_before_authentication_still_disconnects() {
        let fixture = TestServer::new().await;
        let player = fixture.new_bedrock_player().await;
        let client = player.client.bedrock().unwrap();
        client.player.store(Arc::new(None));
        let mut handler = Box::pin(client.handle_resource_pack_response(
            response(SResourcePackClientResponse::STATUS_COMPLETED),
            &fixture.server,
        ));
        let disconnect = first_priority_packet(client, &mut handler).await;
        assert_eq!(decode_packet(&disconnect).await.id, CDisconnect::PACKET_ID);
        assert!(!client.resource_pack_completed.load(Ordering::Acquire));
        drop(handler);
        drop(disconnect);
        drop(player);
        fixture.shutdown().await;
    }
}
