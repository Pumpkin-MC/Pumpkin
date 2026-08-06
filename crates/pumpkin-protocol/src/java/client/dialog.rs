use pumpkin_util::text::TextComponent;

use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

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

    #[allow(clippy::todo)]
    pub fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        match &self.0 {
            DialogNBTSource::Struct(_dialog) => {
                todo!("Serialize Dialog to NbtCompound manually")
            }
            DialogNBTSource::Nbt(nbt) => {
                let tag = pumpkin_nbt::tag::NbtTag::Compound((*nbt).clone());
                write.write_nbt(tag)?;
                Ok(())
            }
        }
    }
}

pub enum DialogNBTSource<'a> {
    Struct(&'a Dialog),
    Nbt(&'a pumpkin_nbt::compound::NbtCompound),
}

pub struct Dialog {
    pub r#type: String,
    pub title: TextComponent,
    pub body: Vec<DialogBody>,
    pub inputs: Vec<DialogInput>,
    pub buttons: Vec<ActionButton>,
    pub links: Vec<DialogLink>,
    pub exit_action: Option<DialogAction>,
    pub after_action: Option<String>,
    pub can_close_with_escape: bool,
    pub external_title: Option<TextComponent>,
}

pub enum DialogBody {
    PlainMessage { contents: TextComponent },
    Item { item: i32 }, // TODO: ItemStack serialization to NBT
}

pub enum DialogInput {
    Boolean {
        label: TextComponent,
        default_value: bool,
    },
    Text {
        label: TextComponent,
        placeholder: TextComponent,
        default_value: String,
    },
    NumberRange {
        label: TextComponent,
        min: f32,
        max: f32,
        initial: f32,
        step: f32,
        label_format: Option<String>,
    },
    SingleOption {
        label: TextComponent,
        options: Vec<TextComponent>,
        initial_index: u32,
    },
}

pub struct ActionButton {
    pub text: TextComponent,
    pub tooltip: Option<TextComponent>,
    pub width: Option<u32>,
    pub action: DialogAction,
}

pub enum DialogAction {
    OpenUrl {
        url: String,
    },
    Custom {
        id: String,
        payload: Option<Vec<u8>>,
    },
}

pub struct DialogLink {
    pub label: crate::Label,
    pub url: String,
}
