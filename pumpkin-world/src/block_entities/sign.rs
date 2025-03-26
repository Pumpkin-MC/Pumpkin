use super::BlockEntity;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};
use pumpkin_util::math::position::BlockPos;

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

impl Into<NbtTag> for DyeColor {
    fn into(self) -> NbtTag {
        NbtTag::Byte(self as i8)
    }
}

// NBT data structure
pub struct SignBlockEntity {
    front_text: Text,
    back_text: Text,
    is_waxed: bool,
    position: BlockPos,
}

#[derive(Clone, Default)]
struct Text {
    has_glowing_text: bool,
    color: DyeColor,
    messages: [String; 4],
}

impl Into<NbtTag> for Text {
    fn into(self) -> NbtTag {
        let mut nbt = NbtCompound::new();
        nbt.put("has_glowing_text", self.has_glowing_text);
        nbt.put("color", self.color);
        nbt.put_list(
            "messages",
            self.messages
                .into_iter()
                .map(|s| NbtTag::String(s))
                .collect(),
        );
        NbtTag::Compound(nbt)
    }
}

impl From<NbtTag> for Text {
    fn from(tag: NbtTag) -> Self {
        let nbt = tag.extract_compound().unwrap();
        let has_glowing_text = nbt.get_bool("has_glowing_text").unwrap_or(false);
        let color = nbt.get_byte("color").unwrap_or(0);
        let messages: Vec<String> = nbt
            .get_list("messages")
            .unwrap()
            .iter()
            .filter_map(|tag| tag.extract_string().cloned())
            .collect();
        Self {
            has_glowing_text,
            color: DyeColor::from_i8(color).unwrap_or(DyeColor::Black),
            messages: [
                messages[0].clone(),
                messages[1].clone(),
                messages[2].clone(),
                messages[3].clone(),
            ],
        }
    }
}

impl Text {
    fn new(messages: [String; 4]) -> Self {
        Self {
            has_glowing_text: false,
            color: DyeColor::Black,
            messages,
        }
    }
}

impl BlockEntity for SignBlockEntity {
    fn identifier(&self) -> &'static str {
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
            is_waxed,
        }
    }

    fn write_nbt(&self, nbt: &mut pumpkin_nbt::compound::NbtCompound) {
        nbt.put("front_text", self.front_text.clone());
        nbt.put("back_text", self.back_text.clone());
        nbt.put_bool("is_waxed", self.is_waxed);
    }
}

impl SignBlockEntity {
    const ID: &'static str = "minecraft:sign";
    pub fn new(position: BlockPos, is_front: bool, messages: [String; 4]) -> Self {
        let formatted_messages = [
            format!("\"{}\"", messages[0]),
            format!("\"{}\"", messages[1]),
            format!("\"{}\"", messages[2]),
            format!("\"{}\"", messages[3]),
        ];

        Self {
            position,
            is_waxed: false,
            front_text: if is_front {
                Text::new(formatted_messages.clone())
            } else {
                Text::default()
            },
            back_text: if !is_front {
                Text::new(formatted_messages.clone())
            } else {
                Text::default()
            },
        }
    }
}
