use super::*;
use crate::data::VanillaData;
use pumpkin_config::{AdvancedConfiguration, BasicConfiguration};
use pumpkin_util::GameMode;
use std::sync::RwLock;
use tokio::net::{TcpListener, TcpStream};
use uuid::Uuid;

#[tokio::test]
async fn disconnected_players_are_released() -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let basic = BasicConfiguration {
        default_level_name: directory.path().to_string_lossy().into_owned(),
        allow_nether: false,
        allow_end: false,
        use_favicon: false,
        ..Default::default()
    };
    let mut advanced = AdvancedConfiguration::default();
    advanced.networking.bedrock.online_mode = false;
    let data = VanillaData {
        banned_ip_list: RwLock::default(),
        banned_player_list: RwLock::default(),
        operator_config: RwLock::default(),
        user_cache: RwLock::default(),
        whitelist_config: RwLock::default(),
    };
    let server = Server::new(basic, advanced, data).await;
    let world = server.get_world_from_dimension(&pumpkin_data::dimension::Dimension::OVERWORLD);
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let mut retained_players = Vec::new();
    for id in 0..16 {
        let remote = TcpStream::connect(listener.local_addr()?).await?;
        let (stream, address) = listener.accept().await?;
        let profile = GameProfile {
            id: Uuid::new_v4(),
            name: format!("reconnect{id}"),
            properties: ArcSwap::from_pointee(Vec::new()),
            profile_actions: None,
        };
        let pending =
            PendingConnection::new(stream, address, id, PacketRateLimiter::new(false, 0.0, 0.0));
        let client = Arc::new(ClientPlatform::Java(JavaClient::from_pending(
            pending,
            profile.clone(),
            PlayerConfig::default(),
        )));
        let player = Arc::new(Player::new(
            client.clone(),
            profile,
            PlayerConfig::default(),
            &world,
            GameMode::Survival,
        ));
        player.screen_handler_sync_handler.store_player(
            &(player.clone() as Arc<dyn pumpkin_inventory::screen_handler::InventoryPlayer>),
        );
        if let ClientPlatform::Java(java) = client.as_ref() {
            java.set_player(&player);
            java.close();
            java.await_tasks().await;
        }
        retained_players.push(Arc::downgrade(&player));
        drop(player);
        drop(client);
        drop(remote);
    }
    server.shutdown().await;
    assert!(
        retained_players
            .iter()
            .all(|player| player.upgrade().is_none()),
        "disconnected players are still owned by their client or inventory sync handler"
    );
    Ok(())
}
