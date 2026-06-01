use std::collections::{BTreeMap, HashMap};
use std::fmt::Display;
use std::hash::{Hash, Hasher};
use crate::Advancement;
use crate::item_stack::ItemStack;
use crate::potion_brewing::ItemRecipe;
use pumpkin_util::resource_location::ResourceLocation;
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::color::{Color, NamedColor};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use std::sync::Arc;
use pumpkin_util::identifier::Identifier;

#[derive(Clone)]
pub struct AdvancementDisplay {
    pub title: &'static str,
    pub description: &'static str,
    pub item_icon: ItemStack,
    pub frame_type: FrameType,
    pub background_texture: Option<&'static str>,
    pub show_toast: bool,
    pub hidden: bool,
    pub announce_to_chat: bool,
    pub x: f32,
    pub y: f32,
}

impl AdvancementDisplay {
    pub fn get_title(&self) -> TextComponent {
        TextComponent::translate(self.title, [])
    }

    pub fn get_description(&self) -> TextComponent {
        TextComponent::translate(self.description, [])
    }

    pub fn has_background(&self) -> bool {
        self.background_texture.is_some()
    }

    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        title: &'static str,
        description: &'static str,
        item_icon: ItemStack,
        frame_type: FrameType,
        background_texture: Option<&'static str>,
        show_toast: bool,
        hidden: bool,
        announce_to_chat: bool,
        x: f32,
        y: f32,
    ) -> Self {
        Self {
            title,
            description,
            frame_type,
            item_icon,
            background_texture,
            show_toast,
            hidden,
            announce_to_chat,
            x,
            y
        }
    }
}

#[derive(Clone, Copy, Deserialize, Serialize, Default, Debug)]
#[repr(i32)]
#[serde(rename_all = "lowercase")]
pub enum FrameType {
    #[default]
    Task = 0,
    Challenge = 1,
    Goal = 2,
}

impl FrameType {
    pub fn get_color(&self) -> NamedColor {
        match self {
            FrameType::Task => NamedColor::Green,
            FrameType::Challenge => NamedColor::DarkPurple,
            FrameType::Goal => NamedColor::Green,
        }
    }

    pub fn get_name(&self) -> &'static str {
        match self {
            FrameType::Task => "task",
            FrameType::Challenge => "challenge",
            FrameType::Goal => "goal",
        }
    }
}

pub struct AdvancementReward {
    pub experience: i32,
    pub recipes: &'static [ItemRecipe],
}

pub struct AdvancementNode {
    pub children: Vec<usize>,
    pub parent: Option<usize>,
    pub value: Advancement,
}

impl AdvancementNode {
    pub fn add_child(&mut self, child: usize) {
        self.children.push(child);
    }

    #[must_use]
    pub fn new(value:Advancement,parent:Option<usize>) -> Self {
        Self {
            value,
            parent,
            children: Vec::new(),
        }
    }

    #[inline]
    #[must_use]
    pub const fn has_display(&self) -> bool {
        self.value.display.is_some()
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
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

#[derive(Default)]
pub struct AdvancementTree {
    pub nodes: HashMap<Identifier, usize>,
    pub nodes_vector: Vec<AdvancementNode>,
    pub roots: Vec<usize>,
    pub tasks: Vec<usize>
}

impl AdvancementTree {
    pub fn add_all(&mut self, advancements: Vec<Advancement>) {
        let mut advancements_to_add: Vec<Advancement> = advancements;

        while !advancements_to_add.is_empty() {
            let len_before = advancements_to_add.len();

            advancements_to_add = advancements_to_add
                .into_iter()
                .filter_map(|advancement| self.try_insert(advancement))
                .collect();

            if advancements_to_add.len() == len_before && !advancements_to_add.is_empty() {
                eprintln!(
                    "Couldn't load advancements: {:?}",
                    advancements_to_add.iter().map(|a| &a.id).collect::<Vec<_>>()
                );
                break;
            }
        }
    }

    pub fn try_insert(&mut self, advancement: Advancement) -> Option<Advancement> {
        let parent_id = &advancement.parent;
        let parent_idx: Option<usize> = match parent_id {
            Some(id) => match self.nodes.get(id) {
                Some(node) => Some(*node),
                None => return Some(advancement),
            },
            None => None,
        };
        let id = advancement.id.clone();
        let node = AdvancementNode::new(advancement);
        let node_idx = self.nodes_vector.len();
        self.nodes.insert(id, node_idx);
        if let Some(parent)  = parent_idx {
            let parent_node = self.nodes_vector.get_mut(parent).unwrap();
            parent_node.add_child(node_idx);
            self.tasks.push(node_idx);
        } else {
            self.roots.push(node_idx);
        }
        self.nodes_vector.push(node);
        None
    }
}

pub trait Criterion {}
