use serde::{Deserialize, Serialize};
use pumpkin_util::resource_location::ResourceLocation;
use pumpkin_util::text::TextComponent;
use crate::item_stack::ItemStack;
use crate::potion_brewing::ItemRecipe;

pub struct AdvancementDisplay {
    pub title: &'static str,
    pub description: &'static str,
    pub item_icon: ItemStack,
    pub frame_type: FrameType,
    pub background_texture: Option<&'static str>,
    pub show_toast: bool,
    pub hidden: bool,
    pub announce_to_chat: bool,
}

#[derive(Clone, Copy, Deserialize, Serialize, Default)]
#[repr(i32)]
#[serde(rename_all = "lowercase")]
pub enum FrameType {
    #[default]
    Task = 0,
    Challenge = 1,
    Goal = 2,
}

pub struct AdvancementReward {
    pub experience: u32,
    pub recipes: &'static [ItemRecipe],
}

pub trait Criterion {
}
