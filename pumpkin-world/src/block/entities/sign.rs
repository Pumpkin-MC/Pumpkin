use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicI8, Ordering},
};

use super::BlockEntity;
use async_trait::async_trait;
use num_derive::FromPrimitive;
use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};
use pumpkin_util::math::position::BlockPos;
use tokio::sync::Mutex;

#[derive(Clone, Default, FromPrimitive)]
#[repr(i8)]
pub enum DyeColor {
    White = 0,
    Orange = 1,
    Magenta = 2,
    LightBlue = 3,
    Yellow = 4,
    Lime = 5,
    Pink = 6,
    Gray = 7,
    LightGray = 8,
    Cyan = 9,
    Purple = 10,
    Blue = 11,
    Brown = 12,
    Green = 13,
    Red = 14,
    #[default]
    Black = 15,
}

impl From<DyeColor> for String {
    fn from(value: DyeColor) -> Self {
        match value {
            DyeColor::White => "white".to_string(),
            DyeColor::Orange => "orange".to_string(),
            DyeColor::Magenta => "magenta".to_string(),
            DyeColor::LightBlue => "light_blue".to_string(),
            DyeColor::Yellow => "yellow".to_string(),
            DyeColor::Lime => "lime".to_string(),
            DyeColor::Pink => "pink".to_string(),
            DyeColor::Gray => "gray".to_string(),
            DyeColor::LightGray => "light_gray".to_string(),
            DyeColor::Cyan => "cyan".to_string(),
            DyeColor::Purple => "purple".to_string(),
            DyeColor::Blue => "blue".to_string(),
            DyeColor::Brown => "brown".to_string(),
            DyeColor::Green => "green".to_string(),
            DyeColor::Red => "red".to_string(),
            DyeColor::Black => "black".to_string(),
        }
    }
}

impl From<&str> for DyeColor {
    fn from(s: &str) -> Self {
        match s {
            "white" => DyeColor::White,
            "orange" => DyeColor::Orange,
            "magenta" => DyeColor::Magenta,
            "light_blue" => DyeColor::LightBlue,
            "yellow" => DyeColor::Yellow,
            "lime" => DyeColor::Lime,
            "pink" => DyeColor::Pink,
            "gray" => DyeColor::Gray,
            "light_gray" => DyeColor::LightGray,
            "cyan" => DyeColor::Cyan,
            "purple" => DyeColor::Purple,
            "blue" => DyeColor::Blue,
            "brown" => DyeColor::Brown,
            "green" => DyeColor::Green,
            "red" => DyeColor::Red,
            "black" => DyeColor::Black,
            _ => DyeColor::default(),
        }
    }
}

impl From<i8> for DyeColor {
    fn from(s: i8) -> Self {
        match s {
            0 => DyeColor::White,
            1 => DyeColor::Orange,
            2 => DyeColor::Magenta,
            3 => DyeColor::LightBlue,
            4 => DyeColor::Yellow,
            5 => DyeColor::Lime,
            6 => DyeColor::Pink,
            7 => DyeColor::Gray,
            8 => DyeColor::LightGray,
            9 => DyeColor::Cyan,
            10 => DyeColor::Purple,
            11 => DyeColor::Blue,
            12 => DyeColor::Brown,
            13 => DyeColor::Green,
            14 => DyeColor::Red,
            15 => DyeColor::Black,
            _ => DyeColor::default(),
        }
    }
}

impl From<DyeColor> for NbtTag {
    fn from(value: DyeColor) -> Self {
        NbtTag::Byte(value as i8)
    }
}

pub struct SignBlockEntity {
    pub front_text: Text,
    pub back_text: Text,
    pub is_waxed: AtomicBool,
    position: BlockPos,
    pub currently_editing_player: Arc<Mutex<Option<uuid::Uuid>>>,
}

pub struct Text {
    pub has_glowing_text: AtomicBool,
    color: AtomicI8,
    pub messages: Arc<std::sync::Mutex<[String; 4]>>,
}

impl Clone for Text {
    fn clone(&self) -> Self {
        Self {
            has_glowing_text: AtomicBool::new(self.has_glowing_text.load(Ordering::Relaxed)),
            color: AtomicI8::new(self.color.load(Ordering::Relaxed)),
            messages: self.messages.clone(),
        }
    }
}

impl Default for Text {
    fn default() -> Self {
        Self {
            has_glowing_text: AtomicBool::new(false),
            color: AtomicI8::new(DyeColor::default() as i8),
            messages: Default::default(),
        }
    }
}

impl From<Text> for NbtTag {
    fn from(value: Text) -> Self {
        let mut nbt = NbtCompound::new();
        nbt.put_bool(
            "has_glowing_text",
            value.has_glowing_text.load(Ordering::Relaxed),
        );
        nbt.put_string("color", value.get_color().into());
        nbt.put_list(
            "messages",
            value
                .messages
                .lock()
                .unwrap()
                .iter()
                .map(|s| NbtTag::String(s.clone()))
                .collect(),
        );
        NbtTag::Compound(nbt)
    }
}

impl From<NbtTag> for Text {
    fn from(tag: NbtTag) -> Self {
        let nbt = tag.extract_compound().unwrap();
        let has_glowing_text = nbt.get_bool("has_glowing_text").unwrap_or(false);
        let color = nbt.get_string("color").unwrap();
        let messages: Vec<String> = nbt
            .get_list("messages")
            .unwrap()
            .iter()
            .filter_map(|tag| tag.extract_string().map(|s| s.to_string()))
            .collect();
        Self {
            has_glowing_text: AtomicBool::new(has_glowing_text),
            color: AtomicI8::new(DyeColor::from(color) as i8),
            messages: Arc::new(std::sync::Mutex::new([
                // its important that we use unwrap_or since otherwise we may crash on older versions
                messages.first().unwrap_or(&"".to_string()).clone(),
                messages.get(1).unwrap_or(&"".to_string()).clone(),
                messages.get(2).unwrap_or(&"".to_string()).clone(),
                messages.get(3).unwrap_or(&"".to_string()).clone(),
            ])),
        }
    }
}

impl Text {
    fn new(messages: [String; 4]) -> Self {
        Self {
            has_glowing_text: AtomicBool::new(false),
            color: AtomicI8::new(DyeColor::default() as i8),
            messages: Arc::new(std::sync::Mutex::new(messages)),
        }
    }

    pub fn get_color(&self) -> DyeColor {
        self.color.load(Ordering::Relaxed).into()
    }

    pub fn set_color(&self, color: DyeColor) {
        self.color.store(color as i8, Ordering::Relaxed);
    }
}

#[async_trait]
impl BlockEntity for SignBlockEntity {
    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn from_nbt(nbt: &pumpkin_nbt::compound::NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        let front_text = Text::from(nbt.get("front_text").unwrap().clone());
        let back_text = Text::from(nbt.get("back_text").unwrap().clone());
        let is_waxed = nbt.get_bool("is_waxed").unwrap_or(false);
        Self {
            position,
            front_text,
            back_text,
            is_waxed: AtomicBool::new(is_waxed),
            currently_editing_player: Arc::new(Mutex::new(None)),
        }
    }

    async fn write_nbt(&self, nbt: &mut pumpkin_nbt::compound::NbtCompound) {
        nbt.put("front_text", self.front_text.clone());
        nbt.put("back_text", self.back_text.clone());
        nbt.put_bool("is_waxed", self.is_waxed.load(Ordering::Relaxed));
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        nbt.put("front_text", self.front_text.clone());
        nbt.put("back_text", self.back_text.clone());
        nbt.put_bool("is_waxed", self.is_waxed.load(Ordering::Relaxed));
        Some(nbt)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_block_entity(&self) -> &dyn BlockEntity {
        self
    }
}

impl SignBlockEntity {
    pub const ID: &'static str = "minecraft:sign";
    pub fn new(position: BlockPos, is_front: bool, messages: [String; 4]) -> Self {
        Self {
            position,
            is_waxed: AtomicBool::new(false),
            front_text: if is_front {
                Text::new(messages.clone())
            } else {
                Text::default()
            },
            back_text: if !is_front {
                Text::new(messages.clone())
            } else {
                Text::default()
            },
            currently_editing_player: Arc::new(Mutex::new(None)),
        }
    }
    pub fn empty(position: BlockPos) -> Self {
        Self {
            position,
            is_waxed: AtomicBool::new(false),
            front_text: Text::default(),
            back_text: Text::default(),
            currently_editing_player: Arc::new(Mutex::new(None)),
        }
    }
}
