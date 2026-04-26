use crate::entity::player::advancement::PlayerAdvancement;
use pumpkin_data::Advancement;
use std::path::PathBuf;

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
}
