use crate::Advancement;
use crate::item_stack::ItemStack;
use crate::potion_brewing::ItemRecipe;
use pumpkin_util::resource_location::ResourceLocation;
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::color::{Color, NamedColor};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use std::sync::Arc;

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

    pub const fn new(
        title: &'static str,
        description: &'static str,
        item_icon: ItemStack,
        frame_type: FrameType,
        background_texture: Option<&'static str>,
        show_toast: bool,
        hidden: bool,
        announce_to_chat: bool,
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
            x: 0f32,
            y: 0f32,
        }
    }

    pub fn set_location(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
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

pub trait Criterion {}
