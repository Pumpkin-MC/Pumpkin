use crate::data::advancement_data::AdvancementManager;
use crate::entity::EntityBase;
use crate::entity::player::Player;
use indexmap::IndexMap;
use pumpkin_data::Advancement;
use pumpkin_data::advancement_data::AdvancementReward;
use pumpkin_util::text::TextComponent;
use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize, Serializer};
use std::collections::{HashMap, HashSet};
use std::fs::create_dir_all;
use std::path::PathBuf;
use std::sync::{Arc, Weak};
use tracing::error;
use uuid::Uuid;

/// Represents the progress of a given advancement for a player.
///
/// Tracks whether the advancement has been fully completed. In the future,
/// this will also track specific criteria progress.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct AdvancementProgress {
    /// Indicates if the advancement is fully completed.
    pub complete: bool,
}

impl AdvancementProgress {
    /// Returns `true` if the advancement is completely done.
    pub fn is_done(&self) -> bool {
        self.complete
    }

    /// Returns `true` if the advancement has any progress. Currently just returns if it is fully complete.
    pub fn has_progress(&self) -> bool {
        self.complete
    }
}

/// Manages a player's collection of advancements.
///
/// This handles saving, loading, and tracking the state of granted / revoked advancements.
pub struct PlayerAdvancement {
    advancements: IndexMap<&'static Advancement, AdvancementProgress>,
    is_first_packet: bool,
    to_update: HashSet<&'static Advancement>,
    manager: Arc<AdvancementManager>,
    path: PathBuf,
    owner: Uuid,
    /// A weak reference to the player who owns these advancements.
    pub player: Weak<Player>,
}

/// Errors that can occur when saving or loading advancement data.
#[derive(Debug, thiserror::Error)]
pub enum AdvancementDataError {
    #[error("IO error: {0}")]
    Io(std::io::Error),
    #[error("JSON error: {0}")]
    Json(serde_json::Error),
}

impl PlayerAdvancement {
    /// Creates a new instance of `PlayerAdvancement`.
    pub(crate) fn new(manager: Arc<AdvancementManager>, uuid: Uuid) -> Self {
        PlayerAdvancement {
            advancements: IndexMap::new(),
            path: manager.advancement_path.join(format!("{}.json", &uuid)),
            manager,
            player: Weak::new(),
            is_first_packet: true,
            owner: uuid,
            to_update: Default::default(),
        }
    }

    /// Associates the `PlayerAdvancement` data with the given player.
    pub fn set_player(&mut self, player: Arc<Player>) {
        self.player = Arc::downgrade(&player);
    }

    /// Returns whether advancement saving is enabled for this player.
    #[must_use]
    pub fn is_save_enabled(&self) -> bool {
        self.manager.save_enabled
    }

    /// Saves the player's advancement progress to disk as JSON.
    pub fn save(&self) -> Result<(), AdvancementDataError> {
        if !self.is_save_enabled() {
            return Ok(());
        }

        if let Some(parent) = &self.path.parent()
            && let Err(e) = create_dir_all(parent)
        {
            let file_name = self
                .path
                .file_prefix()
                .and_then(|prefix| prefix.to_str())
                .unwrap_or("unknown");
            error!(
                "Failed to create player advancement directory for {}: {e}",
                file_name
            );
            return Err(AdvancementDataError::Io(e));
        }
        let file = std::fs::File::create(&self.path).map_err(AdvancementDataError::Io)?;

        serde_json::to_writer_pretty(file, &self).map_err(AdvancementDataError::Json)?;
        Ok(())
    }

    /// Loads the player's advancement progress from disk.
    pub fn load(&mut self) -> Result<(), AdvancementDataError> {
        if !self.path.exists() {
            return Ok(());
        }

        let file = std::fs::File::open(&self.path).map_err(AdvancementDataError::Io)?;

        let loaded_data: HashMap<String, AdvancementProgress> =
            serde_json::from_reader(file).map_err(AdvancementDataError::Json)?;

        self.advancements.clear();
        for (advancement_id, progress) in loaded_data {
            if let Some(advancement_ref) = Advancement::from_name(&advancement_id) {
                self.advancements.insert(advancement_ref, progress);
            } else {
                tracing::warn!("The Advancement name {} is invalid", advancement_id);
            }
        }
        Ok(())
    }

    /// Flushes any pending (dirty) advancement state down to the client.
    pub fn flush_dirty(&mut self, _flag: bool) {
        if self.is_first_packet || !self.to_update.is_empty() {
            todo!("send advancement tree with the complete ones");
        }
        self.is_first_packet = false;
    }

    /// Begins progress on a specific advancement.
    fn start_progress(
        &mut self,
        _advancement: &'static Advancement,
        advancement_progress: AdvancementProgress,
    ) -> AdvancementProgress {
        advancement_progress
    }

    /// Gets the current progress for a given advancement, creating a default uncompleted progress if it doesn't exist.
    pub fn get_or_start_progress(
        &mut self,
        advancement: &'static Advancement,
    ) -> &AdvancementProgress {
        self.get_mut_or_start_progress(advancement)
    }

    /// Gets a mutable reference to the current progress for a given advancement. Creates the state entry if missing.
    pub fn get_mut_or_start_progress(
        &mut self,
        advancement: &'static Advancement,
    ) -> &mut AdvancementProgress {
        self.advancements
            .entry(advancement)
            .or_insert_with(AdvancementProgress::default)
    }

    /// Grants the rewards (like experience) associated with completing an advancement.
    pub async fn grant_reward(player: Arc<Player>, reward: &AdvancementReward) {
        player.add_experience_points(reward.experience).await;
    }

    /// Fully awards an advancement to the player, updating its status to complete and granting rewards if applicable.
    pub async fn award(&mut self, advancement: &'static Advancement) {
        //TODO call and creates Events for plugins
        let player = self.player.upgrade().unwrap().clone();
        let progress = self.get_mut_or_start_progress(advancement);
        let is_done = progress.is_done();
        if !progress.is_done() {
            progress.complete = true;
            Self::grant_reward(player.clone(), advancement.reward).await;
            if let Some(display) = advancement.display
                && display.announce_to_chat
            {
                let component = TextComponent::translate(
                    format!("chat.type.advancement.{}", display.frame_type.get_name()),
                    [player.get_display_name().await, advancement.name()],
                );
                player
                    .world()
                    .broadcast_system_message(&component, false)
                    .await;
            }
        }
        if !is_done && progress.is_done() {
            //TODO update to_update with the advancement
        }
    }

    /// Revokes a previously awarded advancement, clearing its progress state.
    pub fn revoke(&mut self, advancement: &'static Advancement) {
        let progress = self.get_mut_or_start_progress(advancement);
        if progress.is_done() {
            progress.complete = false;
        }
    }
}

impl Serialize for PlayerAdvancement {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.advancements.len()))?;

        for (advancement, progress) in &self.advancements {
            map.serialize_entry(advancement.id, progress)?;
        }
        map.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::advancement_data::AdvancementManager;
    use pumpkin_data::Advancement;
    use tempfile::tempdir;

    #[test]
    fn test_advancement_progress() {
        let progress = AdvancementProgress { complete: false };
        assert!(!progress.is_done());
        assert!(!progress.has_progress());

        let complete_progress = AdvancementProgress { complete: true };
        assert!(complete_progress.is_done());
        assert!(complete_progress.has_progress());
    }

    #[test]
    fn test_new_player_advancement() {
        let temp_dir = tempdir().unwrap();
        let manager = Arc::new(AdvancementManager::new(temp_dir.path(), true));
        let id = Uuid::new_v4();
        let pa = PlayerAdvancement::new(manager, id);
        assert!(pa.is_save_enabled());
        assert!(pa.is_first_packet);
        assert!(pa.to_update.is_empty());
        assert!(pa.advancements.is_empty());
    }

    #[test]
    fn test_get_or_start_progress() {
        let temp_dir = tempdir().unwrap();
        let manager = Arc::new(AdvancementManager::new(temp_dir.path(), true));
        let id = Uuid::new_v4();
        let mut pa = PlayerAdvancement::new(manager, id);
        let adv = Advancement::STORY_ROOT;
        let progress = pa.get_or_start_progress(adv);
        assert!(
            !progress.is_done(),
            "New progress should not be marked done by default"
        );
    }

    #[test]
    fn test_revoke_advancement() {
        let temp_dir = tempdir().unwrap();
        let manager = Arc::new(AdvancementManager::new(temp_dir.path(), true));
        let id = Uuid::new_v4();
        let mut pa = PlayerAdvancement::new(manager, id);
        let adv = Advancement::STORY_ROOT;
        {
            let progress_mut = pa.get_mut_or_start_progress(adv);
            progress_mut.complete = true;
        }
        assert!(pa.get_or_start_progress(adv).is_done());
        pa.revoke(adv);
        assert!(!pa.get_or_start_progress(adv).is_done());
    }

    #[test]
    fn test_save_advancement_progress() {
        let temp_dir = tempdir().unwrap();
        let manager = Arc::new(AdvancementManager::new(temp_dir.path(), true));
        let id = Uuid::new_v4();
        let mut pa = PlayerAdvancement::new(manager, id);

        // Add some advancement progress
        let adv = Advancement::STORY_ROOT;
        {
            let progress_mut = pa.get_mut_or_start_progress(adv);
            progress_mut.complete = true;
        }

        // Save should succeed
        assert!(pa.save().is_ok(), "Save should succeed");

        // File should exist
        assert!(pa.path.exists(), "Saved file should exist");

        // Content should be valid JSON
        let content = std::fs::read_to_string(&pa.path).unwrap();
        assert!(!content.is_empty(), "Saved file should not be empty");
        let _: HashMap<String, AdvancementProgress> =
            serde_json::from_str(&content).expect("Saved content should be valid JSON");
    }

    #[test]
    fn test_save_disabled() {
        let temp_dir = tempdir().unwrap();
        let manager = Arc::new(AdvancementManager::new(temp_dir.path(), false));
        let id = Uuid::new_v4();
        let mut pa = PlayerAdvancement::new(manager, id);

        // Add some advancement progress
        let adv = Advancement::STORY_ROOT;
        {
            let progress_mut = pa.get_mut_or_start_progress(adv);
            progress_mut.complete = true;
        }

        // Save should return Ok but not actually save
        assert!(
            pa.save().is_ok(),
            "Save with disabled saving should return Ok"
        );
        assert!(
            !pa.path.exists(),
            "File should not be created when saving is disabled"
        );
    }

    #[test]
    fn test_load_nonexistent_file() {
        let temp_dir = tempdir().unwrap();
        let manager = Arc::new(AdvancementManager::new(temp_dir.path(), true));
        let id = Uuid::new_v4();
        let mut pa = PlayerAdvancement::new(manager, id);

        // Load from nonexistent file should return Ok (not error)
        assert!(
            pa.load().is_ok(),
            "Loading from nonexistent file should return Ok"
        );
        assert!(
            pa.advancements.is_empty(),
            "Advancements should remain empty"
        );
    }

    #[test]
    fn test_load_advancement_progress() {
        let temp_dir = tempdir().unwrap();
        let manager = Arc::new(AdvancementManager::new(temp_dir.path(), true));

        let id = Uuid::new_v4();
        let mut pa = PlayerAdvancement::new(manager, id);
        // Create a JSON file with advancement data
        let adv = Advancement::STORY_ROOT;
        let data = serde_json::json!({ adv.id: { "complete": true } });
        std::fs::write(&pa.path, data.to_string()).unwrap();

        // Load the file
        assert!(pa.load().is_ok(), "Load should succeed");

        // Verify the advancement was loaded
        let progress = pa.get_or_start_progress(adv);
        assert!(
            progress.is_done(),
            "Loaded advancement should be marked complete"
        );
    }

    #[test]
    fn test_save_load_roundtrip() {
        let temp_dir = tempdir().unwrap();

        // Create and save advancements
        let manager = Arc::new(AdvancementManager::new(temp_dir.path(), true));
        let id = Uuid::new_v4();
        let mut pa = PlayerAdvancement::new(manager.clone(), id);

        let adv = Advancement::STORY_ROOT;
        {
            let progress_mut = pa.get_mut_or_start_progress(adv);
            progress_mut.complete = true;
        }

        assert!(pa.save().is_ok(), "Save should succeed");

        // Load the saved advancements into a new instance
        let mut pa_loaded = PlayerAdvancement::new(manager, id);
        assert!(pa_loaded.load().is_ok(), "Load should succeed");

        // Verify the loaded data matches the saved data
        let loaded_progress = pa_loaded.get_or_start_progress(adv);
        assert!(
            loaded_progress.is_done(),
            "Loaded progress should match saved progress"
        );
        assert_eq!(
            pa_loaded.advancements.len(),
            pa.advancements.len(),
            "Loaded advancements count should match"
        );
    }

    #[test]
    fn test_load_invalid_advancement_id() {
        let temp_dir = tempdir().unwrap();
        let manager = Arc::new(AdvancementManager::new(temp_dir.path(), true));

        // Create a JSON file with invalid advancement ID
        let data = serde_json::json!({
            "invalid_advancement_id_12345": { "complete": true }
        });
        let id = Uuid::new_v4();
        let mut pa = PlayerAdvancement::new(manager, id);
        std::fs::write(&pa.path, data.to_string()).unwrap();

        // Load should still succeed but skip the invalid entry

        assert!(
            pa.load().is_ok(),
            "Load should succeed even with invalid IDs"
        );
        assert!(
            pa.advancements.is_empty(),
            "Invalid advancements should be skipped"
        );
    }

    #[test]
    fn test_save_multiple_advancements() {
        let temp_dir = tempdir().unwrap();
        let manager = Arc::new(AdvancementManager::new(temp_dir.path(), true));
        let id = Uuid::new_v4();
        let mut pa = PlayerAdvancement::new(manager, id);

        // Add multiple advancements
        let adv1 = Advancement::STORY_ROOT;
        let adv2 = Advancement::NETHER_ROOT;

        {
            let progress_mut1 = pa.get_mut_or_start_progress(adv1);
            progress_mut1.complete = true;
        }
        {
            let progress_mut2 = pa.get_mut_or_start_progress(adv2);
            progress_mut2.complete = false;
        }

        assert!(pa.save().is_ok(), "Save should succeed");

        // Verify both were saved
        let content = std::fs::read_to_string(&pa.path).unwrap();
        let saved_data: HashMap<String, AdvancementProgress> =
            serde_json::from_str(&content).unwrap();
        assert_eq!(saved_data.len(), 2, "Should have saved both advancements");
    }
}
