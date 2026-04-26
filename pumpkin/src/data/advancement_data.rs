use crate::entity::player::advancement::PlayerAdvancement;
use pumpkin_data::Advancement;
use std::path::PathBuf;
use std::sync::Arc;
use pumpkin_world::data::player_data::PlayerDataError;
use crate::entity::player::Player;
use crate::server::Server;

pub struct AdvancementManager {
    advancement_path: PathBuf,
}

impl AdvancementManager {
    pub fn new(player_data_path: PathBuf) -> Self {
        AdvancementManager {
            advancement_path: player_data_path.join("advancements"),
        }
    }

    pub fn get_advancements(&self) -> Vec<&'static str> {
        Advancement::get_list().to_vec()
    }

    pub fn get_advancement_path(&self) -> PathBuf {
        self.advancement_path.clone()
    }

    pub fn new_advancement(&self) -> PlayerAdvancement {
        PlayerAdvancement::new(true, self.get_advancement_path())
    }

    pub async fn save_all_players(players : Vec<Arc<Player>>) -> Result<(), PlayerDataError> {
        for player in players {
            player.advancements.lock().await.save().await?;
        }
        Ok(())
    }

    pub async fn save_player(player : Arc<Player>) -> Result<(), PlayerDataError> {
        player.advancements.lock().await.save().await?;
        Ok(())
    }
}
