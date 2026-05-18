use crate::entity::player::Player;
use crate::entity::player::advancement::{AdvancementDataError, PlayerAdvancement};
use pumpkin_data::Advancement;
use pumpkin_util::identifier::Identifier;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display};
use std::fs::create_dir_all;
use std::hash::Hash;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use tracing::{error, info};
use uuid::Uuid;



pub struct AdvancementNode {
    pub children: RwLock<HashSet<Arc<Self>>>,
    pub parent: Option<Arc<Self>>,
    pub value: &'static Advancement,
}

impl AdvancementNode {
    pub fn add_child(&self, child: Arc<Self>) {
        self.children.write().unwrap().insert(child);
    }

    #[must_use]
    pub fn new(value:&'static Advancement, parent: Option<Arc<Self>>) -> Self {
        Self {
            value,
            parent,
            children: RwLock::new(HashSet::new()),
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
    node: Arc<AdvancementNode>,
    parent: Option<Rc<Self>>,
    previous_sibling: Option<Rc<Self>>,
    child_index: u32,
    children: Vec<Rc<Self>>,
    ancestor: Option<Rc<Self>>,
    thread: Option<Rc<Self>>,
    x: u32,
    y: f32,
    r#mod: f32,
    change: f32,
    shift: f32,
}

impl TreeNodePosition {
    fn new(node: Arc<AdvancementNode>, parent:Option<Rc<TreeNodePosition>>, previous_sibling:Option<Rc<Self>>,child_index:u32,x:u32) -> Self {
        Self {
            node,
            parent,
            previous_sibling,
            child_index,
            x,
            y: -1f32,
            r#mod: 0.0,
            change: 0.0,
            ancestor:None,
            children: vec![],
            shift: 0.0,
            thread: None,
        }
    }

    fn previous_or_thread(&self) -> Option<Rc<Self>>{
        if let Some(thread) = self.thread {
            Some(thread.clone())
        } else {
            if !self.children.is_empty() {
                self.children.get(0).map(|child| child.clone())
            }else {
                None
            }
        }
    }

   fn next_or_thread(&self) -> Option<Rc<Self>>{
       if let Some(thread) = self.thread {
           Some(thread.clone())
       } else {
           if !self.children.is_empty() {
               self.children.get(self.children.len() - 1).map(|child| child.clone())
           }else {
               None
           }
       }
    }

    fn move_subtree(&mut self, right : Rc<Self>, shift:f32) {
        let subtrees = (right.child_index - self.child_index) as f32;
        if subtrees != 0.0 {
            right.change -= shift / subtrees;
            self.change += shift / subtrees;
        }

        right.shift += shift;
        right.y += shift;
        right.r#mod += shift;
    }

    fn apportion(self : Rc<Self>, mut default_ancestor: Option<Rc<Self>>) -> Rc<Self> {
        if let Some(value) = &self.previous_sibling {
            let mut vir = self.clone();
            let mut vor = self.clone();
            let mut vil = value.clone();
            let mut vol = if let Some(parent) = &self.parent {
                *parent.children.get(0).unwrap()
            } else {
               return self.clone();
            };
            let mut sir = self.r#mod;
            let mut sor = self.r#mod;
            let mut sil = vil.r#mod;
            let mut sol = vol.r#mod;
            while let Some(next) = vil.next_or_thread() && let Some(previous) = vir.previous_or_thread() {
                vil = next.clone();
                vir = previous.clone();
                vol = previous.clone();
                vor = next.clone();
                vor.ancestor = Some(self.clone());
                let shift = vil.y + sil - (vir.y + sir) + 1.0;
                if shift > 0.0 {
                    vil.ancestor.clone().unwrap_or(default_ancestor.clone().unwrap_or(self.clone())).move_subtree(self.clone(), shift);
                    sir += shift;
                    sor += shift;
                }

                sil += vil.r#mod;
                sir += vir.r#mod;
                sol += vol.r#mod;
                sor += vor.r#mod;
            }

            if let Some(next) = vil.next_or_thread() && vor.next_or_thread().is_none() {
                vor.thread = Some(next);
                vor.r#mod += sil - sor;
            } else {
                if let Some(previous) = vir.previous_or_thread() && vol.previous_or_thread().is_none() {
                    vol.thread = Some(previous);
                    vol.r#mod += sir - sol;
                }
                default_ancestor = Some(self.clone());
            }
        }
        default_ancestor.unwrap_or(self)
    }

    fn execute_shifts(&self) {
        let mut shift = 0.0f32;
        let mut change = 0.0f32;
        for child in self.children {
            child.y += shift;
            child.r#mod += shift;
            change += child.change;
            shift += child.shift + change;
        }
    }

    fn first_walk(&mut self){
        if (self.children.is_empty()) {
            if let Some(prev) = self.previous_sibling {
                self.y = prev.y + 1.0;
            } else {
                self.y = 0.0;
            }
        } else {
            let mut default_ancestor: Option<Rc<Self>> = None;
            for mut child in self.children {
                child.first_walk();
                default_ancestor = Some(child.apportion(default_ancestor));
            }

            self.execute_shift();
            let  f = (((TreeNodePosition)this.children.get(0)).y + ((TreeNodePosition)this.children.get(this.children.size() - 1)).y) / 2.0F;
            if (this.previousSibling != null) {
                this.y = this.previousSibling.y + 1.0F;
                this.mod = this.y - f;
            } else {
                this.y = f;
            }
        }
    }

    pub fn run(root_node: Arc<AdvancementNode>) {
        let mut tree_node_position = TreeNodePosition::new(root_node,None,None,1,0);
        tree_node_position.first_walk();
    }
}

#[derive(Default)]
pub struct AdvancementTree {
    pub nodes: HashMap<Identifier, Arc<AdvancementNode>>,
    pub roots: HashSet<Arc<AdvancementNode>>,
    pub tasks: HashSet<Arc<AdvancementNode>>,
}

impl AdvancementTree {
    pub fn add_all(&mut self, advancements: impl IntoIterator<Item = &'static Advancement>) {
        let mut advancements_to_add : Vec<&'static Advancement> = advancements
            .into_iter()
            .collect();

        while !advancements_to_add.is_empty() {
            let len_before = advancements_to_add.len();
            advancements_to_add.retain(|&val| !self.try_insert(val));
            if len_before == advancements_to_add.len() {
                error!("Couldn't load advancements: {:?}",
                    advancements_to_add.iter().map(|a| &a.id).collect::<Vec<_>>()
                );
                break;
            }
        }
        info!("Loaded {} advancements", self.nodes.len());
    }

    pub fn try_insert(&mut self, advancement: &'static Advancement) -> bool {
        let parent_id = &advancement.parent;

        let parent_node = match parent_id {
            Some(id) => match self.nodes.get_mut(id) {
                Some(node) => Some(Arc::clone(node)),
                None => return false,
            },
            None => None,
        };
        let node = Arc::new(AdvancementNode::new(advancement, parent_node.clone()));
        self.nodes.insert(advancement.id.clone(), Arc::clone(&node));

        if let Some(parent) = parent_node {
            parent.add_child(Arc::clone(&node));
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