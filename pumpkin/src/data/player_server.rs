use crate::{
    entity::{NBTStorage, player::Player},
    server::Server,
};
use crossbeam::atomic::AtomicCell;
use pumpkin_inventory::screen_handler::ScreenHandler;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_storage::StorageError;
use pumpkin_storage::player_data::PlayerDataStorage;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, error};

/// Helper for managing player data in the server context.
///
/// Thin wrapper around an [`Arc<dyn PlayerDataStorage>`] with bookkeeping for
/// periodic saves across all online players. When persistence is disabled the
/// backing storage is a `NullStorage`; when enabled it is whatever backend the
/// server was wired with (currently `VanillaStorage`).
pub struct ServerPlayerData {
    storage: Arc<dyn PlayerDataStorage>,
    save_interval: Duration,
    last_save: AtomicCell<Instant>,
}

impl ServerPlayerData {
    pub fn new(storage: Arc<dyn PlayerDataStorage>, save_interval: Duration) -> Self {
        Self {
            storage,
            save_interval,
            last_save: AtomicCell::new(Instant::now()),
        }
    }

    /// Handles a player leaving the server.
    ///
    /// Closes the player's screen handler and saves their latest NBT.
    pub async fn handle_player_leave(&self, player: &Player) -> Result<(), StorageError> {
        player
            .player_screen_handler
            .lock()
            .await
            .on_closed(player)
            .await;
        player.on_handled_screen_closed().await;

        let mut nbt = NbtCompound::new();
        player.write_nbt(&mut nbt).await;

        self.storage.save(player.gameprofile.id, &nbt).await
    }

    /// Performs periodic maintenance tasks — currently just periodic
    /// snapshot saves of every online player.
    pub async fn tick(&self, server: &Server) -> Result<(), StorageError> {
        let now = Instant::now();
        let last_save = self.last_save.load();
        let should_save = now.duration_since(last_save) >= self.save_interval;

        if should_save {
            self.last_save.store(now);
            for world in server.worlds.load().iter() {
                for player in world.players.load().iter() {
                    let mut nbt = NbtCompound::new();
                    player.write_nbt(&mut nbt).await;

                    if let Err(e) = self.storage.save(player.gameprofile.id, &nbt).await {
                        error!(
                            "Failed to save player data for {}: {e}",
                            player.gameprofile.id,
                        );
                    }
                }
            }

            debug!("Periodic player data save completed");
        }

        Ok(())
    }

    /// Saves every online player's data immediately. Used during shutdown.
    pub async fn save_all_players(&self, server: &Server) -> Result<(), StorageError> {
        let mut total_players = 0;
        for world in server.worlds.load().iter() {
            for player in world.players.load().iter() {
                self.extract_data_and_save_player(player).await?;
                total_players += 1;
            }
        }
        debug!("Saved data for {total_players} online players");
        Ok(())
    }

    /// Loads player data, returning `Ok(None)` when none is stored (fresh
    /// player). Other errors are logged and treated as missing data so a
    /// corrupt file cannot block login.
    pub async fn load_data(
        &self,
        uuid: uuid::Uuid,
    ) -> Result<Option<NbtCompound>, StorageError> {
        match self.storage.load(uuid).await {
            Ok(nbt) => Ok(Some(nbt)),
            Err(e) if e.is_not_found() => Ok(None),
            Err(e) => {
                error!("Error loading player data for {uuid}: {e}");
                Ok(None)
            }
        }
    }

    /// Builds and saves NBT for the given player.
    pub async fn extract_data_and_save_player(
        &self,
        player: &Player,
    ) -> Result<(), StorageError> {
        let mut nbt = NbtCompound::new();
        player.write_nbt(&mut nbt).await;
        self.storage.save(player.gameprofile.id, &nbt).await
    }
}
