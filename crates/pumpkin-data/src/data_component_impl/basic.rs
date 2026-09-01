use crate::data_component_impl::{DataComponentImpl, get_i32_hash, get_str_hash};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::text::TextComponent;
use std::borrow::Cow;

#[derive(Clone, Debug, PartialEq)]
pub struct CustomDataImpl {
    pub data: NbtCompound,
}
impl CustomDataImpl {
    #[must_use]
    pub const fn new(data: NbtCompound) -> Self {
        Self { data }
    }
    #[must_use]
    pub fn read_data(tag: &NbtTag) -> Option<Self> {
        if let NbtTag::Compound(c) = tag {
            Some(Self { data: c.clone() })
        } else {
            None
        }
    }
}
impl DataComponentImpl for CustomDataImpl {
    fn write_data(&self) -> NbtTag {
        NbtTag::Compound(self.data.clone())
    }
    fn get_hash(&self) -> i32 {
        0
    }
    default_impl!(CustomData);
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct MaxStackSizeImpl {
    pub size: u8,
}
impl MaxStackSizeImpl {
    pub fn read_data(data: &NbtTag) -> Option<Self> {
        data.extract_int().map(|size| Self { size: size as u8 })
    }
}
impl DataComponentImpl for MaxStackSizeImpl {
    fn write_data(&self) -> NbtTag {
        NbtTag::Int(self.size as i32)
    }
    fn get_hash(&self) -> i32 {
        get_i32_hash(self.size as i32) as i32
    }
    default_impl!(MaxStackSize);
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct MaxDamageImpl {
    pub max_damage: i32,
}
impl MaxDamageImpl {
    pub fn read_data(data: &NbtTag) -> Option<Self> {
        data.extract_int().map(|max_damage| Self { max_damage })
    }
}
impl DataComponentImpl for MaxDamageImpl {
    fn write_data(&self) -> NbtTag {
        NbtTag::Int(self.max_damage)
    }
    default_impl!(MaxDamage);
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct DamageImpl {
    pub damage: i32,
}
impl DamageImpl {
    pub fn read_data(data: &NbtTag) -> Option<Self> {
        data.extract_int().map(|damage| Self { damage })
    }
}
impl DataComponentImpl for DamageImpl {
    fn write_data(&self) -> NbtTag {
        NbtTag::Int(self.damage)
    }
    fn get_hash(&self) -> i32 {
        get_i32_hash(self.damage) as i32
    }
    default_impl!(Damage);
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct UnbreakableImpl;
impl UnbreakableImpl {
    pub const fn read_data(_data: &NbtTag) -> Option<Self> {
        Some(Self)
    }
}
impl DataComponentImpl for UnbreakableImpl {
    fn write_data(&self) -> NbtTag {
        NbtTag::Compound(NbtCompound::new())
    }
    fn get_hash(&self) -> i32 {
        0
    }
    default_impl!(Unbreakable);
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct CustomNameImpl {
    pub name: TextComponent,
}
impl CustomNameImpl {
    pub fn read_data(data: &NbtTag) -> Option<Self> {
        data.extract_string().map(|name| Self {
            name: TextComponent::text(name.to_string()),
        })
    }
}
impl DataComponentImpl for CustomNameImpl {
    fn write_data(&self) -> NbtTag {
        NbtTag::String(self.name.clone().get_text().into())
    }
    fn get_hash(&self) -> i32 {
        get_str_hash(self.name.clone().get_text().as_str()) as i32
    }
    default_impl!(CustomName);
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ItemNameImpl {
    pub name: Cow<'static, str>,
}
impl ItemNameImpl {
    pub fn read_data(data: &NbtTag) -> Option<Self> {
        let name = match data {
            NbtTag::String(name) => name.to_string(),
            NbtTag::Compound(component) => component
                .get_string("translate")
                .or_else(|| component.get_string("text"))?
                .to_owned(),
            _ => return None,
        };
        Some(Self {
            name: Cow::Owned(name),
        })
    }
}
impl DataComponentImpl for ItemNameImpl {
    fn write_data(&self) -> NbtTag {
        let mut component = NbtCompound::new();
        component.put_string("translate", self.name.to_string());
        NbtTag::Compound(component)
    }
    fn get_hash(&self) -> i32 {
        get_str_hash(&self.name) as i32
    }
    default_impl!(ItemName);
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ItemModelImpl {
    pub id: Cow<'static, str>,
}
impl ItemModelImpl {
    pub fn read_data(data: &NbtTag) -> Option<Self> {
        data.extract_string().map(|id| Self {
            id: Cow::Owned(id.to_string()),
        })
    }
}
impl DataComponentImpl for ItemModelImpl {
    fn write_data(&self) -> NbtTag {
        NbtTag::String(self.id.clone().into_owned().into())
    }
    fn get_hash(&self) -> i32 {
        get_str_hash(self.id.as_ref()) as i32
    }
    default_impl!(ItemModel);
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct LoreImpl {
    pub lines: Vec<TextComponent>,
}
impl LoreImpl {
    pub fn read_data(data: &NbtTag) -> Option<Self> {
        let NbtTag::List(lines) = data else {
            return None;
        };

        Some(Self {
            lines: lines
                .iter()
                .filter_map(NbtTag::extract_string)
                .map(|line| TextComponent::text(line.to_owned()))
                .collect(),
        })
    }
}
impl DataComponentImpl for LoreImpl {
    fn write_data(&self) -> NbtTag {
        NbtTag::List(
            self.lines
                .iter()
                .map(|line| NbtTag::String(line.clone().get_text().into_boxed_str()))
                .collect(),
        )
    }
    default_impl!(Lore);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Rarity {
    #[default]
    Common = 0,
    Uncommon = 1,
    Rare = 2,
    Epic = 3,
}

impl Rarity {
    #[must_use]
    pub fn from_id(id: i32) -> Option<Self> {
        match id {
            0 => Some(Self::Common),
            1 => Some(Self::Uncommon),
            2 => Some(Self::Rare),
            3 => Some(Self::Epic),
            _ => None,
        }
    }

    #[must_use]
    pub fn to_id(self) -> i32 {
        self as i32
    }

    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "common" => Some(Self::Common),
            "uncommon" => Some(Self::Uncommon),
            "rare" => Some(Self::Rare),
            "epic" => Some(Self::Epic),
            _ => None,
        }
    }

    #[must_use]
    pub fn to_name(self) -> &'static str {
        match self {
            Self::Common => "common",
            Self::Uncommon => "uncommon",
            Self::Rare => "rare",
            Self::Epic => "epic",
        }
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct RarityImpl {
    pub rarity: Rarity,
}

impl RarityImpl {
    pub fn read_data(data: &NbtTag) -> Option<Self> {
        let name = data.extract_string()?;
        Some(Self {
            rarity: Rarity::from_name(name)?,
        })
    }
}

impl DataComponentImpl for RarityImpl {
    fn write_data(&self) -> NbtTag {
        NbtTag::String(self.rarity.to_name().into())
    }

    fn get_hash(&self) -> i32 {
        crate::data_component_impl::get_i32_hash(self.rarity.to_id()) as i32
    }

    default_impl!(Rarity);
}

#[derive(Clone, Debug, PartialEq)]
pub struct CustomModelDataImpl {
    pub floats: Vec<f32>,
    pub flags: Vec<bool>,
    pub strings: Vec<String>,
    pub colors: Vec<i32>,
}
impl CustomModelDataImpl {
    pub fn read_data(data: &NbtTag) -> Option<Self> {
        let compound = data.extract_compound()?;
        let floats = compound
            .get_list("floats")
            .map(|l| l.iter().filter_map(NbtTag::extract_float).collect())
            .unwrap_or_default();
        let flags = compound
            .get_list("flags")
            .map(|l| l.iter().filter_map(NbtTag::extract_bool).collect())
            .unwrap_or_default();
        let strings = compound
            .get_list("strings")
            .map(|l| {
                l.iter()
                    .filter_map(|t| t.extract_string().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        // Vanilla encodes the color list as ints, but tolerate a packed int array too.
        let colors = if let Some(arr) = compound.get_int_array("colors") {
            arr.to_vec()
        } else if let Some(l) = compound.get_list("colors") {
            l.iter().filter_map(NbtTag::extract_int).collect()
        } else {
            Vec::new()
        };
        Some(Self {
            floats,
            flags,
            strings,
            colors,
        })
    }
}
impl DataComponentImpl for CustomModelDataImpl {
    fn write_data(&self) -> NbtTag {
        let mut compound = NbtCompound::new();
        compound.put_list(
            "floats",
            self.floats.iter().map(|f| NbtTag::Float(*f)).collect(),
        );
        compound.put_list(
            "flags",
            self.flags.iter().map(|b| NbtTag::Byte(*b as i8)).collect(),
        );
        compound.put_list(
            "strings",
            self.strings
                .iter()
                .map(|s| NbtTag::String(s.clone().into()))
                .collect(),
        );
        compound.put_list(
            "colors",
            self.colors.iter().map(|c| NbtTag::Int(*c)).collect(),
        );
        NbtTag::Compound(compound)
    }
    default_impl!(CustomModelData);
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct TooltipDisplayImpl;
impl DataComponentImpl for TooltipDisplayImpl {
    default_impl!(TooltipDisplay);
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct CreativeSlotLockImpl;
impl DataComponentImpl for CreativeSlotLockImpl {
    default_impl!(CreativeSlotLock);
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct EnchantmentGlintOverrideImpl;
impl DataComponentImpl for EnchantmentGlintOverrideImpl {
    default_impl!(EnchantmentGlintOverride);
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct TooltipStyleImpl {
    pub id: String,
}
impl TooltipStyleImpl {
    pub fn read_data(data: &NbtTag) -> Option<Self> {
        data.extract_string().map(|id| Self { id: id.to_string() })
    }
}
impl DataComponentImpl for TooltipStyleImpl {
    fn write_data(&self) -> NbtTag {
        NbtTag::String(self.id.clone().into())
    }
    fn get_hash(&self) -> i32 {
        get_str_hash(&self.id) as i32
    }
    default_impl!(TooltipStyle);
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct NoteBlockSoundImpl {
    pub sound: String,
}
impl NoteBlockSoundImpl {
    pub fn read_data(data: &NbtTag) -> Option<Self> {
        data.extract_string().map(|sound| Self {
            sound: sound.to_string(),
        })
    }
}
impl DataComponentImpl for NoteBlockSoundImpl {
    fn write_data(&self) -> NbtTag {
        NbtTag::String(self.sound.clone().into())
    }
    fn get_hash(&self) -> i32 {
        get_str_hash(&self.sound) as i32
    }
    default_impl!(NoteBlockSound);
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct BaseColorImpl {
    pub color: String,
}
impl BaseColorImpl {
    pub fn read_data(data: &NbtTag) -> Option<Self> {
        data.extract_string().map(|color| Self {
            color: color.to_string(),
        })
    }
}
impl DataComponentImpl for BaseColorImpl {
    fn write_data(&self) -> NbtTag {
        NbtTag::String(self.color.clone().into())
    }
    fn get_hash(&self) -> i32 {
        get_str_hash(&self.color) as i32
    }
    default_impl!(BaseColor);
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct InstrumentImpl;
impl DataComponentImpl for InstrumentImpl {
    default_impl!(Instrument);
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ProvidesTrimMaterialImpl;
impl DataComponentImpl for ProvidesTrimMaterialImpl {
    default_impl!(ProvidesTrimMaterial);
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ProvidesBannerPatternsImpl;
impl DataComponentImpl for ProvidesBannerPatternsImpl {
    default_impl!(ProvidesBannerPatterns);
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct BannerPatternsImpl;
impl DataComponentImpl for BannerPatternsImpl {
    default_impl!(BannerPatterns);
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct PotDecorationsImpl;
impl DataComponentImpl for PotDecorationsImpl {
    default_impl!(PotDecorations);
}

/// The lock's item predicate, kept as its raw NBT compound since Pumpkin does
/// not yet model item predicates.
// TODO: replace `predicate` with a typed item predicate once item predicates are modelled.
#[derive(Clone, Debug, PartialEq)]
pub struct LockImpl {
    pub predicate: NbtCompound,
}
impl LockImpl {
    pub fn read_data(data: &NbtTag) -> Option<Self> {
        data.extract_compound().map(|predicate| Self {
            predicate: predicate.clone(),
        })
    }
}
impl DataComponentImpl for LockImpl {
    fn write_data(&self) -> NbtTag {
        NbtTag::Compound(self.predicate.clone())
    }
    default_impl!(Lock);
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct BreakSoundImpl;
impl DataComponentImpl for BreakSoundImpl {
    default_impl!(BreakSound);
}

#[derive(Clone, Debug, PartialEq)]
pub struct SoundEvent {
    pub sound_name: String,
    pub range: Option<f32>,
}
impl std::hash::Hash for SoundEvent {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.sound_name.hash(state);
        if let Some(val) = self.range {
            true.hash(state);
            unsafe { (*(&raw const val).cast::<u32>()).hash(state) };
        } else {
            false.hash(state);
        }
    }
}
impl SoundEvent {
    pub const fn new(sound_name: String, range: Option<f32>) -> Self {
        Self { sound_name, range }
    }
}
