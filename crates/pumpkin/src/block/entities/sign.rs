use std::{
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicI8, Ordering},
    },
};

use super::BlockEntity;
use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};
use pumpkin_util::math::position::BlockPos;
use tokio::sync::Mutex;

pub use pumpkin_data::dye_color::DyeColor;

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
    pub messages: Arc<std::sync::Mutex<[Box<str>; 4]>>,
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
            messages: Arc::default(),
        }
    }
}

#[allow(clippy::fallible_impl_from)]
impl From<Text> for NbtTag {
    fn from(value: Text) -> Self {
        let mut nbt = NbtCompound::new();
        nbt.put_bool(
            "has_glowing_text",
            value.has_glowing_text.load(Ordering::Relaxed),
        );
        nbt.put_string("color", value.get_color().name().to_string());
        nbt.put_list(
            "messages",
            value
                .messages
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .iter()
                .map(|s| Self::String(s.clone()))
                .collect(),
        );
        Self::Compound(nbt)
    }
}

impl From<NbtTag> for Text {
    fn from(tag: NbtTag) -> Self {
        let Some(nbt) = tag.extract_compound() else {
            return Self::default();
        };
        let has_glowing_text = nbt.get_bool("has_glowing_text").unwrap_or(false);
        let color = nbt.get_string("color").unwrap_or("black");
        let messages: Vec<Box<str>> = nbt.get_list("messages").map_or_else(Vec::new, |list| {
            list.iter()
                .filter_map(|tag| tag.extract_string().map(Box::from))
                .collect()
        });
        let get_message =
            |i: usize| -> Box<str> { messages.get(i).cloned().unwrap_or_else(|| Box::from("")) };

        Self {
            has_glowing_text: AtomicBool::new(has_glowing_text),
            color: AtomicI8::new(DyeColor::by_name(color).unwrap_or_default().id() as i8),
            messages: Arc::new(std::sync::Mutex::new([
                get_message(0),
                get_message(1),
                get_message(2),
                get_message(3),
            ])),
        }
    }
}

impl Text {
    #[must_use]
    pub fn new(messages: [Box<str>; 4]) -> Self {
        Self {
            has_glowing_text: AtomicBool::new(false),
            color: AtomicI8::new(DyeColor::default().id() as i8),
            messages: Arc::new(std::sync::Mutex::new(messages)),
        }
    }

    pub fn get_color(&self) -> DyeColor {
        let c = self.color.load(Ordering::Relaxed);
        if c >= 0 {
            DyeColor::by_id(c as u8).unwrap_or_default()
        } else {
            DyeColor::default()
        }
    }

    pub fn set_color(&self, color: DyeColor) {
        self.color.store(color.id() as i8, Ordering::Relaxed);
    }
}

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
        let front_text = nbt
            .get("front_text")
            .cloned()
            .map(Text::from)
            .unwrap_or_default();
        let back_text = nbt
            .get("back_text")
            .cloned()
            .map(Text::from)
            .unwrap_or_default();
        let is_waxed = nbt.get_bool("is_waxed").unwrap_or(false);
        Self {
            position,
            front_text,
            back_text,
            is_waxed: AtomicBool::new(is_waxed),
            currently_editing_player: Arc::new(Mutex::new(None)),
        }
    }

    fn write_nbt<'a>(
        &'a self,
        nbt: &'a mut NbtCompound,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            nbt.put("front_text", self.front_text.clone());
            nbt.put("back_text", self.back_text.clone());
            nbt.put_bool("is_waxed", self.is_waxed.load(Ordering::Relaxed));
        })
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
}

impl SignBlockEntity {
    pub const ID: &'static str = "minecraft:sign";
    #[must_use]
    pub fn new(position: BlockPos, is_front: bool, messages: [Box<str>; 4]) -> Self {
        Self {
            position,
            is_waxed: AtomicBool::new(false),
            front_text: if is_front {
                Text::new(messages.clone())
            } else {
                Text::default()
            },
            back_text: if is_front {
                Text::default()
            } else {
                Text::new(messages)
            },
            currently_editing_player: Arc::new(Mutex::new(None)),
        }
    }
    #[must_use]
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
