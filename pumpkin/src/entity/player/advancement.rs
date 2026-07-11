mod visibility_evaluator;

use crate::data::advancement_data::AdvancementManager;
use crate::entity::EntityBase;
use crate::entity::player::Player;
use indexmap::IndexMap;
use pumpkin_data::advancement_data::{
    AdvancementNode, AdvancementProgressData, AdvancementRequirement, AdvancementReward, Criteria,
};
use pumpkin_data::{ADVANCEMENT_TREE, Advancement, translation};
use pumpkin_protocol::java::client::play::{CSelectAdvancementsTab, CUpdateAdvancements};
use pumpkin_util::identifier::Identifier;
use pumpkin_util::text::TextComponent;
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::to_string_pretty;
use std::collections::{HashMap, HashSet};
use std::fs::{create_dir_all, read, write};
use std::path::PathBuf;
use std::str::from_utf8;
use std::sync::{Arc, Weak};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::task::spawn_blocking;
use tracing::{error, warn};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct CriterionProgress(pub Option<SystemTime>);

impl CriterionProgress {
    pub fn grant(&mut self) {
        self.0 = Some(SystemTime::now());
    }

    pub const fn revoke(&mut self) {
        self.0 = None;
    }

    #[must_use]
    pub const fn is_done(&self) -> bool {
        self.0.is_some()
    }
}

/// Represents the progress of a given advancement for a player.
///
/// Tracks whether the advancement has been fully completed. In the future,
/// this will also track specific criteria progress.
#[derive(Debug, Clone, Default)]
pub struct AdvancementProgress {
    /// Indicates the different progress of all criteria currently only a boolean
    pub criteria: HashMap<Arc<str>, CriterionProgress>,
    /// The Requirement for the Advancement to be mark as complete
    pub requirements: AdvancementRequirement,
}

impl AdvancementProgress {
    /// Returns `true` if the advancement is done.
    #[must_use]
    pub fn is_done(&self) -> bool {
        self.requirements.test(|s| self.is_criterion_done(s))
    }

    /// Check if a criterion his mark has complete
    fn is_criterion_done(&self, criterion: &str) -> bool {
        self.criteria
            .get(criterion)
            .is_some_and(CriterionProgress::is_done)
    }

    /// Returns `true` if the advancement has any progress. Currently just returns if it is fully complete.
    #[must_use]
    pub fn has_progress(&self) -> bool {
        for value in self.criteria.values() {
            if value.is_done() {
                return true;
            }
        }
        false
    }

    pub fn grant_progress(&mut self, name: &str) -> bool {
        if let Some(value) = self.criteria.get_mut(name)
            && !value.is_done()
        {
            value.grant();
            true
        } else {
            false
        }
    }

    pub fn revoke_progress(&mut self, name: &str) -> bool {
        if let Some(value) = self.criteria.get_mut(name)
            && value.is_done()
        {
            value.revoke();
            true
        } else {
            false
        }
    }

    pub fn update(&mut self, requirements: AdvancementRequirement) {
        let names = requirements.names();
        self.criteria.retain(|key, _criterion| names.contains(key));
        for name in names {
            self.criteria.entry(name).or_default();
        }
        self.requirements = requirements;
    }

    #[inline]
    pub fn get_remaining_criteria(&self) -> impl Iterator<Item = Arc<str>> {
        self.criteria
            .iter()
            .filter(|&(_id, criterion)| !criterion.is_done())
            .map(|(id, _criterion)| id.clone())
    }

    #[inline]
    pub fn get_completed_criteria(&self) -> impl Iterator<Item = Arc<str>> {
        self.criteria
            .iter()
            .filter(|&(_id, criterion)| criterion.is_done())
            .map(|(id, _criterion)| id.clone())
    }
}

impl Serialize for AdvancementProgress {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let map: HashMap<&Arc<str>, &CriterionProgress> = self
            .criteria
            .iter()
            .filter(|(_key, criteria)| criteria.is_done())
            .collect();
        map.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for AdvancementProgress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let criteria = HashMap::<Arc<str>, CriterionProgress>::deserialize(deserializer)?;
        Ok(Self {
            criteria,
            requirements: AdvancementRequirement::default(),
        })
    }
}

#[derive(Clone, Default)]
pub struct AdvancementProgressMap {
    pub map: IndexMap<&'static Advancement, AdvancementProgress>,
}

impl AdvancementProgressMap {
    /// Gets a mutable reference to the current progress for a given advancement. Creates the state entry if missing.
    pub fn get_mut_or_start_progress(
        &mut self,
        advancement: &'static Advancement,
    ) -> &mut AdvancementProgress {
        self.map.entry(advancement).or_insert_with(|| {
            let mut progress = AdvancementProgress::default();
            progress.update(AdvancementRequirement::from_const(advancement.requirements));
            progress
        })
    }

    #[inline]
    pub fn clear(&mut self) {
        self.map.clear();
    }

    #[inline]
    pub fn insert(&mut self, advancement: &'static Advancement, progress: AdvancementProgress) {
        self.map.insert(advancement, progress);
    }

    #[must_use]
    #[inline]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    #[must_use]
    #[inline]
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

/// Manages a player's collection of advancements.
///
/// This handles saving, loading, and tracking the state of granted / revoked advancements.
pub struct PlayerAdvancement {
    pub progress: AdvancementProgressMap,
    pub is_first_packet: bool,
    pub roots_to_update: HashSet<&'static AdvancementNode>,
    pub visible: HashSet<&'static Advancement>,
    pub progress_changed: HashSet<&'static Advancement>,
    pub manager: Arc<AdvancementManager>,
    pub path: PathBuf,
    pub last_selected_tab: Option<&'static Advancement>,
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
    #[must_use]
    pub fn new(manager: Arc<AdvancementManager>, uuid: Uuid) -> Self {
        Self {
            progress: AdvancementProgressMap::default(),
            path: manager.advancement_path.join(format!("{uuid}.json")),
            manager,
            player: Weak::new(),
            is_first_packet: true,
            roots_to_update: HashSet::default(),
            visible: HashSet::default(),
            progress_changed: HashSet::default(),
            last_selected_tab: None,
        }
    }

    /// Associates the `PlayerAdvancement` data with the given player.
    pub fn set_player(&mut self, player: &Arc<Player>) {
        self.player = Arc::downgrade(player);
    }

    /// Returns whether advancement saving is enabled for this player.
    #[must_use]
    #[inline]
    pub fn is_save_enabled(&self) -> bool {
        self.manager.save_enabled
    }

    ///reload the advancements from the file
    pub async fn reload(&mut self) -> Result<(), AdvancementDataError> {
        //self.stopListening(); TODO
        self.progress.clear();
        self.visible.clear();
        self.roots_to_update.clear();
        self.progress_changed.clear();
        self.is_first_packet = true;
        self.last_selected_tab = None;
        self.load().await
    }

    /// Saves the player's advancement progress to disk as JSON.
    pub async fn save(&self) -> Result<(), AdvancementDataError> {
        if !self.is_save_enabled() {
            return Ok(());
        }
        let json = to_string_pretty(self).map_err(AdvancementDataError::Json)?;
        let path = self.path.clone();
        spawn_blocking(move || {
            if let Err(e) = create_dir_all(path.parent().unwrap()) {
                error!("Failed to create player advancement directory : {e}");
                return Err(AdvancementDataError::Io(e));
            }
            write(path, json).map_err(AdvancementDataError::Io)
        })
        .await
        .expect("spawn_blocking task panicked")
    }

    /// Loads the player's advancement progress from disk.
    pub async fn load(&mut self) -> Result<(), AdvancementDataError> {
        if !self.path.exists() || !self.is_save_enabled() {
            return Ok(());
        }

        let path = self.path.clone();
        let json = spawn_blocking(|| read(path).map_err(AdvancementDataError::Io))
            .await
            .expect("spawn_blocking task panicked")?;

        let loaded_data: HashMap<String, AdvancementProgress> =
            serde_json::from_str(from_utf8(&json).unwrap()).map_err(AdvancementDataError::Json)?;

        self.progress.clear();
        for (advancement_id, mut progress) in loaded_data {
            if let Some(advancement_ref) = Advancement::from_minecraft_name(&advancement_id) {
                progress.update(AdvancementRequirement::from_const(
                    advancement_ref.requirements,
                ));
                self.progress.insert(advancement_ref, progress);
                self.progress_changed.insert(advancement_ref);
                self.mark_for_visibility_update(advancement_ref);
            } else {
                warn!("The Advancement name {} is invalid", advancement_id);
            }
        }
        Ok(())
    }

    fn mark_for_visibility_update(&mut self, advancement: &'static Advancement) {
        let node = ADVANCEMENT_TREE.get_node_from_id(&advancement.id);
        if let Some(node) = node {
            self.roots_to_update.insert(node.root());
        }
    }

    fn update_tree_visibility(
        &mut self,
        root: &AdvancementNode,
        added: &mut Vec<&'static Advancement>,
        removed: &mut Vec<Identifier>,
    ) {
        visibility_evaluator::evaluate_visibility(
            root,
            self,
            &mut |player_advancement, node| {
                player_advancement
                    .progress
                    .get_mut_or_start_progress(node.value)
                    .is_done()
            },
            &mut move |player_advancement, node, should_be_visible| {
                let advancement = node.value;
                if should_be_visible {
                    if player_advancement.visible.insert(advancement) {
                        added.push(advancement);
                        if player_advancement.progress.map.contains_key(advancement) {
                            player_advancement.progress_changed.insert(advancement);
                        }
                    }
                } else if player_advancement.visible.remove(advancement) {
                    removed.push(advancement.id.clone());
                }
            },
        );
    }

    /// Flushes any pending advancement state down to the client.
    pub fn flush_dirty(&mut self, player: &Arc<Player>, show_advancement: bool) {
        if self.is_first_packet || !self.roots_to_update.is_empty() {
            let mut progress: HashMap<Identifier, &AdvancementProgress> = HashMap::new();
            let mut added: Vec<&Advancement> = Vec::new();
            let mut removed: Vec<Identifier> = Vec::new();
            for root in self.roots_to_update.clone() {
                self.update_tree_visibility(root, &mut added, &mut removed);
            }
            self.roots_to_update.clear();
            for advancement in &self.progress_changed {
                if self.visible.contains(advancement) {
                    progress.insert(advancement.id.clone(), &self.progress.map[advancement]);
                }
            }
            self.progress_changed.clear();
            if !progress.is_empty() || !added.is_empty() || !removed.is_empty() {
                let player = player.clone();
                let parsed_progress: Vec<AdvancementProgressData> = progress
                    .into_iter()
                    .map(|(key, val)| AdvancementProgressData {
                        id: key,
                        progress: val
                            .criteria
                            .iter()
                            .map(|(key, val)| Criteria {
                                criterion_id: key.clone(),
                                achieve_date: val.0.map(|time| {
                                    time.duration_since(UNIX_EPOCH).unwrap().as_millis() as i64
                                }),
                            })
                            .collect(),
                    })
                    .collect();
                let first_packet = self.is_first_packet;
                tokio::spawn(async move {
                    player
                        .client
                        .send_packet_now(&CUpdateAdvancements::new(
                            first_packet,
                            added,
                            parsed_progress,
                            removed,
                            show_advancement,
                        ))
                        .await;
                });
            }
        }
        self.is_first_packet = false;
    }

    /// Grants the rewards (like experience) associated with completing an advancement.
    pub fn grant_reward(player: Arc<Player>, reward: &'static AdvancementReward) {
        tokio::spawn(async move {
            tokio::join!(
                player.add_experience_points(reward.experience),
                // more reward later
            );
        });
    }

    /// award a criterion of an advancement to the player, updating its status to complete and granting rewards if applicable.
    pub fn award(&mut self, advancement: &'static Advancement, criterion: &str) -> bool {
        //TODO call and creates Events for plugins
        let mut result = false;
        let player = self.player.upgrade().unwrap();
        let progress = self.progress.get_mut_or_start_progress(advancement);
        let was_done = progress.is_done();
        if progress.grant_progress(criterion) {
            result = true;
            self.progress_changed.insert(advancement);
            if !was_done && progress.is_done() {
                //TODO listener
                Self::grant_reward(player.clone(), advancement.reward);
                if let Some(display) = advancement.display
                    && display.announce_to_chat
                    && player
                        .world()
                        .level_info
                        .load()
                        .game_rules
                        .show_advancement_messages
                {
                    tokio::spawn(async move {
                        let component = TextComponent::translate_cross(
                            display.frame_type.get_translation(),
                            translation::bedrock::CHAT_TYPE_ACHIEVEMENT,
                            [player.get_display_name().await, advancement.name()],
                        );
                        player
                            .world()
                            .broadcast_system_message(&component, false)
                            .await; //send translate component for the event
                    });
                }
            }
        }
        if !was_done && progress.is_done() {
            self.mark_for_visibility_update(advancement);
        }
        result
    }

    /// Revokes a previously awarded advancement, clearing its progress state.
    pub fn revoke(&mut self, advancement: &'static Advancement, criterion: &str) -> bool {
        let mut result = false;
        let progress = self.progress.get_mut_or_start_progress(advancement);
        let was_done = progress.is_done();
        if progress.revoke_progress(criterion) {
            //TODO listener
            self.progress_changed.insert(advancement);
            result = true;
        }

        if was_done && !progress.is_done() {
            self.mark_for_visibility_update(advancement);
        }
        result
    }

    /// set the selected advancement tab of the player
    pub async fn set_selected_tab(&mut self, advancement: Option<&'static Advancement>) {
        let old = self.last_selected_tab;
        if let Some(value) = advancement
            && value.is_root()
            && value.display.is_some()
        {
            self.last_selected_tab = advancement;
        } else {
            self.last_selected_tab = None;
        }
        if old != self.last_selected_tab
            && let Some(player) = self.player.upgrade()
        {
            player
                .client
                .send_packet_now(&CSelectAdvancementsTab::new(
                    self.last_selected_tab.map(|adv| adv.id.clone()),
                ))
                .await;
        }
    }
}

impl Serialize for PlayerAdvancement {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let filtered_map: HashMap<&'static Advancement, &AdvancementProgress> = self
            .progress
            .map
            .iter()
            .filter(|(_key, value)| value.has_progress())
            .map(|(&key, val)| (key, val))
            .collect();
        let mut map = serializer.serialize_map(Some(filtered_map.len()))?;

        for (advancement, progress) in &filtered_map {
            map.serialize_entry(&advancement.id, &progress.criteria)?;
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
    fn advancement_progress() {
        let mut criteria = HashMap::new();
        criteria.insert(Arc::from("testCriteria"), CriterionProgress::default());
        criteria.insert(Arc::from("testCriteria2"), CriterionProgress::default());
        let requirements = AdvancementRequirement {
            requirements: vec![
                vec![Arc::from("testCriteria")],
                vec![Arc::from("testCriteria2")],
            ],
        };
        let mut progress = AdvancementProgress {
            criteria,
            requirements,
        };
        assert!(!progress.is_done());
        assert!(!progress.has_progress());
        progress.grant_progress("testCriteria");
        assert!(!progress.is_done());
        assert!(progress.has_progress());
        progress.grant_progress("testCriteria2");
        assert!(progress.is_done());
        assert!(progress.has_progress());
    }

    #[test]
    fn new_player_advancement() {
        let temp_dir = tempdir().unwrap();
        let manager = Arc::new(AdvancementManager::new(temp_dir.path(), true));
        let id = Uuid::new_v4();
        let pa = PlayerAdvancement::new(manager, id);
        assert!(pa.is_save_enabled());
        assert!(pa.is_first_packet);
        assert!(pa.roots_to_update.is_empty());
        assert!(pa.progress.is_empty());
    }

    #[test]
    fn get_or_start_progress() {
        let temp_dir = tempdir().unwrap();
        let manager = Arc::new(AdvancementManager::new(temp_dir.path(), true));
        let id = Uuid::new_v4();
        let mut pa = PlayerAdvancement::new(manager, id);
        let adv = Advancement::STORY_ROOT;
        let progress = pa.progress.get_mut_or_start_progress(adv);
        assert!(
            !progress.is_done(),
            "New progress should not be marked done by default"
        );
    }

    #[test]
    fn revoke_advancement() {
        let temp_dir = tempdir().unwrap();
        let manager = Arc::new(AdvancementManager::new(temp_dir.path(), true));
        let id = Uuid::new_v4();
        let mut pa = PlayerAdvancement::new(manager, id);
        let adv = Advancement::STORY_ROOT;
        {
            let progress_mut = pa.progress.get_mut_or_start_progress(adv);
            progress_mut.grant_progress("crafting_table");
        };
        assert!(pa.progress.get_mut_or_start_progress(adv).is_done());
        pa.revoke(adv, "crafting_table");
        assert!(!pa.progress.get_mut_or_start_progress(adv).is_done());
    }

    #[tokio::test]
    async fn save_advancement_progress() {
        let temp_dir = tempdir().unwrap();
        let manager = Arc::new(AdvancementManager::new(temp_dir.path(), true));
        let id = Uuid::new_v4();
        let mut pa = PlayerAdvancement::new(manager, id);

        // Add some advancement progress
        let adv = Advancement::STORY_ROOT;
        {
            let progress_mut = pa.progress.get_mut_or_start_progress(adv);
            progress_mut.grant_progress("crafting_table");
        };

        // Save should succeed
        assert!(pa.save().await.is_ok(), "Save should succeed");

        // File should exist
        assert!(pa.path.exists(), "Saved file should exist");

        // Content should be valid JSON
        let content = std::fs::read_to_string(&pa.path).unwrap();
        assert!(!content.is_empty(), "Saved file should not be empty");
        let _: HashMap<String, AdvancementProgress> =
            serde_json::from_str(&content).expect("Saved content should be valid JSON");
    }

    #[tokio::test]
    async fn save_disabled() {
        let temp_dir = tempdir().unwrap();
        let manager = Arc::new(AdvancementManager::new(temp_dir.path(), false));
        let id = Uuid::new_v4();
        let mut pa = PlayerAdvancement::new(manager, id);

        // Add some advancement progress
        let adv = Advancement::STORY_ROOT;
        {
            let progress_mut = pa.progress.get_mut_or_start_progress(adv);
            progress_mut.grant_progress("crafting_table");
        };

        // Save should return Ok but not actually save
        assert!(
            pa.save().await.is_ok(),
            "Save with disabled saving should return Ok"
        );
        assert!(
            !pa.path.exists(),
            "File should not be created when saving is disabled"
        );
    }

    #[tokio::test]
    async fn load_nonexistent_file() {
        let temp_dir = tempdir().unwrap();
        let manager = Arc::new(AdvancementManager::new(temp_dir.path(), true));
        let id = Uuid::new_v4();
        let mut pa = PlayerAdvancement::new(manager, id);

        // Load from nonexistent file should return Ok (not error)
        assert!(
            pa.load().await.is_ok(),
            "Loading from nonexistent file should return Ok"
        );
        assert!(pa.progress.is_empty(), "Advancements should remain empty");
    }

    #[tokio::test]
    async fn load_advancement_progress() {
        let temp_dir = tempdir().unwrap();
        let manager = Arc::new(AdvancementManager::new(temp_dir.path(), true));

        let id = Uuid::new_v4();
        let mut pa = PlayerAdvancement::new(manager, id);
        // Create a JSON file with advancement data
        let adv = Advancement::STORY_ROOT;
        let mut progress = AdvancementProgress::default();
        progress.update(AdvancementRequirement::from_const(adv.requirements));
        progress.grant_progress("crafting_table");
        let data = serde_json::json!({ adv.id.to_string():progress });
        std::fs::write(&pa.path, data.to_string()).unwrap();

        // Load the file
        assert!(pa.load().await.is_ok(), "Load should succeed");

        // Verify the advancement was loaded
        let loaded_progress = pa.progress.get_mut_or_start_progress(adv);
        assert!(
            loaded_progress.is_done(),
            "Loaded advancement should be marked complete"
        );
    }

    #[tokio::test]
    async fn save_load_roundtrip() {
        let temp_dir = tempdir().unwrap();

        // Create and save advancements
        let manager = Arc::new(AdvancementManager::new(temp_dir.path(), true));
        let id = Uuid::new_v4();
        let mut pa = PlayerAdvancement::new(manager.clone(), id);

        let adv = Advancement::STORY_ROOT;
        {
            let progress_mut = pa.progress.get_mut_or_start_progress(adv);
            progress_mut.grant_progress("crafting_table");
        };

        assert!(pa.save().await.is_ok(), "Save should succeed");

        // Load the saved advancements into a new instance
        let mut pa_loaded = PlayerAdvancement::new(manager, id);
        assert!(pa_loaded.load().await.is_ok(), "Load should succeed");

        // Verify the loaded data matches the saved data
        let loaded_progress = pa_loaded.progress.get_mut_or_start_progress(adv);
        assert!(
            loaded_progress.is_done(),
            "Loaded progress should match saved progress"
        );
        assert_eq!(
            pa_loaded.progress.len(),
            pa.progress.len(),
            "Loaded advancements count should match"
        );
    }

    #[tokio::test]
    async fn load_invalid_advancement_id() {
        let temp_dir = tempdir().unwrap();
        let manager = Arc::new(AdvancementManager::new(temp_dir.path(), true));

        // Create a JSON file with invalid advancement ID
        let mut criteria = HashMap::new();
        criteria.insert(Arc::from("testCriteria"), CriterionProgress::default());
        let requirements = AdvancementRequirement {
            requirements: vec![vec![Arc::from("testCriteria")]],
        };
        let progress = AdvancementProgress {
            criteria,
            requirements,
        };
        let data = serde_json::json!({
            "invalid_advancement_id_12345": progress
        });
        let id = Uuid::new_v4();
        let mut pa = PlayerAdvancement::new(manager, id);
        std::fs::write(&pa.path, data.to_string()).unwrap();

        // Load should still succeed but skip the invalid entry

        assert!(
            pa.load().await.is_ok(),
            "Load should succeed even with invalid IDs"
        );
        assert!(
            pa.progress.is_empty(),
            "Invalid advancements should be skipped"
        );
    }

    #[tokio::test]
    async fn save_multiple_advancements() {
        let temp_dir = tempdir().unwrap();
        let manager = Arc::new(AdvancementManager::new(temp_dir.path(), true));
        let id = Uuid::new_v4();
        let mut pa = PlayerAdvancement::new(manager, id);

        // Add multiple advancements
        let adv1 = Advancement::STORY_ROOT;
        let adv2 = Advancement::NETHER_ROOT;

        {
            let progress_mut1 = pa.progress.get_mut_or_start_progress(adv1);
            progress_mut1.grant_progress("crafting_table");
        };
        {
            let progress_mut2 = pa.progress.get_mut_or_start_progress(adv2);
            progress_mut2.grant_progress("entered_nether");
        };

        assert!(pa.save().await.is_ok(), "Save should succeed");

        // Verify both were saved
        let content = std::fs::read_to_string(&pa.path).unwrap();
        let saved_data: HashMap<String, AdvancementProgress> =
            serde_json::from_str(&content).unwrap();
        assert_eq!(saved_data.len(), 2, "Should have saved both advancements");
    }

    #[tokio::test]
    async fn ignore_loading() {
        let temp_dir = tempdir().unwrap();
        let manager = Arc::new(AdvancementManager::new(temp_dir.path(), false));

        let id = Uuid::new_v4();
        let mut pa = PlayerAdvancement::new(manager, id);
        // Create a JSON file with advancement data
        let adv = Advancement::STORY_ROOT;
        let data = serde_json::json!({ adv.id.to_string(): { "complete": true } });
        std::fs::write(&pa.path, data.to_string()).unwrap();

        //try load the file
        assert!(pa.load().await.is_ok(), "Load should succeed");

        // Verify that the advancement was not loaded
        assert!(
            pa.progress.is_empty(),
            "The advancement shouldn't have been loaded"
        );
    }
}
