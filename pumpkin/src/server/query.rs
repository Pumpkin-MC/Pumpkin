use super::Server;
use super::connection_cache::CachedStatus;
use super::key_store::KeyStore;
use crate::net::EncryptionError;
use crate::plugin::server::server_broadcast::ServerBroadcastEvent;
use pumpkin_macros::send_cancellable;
use pumpkin_protocol::java::client::login::CEncryptionRequest;
use pumpkin_protocol::java::client::play::{CChangeDifficulty, CTabList};
use pumpkin_protocol::{ClientPacket, java::client::config::CPluginMessage};
use pumpkin_util::Difficulty;
use pumpkin_util::text::TextComponent;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use tokio::sync::Mutex;

impl Server {
    /// Broadcasts a packet to all players in all worlds.
    ///
    /// This function sends the specified packet to every connected player in every world managed by the server.
    ///
    /// # Arguments
    ///
    /// * `packet`: A reference to the packet to be broadcast. The packet must implement the `ClientPacket` trait.
    pub fn broadcast_packet_all<P: ClientPacket>(&self, packet: &P) {
        for world in self.worlds.load().iter() {
            world.broadcast_packet_all(packet);
        }
    }

    pub async fn broadcast_tab_list_header_footer(
        &self,
        header: &TextComponent,
        footer: &TextComponent,
    ) {
        let packet = CTabList::new(header, footer);
        for world in self.worlds.load().iter() {
            for player in world.players.load().iter() {
                *player.tab_list_header.lock().await = header.clone();
                *player.tab_list_footer.lock().await = footer.clone();
                player.client.enqueue_packet(&packet).await;
            }
        }
    }

    pub async fn broadcast_message(
        &self,
        message: &TextComponent,
        sender_name: &TextComponent,
        chat_type: u8,
        target_name: Option<&TextComponent>,
    ) {
        send_cancellable! {{
            self;
            ServerBroadcastEvent::new(message.clone(), sender_name.clone());

            'after: {
                for world in self.worlds.load().iter() {
                    world
                        .broadcast_message(&event.message, &event.sender, chat_type, target_name)
                        .await;
                }
            }
        }}
    }

    /// Gets the current difficulty of the server.
    pub fn get_difficulty(&self) -> Difficulty {
        self.level_info.load().difficulty
    }

    /// Sets the difficulty of the server.
    ///
    /// This function updates the difficulty level of the server and broadcasts the change to all players.
    /// It also iterates through all worlds to ensure the difficulty is applied consistently.
    /// If `force_update` is `Some(true)`, the difficulty will be set regardless of the current state.
    /// If `force_update` is `Some(false)` or `None`, the difficulty will only be updated if it is not locked.
    ///
    /// # Arguments
    ///
    /// * `difficulty`: The new difficulty level to set. This should be one of the variants of the `Difficulty` enum.
    /// * `force_update`: An optional boolean that, if set to `Some(true)`, forces the difficulty to be updated even if it is currently locked.
    ///
    /// # Note
    ///
    /// This function does not handle the actual mob spawn options update, which is a TODO item for future implementation.
    pub async fn set_difficulty(&self, difficulty: Difficulty, force_update: bool) {
        let current_info = self.level_info.load();
        if current_info.difficulty_locked && !force_update {
            return;
        }

        let new_difficulty = if self.basic_config.hardcore {
            Difficulty::Hard
        } else {
            difficulty
        };

        let mut new_info = (**current_info).clone();

        new_info.difficulty = new_difficulty;
        let locked = new_info.difficulty_locked;
        self.level_info.store(Arc::new(new_info));

        for world in self.worlds.load().iter() {
            world.set_difficulty(difficulty);
            world
                .broadcast_editioned(
                    &CChangeDifficulty::new(difficulty as u8, locked),
                    &pumpkin_protocol::bedrock::client::CSetDifficulty::new(difficulty as u32),
                )
                .await;
        }
    }

    /// Generates a new container id.
    pub fn new_container_id(&self) -> u32 {
        self.container_id.fetch_add(1, Ordering::SeqCst)
    }

    /// Generates a new map id.
    pub fn next_map_id(&self) -> i32 {
        let id = self.map_id.fetch_add(1, Ordering::SeqCst);
        self.level_info.rcu(|level_info| {
            let mut new_level_info = (**level_info).clone();
            new_level_info.map_id = self.map_id.load(Ordering::SeqCst);
            new_level_info
        });
        id
    }

    pub fn get_branding(&self) -> CPluginMessage<'_> {
        self.branding.get_branding()
    }

    pub const fn get_status(&self) -> &Mutex<CachedStatus> {
        &self.listing
    }

    pub async fn encryption_request<'a>(
        &'a self,
        verification_token: &'a [u8; 4],
        should_authenticate: bool,
    ) -> CEncryptionRequest<'a> {
        self.key_store
            .get_or_init(|| async { Arc::new(KeyStore::new()) })
            .await
            .encryption_request("", verification_token, should_authenticate)
    }

    pub async fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        self.key_store
            .get_or_init(|| async { Arc::new(KeyStore::new()) })
            .await
            .decrypt(data)
    }

    pub async fn digest_secret(&self, secret: &[u8]) -> String {
        self.key_store
            .get_or_init(|| async { Arc::new(KeyStore::new()) })
            .await
            .get_digest(secret)
    }
}
