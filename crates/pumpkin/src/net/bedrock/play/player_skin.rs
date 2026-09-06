use std::{sync::Arc, time::Duration};

use pumpkin_protocol::bedrock::{
    client::player_skin::CPlayerSkin, server::player_skin::SPlayerSkin,
};

use crate::{
    entity::player::Player,
    net::{ClientPlatform, bedrock::BedrockClient},
    server::Server,
};

impl BedrockClient {
    pub async fn handle_player_skin(
        &self,
        player: &Arc<Player>,
        server: &Server,
        packet: SPlayerSkin,
    ) {
        if packet.uuid != player.gameprofile.id {
            tracing::warn!(
                player = %player.gameprofile.name,
                claimed_uuid = %packet.uuid,
                "Rejected a Bedrock skin update for another player"
            );
            return;
        }

        let config = &server.advanced_config.networking.bedrock.skins;
        let (skin, changed) = server.bedrock_skin_packs.accept(
            packet.uuid,
            packet.skin,
            config.trusted_only,
            Duration::from_secs(config.change_cooldown_seconds),
        );
        if !changed {
            return;
        }

        player.bedrock_skin.store(Arc::new(skin.clone()));
        let previous_pack = server.bedrock_skin_packs.current().await;
        let _ = server
            .bedrock_skin_packs
            .register(player.gameprofile.id, &skin)
            .await;
        if let Some(pack) = server.bedrock_skin_packs.current().await
            && previous_pack.as_ref().map(|pack| pack.id) != Some(pack.id)
        {
            server.push_bedrock_skin_pack(pack).await;
        }
        let update = CPlayerSkin {
            uuid: player.gameprofile.id,
            skin: &skin,
            new_skin_name: &packet.new_skin_name,
            old_skin_name: &packet.old_skin_name,
        };
        for recipient in player.world().players.load().iter() {
            if recipient.gameprofile.id != player.gameprofile.id
                && let ClientPlatform::Bedrock(client) = recipient.client.as_ref()
            {
                client.try_enqueue_client_packet(&update);
            }
        }
    }
}
