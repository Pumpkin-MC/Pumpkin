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
            .to_nbt_tag_for_version(&crate::packet::CURRENT_MC_VERSION)
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
                .to_nbt_tag_for_version(&crate::packet::CURRENT_MC_VERSION),
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

/// A pot face's item template, retaining its full wire count and component patch.
#[derive(Clone)]
pub struct PotDecoration {
    pub item: &'static crate::item::Item,
    pub count: i32,
    pub patch: Vec<(
        crate::data_component::DataComponent,
        Option<Box<dyn DataComponentImpl>>,
    )>,
}

impl PotDecoration {
    /// Reads a registry name or item template, rejecting invalid counts and component patches.
    fn read_data(data: &NbtTag) -> Option<Self> {
        if let NbtTag::String(name) = data {
            let item = crate::item::Item::from_registry_key(name)?;
            if item.id == crate::item::Item::AIR.id {
                return None;
            }
            return Some(Self {
                item,
                count: 1,
                patch: Vec::new(),
            });
        }
        let compound = data.extract_compound()?;
        let item = crate::item::Item::from_registry_key(compound.get_string("id")?)?;
        if item.id == crate::item::Item::AIR.id {
            return None;
        }
        let count = match compound.get("count") {
            Some(count) => count.extract_int()?,
            None => 1,
        };
        if !(1..=99).contains(&count) {
            return None;
        }
        let mut patch = Vec::new();
        if let Some(components) = compound.get("components") {
            let components = components.extract_compound()?;
            let mut seen = [false; 256];
            for (name, value) in &components.child_tags {
                let (name, removed) = name
                    .strip_prefix('!')
                    .map_or((name.as_ref(), false), |name| (name, true));
                let id = crate::data_component::DataComponent::try_from_name(name)?;
                if !Self::is_persistent_component(id) {
                    return None;
                }
                if std::mem::replace(&mut seen[usize::from(id.to_id())], true) {
                    return None;
                }
                let value = if removed {
                    if !value.extract_compound()?.child_tags.is_empty() {
                        return None;
                    }
                    None
                } else {
                    Some(crate::data_component_impl::read_data(id, value)?)
                };
                patch.push((id, value));
            }
        }
        Some(Self { item, count, patch })
    }

    /// Persists a named item template, omitting default count and empty component fields.
    fn write_data(&self) -> NbtTag {
        let mut compound = NbtCompound::new();
        compound.put_string("id", format!("minecraft:{}", self.item.registry_key));
        if self.count != 1 {
            compound.put_int("count", self.count);
        }
        let mut components = NbtCompound::new();
        for (id, value) in &self.patch {
            if Self::is_persistent_component(*id) {
                match value {
                    Some(value) => components.put(id.to_name(), value.write_data()),
                    None => components
                        .put_compound(format!("!{}", id.to_name()).as_str(), NbtCompound::new()),
                }
            }
        }
        if !components.child_tags.is_empty() {
            compound.put_compound("components", components);
        }
        NbtTag::Compound(compound)
    }

    /// Excludes network-only components from template NBT for both additions and removals.
    const fn is_persistent_component(id: crate::data_component::DataComponent) -> bool {
        !matches!(
            id,
            crate::data_component::DataComponent::CreativeSlotLock
                | crate::data_component::DataComponent::AdditionalTradeCost
                | crate::data_component::DataComponent::MapPostProcessing
        )
    }
}

impl std::fmt::Debug for PotDecoration {
    /// Formats the complete persistent template without a runtime item-stack identifier.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.write_data(), formatter)
    }
}

impl PartialEq for PotDecoration {
    /// Compares item, count, and component values independently of patch entry order.
    fn eq(&self, other: &Self) -> bool {
        self.item.id == other.item.id
            && self.count == other.count
            && self.patch.len() == other.patch.len()
            && self.patch.iter().all(|(id, value)| {
                other.patch.iter().any(|(other_id, other_value)| {
                    id == other_id
                        && match (value, other_value) {
                            (Some(value), Some(other_value)) => value.equal(other_value.as_ref()),
                            (None, None) => true,
                            _ => false,
                        }
                })
            })
    }
}

/// The four optional decorated-pot faces, ordered back, left, right, then front.
#[derive(Clone, Debug, PartialEq)]
pub struct PotDecorationsImpl {
    pub decorations: [Option<PotDecoration>; 4],
}
impl PotDecorationsImpl {
    pub const EMPTY: Self = Self {
        decorations: [const { None }; 4],
    };
    const FACES: [&'static str; 4] = ["back", "left", "right", "front"];

    /// Reads named optional templates; absent faces remain distinct from explicit bricks.
    pub fn read_data(data: &NbtTag) -> Option<Self> {
        let compound = data.extract_compound()?;
        let mut decorations = Self::EMPTY.decorations;
        for (face, name) in decorations.iter_mut().zip(Self::FACES) {
            if let Some(data) = compound.get(name) {
                *face = Some(PotDecoration::read_data(data)?);
            }
        }
        Some(Self { decorations })
    }
}
impl DataComponentImpl for PotDecorationsImpl {
    /// Persists present faces as named complete item templates without filling missing faces.
    fn write_data(&self) -> NbtTag {
        let mut compound = NbtCompound::new();
        for (face, name) in self.decorations.iter().zip(Self::FACES) {
            if let Some(face) = face {
                compound.put(name, face.write_data());
            }
        }
        NbtTag::Compound(compound)
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
        let mut faces = NbtCompound::new();
        for (face, name) in [
            ("back", "angler"),
            ("left", "archer"),
            ("right", "arms_up"),
            ("front", "blade"),
        ] {
            let mut template = NbtCompound::new();
            template.put_string("id", format!("minecraft:{name}_pottery_sherd"));
            faces.put_compound(face, template);
        }
        let expected = NbtTag::Compound(faces);
        assert_eq!(
            PotDecorationsImpl::read_data(&expected).map(|value| value.write_data()),
            Some(expected)
        );
    }

    /// Named pot faces preserve complete item templates and distinguish absent faces from bricks.
    #[test]
    fn pot_decorations_read_named_item_templates() {
        let mut patch = NbtCompound::new();
        patch.put_int("minecraft:repair_cost", 7);
        patch.put_compound("!minecraft:custom_name", NbtCompound::new());
        let mut back = NbtCompound::new();
        back.put_string("id", "minecraft:angler_pottery_sherd".into());
        back.put_int("count", 3);
        back.put_compound("components", patch);
        let mut brick = NbtCompound::new();
        brick.put_string("id", "minecraft:brick".into());
        let mut faces = NbtCompound::new();
        faces.put_compound("back", back);
        faces.put_compound("front", brick);
        let expected = NbtTag::Compound(faces);
        assert_eq!(
            PotDecorationsImpl::read_data(&expected).map(|value| value.write_data()),
            Some(expected)
        );
    }

    /// Bare registry names acquire the default count while an empty compound has no decoration faces.
    #[test]
    fn pot_decorations_read_bare_names_and_absent_faces() -> Result<(), Box<dyn std::error::Error>>
    {
        assert_eq!(
            PotDecorationsImpl::EMPTY.write_data(),
            NbtTag::Compound(NbtCompound::new())
        );
        let mut input = NbtCompound::new();
        input.put_string("front", "minecraft:brick".into());
        let decoded =
            PotDecorationsImpl::read_data(&NbtTag::Compound(input)).ok_or("Missing pot faces")?;
        assert!(decoded.decorations[..3].iter().all(Option::is_none));
        let front = decoded.decorations[3].as_ref().ok_or("Missing brick")?;
        assert_eq!(front.item.id, crate::item::Item::BRICK.id);
        assert_eq!(front.count, 1);
        assert!(front.patch.is_empty());
        Ok(())
    }

    /// Invalid persistent counts, unknown items, and malformed component values reject the whole face.
    #[test]
    fn pot_decorations_reject_invalid_templates() {
        for count in [-1, 0, 100, 256, i32::MAX] {
            let mut template = NbtCompound::new();
            template.put_string("id", "minecraft:brick".into());
            template.put_int("count", count);
            let mut faces = NbtCompound::new();
            faces.put_compound("back", template);
            assert!(PotDecorationsImpl::read_data(&NbtTag::Compound(faces)).is_none());
        }
        for value in [
            NbtTag::String("minecraft:unknown_item".into()),
            NbtTag::String("minecraft:air".into()),
            NbtTag::Int(1),
            NbtTag::List(Vec::new()),
        ] {
            let mut faces = NbtCompound::new();
            faces.put("back", value);
            assert!(PotDecorationsImpl::read_data(&NbtTag::Compound(faces)).is_none());
        }
        let mut air = NbtCompound::new();
        air.put_string("id", "minecraft:air".into());
        let mut faces = NbtCompound::new();
        faces.put_compound("back", air);
        assert!(PotDecorationsImpl::read_data(&NbtTag::Compound(faces)).is_none());
        for (name, value) in [
            ("minecraft:unknown_component", NbtTag::Int(1)),
            ("minecraft:repair_cost", NbtTag::String("invalid".into())),
            ("!minecraft:custom_name", NbtTag::Int(1)),
        ] {
            let mut patch = NbtCompound::new();
            patch.put(name, value);
            let mut template = NbtCompound::new();
            template.put_string("id", "minecraft:brick".into());
            template.put_compound("components", patch);
            let mut faces = NbtCompound::new();
            faces.put_compound("back", template);
            assert!(PotDecorationsImpl::read_data(&NbtTag::Compound(faces)).is_none());
        }
    }

    /// Transient component additions and removals are rejected in NBT and omitted from saved templates.
    #[test]
    fn pot_decorations_omit_transient_components_from_nbt() {
        use crate::data_component::DataComponent;
        for id in [
            DataComponent::CreativeSlotLock,
            DataComponent::AdditionalTradeCost,
            DataComponent::MapPostProcessing,
        ] {
            for name in [id.to_name().to_owned(), format!("!{}", id.to_name())] {
                let mut patch = NbtCompound::new();
                patch.put_compound(&name, NbtCompound::new());
                let mut template = NbtCompound::new();
                template.put_string("id", "minecraft:brick".into());
                template.put_compound("components", patch);
                let mut faces = NbtCompound::new();
                faces.put_compound("front", template);
                assert!(PotDecorationsImpl::read_data(&NbtTag::Compound(faces)).is_none());
            }
        }
        let face = PotDecoration {
            item: &crate::item::Item::BRICK,
            count: 1,
            patch: vec![
                (
                    DataComponent::CreativeSlotLock,
                    Some(Box::new(CreativeSlotLockImpl)),
                ),
                (DataComponent::AdditionalTradeCost, None),
                (DataComponent::MapPostProcessing, None),
            ],
        };
        let mut expected = NbtCompound::new();
        expected.put_string("id", "minecraft:brick".into());
        assert_eq!(face.write_data(), NbtTag::Compound(expected));
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
