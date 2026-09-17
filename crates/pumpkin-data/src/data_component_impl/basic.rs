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
    /// Reads a structured text component without flattening its styles or children.
    pub fn read_data(data: &NbtTag) -> Option<Self> {
        TextComponent::try_from_nbt(data)
            .ok()
            .map(|name| Self { name })
    }
}
impl DataComponentImpl for CustomNameImpl {
    /// Persists the untranslated text tree, including styles and child components.
    fn write_data(&self) -> NbtTag {
        self.name
            .0
            .to_nbt_tag_for_version(&pumpkin_util::version::JavaMinecraftVersion::V_26_2)
    }
    fn get_hash(&self) -> i32 {
        get_str_hash(self.name.clone().get_text().as_str()) as i32
    }
    default_impl!(CustomName);
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum ItemNameImpl {
    /// A generated item's constant translation key.
    Translation(Cow<'static, str>),
    /// A stored name with arbitrary text content, styles, and children.
    Component(TextComponent),
}
impl ItemNameImpl {
    /// Reads item names without flattening structured or literal text into a translation key.
    pub fn read_data(data: &NbtTag) -> Option<Self> {
        if let NbtTag::Compound(component) = data
            && component.child_tags.len() == 1
            && let Some(key) = component.get_string("translate")
        {
            return Some(Self::Translation(Cow::Owned(key.to_owned())));
        }
        TextComponent::try_from_nbt(data).ok().map(Self::Component)
    }

    /// Returns a display component while retaining translation and formatting semantics.
    #[must_use]
    #[allow(
        deprecated,
        reason = "Stored translation keys are supplied at runtime."
    )]
    pub fn to_text_component(&self) -> TextComponent {
        match self {
            Self::Translation(key) => TextComponent::translate(key.clone(), &[]),
            Self::Component(component) => component.clone(),
        }
    }
}
impl DataComponentImpl for ItemNameImpl {
    /// Persists generated translation keys and stored text trees in the same NBT component format.
    fn write_data(&self) -> NbtTag {
        match self {
            Self::Translation(key) => {
                let mut component = NbtCompound::new();
                component.put_string("translate", key.to_string());
                NbtTag::Compound(component)
            }
            Self::Component(component) => component
                .0
                .to_nbt_tag_for_version(&pumpkin_util::version::JavaMinecraftVersion::V_26_2),
        }
    }
    /// Hashes the translation key or displayed literal text using the existing string component hash.
    fn get_hash(&self) -> i32 {
        match self {
            Self::Translation(key) => get_str_hash(key) as i32,
            Self::Component(component) => get_str_hash(&component.clone().get_text()) as i32,
        }
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
pub struct TooltipDisplayImpl {
    pub hide_tooltip: bool,
    pub hidden_components: Vec<crate::data_component::DataComponent>,
}
impl TooltipDisplayImpl {
    pub const DEFAULT: Self = Self {
        hide_tooltip: false,
        hidden_components: Vec::new(),
    };

    /// Reads tooltip visibility and its ordered set of hidden component registry names.
    pub fn read_data(data: &NbtTag) -> Option<Self> {
        let compound = data.extract_compound()?;
        let hide_tooltip = match compound.get("hide_tooltip") {
            None => false,
            Some(NbtTag::Byte(value)) => *value != 0,
            _ => return None,
        };
        let mut hidden_components = Vec::new();
        if let Some(tag) = compound.get("hidden_components") {
            let NbtTag::List(list) = tag else {
                return None;
            };
            for tag in list {
                let component =
                    crate::data_component::DataComponent::try_from_name(tag.extract_string()?)?;
                if !hidden_components.contains(&component) {
                    hidden_components.push(component);
                }
            }
        }
        Some(Self {
            hide_tooltip,
            hidden_components,
        })
    }
}
impl DataComponentImpl for TooltipDisplayImpl {
    /// Persists visibility and hidden registry names without coupling them to protocol IDs.
    fn write_data(&self) -> NbtTag {
        let mut compound = NbtCompound::new();
        compound.put_bool("hide_tooltip", self.hide_tooltip);
        compound.put_list(
            "hidden_components",
            self.hidden_components
                .iter()
                .map(|component| NbtTag::String(component.to_name().into()))
                .collect(),
        );
        NbtTag::Compound(compound)
    }
    default_impl!(TooltipDisplay);
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct CreativeSlotLockImpl;
impl CreativeSlotLockImpl {
    pub const fn read_data(_data: &NbtTag) -> Option<Self> {
        Some(Self)
    }
}
impl DataComponentImpl for CreativeSlotLockImpl {
    default_impl!(CreativeSlotLock);
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct EnchantmentGlintOverrideImpl;
impl EnchantmentGlintOverrideImpl {
    pub const fn read_data(_data: &NbtTag) -> Option<Self> {
        Some(Self)
    }
}
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
impl InstrumentImpl {
    pub const fn read_data(_data: &NbtTag) -> Option<Self> {
        Some(Self)
    }
}
impl DataComponentImpl for InstrumentImpl {
    default_impl!(Instrument);
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ProvidesTrimMaterialImpl;
impl ProvidesTrimMaterialImpl {
    pub const fn read_data(_data: &NbtTag) -> Option<Self> {
        Some(Self)
    }
}
impl DataComponentImpl for ProvidesTrimMaterialImpl {
    default_impl!(ProvidesTrimMaterial);
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ProvidesBannerPatternsImpl;
impl ProvidesBannerPatternsImpl {
    pub const fn read_data(_data: &NbtTag) -> Option<Self> {
        Some(Self)
    }
}
impl DataComponentImpl for ProvidesBannerPatternsImpl {
    default_impl!(ProvidesBannerPatterns);
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct BannerPatternLayer {
    pub pattern: String,
    pub color: crate::dye_color::DyeColor,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Default)]
pub struct BannerPatternsImpl {
    pub layers: Vec<BannerPatternLayer>,
}

impl BannerPatternsImpl {
    pub const EMPTY: Self = Self { layers: Vec::new() };

    #[must_use]
    pub fn read_data(data: &NbtTag) -> Option<Self> {
        let mut layers = Vec::new();
        if let NbtTag::List(list) = data {
            for tag in list {
                if let Some(compound) = tag.extract_compound() {
                    let pattern = compound.get_string("pattern")?.to_string();
                    let color_str = compound.get_string("color")?;
                    let color = crate::dye_color::DyeColor::by_name(color_str).unwrap_or_default();
                    layers.push(BannerPatternLayer { pattern, color });
                }
            }
        }
        Some(Self { layers })
    }
}

impl DataComponentImpl for BannerPatternsImpl {
    fn write_data(&self) -> NbtTag {
        let mut list = Vec::new();
        for layer in &self.layers {
            let mut compound = NbtCompound::new();
            compound.put_string("pattern", layer.pattern.clone());
            compound.put_string("color", layer.color.name().to_string());
            list.push(NbtTag::Compound(compound));
        }
        NbtTag::List(list)
    }

    default_impl!(BannerPatterns);
}

/// The four decorated-pot faces, ordered back, left, right, then front.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct PotDecorationsImpl {
    pub decorations: [&'static crate::item::Item; 4],
}
impl PotDecorationsImpl {
    pub const EMPTY: Self = Self {
        decorations: [&crate::item::Item::BRICK; 4],
    };

    /// Reads up to four item names, filling unspecified faces with plain bricks.
    pub fn read_data(data: &NbtTag) -> Option<Self> {
        let NbtTag::List(list) = data else {
            return None;
        };
        if list.len() > 4 {
            return None;
        }
        let mut decorations = Self::EMPTY.decorations;
        for (face, tag) in decorations.iter_mut().zip(list) {
            *face = crate::item::Item::from_registry_key(tag.extract_string()?)?;
        }
        Some(Self { decorations })
    }
}
impl DataComponentImpl for PotDecorationsImpl {
    /// Persists all four faces as registry names in their placement order.
    fn write_data(&self) -> NbtTag {
        NbtTag::List(
            self.decorations
                .iter()
                .map(|item| NbtTag::String(format!("minecraft:{}", item.registry_key).into()))
                .collect(),
        )
    }
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
impl BreakSoundImpl {
    pub const fn read_data(_data: &NbtTag) -> Option<Self> {
        Some(Self)
    }
}
impl DataComponentImpl for BreakSoundImpl {
    default_impl!(BreakSound);
}

#[cfg(test)]
mod copy_component_tests {
    use super::*;

    /// Custom names retain formatting when persisted as item components.
    #[test]
    fn custom_name_preserves_style() {
        let original = CustomNameImpl {
            name: TextComponent::text("Stored name").bold(),
        };
        assert_eq!(
            CustomNameImpl::read_data(&original.write_data()),
            Some(original)
        );
    }

    /// Vanilla colored names retain named colors and explicitly enabled or disabled styles.
    #[test]
    fn custom_name_reads_vanilla_colored_text() -> Result<(), Box<dyn std::error::Error>> {
        for (text, color, style) in [
            ("Archive", "gold", Some(("bold", true))),
            ("Alex fixture", "green", None),
            ("Stored shulker", "aqua", Some(("italic", false))),
        ] {
            let mut compound = NbtCompound::new();
            compound.put_string("text", text.into());
            compound.put_string("color", color.into());
            if let Some((style, enabled)) = style {
                compound.put_bool(style, enabled);
            }
            let original = NbtTag::Compound(compound);
            let name = TextComponent::try_from_nbt(&original)?;
            assert_eq!(CustomNameImpl { name }.write_data(), original);
        }
        Ok(())
    }

    /// Pot decorations preserve all four faces in back, left, right, front order.
    #[test]
    fn pot_decorations_preserve_faces() {
        let expected = NbtTag::List(
            ["angler", "archer", "arms_up", "blade"]
                .into_iter()
                .map(|name| NbtTag::String(format!("minecraft:{name}_pottery_sherd").into()))
                .collect(),
        );
        assert_eq!(
            PotDecorationsImpl::read_data(&expected).map(|value| value.write_data()),
            Some(expected)
        );
    }

    /// Banner tooltip visibility persists alongside the selected hidden component types.
    #[test]
    fn tooltip_display_preserves_hidden_components() {
        let mut compound = NbtCompound::new();
        compound.put_bool("hide_tooltip", true);
        compound.put_list(
            "hidden_components",
            vec![
                NbtTag::String("minecraft:banner_patterns".into()),
                NbtTag::String("minecraft:rarity".into()),
            ],
        );
        let expected = NbtTag::Compound(compound);
        assert_eq!(
            TooltipDisplayImpl::read_data(&expected).map(|value| value.write_data()),
            Some(expected)
        );
    }

    /// Stored item names retain styles and literal text independently of generated translation keys.
    #[test]
    fn item_name_preserves_structured_text() {
        let original = ItemNameImpl::Component(TextComponent::text("Named banner").bold());
        assert_eq!(
            ItemNameImpl::read_data(&original.write_data()),
            Some(original)
        );
        assert_eq!(
            ItemNameImpl::read_data(&NbtTag::String("Literal name".into())),
            Some(ItemNameImpl::Component(TextComponent::text("Literal name")))
        );
    }
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
