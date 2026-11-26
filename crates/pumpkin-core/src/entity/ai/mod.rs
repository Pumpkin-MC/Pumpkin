pub mod behavior;
pub mod brain;
pub mod control;
pub mod goal;
pub mod pathfinder;
pub mod random_pos;
pub mod sensing;
pub mod target_match;
pub mod target_predicate;
pub mod util;

#[cfg(test)]
pub(crate) mod test_support {
    use std::sync::{Arc, Weak};

    use arc_swap::ArcSwap;
    use pumpkin_config::{
        AdvancedConfiguration, BasicConfiguration, TelemetryConfig, world::LevelConfig,
    };
    use pumpkin_data::dimension::Dimension;
    use pumpkin_util::world_seed::Seed;
    use pumpkin_world::{level::Level, world_info::LevelData};

    use crate::{
        block::registry::default_registry, data::VanillaData, server::Server, world::World,
    };

    pub async fn server(
        folder: &tempfile::TempDir,
    ) -> Result<Arc<Server>, pumpkin_world::world_info::WorldInfoError> {
        let basic_config = BasicConfiguration {
            default_level_name: folder.path().to_string_lossy().into_owned(),
            allow_nether: false,
            allow_end: false,
            allow_chat_reports: false,
            ..Default::default()
        };
        let mut advanced_config = AdvancedConfiguration::default();
        advanced_config.update_checker.enabled = false;
        advanced_config.networking.java.online_mode = false;
        advanced_config.networking.bedrock.enabled = false;
        advanced_config.networking.bedrock.authentication.enabled = false;
        Server::new(
            basic_config,
            advanced_config,
            TelemetryConfig::default(),
            VanillaData::default(),
            Vec::new(),
        )
        .await
    }

    pub fn java_sound_recipient(
        server: &Arc<Server>,
        world: &Arc<World>,
        name: &str,
    ) -> Result<
        (Arc<crate::entity::player::Player>, tokio::io::DuplexStream),
        Box<dyn std::error::Error>,
    > {
        use crate::entity::player::Player;
        use crate::net::{
            ClientPlatform, GameProfile, PacketRateLimiter, PlayerConfig,
            java::{JavaClient, pending::PendingConnection},
        };
        use pumpkin_protocol::ConnectionState;
        use pumpkin_util::GameMode;

        let (transport, peer) = tokio::io::duplex(65536);
        let (read, write) = tokio::io::split(transport);
        let pending = PendingConnection::from_transport(
            Box::new(read),
            Box::new(write),
            "127.0.0.1:25565".parse()?,
            0,
            PacketRateLimiter::new(false, 0.0, 0.0),
            Arc::downgrade(server),
        );
        let profile = GameProfile {
            id: uuid::Uuid::new_v4(),
            name: name.into(),
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
            world,
            GameMode::Survival,
        ));
        player
            .client
            .java()
            .ok_or("Expected Java client")?
            .set_player(player.clone());
        world.add_player(&player)?;

        Ok((player, peer))
    }

    pub fn world(folder: &tempfile::TempDir) -> Arc<World> {
        world_in_dimension(folder, Dimension::OVERWORLD.clone())
    }

    pub fn world_in_dimension(folder: &tempfile::TempDir, dimension: Dimension) -> Arc<World> {
        Arc::new(World::load(
            Level::from_root_folder(
                &LevelConfig::default(),
                folder.path().to_path_buf(),
                0,
                dimension.clone(),
            ),
            Arc::new(ArcSwap::from_pointee(LevelData::default(Seed(0)))),
            dimension,
            default_registry(),
            Weak::new(),
        ))
    }
}
