#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub fn handle_confirm_teleport(&self, player: &Player, confirm_teleport: &SConfirmTeleport) {
        let confirm_id = confirm_teleport.teleport_id;
        {
            let mut awaiting_teleport = player
                .awaiting_teleport
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            match *awaiting_teleport {
                Some((id, position)) if id == confirm_id => {
                    player.get_entity().set_pos(position);
                    *awaiting_teleport = None;
                    drop(awaiting_teleport);
                    // Vanilla `handleAcceptTeleportPacket` -> `getChunkSource().move(player)`.
                    // Without it the view only follows the next client move, which never comes
                    // while the client stands in chunks it does not have.
                    if let Some(player) = player.world().get_player_by_uuid(player.gameprofile.id) {
                        chunker::update_position(&player);
                    }
                    return;
                }
                Some(_) => return,
                None if confirm_id.0 != player.teleport_id_count.load(Ordering::Relaxed) => return,
                None => {}
            }
        }
        self.try_kick(&TextComponent::text(
            "Send Teleport confirm, but we did not teleport",
        ));
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, atomic::Ordering};

    use arc_swap::ArcSwap;
    use pumpkin_config::{AdvancedConfiguration, BasicConfiguration, TelemetryConfig};
    use pumpkin_protocol::java::server::play::SConfirmTeleport;
    use pumpkin_protocol::{ConnectionState, codec::var_int::VarInt};
    use pumpkin_util::{
        GameMode,
        math::{vector2::Vector2, vector3::Vector3},
    };
    use uuid::Uuid;

    use crate::{
        data::VanillaData,
        entity::{EntityBase, player::Player},
        net::{
            ClientPlatform, GameProfile, PacketRateLimiter, PlayerConfig,
            java::{JavaClient, pending::PendingConnection},
        },
        server::Server,
        world::chunker,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn a_teleport_moves_the_view_once_confirmed_without_a_client_move()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempfile::tempdir()?;
        let basic = BasicConfiguration {
            default_level_name: directory
                .path()
                .join("world")
                .to_string_lossy()
                .into_owned(),
            allow_nether: false,
            allow_end: false,
            ..Default::default()
        };
        let mut advanced = AdvancedConfiguration::default();
        advanced.networking.bedrock.online_mode = false;
        let server = Server::new(
            basic,
            advanced,
            TelemetryConfig::default(),
            VanillaData::default(),
            vec![],
        )
        .await?;
        let world = server.worlds.load().first().cloned().ok_or("No world")?;
        let (transport, _peer) = tokio::io::duplex(1 << 20);
        let (read, write) = tokio::io::split(transport);
        let pending = PendingConnection::from_transport(
            Box::new(read),
            Box::new(write),
            "127.0.0.1:25565".parse()?,
            0,
            PacketRateLimiter::new(false, 0.0, 0.0),
            Arc::downgrade(&server),
        );
        let profile = GameProfile {
            id: Uuid::new_v4(),
            name: "TeleportTest".into(),
            properties: ArcSwap::from_pointee(vec![]),
            profile_actions: None,
        };
        let mut client =
            JavaClient::from_pending(pending, profile.clone(), PlayerConfig::default());
        client.connection_state.store(ConnectionState::Play);
        client.start_outgoing_packet_task();
        let player = Arc::new(Player::new(
            Arc::new(ClientPlatform::Java(client)),
            profile,
            PlayerConfig::default(),
            &world,
            GameMode::Creative,
        ));
        world.add_player(&player)?;
        player.get_entity().set_pos(Vector3::new(0.5, 100.0, 0.5));
        chunker::update_position(&player);
        assert_eq!(player.watched_section.load().center, Vector2::new(0, 0));

        // `/tp 1000 100 1000`: chunks there were never sent, so the client cannot move.
        let target = Vector3::new(1000.5, 100.0, 1000.5);
        player.teleport(target, None, None, world);
        assert_eq!(player.watched_section.load().center, Vector2::new(0, 0));

        let ClientPlatform::Java(java) = player.client.as_ref() else {
            return Err("not a Java client".into());
        };
        java.handle_confirm_teleport(
            &player,
            &SConfirmTeleport {
                teleport_id: VarInt(player.teleport_id_count.load(Ordering::Relaxed)),
                position: target,
                yaw: 0.0,
                pitch: 0.0,
            },
        );
        assert!(player.awaiting_teleport.lock().unwrap().is_none());
        assert_eq!(player.watched_section.load().center, Vector2::new(62, 62));
        Ok(())
    }
}
