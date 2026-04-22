use std::fs::create_dir_all;
use std::path::PathBuf;
use std::sync::{Arc, Weak};
use futures::SinkExt;
use indexmap::IndexMap;
use rayon::broadcast;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tracing::error;
use pumpkin_data::Advancement;
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
    advancement_path: PathBuf,
    pub player: Weak<Player>,
    server : Weak<Server>
}

pub enum AdvancementDataError {
    Io(std::io::Error),
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
            server: Weak::new()
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
                self.advancement_path.file_prefix());
            return Err(AdvancementDataError::Io(e));
        }

    }

    pub fn load(&self){

    }
    pub fn flush_dirty(&self, flush: bool) {
        todo!()
    }

    fn start_progress(&mut self,advancement : &Advancement, advancement_progress : AdvancementProgress){
        self.advancements.insert(advancement, advancement_progress);
    }

    pub fn get_or_start_progress(&mut self,advancement:&Advancement) -> AdvancementProgress{
        self.advancements.get(advancement).map(|progress| *progress).unwrap_or_else(|| {
            let progress = AdvancementProgress::default();
            self.start_progress(advancement, progress.clone());
            progress
        })

    }

    pub async fn award(&mut self,advancement:&Advancement){
        //TODO call and creates Events
        let progress = self.get_or_start_progress(advancement);
        if !progress.is_done() {
            let player = self.player.upgrade().unwrap().clone();
            advancement.reward.grant(player.clone());
            if let Some(display) = advancement.display && display.announce_to_chat {
                let component = TextComponent::translate(
                    format!("chat.type.advancement.{}", display.frame_type.get_name()),
                    [player.get_display_name().await, advancement.name()]);
                player.world().broadcast_system_message(component,false).await;
            }
        }
    }

    pub fn revoke(&self,advancement:&Advancement){}
}

impl Serialize for PlayerAdvancement {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        todo!()
    }
}

impl Deserialize for PlayerAdvancement {
    fn deserialize<'de,D>(deserializer:  D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        todo!()
    }
}