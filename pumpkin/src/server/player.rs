use super::Server;
use crate::entity::NBTStorage;
use crate::entity::player::Player;
use crate::net::{ClientPlatform, DisconnectReason, GameProfile, PlayerConfig};
use crate::plugin::player::player_login::PlayerLoginEvent;
use crate::world::World;
use pumpkin_data::dimension::Dimension;
use pumpkin_macros::send_cancellable;
use pumpkin_util::text::TextComponent;
use rand::seq::IndexedRandom;
use std::net::IpAddr;
use std::sync::Arc;
use tracing::warn;

impl Server {
    /// Adds a new player to the server.
    ///
    /// This function takes an `Arc<Client>` representing the connected client and performs the following actions:
    ///
    /// 1. Generates a new entity ID for the player.
    /// 2. Determines the player's gamemode (defaulting to Survival if not specified in configuration).
    /// 3. **(TODO: Select default from config)** Selects the world for the player (currently uses the first world).
    /// 4. Creates a new `Player` instance using the provided information.
    /// 5. Adds the player to the chosen world.
    /// 6. **(TODO: Config if we want increase online)** Optionally updates server listing information based on the player's configuration.
    ///
    /// # Arguments
    ///
    /// * `client`: An `Arc<Client>` representing the connected client.
    ///
    /// # Returns
    ///
    /// A tuple containing:
    ///
    /// - `Arc<Player>`: A reference to the newly created player object.
    /// - `Arc<World>`: A reference to the world the player was added to.
    ///
    /// # Note
    ///
    /// You still have to spawn the `Player` in a `World` to let them join and make them visible.
    pub async fn add_player(
        &self,
        client: Arc<ClientPlatform>,
        profile: GameProfile,
        config: Option<PlayerConfig>,
    ) -> Option<(Arc<Player>, Arc<World>)> {
        let gamemode = self.defaultgamemode.lock().await.gamemode;

        let (world, nbt) =
            if let Ok(Some(data)) = self.player_data_storage.load_data(&profile.id).await {
                if let Some(dimension_key) = data.get_string("Dimension") {
                    if let Some(dimension) = Dimension::from_name(dimension_key) {
                        let world = self.get_world_from_dimension(dimension);
                        (world, Some(data))
                    } else {
                        warn!("Invalid dimension key in player data: {dimension_key}");
                        let default_world = self
                            .worlds
                            .load()
                            .first()
                            .expect("Default world should exist")
                            .clone();
                        (default_world, Some(data))
                    }
                } else {
                    // Player data exists but doesn't have a "Dimension" key.
                    let default_world = self
                        .worlds
                        .load()
                        .first()
                        .expect("Default world should exist")
                        .clone();
                    (default_world, Some(data))
                }
            } else {
                // No player data found or an error occurred, default to the Overworld.
                let default_world = self
                    .worlds
                    .load()
                    .first()
                    .expect("Default world should exist")
                    .clone();
                (default_world, None)
            };

        let mut player = Player::new(
            client,
            profile,
            config.clone().unwrap_or_default(),
            world.clone(),
            gamemode,
        )
        .await;

        if let Some(mut nbt_data) = nbt {
            player.read_nbt(&mut nbt_data).await;
        }

        // Wrap in Arc after data is loaded
        let player = Arc::new(player);
        {
            let mut advancements = player.advancements.lock().await;
            if let Err(e) = advancements.load().await {
                warn!("Error loading player {}: {e}", player.gameprofile.id);
            }
            advancements.player = Arc::downgrade(&player);
        };

        send_cancellable! {{
            self;
            PlayerLoginEvent::new(player.clone(), TextComponent::text("You have been kicked from the server"));
            'after: {
                player.screen_handler_sync_handler.store_player(player.clone()).await;
                if world
                    .add_player(&player)
                    .is_ok() {
                    let mut user_cache = self.data.user_cache.write().await;
                    user_cache.upsert(player.gameprofile.id, player.gameprofile.name.clone());

                    // TODO: Config if we want increase online
                    if let Some(config) = config {
                        // TODO: Config so we can also just ignore this hehe
                        if config.server_listing {
                            self.listing.lock().await.add_player(&player);
                        }
                    }

                    Some((player, world.clone()))
                } else {
                    None
                }
            }

            'cancelled: {
                player.kick(DisconnectReason::Kicked, event.kick_message).await;
                None
            }
        }}
    }

    pub async fn remove_player(&self, player: &Player) {
        player
            .increment_stat(
                pumpkin_data::statistic::StatisticCategory::Custom,
                pumpkin_data::statistic::CustomStatistic::LeaveGame as i32,
                1,
            )
            .await;
        // TODO: Config if we want decrease online
        self.listing.lock().await.remove_player(player);
    }

    /// Searches for a player by their username across all worlds.
    ///
    /// This function iterates through each world managed by the server and attempts to find a player with the specified username.
    /// If a player is found in any world, it returns an `Arc<Player>` reference to that player. Otherwise, it returns `None`.
    ///
    /// # Arguments
    ///
    /// * `name`: The username of the player to search for.
    ///
    /// # Returns
    ///
    /// An `Option<Arc<Player>>` containing the player if found, or `None` if not found.
    pub fn get_player_by_name(&self, name: &str) -> Option<Arc<Player>> {
        for world in self.worlds.load().iter() {
            if let Some(player) = world.get_player_by_name(name) {
                return Some(player);
            }
        }
        None
    }

    pub async fn get_players_by_ip(&self, ip: IpAddr) -> Vec<Arc<Player>> {
        let mut players = Vec::<Arc<Player>>::new();

        for world in self.worlds.load().iter() {
            for player in world.players.load().iter() {
                if player.client.address().await.ip() == ip {
                    players.push(player.clone());
                }
            }
        }

        players
    }

    /// Returns all players from all worlds.
    pub fn get_all_players(&self) -> Vec<Arc<Player>> {
        let mut players = Vec::<Arc<Player>>::new();

        for world in self.worlds.load().iter() {
            players.extend(world.players.load().iter().cloned());
        }

        players
    }

    pub fn for_each_player<F>(&self, mut f: F)
    where
        F: FnMut(&Arc<Player>),
    {
        let worlds = self.worlds.load();

        for world in worlds.iter() {
            let players = world.players.load();
            for player in players.iter() {
                f(player);
            }
        }
    }

    /// Returns a random player from any of the worlds, or `None` if all worlds are empty.
    pub fn get_random_player(&self) -> Option<Arc<Player>> {
        let players = self.get_all_players();
        players.choose(&mut rand::rng()).map(Arc::<_>::clone)
    }

    /// Searches for a player by their UUID across all worlds.
    ///
    /// This function iterates through each world managed by the server and attempts to find a player with the specified UUID.
    /// If a player is found in any world, it returns an `Arc<Player>` reference to that player. Otherwise, it returns `None`.
    ///
    /// # Arguments
    ///
    /// * `id`: The UUID of the player to search for.
    ///
    /// # Returns
    ///
    /// An `Option<Arc<Player>>` containing the player if found, or `None` if not found.
    pub fn get_player_by_uuid(&self, id: uuid::Uuid) -> Option<Arc<Player>> {
        for world in self.worlds.load().iter() {
            if let Some(player) = world.get_player_by_uuid(id) {
                return Some(player);
            }
        }
        None
    }

    /// Counts the total number of players across all worlds.
    ///
    /// This function iterates through each world and sums up the number of players currently connected to that world.
    ///
    /// # Returns
    ///
    /// The total number of players connected to the server.
    pub fn get_player_count(&self) -> usize {
        let mut count = 0;
        for world in self.worlds.load().iter() {
            count += world.players.load().len();
        }
        count
    }

    /// Similar to [`Server::get_player_count`] >= n, but may be more efficient since it stops its iteration through all worlds as soon as n players were found.
    pub fn has_n_players(&self, n: usize) -> bool {
        let mut count = 0;
        for world in self.worlds.load().iter() {
            count += world.players.load().len();
            if count >= n {
                return true;
            }
        }
        false
    }
}
