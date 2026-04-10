use serde::{Deserialize, Serialize};
use pumpkin_util::resource_location::ResourceLocation;
use pumpkin_util::text::TextComponent;
use crate::item_stack::ItemStack;
use crate::potion_brewing::ItemRecipe;

#[derive(Serialize)]
pub struct AdvancementDisplay {
    pub title: TextComponent,
    pub description: TextComponent,
    #[serde(rename = "icon", deserialize_with = "deserialize_icon_id")]
    pub item_icon: ItemStack,
    #[serde(default, rename = "frame")]
    pub frame_type: FrameType,
    #[serde(default, rename = "background")]
    pub background_texture: Option<&'static str>,
    #[serde(default)]
    pub show_toast: bool,
    #[serde(default)]
    pub hidden: bool,
    #[serde(default)]
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
