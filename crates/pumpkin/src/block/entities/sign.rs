use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicI8, Ordering},
};

use super::BlockEntity;
use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::click::ClickEvent;

pub use pumpkin_data::dye_color::DyeColor;

pub struct SignBlockEntity {
    pub front_text: SignText,
    pub back_text: SignText,
    pub is_waxed: AtomicBool,
    position: BlockPos,
    pub currently_editing_player: Arc<Mutex<Option<uuid::Uuid>>>,
}

pub type Text = SignText;

pub struct SignText {
    pub has_glowing_text: AtomicBool,
    color: AtomicI8,
    pub messages: Arc<Mutex<[Box<str>; 4]>>,
    pub filtered_messages: Arc<Mutex<[Box<str>; 4]>>,
}

impl Clone for SignText {
    fn clone(&self) -> Self {
        Self {
            has_glowing_text: AtomicBool::new(self.has_glowing_text.load(Ordering::Relaxed)),
            color: AtomicI8::new(self.color.load(Ordering::Relaxed)),
            messages: self.messages.clone(),
            filtered_messages: self.filtered_messages.clone(),
        }
    }
}

impl Default for SignText {
    fn default() -> Self {
        Self {
            has_glowing_text: AtomicBool::new(false),
            color: AtomicI8::new(DyeColor::Black as i8),
            messages: Arc::new(Mutex::new(Self::empty_messages())),
            filtered_messages: Arc::new(Mutex::new(Self::empty_messages())),
        }
    }
}

#[allow(clippy::fallible_impl_from)]
impl From<SignText> for NbtTag {
    fn from(value: SignText) -> Self {
        let mut nbt = NbtCompound::new();
        nbt.put_bool(
            "has_glowing_text",
            value.has_glowing_text.load(Ordering::Relaxed),
        );
        nbt.put_string("color", value.get_color().name().to_string());

        let messages = value
            .messages
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        nbt.put_list(
            "messages",
            messages.iter().map(|s| Self::String(s.clone())).collect(),
        );

        let filtered_messages = value
            .filtered_messages
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if *filtered_messages != *messages {
            nbt.put_list(
                "filtered_messages",
                filtered_messages
                    .iter()
                    .map(|s| Self::String(s.clone()))
                    .collect(),
            );
        }

        Self::Compound(nbt)
    }
}

impl From<NbtTag> for SignText {
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

        let parsed_messages = [
            get_message(0),
            get_message(1),
            get_message(2),
            get_message(3),
        ];

        let filtered_messages: Option<Vec<Box<str>>> =
            nbt.get_list("filtered_messages").map(|list| {
                list.iter()
                    .filter_map(|tag| tag.extract_string().map(Box::from))
                    .collect()
            });

        let parsed_filtered = filtered_messages.map_or_else(
            || parsed_messages.clone(),
            |filtered| {
                let get_filtered = |i: usize| -> Box<str> {
                    filtered.get(i).cloned().unwrap_or_else(|| Box::from(""))
                };
                [
                    get_filtered(0),
                    get_filtered(1),
                    get_filtered(2),
                    get_filtered(3),
                ]
            },
        );

        Self {
            has_glowing_text: AtomicBool::new(has_glowing_text),
            color: AtomicI8::new(DyeColor::by_name(color).unwrap_or(DyeColor::Black).id() as i8),
            messages: Arc::new(Mutex::new(parsed_messages)),
            filtered_messages: Arc::new(Mutex::new(parsed_filtered)),
        }
    }
}

impl SignText {
    pub const LINES: usize = 4;

    #[must_use]
    pub fn empty_messages() -> [Box<str>; 4] {
        [Box::from(""), Box::from(""), Box::from(""), Box::from("")]
    }

    #[must_use]
    pub fn new(
        messages: [Box<str>; 4],
        filtered_messages: Option<[Box<str>; 4]>,
        color: DyeColor,
        has_glowing_text: bool,
    ) -> Self {
        let filtered = filtered_messages.unwrap_or_else(|| messages.clone());
        Self {
            has_glowing_text: AtomicBool::new(has_glowing_text),
            color: AtomicI8::new(color.id() as i8),
            messages: Arc::new(Mutex::new(messages)),
            filtered_messages: Arc::new(Mutex::new(filtered)),
        }
    }

    #[must_use]
    pub fn from_messages(messages: [Box<str>; 4]) -> Self {
        Self::new(messages, None, DyeColor::Black, false)
    }

    #[must_use]
    pub fn has_glowing_text(&self) -> bool {
        self.has_glowing_text.load(Ordering::Relaxed)
    }

    pub fn set_has_glowing_text(&self, has_glowing_text: bool) {
        self.has_glowing_text
            .store(has_glowing_text, Ordering::Relaxed);
    }

    #[must_use]
    pub fn get_color(&self) -> DyeColor {
        let c = self.color.load(Ordering::Relaxed);
        if c >= 0 {
            DyeColor::by_id(c as u8).unwrap_or(DyeColor::Black)
        } else {
            DyeColor::Black
        }
    }

    pub fn set_color(&self, color: DyeColor) {
        self.color.store(color.id() as i8, Ordering::Relaxed);
    }

    #[must_use]
    pub fn get_message(&self, index: usize, should_filter: bool) -> Box<str> {
        if index >= Self::LINES {
            return Box::from("");
        }
        let lock = if should_filter {
            self.filtered_messages.lock()
        } else {
            self.messages.lock()
        };
        lock.unwrap_or_else(std::sync::PoisonError::into_inner)[index].clone()
    }

    pub fn set_message(&self, index: usize, message: Box<str>, filtered_message: Option<Box<str>>) {
        if index >= Self::LINES {
            return;
        }
        let filtered = filtered_message.unwrap_or_else(|| message.clone());
        self.messages
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)[index] = message;
        self.filtered_messages
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)[index] = filtered;
    }

    #[must_use]
    pub fn get_messages(&self, should_filter: bool) -> [Box<str>; 4] {
        let lock = if should_filter {
            self.filtered_messages.lock()
        } else {
            self.messages.lock()
        };
        lock.unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    #[must_use]
    pub fn has_message(&self, should_filter: bool) -> bool {
        let messages = self.get_messages(should_filter);
        messages.iter().any(|msg| !msg.is_empty())
    }

    #[must_use]
    pub fn has_any_click_commands(&self, should_filter: bool) -> bool {
        let messages = self.get_messages(should_filter);
        for msg in &messages {
            if msg.is_empty() || !msg.starts_with('{') {
                continue;
            }
            if let Ok(component) = serde_json::from_str::<TextComponent>(msg)
                && let Some(ClickEvent::RunCommand { .. }) = &component.0.style.click_event
            {
                return true;
            }
        }
        false
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
            .map(SignText::from)
            .unwrap_or_default();
        let back_text = nbt
            .get("back_text")
            .cloned()
            .map(SignText::from)
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

    fn write_nbt(&self, nbt: &mut NbtCompound) {
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
}

impl SignBlockEntity {
    pub const ID: &'static str = "minecraft:sign";

    #[must_use]
    pub fn new(position: BlockPos, is_front: bool, messages: [Box<str>; 4]) -> Self {
        Self {
            position,
            is_waxed: AtomicBool::new(false),
            front_text: if is_front {
                SignText::from_messages(messages.clone())
            } else {
                SignText::default()
            },
            back_text: if is_front {
                SignText::default()
            } else {
                SignText::from_messages(messages)
            },
            currently_editing_player: Arc::new(Mutex::new(None)),
        }
    }

    #[must_use]
    pub fn empty(position: BlockPos) -> Self {
        Self {
            position,
            is_waxed: AtomicBool::new(false),
            front_text: SignText::default(),
            back_text: SignText::default(),
            currently_editing_player: Arc::new(Mutex::new(None)),
        }
    }
}

pub enum SignEntityRef<'a> {
    Sign(&'a SignBlockEntity),
    Hanging(&'a super::hanging_sign::HangingSignBlockEntity),
}

impl<'a> SignEntityRef<'a> {
    pub fn from_block_entity(entity: &'a dyn BlockEntity) -> Option<Self> {
        entity
            .as_any()
            .downcast_ref::<SignBlockEntity>()
            .map(Self::Sign)
            .or_else(|| {
                entity
                    .as_any()
                    .downcast_ref::<super::hanging_sign::HangingSignBlockEntity>()
                    .map(Self::Hanging)
            })
    }

    #[must_use]
    pub const fn front_text(&self) -> &'a SignText {
        match self {
            Self::Sign(s) => &s.front_text,
            Self::Hanging(s) => &s.front_text,
        }
    }

    #[must_use]
    pub const fn back_text(&self) -> &'a SignText {
        match self {
            Self::Sign(s) => &s.back_text,
            Self::Hanging(s) => &s.back_text,
        }
    }

    #[must_use]
    pub const fn get_text(&self, is_front: bool) -> &'a SignText {
        if is_front {
            self.front_text()
        } else {
            self.back_text()
        }
    }

    #[must_use]
    pub fn is_waxed(&self) -> bool {
        match self {
            Self::Sign(s) => s.is_waxed.load(Ordering::Relaxed),
            Self::Hanging(s) => s.is_waxed.load(Ordering::Relaxed),
        }
    }

    pub fn set_waxed(&self, waxed: bool) {
        match self {
            Self::Sign(s) => s.is_waxed.store(waxed, Ordering::Relaxed),
            Self::Hanging(s) => s.is_waxed.store(waxed, Ordering::Relaxed),
        }
    }

    #[must_use]
    pub const fn currently_editing_player(&self) -> &'a Arc<Mutex<Option<uuid::Uuid>>> {
        match self {
            Self::Sign(s) => &s.currently_editing_player,
            Self::Hanging(s) => &s.currently_editing_player,
        }
    }
}
