use crate::entity::player::Player;
use crate::entity::player::advancement::{AdvancementDataError, PlayerAdvancement};
use pumpkin_data::Advancement;
use pumpkin_util::identifier::Identifier;
use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::fs::create_dir_all;
use std::hash::Hash;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tracing::{error, info};
use uuid::Uuid;

pub struct AdvancementNode {
    pub children: HashSet<Arc<RwLock<AdvancementNode>>>,
    pub parent: Option<Arc<RwLock<AdvancementNode>>>,
    pub value: &'static Advancement,
}

impl AdvancementNode {
    pub fn add_child(&mut self, child: Arc<RwLock<AdvancementNode>>) {
        self.children.insert(child);
    }

    #[must_use]
    pub fn new(value: &'static Advancement, parent: Option<Arc<RwLock<AdvancementNode>>>) -> Self {
        Self {
            value,
            parent,
            children: HashSet::new(),
        }
    }
}

impl PartialEq<Self> for AdvancementNode {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for AdvancementNode {}

impl Display for AdvancementNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value.id)
    }
}

impl Hash for AdvancementNode {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

struct TreeNodePosition {
    node: Arc<RwLock<AdvancementNode>>,
    parent: Arc<TreeNodePosition>,
    previous_sibling: Option<Arc<TreeNodePosition>>,
    child_index: i32,
    children: Vec<Arc<TreeNodePosition>>,
    x: i32,
    y: f32,
    r#mod: f32,
    change: f32,
    shift: f32,
}

impl TreeNodePosition {
    pub fn run(_root_node: Arc<RwLock<AdvancementNode>>) {
        //TODO implement the position of the Advancement
    }
}

#[derive(Default)]
pub struct AdvancementTree {
    pub nodes: HashMap<Identifier, Arc<RwLock<AdvancementNode>>>,
    pub roots: HashSet<Arc<RwLock<AdvancementNode>>>,
    pub tasks: HashSet<Arc<RwLock<AdvancementNode>>>,
}

impl AdvancementTree {
    pub fn add_all(&mut self, advancements: impl IntoIterator<Item: Into<Advancement>>) {
        let mut advancements_to_add : Vec<Advancement> = advancements
            .into_iter()
            .map(|a| a.into())
            .collect();

        while !advancements_to_add.is_empty() {
            let len_before = advancements_to_add.len();
            advancements_to_add.retain(|val| ! self.try_insert(val));
            if len_before == advancements_to_add.len() {
                error!("Couldn't load advancements: {:?}", advancements_to_add);
                break;
            }
        }
        info!("Loaded {} advancements", self.nodes.len());
    }

    pub fn try_insert(&mut self, advancement: &'static Advancement) -> bool {
        let parent_id = &advancement.parent;

        let parent_node = match parent_id {
            Some(id) => match self.nodes.get(&id) {
                Some(node) => Some(Arc::clone(node)),
                None => return false,
            },
            None => None,
        };

        let node = Arc::new(RwLock::new(AdvancementNode::new(advancement, parent_node.clone())));
        self.nodes.insert(advancement.id.clone(), Arc::clone(&node));

        if let Some(parent) = parent_node {
            parent.read().unwrap().add_child(Arc::clone(&node));
            self.tasks.insert(node);
        } else {
            self.roots.insert(node);
        }
        true
    }
}

/// Manages player advancements, including data creation and saving.
pub struct AdvancementManager {
    pub advancement_path: PathBuf,
    pub save_enabled: bool,
    pub tree: AdvancementTree,
}

impl AdvancementManager {
    /// Creates a new instance of `AdvancementManager` using the player data path.
    pub fn new(player_data_path: impl Into<PathBuf>, save_enabled: bool) -> Self {
        let path = player_data_path.into().join("advancements");
        if !path.exists()
            && let Err(e) = create_dir_all(&path)
        {
            error!(
                "Failed to create player data directory at {}: {e}",
                path.display()
            );
        }
        Self {
            advancement_path: path,
            save_enabled,
            tree: AdvancementTree::default(),
        }
    }

    /// Retrieves the list of all available advancements in the game.
    #[must_use]
    pub fn get_advancements(&self) -> Vec<Identifier> {
        Advancement::get_list().to_vec()
    }

    /// Creates and returns a new instance of `PlayerAdvancement` with the configured path.
    #[must_use]
    pub fn new_advancement(self: Arc<Self>, owner: Uuid) -> PlayerAdvancement {
        PlayerAdvancement::new(self, owner)
    }

    /// Saves the advancements of all provided players.
    pub async fn save_all_players(players: Vec<Arc<Player>>) -> Result<(), AdvancementDataError> {
        for player in players {
            player.advancements.lock().await.save()?;
        }
        Ok(())
    }

    /// Saves the advancements of a specific player.
    pub async fn save_player(player: &Player) -> Result<(), AdvancementDataError> {
        player.advancements.lock().await.save()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn advancement_manager_new() {
        let path = PathBuf::from("test_data");
        let manager = AdvancementManager::new(path, true);
        assert_eq!(
            manager.advancement_path,
            PathBuf::from("test_data/advancements")
        );
    }

    #[test]
    fn get_advancement_path() {
        let path = PathBuf::from("world/playerdata");
        let manager = AdvancementManager::new(path, true);
        let advancement_path = manager.advancement_path;
        assert!(advancement_path.ends_with("advancements"));
    }
}