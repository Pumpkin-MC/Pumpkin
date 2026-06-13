use crate::item_stack::ItemStack;
use crate::potion_brewing::ItemRecipe;
use crate::{ADVANCEMENT_TREE, Advancement};
use pumpkin_util::identifier::Identifier;
use pumpkin_util::resource_location::ResourceLocation;
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::color::{Color, NamedColor};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeMap;
use std::fmt::Display;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use crate::translation::java::{CHAT_TYPE_ADVANCEMENT_CHALLENGE, CHAT_TYPE_ADVANCEMENT_GOAL, CHAT_TYPE_ADVANCEMENT_TASK};

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
            y,
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

    pub fn get_translation(&self) -> &'static str {
        match self {
            FrameType::Task => CHAT_TYPE_ADVANCEMENT_TASK,
            FrameType::Challenge => CHAT_TYPE_ADVANCEMENT_CHALLENGE,
            FrameType::Goal => CHAT_TYPE_ADVANCEMENT_GOAL,
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
    pub value: &'static Advancement,
}

impl AdvancementNode {
    pub fn add_child(&mut self, child: usize) {
        self.children.push(child);
    }

    #[must_use]
    pub fn new(value: &'static Advancement, parent: Option<usize>) -> Self {
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

    pub fn root(&self) -> &AdvancementNode {
        let mut advancement_node = self;
        while let Some(parent) = &advancement_node.parent {
            advancement_node = &ADVANCEMENT_TREE.nodes_vector[*parent];
        }
        advancement_node
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
    pub nodes: BTreeMap<Identifier, usize>,
    pub nodes_vector: Vec<AdvancementNode>,
    pub roots: Vec<usize>,
    pub tasks: Vec<usize>,
}

impl AdvancementTree {
    pub fn get_node_from_id(&self, id: &Identifier) -> Option<&AdvancementNode> {
        if let Some(idx) = self.nodes.get(id) {
            self.nodes_vector.get(*idx)
        } else {
            None
        }
    }

    pub fn get_node_from_idx(&self, idx: usize) -> Option<&AdvancementNode> {
        self.nodes_vector.get(idx)
    }

    pub fn get_idx(&self, id: &Identifier) -> Option<usize> {
        self.nodes.get(id).copied()
    }
}

#[derive(Serialize)]
pub struct AdvancementProgress {
    pub id: Identifier,
    pub progress: Vec<Criteria>,
}

#[derive(Serialize)]
pub struct Criteria {
    pub criterion_id: Identifier,
    pub achieve_date: Option<i64>,
}
/// Represents the requirements needed to complete an advancement.
/// To mark as done, at least one criterion from each requirement group must be satisfied.
#[derive(Serialize,Deserialize,Clone,Debug,Default)]
pub struct AdvancementRequirement {
    pub requirements: Vec<Vec<Arc<str>>>,
}

impl AdvancementRequirement {
    pub fn from_const(static_requirements: &[&[&str]]) -> Self{
        let requirements = static_requirements.iter()
            .map(|inner| inner.iter().map(|s| Arc::from(*s)).collect())
            .collect();
        AdvancementRequirement { requirements }
    }

    /// test if the requirements is complete
    pub fn test(&self,predicate: impl Fn(&str) -> bool) -> bool {
        if self.requirements.is_empty() {
            false
        } else {
            for requirement in &self.requirements {
                if !Self::any_match(requirement, &predicate) {
                    return false;
                }
            }
            true
        }
    }

    /// check if any test pass
    fn any_match(requirements: &Vec<Arc<str>>,predicate: impl Fn(&str) -> bool) -> bool {
        for requirement in requirements {
            if predicate(requirement) {
                return true;
            }
        }
        false
    }

    pub fn names(&self) -> Vec<Arc<str>> {
        self.requirements.iter().flatten().cloned().collect()
    }
}
pub trait Criterion {}
