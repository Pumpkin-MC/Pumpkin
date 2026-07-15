use pumpkin_util::text::TextComponent;
use serde::Serialize;

#[derive(Serialize)]
pub struct DialogNBT<'a>(pub DialogNBTSource<'a>);

impl<'a> DialogNBT<'a> {
    #[must_use]
    pub const fn from_dialog(dialog: &'a Dialog) -> Self {
        Self(DialogNBTSource::Struct(dialog))
    }

    #[must_use]
    pub const fn from_nbt(compound: &'a pumpkin_nbt::compound::NbtCompound) -> Self {
        Self(DialogNBTSource::Nbt(compound))
    }
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum DialogNBTSource<'a> {
    Struct(&'a Dialog),
    Nbt(&'a pumpkin_nbt::compound::NbtCompound),
}

#[derive(Serialize)]
pub struct Dialog {
    pub r#type: String,
    pub title: TextComponent,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub body: Vec<DialogBody>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub inputs: Vec<DialogInput>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub buttons: Vec<ActionButton>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<DialogLink>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_action: Option<DialogAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after_action: Option<String>,
    pub can_close_with_escape: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_title: Option<TextComponent>,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DialogBody {
    #[serde(rename = "minecraft:plain_message")]
    PlainMessage { contents: TextComponent },
    #[serde(rename = "minecraft:item")]
    Item { item: i32 }, // TODO: ItemStack serialization to NBT
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DialogInput {
    #[serde(rename = "minecraft:boolean")]
    Boolean {
        label: TextComponent,
        default_value: bool,
    },
    #[serde(rename = "minecraft:text")]
    Text {
        label: TextComponent,
        placeholder: TextComponent,
        default_value: String,
    },
    #[serde(rename = "minecraft:number_range")]
    NumberRange {
        label: TextComponent,
        min: f32,
        max: f32,
        initial: f32,
        step: f32,
        #[serde(skip_serializing_if = "Option::is_none")]
        label_format: Option<String>,
    },
    #[serde(rename = "minecraft:single_option")]
    SingleOption {
        label: TextComponent,
        options: Vec<TextComponent>,
        initial_index: u32,
    },
}

#[derive(Serialize)]
pub struct ActionButton {
    pub text: TextComponent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tooltip: Option<TextComponent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    pub action: DialogAction,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DialogAction {
    #[serde(rename = "minecraft:open_url")]
    OpenUrl { url: String },
    #[serde(rename = "minecraft:custom")]
    Custom {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        payload: Option<Vec<u8>>,
    },
}

#[derive(Serialize)]
pub struct DialogLink {
    pub label: crate::Label,
    pub url: String,
}
