#[allow(clippy::wildcard_imports)]
use super::*;
use pumpkin_protocol::java::{
    client::play::CRemoveResourcePack,
    server::play::{PlayResourcePackResult, SPlayResourcePack},
};
use pumpkin_util::{text::TextComponent, version::JavaMinecraftVersion};

use crate::plugin::api::events::player::player_resource_pack_status::PlayerResourcePackStatusEvent;

#[derive(Debug, PartialEq, Eq)]
enum SkinPackUpdate {
    Ignored,
    Completed { refresh: bool },
    Unloaded,
}

impl JavaClient {
    pub fn handle_play_resource_pack_response(
        &self,
        server: &Arc<Server>,
        player: &Arc<Player>,
        packet: &SPlayResourcePack,
    ) {
        let result = packet.response_result();
        debug!(
            "Player {} resource pack response for {}: {:?}",
            player.gameprofile.name, packet.uuid, result
        );

        let mut event = PlayerResourcePackStatusEvent::new(
            player.clone(),
            packet.uuid.to_string(),
            format!("{result:?}"),
        );
        server.plugin_manager.fire_blocking(server, &mut event);

        if self.handle_configured_resource_pack_response(
            &server.advanced_config.resource_pack.java,
            packet.uuid,
            &result,
        ) {
            return;
        }

        let player_c = player.clone();
        let server_c = server.clone();
        let id = packet.uuid;
        player.spawn_task(async move {
            if let Some(client) = player_c.client.java() {
                client
                    .handle_bedrock_skin_pack_response(&server_c, &player_c, id, result)
                    .await;
            }
        });
    }

    fn handle_configured_resource_pack_response(
        &self,
        config: &pumpkin_config::resource_pack::JavaResourcePackConfig,
        id: uuid::Uuid,
        result: &PlayResourcePackResult,
    ) -> bool {
        let configured_id = uuid::Uuid::new_v3(&uuid::Uuid::NAMESPACE_DNS, config.url.as_bytes());
        let configured_response = id == configured_id
            || (self.version.load() < JavaMinecraftVersion::V_1_20_3 && id.is_nil());
        if !config.enabled || !configured_response {
            return false;
        }
        if config.force
            && matches!(
                result,
                PlayResourcePackResult::Declined | PlayResourcePackResult::DownloadFail
            )
        {
            self.try_kick(&TextComponent::text(
                "You must accept the resource pack to play on this server.",
            ));
        }
        true
    }

    async fn handle_bedrock_skin_pack_response(
        &self,
        server: &Server,
        player: &Player,
        id: uuid::Uuid,
        result: PlayResourcePackResult,
    ) {
        let mut pending = self.pending_bedrock_skin_pack.lock().await;
        if self.is_closed() {
            return;
        }
        match self
            .apply_bedrock_skin_pack_response(&mut pending, id, result)
            .await
        {
            SkinPackUpdate::Ignored => return,
            SkinPackUpdate::Unloaded => {
                player
                    .world()
                    .refresh_bedrock_players_for_java(player)
                    .await;
                return;
            }
            SkinPackUpdate::Completed { refresh } => {
                if refresh {
                    player
                        .world()
                        .refresh_bedrock_players_for_java(player)
                        .await;
                }
            }
        }

        // Coalesce changes that arrived during download into one latest offer.
        drop(pending);
        if let Some(latest) = server.bedrock_skin_packs.current().await
            && latest.id != id
        {
            self.push_bedrock_skin_pack(server, latest).await;
        }
    }

    async fn apply_bedrock_skin_pack_response(
        &self,
        pending: &mut Option<Arc<crate::net::bedrock::skin_pack::BedrockSkinPack>>,
        id: uuid::Uuid,
        result: PlayResourcePackResult,
    ) -> SkinPackUpdate {
        let in_progress = matches!(
            result,
            PlayResourcePackResult::Accepted | PlayResourcePackResult::Downloaded
        );
        if pending.as_ref().is_some_and(|pack| pack.id == id) {
            if in_progress {
                return SkinPackUpdate::Ignored;
            }
            // Take only this session's outstanding offer, never an arbitrary pack
            // from the global registry or a duplicate response to an older offer.
            let Some(pack) = pending.take() else {
                return SkinPackUpdate::Ignored;
            };
            let refresh = result == PlayResourcePackResult::DownloadSuccess;
            if refresh {
                let previous = self.bedrock_skin_pack.swap(Some(pack));
                if let Some(previous) = previous
                    && previous.id != id
                {
                    self.enqueue_client_packet(&CRemoveResourcePack::new(Some(&previous.id)))
                        .await;
                }
            }
            SkinPackUpdate::Completed { refresh }
        } else if self
            .bedrock_skin_pack
            .load_full()
            .is_some_and(|pack| pack.id == id)
            && !in_progress
            && result != PlayResourcePackResult::DownloadSuccess
        {
            self.bedrock_skin_pack.store(None);
            SkinPackUpdate::Unloaded
        } else {
            SkinPackUpdate::Ignored
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::bedrock::skin_pack::{BedrockSkinPack, BedrockSkinPacks};
    use crate::net::java::pending::PendingConnection;
    use crate::net::{GameProfile, PacketRateLimiter, PlayerConfig};
    use pumpkin_protocol::{
        ConnectionState, bedrock::client::Skin, java::client::play::CAddResourcePack,
    };
    use tokio::net::{TcpListener, TcpStream};

    async fn client() -> (JavaClient, TcpStream) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let (peer, accepted) = tokio::join!(
            TcpStream::connect(listener.local_addr().unwrap()),
            listener.accept(),
        );
        let (socket, address) = accepted.unwrap();
        let mut pending = PendingConnection::new(
            socket,
            address,
            1,
            PacketRateLimiter::new(false, 100.0, 100.0),
        );
        pending.server_address = "127.0.0.1".to_string();
        pending.version.store(JavaMinecraftVersion::V_26_2);
        pending.connection_state.store(ConnectionState::Play);
        let profile = GameProfile {
            id: uuid::Uuid::from_u128(1),
            name: "SkinPackTest".to_string(),
            properties: arc_swap::ArcSwap::from_pointee(Vec::new()),
            profile_actions: None,
        };
        (
            JavaClient::from_pending(pending, profile, PlayerConfig::default()),
            peer.unwrap(),
        )
    }

    async fn revision(packs: &BedrockSkinPacks, marker: u8) -> Arc<BedrockSkinPack> {
        let mut skin = Skin::steve();
        skin.skin_data[0] = marker;
        packs
            .register(uuid::Uuid::from_u128(7), &skin)
            .await
            .unwrap();
        packs.current().await.unwrap()
    }

    async fn reply(
        client: &JavaClient,
        id: uuid::Uuid,
        result: PlayResourcePackResult,
    ) -> SkinPackUpdate {
        let mut pending = client.pending_bedrock_skin_pack.lock().await;
        client
            .apply_bedrock_skin_pack_response(&mut pending, id, result)
            .await
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn concurrent_offers_are_owned_and_coalesce_to_the_latest_revision() {
        let (mut client, _peer) = client().await;
        let mut outbound = client.outgoing_packet_queue_recv.take().unwrap();
        let client = Arc::new(client);
        let packs = Arc::new(BedrockSkinPacks::default());
        let first = revision(&packs, 1).await;
        let barrier = Arc::new(tokio::sync::Barrier::new(8));
        let mut offers = tokio::task::JoinSet::new();
        for _ in 0..8 {
            let (client, packs, first, barrier) = (
                client.clone(),
                packs.clone(),
                first.clone(),
                barrier.clone(),
            );
            offers.spawn(async move {
                barrier.wait().await;
                client
                    .offer_bedrock_skin_pack(&packs, first, 19132, None)
                    .await;
            });
        }
        while let Some(result) = offers.join_next().await {
            result.unwrap();
        }
        assert!(outbound.try_recv().is_ok());
        assert!(outbound.try_recv().is_err());
        assert!(client.bedrock_skin_pack.load().is_none());

        let second = revision(&packs, 2).await;
        let latest = revision(&packs, 3).await;
        client
            .offer_bedrock_skin_pack(&packs, second.clone(), 19132, None)
            .await;
        assert!(outbound.try_recv().is_err());
        assert_eq!(
            reply(&client, latest.id, PlayResourcePackResult::DownloadSuccess).await,
            SkinPackUpdate::Ignored
        );
        assert_eq!(
            reply(&client, first.id, PlayResourcePackResult::Accepted).await,
            SkinPackUpdate::Ignored
        );
        assert_eq!(
            client
                .pending_bedrock_skin_pack
                .lock()
                .await
                .as_ref()
                .unwrap()
                .id,
            first.id
        );
        assert!(client.bedrock_skin_pack.load().is_none());
        assert_eq!(
            reply(&client, first.id, PlayResourcePackResult::DownloadSuccess).await,
            SkinPackUpdate::Completed { refresh: true }
        );

        // Even a stale broadcast must offer the registry's latest revision next.
        client
            .offer_bedrock_skin_pack(&packs, second, 19132, None)
            .await;
        let url = format!("http://127.0.0.1:19132/v1/skin-packs/{}", latest.id);
        let expected = client
            .serialize_packet(&CAddResourcePack::new(
                &latest.id,
                &url,
                &latest.hash,
                false,
                None,
            ))
            .unwrap();
        assert_eq!(outbound.try_recv().unwrap().data, expected);
        assert!(outbound.try_recv().is_err());
        assert_eq!(
            client
                .pending_bedrock_skin_pack
                .lock()
                .await
                .as_ref()
                .unwrap()
                .id,
            latest.id
        );
        assert_eq!(
            reply(&client, first.id, PlayResourcePackResult::DownloadSuccess).await,
            SkinPackUpdate::Ignored
        );
        assert_eq!(
            client
                .pending_bedrock_skin_pack
                .lock()
                .await
                .as_ref()
                .unwrap()
                .id,
            latest.id
        );
    }

    #[tokio::test]
    async fn replacement_pops_only_the_previous_pack_and_optional_failure_keeps_it_loaded() {
        let (mut client, _peer) = client().await;
        let mut outbound = client.outgoing_packet_queue_recv.take().unwrap();
        let packs = BedrockSkinPacks::default();
        let first = revision(&packs, 1).await;
        client
            .offer_bedrock_skin_pack(&packs, first.clone(), 19132, None)
            .await;
        outbound.try_recv().unwrap();
        assert_eq!(
            reply(&client, first.id, PlayResourcePackResult::DownloadSuccess).await,
            SkinPackUpdate::Completed { refresh: true }
        );
        assert!(outbound.try_recv().is_err());

        let second = revision(&packs, 2).await;
        client
            .offer_bedrock_skin_pack(&packs, second.clone(), 19132, None)
            .await;
        outbound.try_recv().unwrap();
        assert_eq!(
            reply(&client, second.id, PlayResourcePackResult::Downloaded).await,
            SkinPackUpdate::Ignored
        );
        assert_eq!(client.bedrock_skin_pack.load_full().unwrap().id, first.id);
        assert_eq!(
            reply(&client, second.id, PlayResourcePackResult::DownloadSuccess).await,
            SkinPackUpdate::Completed { refresh: true }
        );
        let expected = client
            .serialize_packet(&CRemoveResourcePack::new(Some(&first.id)))
            .unwrap();
        assert_eq!(outbound.try_recv().unwrap().data, expected);
        assert!(outbound.try_recv().is_err());
        assert_eq!(client.bedrock_skin_pack.load_full().unwrap().id, second.id);

        let rejected = revision(&packs, 3).await;
        client
            .offer_bedrock_skin_pack(&packs, rejected.clone(), 19132, None)
            .await;
        outbound.try_recv().unwrap();
        let required = pumpkin_config::resource_pack::JavaResourcePackConfig {
            enabled: true,
            force: true,
            url: "https://example.org/required.zip".to_string(),
            ..Default::default()
        };
        assert!(!client.handle_configured_resource_pack_response(
            &required,
            rejected.id,
            &PlayResourcePackResult::Declined
        ));
        assert!(!client.is_closed());
        assert_eq!(
            reply(&client, rejected.id, PlayResourcePackResult::Declined).await,
            SkinPackUpdate::Completed { refresh: false }
        );
        assert_eq!(client.bedrock_skin_pack.load_full().unwrap().id, second.id);
        assert!(client.pending_bedrock_skin_pack.lock().await.is_none());
        assert!(outbound.try_recv().is_err());

        let required_id = uuid::Uuid::new_v3(&uuid::Uuid::NAMESPACE_DNS, required.url.as_bytes());
        assert!(client.handle_configured_resource_pack_response(
            &required,
            required_id,
            &PlayResourcePackResult::Declined
        ));
        assert!(client.is_closed());
    }
}
