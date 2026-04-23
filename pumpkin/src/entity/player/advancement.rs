use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs::create_dir_all;
use std::path::PathBuf;
use std::sync::{Arc, Weak};
use futures::SinkExt;
use indexmap::IndexMap;
use rayon::broadcast;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::ser::SerializeMap;
use tracing::error;
use wasmparser::collections::Set;
use pumpkin_data::Advancement;
use pumpkin_data::advancement_data::AdvancementReward;
use pumpkin_util::identifier::Identifier;
use pumpkin_util::text::TextComponent;
use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::server::Server;

#[derive(Debug, Clone,Copy, Serialize, Deserialize,Default)]
pub struct AdvancementProgress {
    pub complete: bool,
}

impl AdvancementProgress {

    pub fn is_done(&self) -> bool {
        self.complete
    }

    pub fn has_progress(&self) -> bool {
        self.complete
    }
}

pub struct PlayerAdvancement {
    advancements: IndexMap<&'static Advancement, AdvancementProgress>,
    save_enabled: bool,
    is_first_packet: bool,
    to_update : HashSet<&'static Advancement>,
    advancement_path: PathBuf,
    pub player: Weak<Player>,
}

pub enum AdvancementDataError {
    Io(std::io::Error),
    Json(serde_json::Error),
}

impl PlayerAdvancement {

    const PATH: &'static str = "advancement";

    pub(crate) fn new(save_enabled:bool,advancement_path:impl Into<PathBuf>) -> Self {
        let path = advancement_path.into();
        if !path.exists()
            && let Err(e) = create_dir_all(&path)
        {
            error!(
                "Failed to create player data directory at {}: {e}",
                path.display()
            );
        }
        PlayerAdvancement {
            advancements:IndexMap::new(),
            save_enabled,
            advancement_path: path,
            player: Weak::new(),
            is_first_packet : true,
            to_update : Default::default(),
        }
    }

    pub fn set_player(&mut self, player: Arc<Player>) {
        self.player = Arc::downgrade(&player);
    }

    #[must_use]
    pub const fn is_save_enabled(&self) -> bool {
        self.save_enabled
    }

    pub fn save(&self) -> Result<(), AdvancementDataError>{
        if !self.is_save_enabled() {
            return Ok(());
        }
        if let Some(parent) = &self.advancement_path.parent()
            && let Err(e) = create_dir_all(parent){
            error!("Failed to create player advancement directory for {}: {e}",
                self.advancement_path.file_prefix().to_string());
            return Err(AdvancementDataError::Io(e));
        }
        let file = std::fs::File::create(&self.advancement_path)
            .map_err(AdvancementDataError::Io)?;

        serde_json::to_writer_pretty(file, &self)
            .map_err(AdvancementDataError::Json)?;
        Ok(())
    }

    pub fn load(&mut self) -> Result<(), AdvancementDataError> {
        if !self.advancement_path.exists() {
            return Ok(()); // Fichier inexistant, on garde la map vide
        }

        let file = std::fs::File::open(&self.advancement_path)
            .map_err(AdvancementDataError::Io)?;

        let loaded_data: HashMap<String, AdvancementProgress> =
            serde_json::from_reader(file).map_err(AdvancementDataError::Json)?;

        self.advancements.clear();
        for (advancement_id, progress) in loaded_data {
            if let Some(advancement_ref) = Advancement::from_name(&advancement_id) {
                self.advancements.insert(advancement_ref, progress);
            } else {
                tracing::warn!("The Advancement {} is invalid", advancement_id);
            }
        }
        Ok(())
    }

    pub fn flush_dirty(&mut self, flush: bool) {
        if self.is_first_packet || !self.to_update.is_empty() {
            todo!("send advancement tree with the complete ones");
        }
        self.is_first_packet = false;
    }

    fn start_progress(&mut self,advancement : &'static Advancement, advancement_progress : AdvancementProgress){
        self.advancements.insert(advancement, advancement_progress);
    }

    pub fn get_or_start_progress(&mut self,advancement:&'static Advancement) -> &AdvancementProgress{
        self.advancements.entry(advancement).or_insert_with(AdvancementProgress::default)
    }

    pub fn get_mut_or_start_progress(&mut self,advancement:&'static Advancement) -> &mut AdvancementProgress{
        self.advancements.get_mut(advancement).map(|progress| progress).unwrap_or_else(|| {
            self.start_progress(advancement, AdvancementProgress::default());
            self.advancements.get_mut(advancement).map(|progress| progress).unwrap()
        })
    }

    pub async fn grant_reward(player:Arc<Player>,reward:&AdvancementReward){
        player.add_experience_points(reward.experience).await;
    }

    pub async fn award(&mut self,advancement:&'static Advancement){
        //TODO call and creates Events
        let player = self.player.upgrade().unwrap().clone();
        let mut progress = self.get_mut_or_start_progress(advancement);
        let is_done = progress.is_done();
        if !progress.is_done() {
            progress.complete = true;
            Self::grant_reward(player.clone(),advancement.reward).await;
            if let Some(display) = advancement.display && display.announce_to_chat {
                let component = TextComponent::translate(
                    format!("chat.type.advancement.{}", display.frame_type.get_name()),
                    [player.get_display_name().await, advancement.name()]);
                player.world().broadcast_system_message(&component,false).await;
            }
        }
        if !is_done && progress.is_done() {
           //TODO update to_update with the advancement
        }

    }

    pub fn revoke(&mut self,advancement:&'static Advancement){
        let progress = self.get_mut_or_start_progress(advancement);
        if progress.is_done() {
            progress.complete = false;
        }
    }
}

impl Serialize for PlayerAdvancement {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        let mut map = serializer.serialize_map(Some(self.advancements.len()))?;

        for (advancement, progress) in &self.advancements {
            map.serialize_entry(advancement.id, progress)?;
        }
        map.end()
    }
}