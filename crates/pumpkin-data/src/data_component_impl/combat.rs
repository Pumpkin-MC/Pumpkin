use super::Digest;
use crate::Block;
use crate::Enchantment;
use crate::attributes::Attributes;
use crate::damage::DamageType;
use crate::data_component_impl::basic::SoundEvent;
use crate::data_component_impl::{
    DataComponentImpl, EquipmentSlot, IDSet, IdOr, get_f32_hash, get_i32_hash, get_idor,
    get_idor_hash, get_idset_hash, get_str_hash, put_idor,
};
use crate::entity_type::EntityType;
use crate::item::Item;
use crate::item_stack::ItemStack;
use crate::sound::Sound;
use crate::tag::Taggable;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use std::borrow::Cow;
use std::hash::Hash;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Operation {
    AddValue = 0,
    AddMultipliedBase = 1,
    AddMultipliedTotal = 2,
}

impl Operation {
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "add_value" => Some(Self::AddValue),
            "add_multiplied_base" => Some(Self::AddMultipliedBase),
            "add_multiplied_total" => Some(Self::AddMultipliedTotal),
            _ => None,
        }
    }

    #[must_use]
    pub const fn to_name(self) -> &'static str {
        match self {
            Self::AddValue => "add_value",
            Self::AddMultipliedBase => "add_multiplied_base",
            Self::AddMultipliedTotal => "add_multiplied_total",
        }
    }

    #[must_use]
    pub const fn from_id(id: i32) -> Option<Self> {
        match id {
            0 => Some(Self::AddValue),
            1 => Some(Self::AddMultipliedBase),
            2 => Some(Self::AddMultipliedTotal),
            _ => None,
        }
    }
}

#[must_use]
pub fn slot_from_name(name: &str) -> Option<crate::AttributeModifierSlot> {
    match name {
        "any" => Some(crate::AttributeModifierSlot::Any),
        "mainhand" => Some(crate::AttributeModifierSlot::MainHand),
        "offhand" => Some(crate::AttributeModifierSlot::OffHand),
        "hand" => Some(crate::AttributeModifierSlot::Hand),
        "feet" => Some(crate::AttributeModifierSlot::Feet),
        "legs" => Some(crate::AttributeModifierSlot::Legs),
        "chest" => Some(crate::AttributeModifierSlot::Chest),
        "head" => Some(crate::AttributeModifierSlot::Head),
        "armor" => Some(crate::AttributeModifierSlot::Armor),
        "body" => Some(crate::AttributeModifierSlot::Body),
        "saddle" => Some(crate::AttributeModifierSlot::Saddle),
        _ => None,
    }
}

#[must_use]
pub const fn slot_to_name(slot: &crate::AttributeModifierSlot) -> &'static str {
    match slot {
        crate::AttributeModifierSlot::Any => "any",
        crate::AttributeModifierSlot::MainHand => "mainhand",
        crate::AttributeModifierSlot::OffHand => "offhand",
        crate::AttributeModifierSlot::Hand => "hand",
        crate::AttributeModifierSlot::Feet => "feet",
        crate::AttributeModifierSlot::Legs => "legs",
        crate::AttributeModifierSlot::Chest => "chest",
        crate::AttributeModifierSlot::Head => "head",
        crate::AttributeModifierSlot::Armor => "armor",
        crate::AttributeModifierSlot::Body => "body",
        crate::AttributeModifierSlot::Saddle => "saddle",
    }
}

#[must_use]
pub const fn slot_from_id(id: i32) -> Option<crate::AttributeModifierSlot> {
    match id {
        0 => Some(crate::AttributeModifierSlot::Any),
        1 => Some(crate::AttributeModifierSlot::MainHand),
        2 => Some(crate::AttributeModifierSlot::OffHand),
        3 => Some(crate::AttributeModifierSlot::Hand),
        4 => Some(crate::AttributeModifierSlot::Feet),
        5 => Some(crate::AttributeModifierSlot::Legs),
        6 => Some(crate::AttributeModifierSlot::Chest),
        7 => Some(crate::AttributeModifierSlot::Head),
        8 => Some(crate::AttributeModifierSlot::Armor),
        9 => Some(crate::AttributeModifierSlot::Body),
        10 => Some(crate::AttributeModifierSlot::Saddle),
        _ => None,
    }
}

#[must_use]
pub const fn slot_to_id(slot: &crate::AttributeModifierSlot) -> i32 {
    match slot {
        crate::AttributeModifierSlot::Any => 0,
        crate::AttributeModifierSlot::MainHand => 1,
        crate::AttributeModifierSlot::OffHand => 2,
        crate::AttributeModifierSlot::Hand => 3,
        crate::AttributeModifierSlot::Feet => 4,
        crate::AttributeModifierSlot::Legs => 5,
        crate::AttributeModifierSlot::Chest => 6,
        crate::AttributeModifierSlot::Head => 7,
        crate::AttributeModifierSlot::Armor => 8,
        crate::AttributeModifierSlot::Body => 9,
        crate::AttributeModifierSlot::Saddle => 10,
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub enum ModifierDisplay {
    #[default]
    Default,
    Hidden,
    Override(pumpkin_util::text::TextComponent),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Modifier {
    pub r#type: &'static Attributes,
    pub id: Cow<'static, str>,
    pub amount: f64,
    pub operation: Operation,
    pub slot: crate::AttributeModifierSlot,
    pub display: ModifierDisplay,
}
impl Hash for Modifier {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.r#type.hash(state);
        self.id.hash(state);
        self.amount.to_bits().hash(state);
        self.operation.hash(state);
        self.slot.hash(state);
        self.display.hash(state);
    }
}

#[must_use]
pub fn attribute_from_name(name: &str) -> Option<&'static Attributes> {
    let stripped = name.strip_prefix("minecraft:").unwrap_or(name);
    Attributes::ALL
        .iter()
        .find(|attr| attr.name == name || attr.name.strip_prefix("minecraft:") == Some(stripped))
}

#[must_use]
pub fn attribute_from_id(id: u8) -> Option<&'static Attributes> {
    Attributes::ALL.iter().find(|attr| attr.id == id)
}

#[derive(Clone, Debug, Hash, PartialEq)]
pub struct AttributeModifiersImpl {
    pub attribute_modifiers: Cow<'static, [Modifier]>,
}
impl AttributeModifiersImpl {
    pub fn read_data(data: &NbtTag) -> Option<Self> {
        let list = match data {
            NbtTag::List(list) => list.as_slice(),
            NbtTag::Compound(c) => c.get_list("modifiers")?,
            _ => return None,
        };
        let mut modifiers = Vec::with_capacity(list.len());
        for entry in list {
            let comp = entry.extract_compound()?;
            let type_name = comp.get_string("type")?;
            let r#type = attribute_from_name(type_name)?;
            let id = Cow::Owned(comp.get_string("id")?.to_string());
            let amount = comp
                .get_double("amount")
                .or_else(|| comp.get_float("amount").map(f64::from))?;
            let operation = Operation::from_name(comp.get_string("operation")?)?;
            let slot = comp
                .get_string("slot")
                .and_then(slot_from_name)
                .unwrap_or(crate::AttributeModifierSlot::Any);
            let display = if let Some(disp_comp) = comp.get_compound("display") {
                match disp_comp.get_string("type") {
                    Some("hidden") => ModifierDisplay::Hidden,
                    Some("override") => {
                        disp_comp
                            .get("value")
                            .map_or(ModifierDisplay::Default, |v| {
                                ModifierDisplay::Override(
                                    pumpkin_util::text::TextComponent::from_nbt(v),
                                )
                            })
                    }
                    _ => ModifierDisplay::Default,
                }
            } else {
                ModifierDisplay::Default
            };
            modifiers.push(Modifier {
                r#type,
                id,
                amount,
                operation,
                slot,
                display,
            });
        }
        Some(Self {
            attribute_modifiers: Cow::Owned(modifiers),
        })
    }
}
impl DataComponentImpl for AttributeModifiersImpl {
    fn write_data(&self) -> NbtTag {
        let mut list = Vec::with_capacity(self.attribute_modifiers.len());
        for m in self.attribute_modifiers.iter() {
            let mut comp = NbtCompound::new();
            comp.put_string("type", m.r#type.name.to_string());
            comp.put_string("id", m.id.to_string());
            comp.put_double("amount", m.amount);
            comp.put_string("operation", m.operation.to_name().to_string());
            if !matches!(m.slot, crate::AttributeModifierSlot::Any) {
                comp.put_string("slot", slot_to_name(&m.slot).to_string());
            }
            match &m.display {
                ModifierDisplay::Default => {}
                ModifierDisplay::Hidden => {
                    let mut disp = NbtCompound::new();
                    disp.put_string("type", "hidden".to_string());
                    comp.put_compound("display", disp);
                }
                ModifierDisplay::Override(text) => {
                    let mut disp = NbtCompound::new();
                    disp.put_string("type", "override".to_string());
                    disp.put_string("value", text.clone().get_text());
                    comp.put_compound("display", disp);
                }
            }
            list.push(NbtTag::Compound(comp));
        }
        NbtTag::List(list)
    }
    default_impl!(AttributeModifiers);
}

/// Vanilla `ItemEnchantments.LEVEL_CODEC` bounds encoded levels to 1..=255.
pub const MAX_ENCHANTMENT_LEVEL: i32 = 255;

/// Reads the `ItemEnchantments` map shared by `enchantments` and
/// `stored_enchantments`, rejecting levels the client refuses to decode.
fn read_enchantment_levels(data: &NbtTag) -> Option<Vec<(&'static Enchantment, i32)>> {
    let compound = data.extract_compound()?;
    let levels = if let Some(NbtTag::Compound(levels)) = compound.child_tags.get("levels") {
        &levels.child_tags
    } else {
        &compound.child_tags
    };
    let mut enchantments = Vec::with_capacity(levels.len());
    for (name, level) in levels {
        let enchantment = Enchantment::from_name(name.as_ref())
            .or_else(|| Enchantment::from_name(&format!("minecraft:{name}")))?;
        let level = level.extract_int()?;
        if !(1..=MAX_ENCHANTMENT_LEVEL).contains(&level) {
            return None;
        }
        enchantments.push((enchantment, level));
    }
    Some(enchantments)
}

#[derive(Clone, Hash, PartialEq, Eq, Default)]
pub struct EnchantmentsImpl {
    pub enchantment: Cow<'static, [(&'static Enchantment, i32)]>,
}
impl EnchantmentsImpl {
    pub fn read_data(data: &NbtTag) -> Option<Self> {
        Some(Self {
            enchantment: Cow::from(read_enchantment_levels(data)?),
        })
    }
}
impl DataComponentImpl for EnchantmentsImpl {
    fn write_data(&self) -> NbtTag {
        let mut data = NbtCompound::new();
        for (enc, level) in self.enchantment.iter() {
            data.put_int(enc.name, *level);
        }
        NbtTag::Compound(data)
    }
    fn get_hash(&self) -> i32 {
        let mut digest = Digest::new();
        digest.update(&[2u8]);
        for (enc, level) in self.enchantment.iter() {
            digest.update(&get_str_hash(enc.name).to_le_bytes());
            digest.update(&get_i32_hash(*level).to_le_bytes());
        }
        digest.update(&[3u8]);
        digest.finalize() as i32
    }
    default_impl!(Enchantments);
}

fn block_predicate_matches(predicate: &NbtTag, block: &Block) -> bool {
    match predicate {
        NbtTag::Compound(c) => {
            if let Some(list) = c.get_list("predicates") {
                return list.iter().any(|p| block_predicate_matches(p, block));
            }
            if let Some(blocks_tag) = c.get("blocks") {
                IDSet::<Block>::read(blocks_tag).is_some_and(|idset| match &idset {
                    IDSet::Tag(tag) => block.is_tagged_with(tag).unwrap_or(false),
                    IDSet::IDs(blocks) => blocks.iter().any(|b| b.id == block.id),
                })
            } else {
                true
            }
        }
        NbtTag::List(list) => list.iter().any(|p| block_predicate_matches(p, block)),
        _ => false,
    }
}

/// An adventure-mode block predicate, kept as its raw NBT (a compound or list)
/// since Pumpkin does not yet model block predicates.
// TODO: replace `predicate` with a typed block predicate once block predicates are modelled.
#[derive(Clone, Debug, PartialEq)]
pub struct CanPlaceOnImpl {
    pub predicate: NbtTag,
}
impl CanPlaceOnImpl {
    pub fn read_data(data: &NbtTag) -> Option<Self> {
        Some(Self {
            predicate: data.clone(),
        })
    }

    #[must_use]
    pub fn matches_block(&self, block: &Block) -> bool {
        block_predicate_matches(&self.predicate, block)
    }
}
impl DataComponentImpl for CanPlaceOnImpl {
    fn write_data(&self) -> NbtTag {
        self.predicate.clone()
    }
    default_impl!(CanPlaceOn);
}

/// An adventure-mode block predicate, kept as its raw NBT (a compound or list)
/// since Pumpkin does not yet model block predicates.
// TODO: replace `predicate` with a typed block predicate once block predicates are modelled.
#[derive(Clone, Debug, PartialEq)]
pub struct CanBreakImpl {
    pub predicate: NbtTag,
}
impl CanBreakImpl {
    pub fn read_data(data: &NbtTag) -> Option<Self> {
        Some(Self {
            predicate: data.clone(),
        })
    }

    #[must_use]
    pub fn matches_block(&self, block: &Block) -> bool {
        block_predicate_matches(&self.predicate, block)
    }
}
impl DataComponentImpl for CanBreakImpl {
    fn write_data(&self) -> NbtTag {
        self.predicate.clone()
    }
    default_impl!(CanBreak);
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Default)]
pub struct RepairCostImpl {
    pub cost: i32,
}
impl RepairCostImpl {
    pub const DEFAULT: Self = Self { cost: 0 };

    pub fn read_data(data: &NbtTag) -> Option<Self> {
        Some(Self {
            cost: data.extract_int()?,
        })
    }
}
impl DataComponentImpl for RepairCostImpl {
    fn write_data(&self) -> NbtTag {
        NbtTag::Int(self.cost)
    }
    default_impl!(RepairCost);
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct IntangibleProjectileImpl;
impl IntangibleProjectileImpl {
    pub const fn read_data(_data: &NbtTag) -> Option<Self> {
        Some(Self)
    }
}
impl DataComponentImpl for IntangibleProjectileImpl {
    fn write_data(&self) -> NbtTag {
        NbtTag::Compound(NbtCompound::new())
    }
    default_impl!(IntangibleProjectile);
}

#[derive(Clone, Debug, PartialEq)]
pub struct DamageResistantImpl {
    pub types: IDSet<DamageType>,
}
impl Hash for DamageResistantImpl {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        get_idset_hash(&self.types).hash(state);
    }
}
impl DamageResistantImpl {
    pub fn read_data(data: &NbtTag) -> Option<Self> {
        let compound = data.extract_compound()?;
        let types = IDSet::<DamageType>::read(compound.get("types")?)?;
        Some(Self { types })
    }
}
impl DataComponentImpl for DamageResistantImpl {
    fn write_data(&self) -> NbtTag {
        let mut compound = NbtCompound::new();
        self.types.write(&mut compound, "types");
        NbtTag::Compound(compound)
    }
    fn get_hash(&self) -> i32 {
        get_idset_hash(&self.types) as i32
    }
    default_impl!(DamageResistant);
}

#[derive(Clone, PartialEq)]
pub struct ToolRule {
    pub blocks: IDSet<Block>,
    pub speed: Option<f32>,
    pub correct_for_drops: Option<bool>,
}
impl Hash for ToolRule {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.blocks.hash(state);
        if let Some(val) = self.speed {
            true.hash(state);
            unsafe { (*(&raw const val).cast::<u32>()).hash(state) };
        } else {
            false.hash(state);
        }
        self.correct_for_drops.hash(state);
    }
}

#[derive(Clone, PartialEq)]
pub struct ToolImpl {
    pub rules: Cow<'static, [ToolRule]>,
    pub default_mining_speed: f32,
    pub damage_per_block: u32,
    pub can_destroy_blocks_in_creative: bool,
}
impl ToolImpl {
    pub fn read_data(data: &NbtTag) -> Option<Self> {
        let compound = data.extract_compound()?;
        let mut rules = Vec::new();
        if let Some(list) = compound.get_list("rules") {
            for rule_tag in list {
                if let Some(rule_compound) = rule_tag.extract_compound()
                    && let Some(blocks_tag) = rule_compound.get("blocks")
                    && let Some(blocks) = IDSet::<Block>::read(blocks_tag)
                {
                    rules.push(ToolRule {
                        blocks,
                        speed: rule_compound.get_float("speed"),
                        correct_for_drops: rule_compound.get_bool("correct_for_drops"),
                    });
                }
            }
        }
        let default_mining_speed = compound.get_float("default_mining_speed").unwrap_or(1.0);
        let damage_per_block = compound.get_int("damage_per_block").unwrap_or(1).max(0) as u32;
        let can_destroy_blocks_in_creative = compound
            .get_bool("can_destroy_blocks_in_creative")
            .unwrap_or(true);
        Some(Self {
            rules: Cow::Owned(rules),
            default_mining_speed,
            damage_per_block,
            can_destroy_blocks_in_creative,
        })
    }
}
impl DataComponentImpl for ToolImpl {
    fn write_data(&self) -> NbtTag {
        let mut compound = NbtCompound::new();
        let mut rules_list = Vec::new();
        for rule in self.rules.iter() {
            let mut rule_compound = NbtCompound::new();
            rule.blocks.write(&mut rule_compound, "blocks");
            if let Some(speed) = rule.speed {
                rule_compound.put_float("speed", speed);
            }
            if let Some(correct_for_drops) = rule.correct_for_drops {
                rule_compound.put_bool("correct_for_drops", correct_for_drops);
            }
            rules_list.push(NbtTag::Compound(rule_compound));
        }
        compound.put_list("rules", rules_list);
        compound.put_float("default_mining_speed", self.default_mining_speed);
        compound.put_int("damage_per_block", self.damage_per_block as i32);
        compound.put_bool(
            "can_destroy_blocks_in_creative",
            self.can_destroy_blocks_in_creative,
        );
        NbtTag::Compound(compound)
    }
    default_impl!(Tool);
}
impl Hash for ToolImpl {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.rules.hash(state);
        unsafe { (*(&raw const self.default_mining_speed).cast::<u32>()).hash(state) };
        self.damage_per_block.hash(state);
        self.can_destroy_blocks_in_creative.hash(state);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WeaponImpl {
    pub item_damage_per_attack: u32,
    pub disable_blocking_for_seconds: f32,
}
impl Hash for WeaponImpl {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.item_damage_per_attack.hash(state);
        self.disable_blocking_for_seconds.to_bits().hash(state);
    }
}
impl WeaponImpl {
    pub fn read_data(data: &NbtTag) -> Option<Self> {
        let compound = data.extract_compound()?;
        let item_damage_per_attack = compound
            .get_int("item_damage_per_attack")
            .unwrap_or(1)
            .max(0) as u32;
        let disable_blocking_for_seconds = compound
            .get_float("disable_blocking_for_seconds")
            .unwrap_or(0.0);
        Some(Self {
            item_damage_per_attack,
            disable_blocking_for_seconds,
        })
    }
}
impl DataComponentImpl for WeaponImpl {
    fn write_data(&self) -> NbtTag {
        let mut compound = NbtCompound::new();
        compound.put_int("item_damage_per_attack", self.item_damage_per_attack as i32);
        compound.put_float(
            "disable_blocking_for_seconds",
            self.disable_blocking_for_seconds,
        );
        NbtTag::Compound(compound)
    }
    fn get_hash(&self) -> i32 {
        let mut digest = Digest::new();
        digest.update(&get_i32_hash(self.item_damage_per_attack as i32).to_le_bytes());
        digest.update(&get_f32_hash(self.disable_blocking_for_seconds).to_le_bytes());
        digest.finalize() as i32
    }
    default_impl!(Weapon);
}

#[derive(Clone, Debug, PartialEq)]
pub struct AttackRangeImpl {
    pub min_reach: f32,
    pub max_reach: f32,
    pub min_creative_reach: f32,
    pub max_creative_reach: f32,
    pub hitbox_margin: f32,
    pub mob_factor: f32,
}
impl AttackRangeImpl {
    pub fn read_data(data: &NbtTag) -> Option<Self> {
        let compound = data.extract_compound()?;
        Some(Self {
            min_reach: compound.get_float("min_reach").unwrap_or(0.0),
            max_reach: compound.get_float("max_reach").unwrap_or(3.0),
            min_creative_reach: compound.get_float("min_creative_reach").unwrap_or(0.0),
            max_creative_reach: compound.get_float("max_creative_reach").unwrap_or(5.0),
            hitbox_margin: compound.get_float("hitbox_margin").unwrap_or(0.3),
            mob_factor: compound.get_float("mob_factor").unwrap_or(1.0),
        })
    }
    fn values(&self) -> [f32; 6] {
        [
            self.min_reach,
            self.max_reach,
            self.min_creative_reach,
            self.max_creative_reach,
            self.hitbox_margin,
            self.mob_factor,
        ]
    }
}
impl DataComponentImpl for AttackRangeImpl {
    fn write_data(&self) -> NbtTag {
        let mut compound = NbtCompound::new();
        compound.put_float("min_reach", self.min_reach);
        compound.put_float("max_reach", self.max_reach);
        compound.put_float("min_creative_reach", self.min_creative_reach);
        compound.put_float("max_creative_reach", self.max_creative_reach);
        compound.put_float("hitbox_margin", self.hitbox_margin);
        compound.put_float("mob_factor", self.mob_factor);
        NbtTag::Compound(compound)
    }
    fn get_hash(&self) -> i32 {
        let mut digest = Digest::new();
        for value in self.values() {
            digest.update(&get_f32_hash(value).to_le_bytes());
        }
        digest.finalize() as i32
    }
    default_impl!(AttackRange);
}
impl Hash for AttackRangeImpl {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for value in self.values() {
            value.to_bits().hash(state);
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct EnchantableImpl {
    pub value: i32,
}
impl EnchantableImpl {
    pub fn read_data(data: &NbtTag) -> Option<Self> {
        let compound = data.extract_compound()?;
        let value = compound.get_int("value")?;
        Some(Self { value })
    }
}
impl DataComponentImpl for EnchantableImpl {
    fn write_data(&self) -> NbtTag {
        let mut compound = NbtCompound::new();
        compound.put_int("value", self.value);
        NbtTag::Compound(compound)
    }
    default_impl!(Enchantable);
}

#[derive(Clone, Hash, PartialEq)]
pub struct EquippableImpl {
    pub slot: &'static EquipmentSlot,
    pub equip_sound: IdOr<SoundEvent>,
    pub asset_id: Option<Cow<'static, str>>,
    pub camera_overlay: Option<Cow<'static, str>>,
    pub allowed_entities: Option<IDSet<EntityType>>,
    pub dispensable: bool,
    pub swappable: bool,
    pub damage_on_hurt: bool,
    pub equip_on_interact: bool,
    pub can_be_sheared: bool,
    pub shearing_sound: IdOr<SoundEvent>,
}
impl EquippableImpl {
    pub fn read_data(data: &NbtTag) -> Option<Self> {
        let compound = data.extract_compound()?;
        let slot = EquipmentSlot::get_from_name(compound.get_string("slot")?)?;
        let asset_id = compound
            .get_string("asset_id")
            .map(|str| Cow::Owned(str.to_owned()));
        let camera_overlay = compound
            .get_string("camera_overlay")
            .map(|str| Cow::Owned(str.to_owned()));
        let dispensable = compound.get_bool("dispensable").unwrap_or(true);
        let swappable = compound.get_bool("swappable").unwrap_or(true);
        let damage_on_hurt = compound.get_bool("damage_on_hurt").unwrap_or(true);
        let equip_on_interact = compound.get_bool("equip_on_interact").unwrap_or(false);
        let can_be_sheared = compound.get_bool("can_be_sheared").unwrap_or(false);
        let equip_sound = get_idor(compound, "equip_sound", Sound::ItemArmorEquipGeneric);
        let shearing_sound = get_idor(compound, "shearing_sound", Sound::ItemShearsSnip);
        let allowed_entities = if let Some(nbt) = compound.get("allowed_entities") {
            IDSet::<EntityType>::read(nbt)
        } else {
            None
        };
        Some(Self {
            slot,
            equip_sound,
            asset_id,
            camera_overlay,
            allowed_entities,
            dispensable,
            swappable,
            damage_on_hurt,
            equip_on_interact,
            can_be_sheared,
            shearing_sound,
        })
    }
}
impl DataComponentImpl for EquippableImpl {
    fn get_hash(&self) -> i32 {
        let mut digest = Digest::new();
        digest.update(&[16u8]);
        digest.update(&get_i32_hash(self.slot.get_slot_index()).to_le_bytes());
        digest.update(&get_idor_hash(&self.equip_sound).to_le_bytes());
        if let Some(asset) = &self.asset_id {
            digest.update(&[1u8]);
            digest.update(&get_str_hash(asset).to_le_bytes());
        }
        if let Some(overlay) = &self.camera_overlay {
            digest.update(&[2u8]);
            digest.update(&get_str_hash(overlay).to_le_bytes());
        }
        if let Some(allowed_entities) = &self.allowed_entities {
            digest.update(&[3u8]);
            digest.update(&get_idset_hash(allowed_entities).to_le_bytes());
        }
        digest.update(&[self.dispensable as u8]);
        digest.update(&[self.swappable as u8]);
        digest.update(&[self.damage_on_hurt as u8]);
        digest.update(&[self.equip_on_interact as u8]);
        digest.update(&[self.can_be_sheared as u8]);
        digest.update(&get_idor_hash(&self.shearing_sound).to_le_bytes());
        digest.finalize() as i32
    }
    fn write_data(&self) -> NbtTag {
        let mut compound = NbtCompound::new();
        compound.put_string("slot", self.slot.to_name().to_string());
        put_idor(&mut compound, "equip_sound", &self.equip_sound);
        put_idor(&mut compound, "shearing_sound", &self.shearing_sound);
        if let Some(asset_id) = &self.asset_id {
            compound.put_string("asset_id", asset_id.to_string());
        }
        if let Some(camera_overlay) = &self.camera_overlay {
            compound.put_string("camera_overlay", camera_overlay.to_string());
        }
        if let Some(allowed_entities) = &self.allowed_entities {
            allowed_entities.write(&mut compound, "allowed_entities");
        }
        compound.put_bool("dispensable", self.dispensable);
        compound.put_bool("swappable", self.swappable);
        compound.put_bool("damage_on_hurt", self.damage_on_hurt);
        compound.put_bool("equip_on_interact", self.equip_on_interact);
        compound.put_bool("can_be_sheared", self.can_be_sheared);
        NbtTag::Compound(compound)
    }
    default_impl!(Equippable);
}

#[derive(Clone, Debug, PartialEq)]
pub struct RepairableImpl {
    pub items: IDSet<Item>,
}

impl Hash for RepairableImpl {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        get_idset_hash(&self.items).hash(state);
    }
}

impl RepairableImpl {
    #[must_use]
    pub fn is_valid_repair_item(&self, repair_item: &ItemStack) -> bool {
        if repair_item.is_empty() {
            return false;
        }
        match &self.items {
            IDSet::Tag(tag) => repair_item.item.is_tagged_with(tag).unwrap_or(false),
            IDSet::IDs(items) => items.iter().any(|item| item.id == repair_item.item.id),
        }
    }

    pub fn read_data(data: &NbtTag) -> Option<Self> {
        if let NbtTag::Compound(c) = data {
            let items_tag = c.get("items")?;
            let items = IDSet::read(items_tag)?;
            Some(Self { items })
        } else if let Some(items) = IDSet::read(data) {
            Some(Self { items })
        } else {
            None
        }
    }
}

impl DataComponentImpl for RepairableImpl {
    fn write_data(&self) -> NbtTag {
        let mut compound = NbtCompound::new();
        self.items.write(&mut compound, "items");
        NbtTag::Compound(compound)
    }

    default_impl!(Repairable);
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct GliderImpl;
impl GliderImpl {
    pub const fn read_data(_data: &NbtTag) -> Option<Self> {
        Some(Self)
    }
}
impl DataComponentImpl for GliderImpl {
    fn write_data(&self) -> NbtTag {
        NbtTag::Compound(NbtCompound::new())
    }
    default_impl!(Glider);
}

#[derive(Clone, Debug, Hash, PartialEq)]
pub struct DeathProtectionImpl {
    pub death_effects: Cow<'static, [crate::data_component_impl::food::ConsumeEffect]>,
}
impl DeathProtectionImpl {
    pub const EMPTY: Self = Self {
        death_effects: Cow::Borrowed(&[]),
    };

    pub fn read_data(data: &NbtTag) -> Option<Self> {
        let Some(compound) = data.extract_compound() else {
            return Some(Self::EMPTY);
        };
        let death_effects = compound
            .get_list("death_effects")
            .map(|list| {
                list.iter()
                    .filter_map(crate::data_component_impl::food::ConsumeEffect::read_data)
                    .collect::<Vec<_>>()
            })
            .map_or(Cow::Borrowed(&[][..]), Cow::Owned);
        Some(Self { death_effects })
    }
}
impl DataComponentImpl for DeathProtectionImpl {
    fn write_data(&self) -> NbtTag {
        let mut compound = NbtCompound::new();
        if !self.death_effects.is_empty() {
            compound.put_list(
                "death_effects",
                self.death_effects.iter().map(|e| e.as_nbt()).collect(),
            );
        }
        NbtTag::Compound(compound)
    }
    fn get_hash(&self) -> i32 {
        let mut digest = Digest::new();
        for effect in self.death_effects.iter() {
            digest.update(&effect.get_hash().to_le_bytes());
        }
        digest.finalize() as i32
    }
    default_impl!(DeathProtection);
}

#[derive(Clone, Debug, PartialEq)]
pub struct DamageReduction {
    pub horizontal_blocking_angle: f32,
    pub damage_type: Option<IDSet<DamageType>>,
    pub base: f32,
    pub factor: f32,
}
impl Hash for DamageReduction {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.horizontal_blocking_angle.to_bits().hash(state);
        self.damage_type.as_ref().map(get_idset_hash).hash(state);
        self.base.to_bits().hash(state);
        self.factor.to_bits().hash(state);
    }
}
impl DamageReduction {
    pub const DEFAULT: Self = Self {
        horizontal_blocking_angle: 90.0,
        damage_type: None,
        base: 0.0,
        factor: 1.0,
    };

    #[must_use]
    pub fn resolve(&self, damage_type: &DamageType, dealt_damage: f32, angle_rad: f64) -> f32 {
        let max_rad = f64::from((std::f32::consts::PI / 180.0) * self.horizontal_blocking_angle);
        if angle_rad > max_rad {
            return 0.0;
        }
        if let Some(types) = &self.damage_type
            && !types.contains(damage_type)
        {
            return 0.0;
        }
        (self.base + self.factor * dealt_damage).clamp(0.0, dealt_damage)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ItemDamageFunction {
    pub threshold: f32,
    pub base: f32,
    pub factor: f32,
}
impl Hash for ItemDamageFunction {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.threshold.to_bits().hash(state);
        self.base.to_bits().hash(state);
        self.factor.to_bits().hash(state);
    }
}
impl ItemDamageFunction {
    pub const DEFAULT: Self = Self {
        threshold: 1.0,
        base: 0.0,
        factor: 1.0,
    };

    #[must_use]
    pub fn apply(&self, dealt_damage: f32) -> i32 {
        if dealt_damage < self.threshold {
            0
        } else {
            (self.base + self.factor * dealt_damage).floor() as i32
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BlocksAttacksImpl {
    pub block_delay_seconds: f32,
    pub disable_cooldown_scale: f32,
    pub damage_reductions: Cow<'static, [DamageReduction]>,
    pub item_damage: ItemDamageFunction,
    pub bypassed_by: Option<IDSet<DamageType>>,
    pub block_sound: Option<IdOr<SoundEvent>>,
    pub disabled_sound: Option<IdOr<SoundEvent>>,
}
impl Hash for BlocksAttacksImpl {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.block_delay_seconds.to_bits().hash(state);
        self.disable_cooldown_scale.to_bits().hash(state);
        self.damage_reductions.hash(state);
        self.item_damage.hash(state);
        self.bypassed_by.as_ref().map(get_idset_hash).hash(state);
        self.block_sound.hash(state);
        self.disabled_sound.hash(state);
    }
}
impl BlocksAttacksImpl {
    pub const DEFAULT_DAMAGE_REDUCTIONS: [DamageReduction; 1] = [DamageReduction::DEFAULT];

    pub const DEFAULT: Self = Self {
        block_delay_seconds: 0.0,
        disable_cooldown_scale: 1.0,
        damage_reductions: Cow::Borrowed(&Self::DEFAULT_DAMAGE_REDUCTIONS),
        item_damage: ItemDamageFunction::DEFAULT,
        bypassed_by: None,
        block_sound: None,
        disabled_sound: None,
    };

    #[must_use]
    pub fn block_delay_ticks(&self) -> i32 {
        (self.block_delay_seconds * 20.0).round() as i32
    }

    #[must_use]
    pub fn disable_blocking_for_ticks(&self, base_seconds: f32) -> i32 {
        let seconds = base_seconds * self.disable_cooldown_scale;
        if seconds > 0.0 {
            (seconds * 20.0).round() as i32
        } else {
            0
        }
    }

    #[must_use]
    pub fn resolve_blocked_damage(
        &self,
        damage_type: &DamageType,
        dealt_damage: f32,
        angle_rad: f64,
    ) -> f32 {
        let mut blocked = 0.0f32;
        for reduction in self.damage_reductions.iter() {
            blocked += reduction.resolve(damage_type, dealt_damage, angle_rad);
        }
        blocked.clamp(0.0, dealt_damage)
    }

    pub fn read_data(data: &NbtTag) -> Option<Self> {
        let Some(compound) = data.extract_compound() else {
            return Some(Self::DEFAULT);
        };
        let block_delay_seconds = compound
            .get_float("block_delay_seconds")
            .unwrap_or(0.0)
            .max(0.0);
        let disable_cooldown_scale = compound
            .get_float("disable_cooldown_scale")
            .unwrap_or(1.0)
            .max(0.0);
        let damage_reductions = if let Some(list) = compound.get_list("damage_reductions") {
            let mut reductions = Vec::with_capacity(list.len());
            for item in list {
                let c = item.extract_compound()?;
                reductions.push(DamageReduction {
                    horizontal_blocking_angle: c
                        .get_float("horizontal_blocking_angle")
                        .unwrap_or(90.0),
                    damage_type: c.get("type").and_then(IDSet::<DamageType>::read),
                    base: c.get_float("base")?,
                    factor: c.get_float("factor")?,
                });
            }
            Cow::Owned(reductions)
        } else {
            Cow::Borrowed(&Self::DEFAULT_DAMAGE_REDUCTIONS[..])
        };
        let item_damage = if let Some(c) = compound.get_compound("item_damage") {
            ItemDamageFunction {
                threshold: c.get_float("threshold")?.max(0.0),
                base: c.get_float("base")?,
                factor: c.get_float("factor")?,
            }
        } else {
            ItemDamageFunction::DEFAULT
        };
        let bypassed_by = compound
            .get("bypassed_by")
            .and_then(IDSet::<DamageType>::read);
        let block_sound = get_optional_idor(compound, "block_sound");
        let disabled_sound = get_optional_idor(compound, "disabled_sound");
        Some(Self {
            block_delay_seconds,
            disable_cooldown_scale,
            damage_reductions,
            item_damage,
            bypassed_by,
            block_sound,
            disabled_sound,
        })
    }
}
impl DataComponentImpl for BlocksAttacksImpl {
    fn write_data(&self) -> NbtTag {
        let mut compound = NbtCompound::new();
        compound.put_float("block_delay_seconds", self.block_delay_seconds);
        compound.put_float("disable_cooldown_scale", self.disable_cooldown_scale);
        let reductions = self
            .damage_reductions
            .iter()
            .map(|r| {
                let mut rc = NbtCompound::new();
                rc.put_float("horizontal_blocking_angle", r.horizontal_blocking_angle);
                if let Some(dt) = &r.damage_type {
                    dt.write(&mut rc, "type");
                }
                rc.put_float("base", r.base);
                rc.put_float("factor", r.factor);
                NbtTag::Compound(rc)
            })
            .collect();
        compound.put_list("damage_reductions", reductions);
        let mut id_comp = NbtCompound::new();
        id_comp.put_float("threshold", self.item_damage.threshold);
        id_comp.put_float("base", self.item_damage.base);
        id_comp.put_float("factor", self.item_damage.factor);
        compound.put_compound("item_damage", id_comp);
        if let Some(bypassed) = &self.bypassed_by {
            bypassed.write(&mut compound, "bypassed_by");
        }
        put_optional_idor(&mut compound, "block_sound", self.block_sound.as_ref());
        put_optional_idor(
            &mut compound,
            "disabled_sound",
            self.disabled_sound.as_ref(),
        );
        NbtTag::Compound(compound)
    }
    fn get_hash(&self) -> i32 {
        let mut digest = Digest::new();
        digest.update(&get_f32_hash(self.block_delay_seconds).to_le_bytes());
        digest.update(&get_f32_hash(self.disable_cooldown_scale).to_le_bytes());
        for r in self.damage_reductions.iter() {
            digest.update(&get_f32_hash(r.horizontal_blocking_angle).to_le_bytes());
            if let Some(dt) = &r.damage_type {
                digest.update(&[1u8]);
                digest.update(&get_idset_hash(dt).to_le_bytes());
            } else {
                digest.update(&[0u8]);
            }
            digest.update(&get_f32_hash(r.base).to_le_bytes());
            digest.update(&get_f32_hash(r.factor).to_le_bytes());
        }
        digest.update(&get_f32_hash(self.item_damage.threshold).to_le_bytes());
        digest.update(&get_f32_hash(self.item_damage.base).to_le_bytes());
        digest.update(&get_f32_hash(self.item_damage.factor).to_le_bytes());
        if let Some(bypassed) = &self.bypassed_by {
            digest.update(&[1u8]);
            digest.update(&get_idset_hash(bypassed).to_le_bytes());
        } else {
            digest.update(&[0u8]);
        }
        update_optional_idor(&mut digest, self.block_sound.as_ref());
        update_optional_idor(&mut digest, self.disabled_sound.as_ref());
        digest.finalize() as i32
    }
    default_impl!(BlocksAttacks);
}

fn get_optional_idor(compound: &NbtCompound, key: &str) -> Option<IdOr<SoundEvent>> {
    compound
        .get(key)
        .map(|_| get_idor(compound, key, Sound::IntentionallyEmpty))
}

fn put_optional_idor(compound: &mut NbtCompound, key: &str, sound: Option<&IdOr<SoundEvent>>) {
    if let Some(sound) = sound {
        put_idor(compound, key, sound);
    }
}

fn update_optional_idor(digest: &mut Digest, sound: Option<&IdOr<SoundEvent>>) {
    if let Some(sound) = sound {
        digest.update(&[1u8]);
        digest.update(&get_idor_hash(sound).to_le_bytes());
    } else {
        digest.update(&[0u8]);
    }
}

#[derive(Clone, Debug, Hash, PartialEq)]
pub struct PiercingWeaponImpl {
    pub deals_knockback: bool,
    pub dismounts: bool,
    pub sound: Option<IdOr<SoundEvent>>,
    pub hit_sound: Option<IdOr<SoundEvent>>,
}
impl PiercingWeaponImpl {
    pub fn read_data(data: &NbtTag) -> Option<Self> {
        let compound = data.extract_compound()?;
        Some(Self {
            deals_knockback: compound.get_bool("deals_knockback").unwrap_or(true),
            dismounts: compound.get_bool("dismounts").unwrap_or(false),
            sound: get_optional_idor(compound, "sound"),
            hit_sound: get_optional_idor(compound, "hit_sound"),
        })
    }
}
impl DataComponentImpl for PiercingWeaponImpl {
    fn write_data(&self) -> NbtTag {
        let mut compound = NbtCompound::new();
        compound.put_bool("deals_knockback", self.deals_knockback);
        compound.put_bool("dismounts", self.dismounts);
        put_optional_idor(&mut compound, "sound", self.sound.as_ref());
        put_optional_idor(&mut compound, "hit_sound", self.hit_sound.as_ref());
        NbtTag::Compound(compound)
    }
    fn get_hash(&self) -> i32 {
        let mut digest = Digest::new();
        digest.update(&[self.deals_knockback as u8, self.dismounts as u8]);
        update_optional_idor(&mut digest, self.sound.as_ref());
        update_optional_idor(&mut digest, self.hit_sound.as_ref());
        digest.finalize() as i32
    }
    default_impl!(PiercingWeapon);
}

#[derive(Clone, Debug, PartialEq)]
pub struct KineticConditionImpl {
    pub max_duration_ticks: i32,
    pub min_speed: f32,
    pub min_relative_speed: f32,
}
impl KineticConditionImpl {
    /// `entity_factor` scales both speed thresholds: 1 for players, 0.2 for mobs.
    pub fn test(
        &self,
        ticks_used: i32,
        attacker_speed: f64,
        relative_speed: f64,
        entity_factor: f64,
    ) -> bool {
        ticks_used <= self.max_duration_ticks
            && attacker_speed >= f64::from(self.min_speed) * entity_factor
            && relative_speed >= f64::from(self.min_relative_speed) * entity_factor
    }
    fn read(compound: &NbtCompound, key: &str) -> Option<Self> {
        let condition = compound.get_compound(key)?;
        Some(Self {
            max_duration_ticks: condition.get_int("max_duration_ticks")?,
            min_speed: condition.get_float("min_speed").unwrap_or(0.0),
            min_relative_speed: condition.get_float("min_relative_speed").unwrap_or(0.0),
        })
    }
    fn write(&self, compound: &mut NbtCompound, key: &str) {
        let mut condition = NbtCompound::new();
        condition.put_int("max_duration_ticks", self.max_duration_ticks);
        condition.put_float("min_speed", self.min_speed);
        condition.put_float("min_relative_speed", self.min_relative_speed);
        compound.put(key, NbtTag::Compound(condition));
    }
    fn update_digest(&self, digest: &mut Digest) {
        digest.update(&get_i32_hash(self.max_duration_ticks).to_le_bytes());
        digest.update(&get_f32_hash(self.min_speed).to_le_bytes());
        digest.update(&get_f32_hash(self.min_relative_speed).to_le_bytes());
    }
}
impl Hash for KineticConditionImpl {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.max_duration_ticks.hash(state);
        self.min_speed.to_bits().hash(state);
        self.min_relative_speed.to_bits().hash(state);
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct KineticWeaponImpl {
    pub contact_cooldown_ticks: i32,
    pub delay_ticks: i32,
    pub dismount_conditions: Option<KineticConditionImpl>,
    pub knockback_conditions: Option<KineticConditionImpl>,
    pub damage_conditions: Option<KineticConditionImpl>,
    pub forward_movement: f32,
    pub damage_multiplier: f32,
    pub sound: Option<IdOr<SoundEvent>>,
    pub hit_sound: Option<IdOr<SoundEvent>>,
}
impl KineticWeaponImpl {
    pub fn read_data(data: &NbtTag) -> Option<Self> {
        let compound = data.extract_compound()?;
        Some(Self {
            contact_cooldown_ticks: compound.get_int("contact_cooldown_ticks").unwrap_or(10),
            delay_ticks: compound.get_int("delay_ticks").unwrap_or(0),
            dismount_conditions: KineticConditionImpl::read(compound, "dismount_conditions"),
            knockback_conditions: KineticConditionImpl::read(compound, "knockback_conditions"),
            damage_conditions: KineticConditionImpl::read(compound, "damage_conditions"),
            forward_movement: compound.get_float("forward_movement").unwrap_or(0.0),
            damage_multiplier: compound.get_float("damage_multiplier").unwrap_or(1.0),
            sound: get_optional_idor(compound, "sound"),
            hit_sound: get_optional_idor(compound, "hit_sound"),
        })
    }
    fn conditions(&self) -> [(&str, Option<&KineticConditionImpl>); 3] {
        [
            ("dismount_conditions", self.dismount_conditions.as_ref()),
            ("knockback_conditions", self.knockback_conditions.as_ref()),
            ("damage_conditions", self.damage_conditions.as_ref()),
        ]
    }
}
impl DataComponentImpl for KineticWeaponImpl {
    fn write_data(&self) -> NbtTag {
        let mut compound = NbtCompound::new();
        compound.put_int("contact_cooldown_ticks", self.contact_cooldown_ticks);
        compound.put_int("delay_ticks", self.delay_ticks);
        for (key, condition) in self.conditions() {
            if let Some(condition) = condition {
                condition.write(&mut compound, key);
            }
        }
        compound.put_float("forward_movement", self.forward_movement);
        compound.put_float("damage_multiplier", self.damage_multiplier);
        put_optional_idor(&mut compound, "sound", self.sound.as_ref());
        put_optional_idor(&mut compound, "hit_sound", self.hit_sound.as_ref());
        NbtTag::Compound(compound)
    }
    fn get_hash(&self) -> i32 {
        let mut digest = Digest::new();
        digest.update(&get_i32_hash(self.contact_cooldown_ticks).to_le_bytes());
        digest.update(&get_i32_hash(self.delay_ticks).to_le_bytes());
        for (_, condition) in self.conditions() {
            if let Some(condition) = condition {
                digest.update(&[1u8]);
                condition.update_digest(&mut digest);
            } else {
                digest.update(&[0u8]);
            }
        }
        digest.update(&get_f32_hash(self.forward_movement).to_le_bytes());
        digest.update(&get_f32_hash(self.damage_multiplier).to_le_bytes());
        update_optional_idor(&mut digest, self.sound.as_ref());
        update_optional_idor(&mut digest, self.hit_sound.as_ref());
        digest.finalize() as i32
    }
    default_impl!(KineticWeapon);
}
impl Hash for KineticWeaponImpl {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.contact_cooldown_ticks.hash(state);
        self.delay_ticks.hash(state);
        self.dismount_conditions.hash(state);
        self.knockback_conditions.hash(state);
        self.damage_conditions.hash(state);
        self.forward_movement.to_bits().hash(state);
        self.damage_multiplier.to_bits().hash(state);
        self.sound.hash(state);
        self.hit_sound.hash(state);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum SwingAnimationType {
    #[default]
    Whack = 0,
    Stab = 1,
    None = 2,
}

impl SwingAnimationType {
    #[must_use]
    pub fn from_id(id: i32) -> Option<Self> {
        match id {
            0 => Some(Self::Whack),
            1 => Some(Self::Stab),
            2 => Some(Self::None),
            _ => None,
        }
    }

    #[must_use]
    pub fn to_id(&self) -> i32 {
        *self as i32
    }

    #[must_use]
    pub fn to_name(&self) -> &'static str {
        match self {
            Self::Whack => "whack",
            Self::Stab => "stab",
            Self::None => "none",
        }
    }

    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "whack" => Some(Self::Whack),
            "stab" => Some(Self::Stab),
            "none" => Some(Self::None),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct SwingAnimationImpl {
    pub animation_type: SwingAnimationType,
    pub duration: i32,
}

impl Default for SwingAnimationImpl {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl SwingAnimationImpl {
    pub const DEFAULT: Self = Self {
        animation_type: SwingAnimationType::Whack,
        duration: 6,
    };

    pub fn read_data(data: &NbtTag) -> Option<Self> {
        match data {
            NbtTag::Compound(compound) => {
                let animation_type = compound
                    .get("type")
                    .and_then(|tag| match tag {
                        NbtTag::String(name) => SwingAnimationType::from_name(name),
                        NbtTag::Int(id) => SwingAnimationType::from_id(*id),
                        NbtTag::Byte(id) => SwingAnimationType::from_id(*id as i32),
                        _ => None,
                    })
                    .unwrap_or(Self::DEFAULT.animation_type);
                let duration = compound
                    .get("duration")
                    .and_then(|tag| tag.extract_int())
                    .unwrap_or(Self::DEFAULT.duration);
                Some(Self {
                    animation_type,
                    duration,
                })
            }
            _ => Some(Self::DEFAULT),
        }
    }
}

impl DataComponentImpl for SwingAnimationImpl {
    fn write_data(&self) -> NbtTag {
        let mut compound = NbtCompound::new();
        compound.put_string("type", self.animation_type.to_name().to_string());
        compound.put_int("duration", self.duration);
        NbtTag::Compound(compound)
    }

    default_impl!(AttackAnimation);
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct InteractAnimationImpl {
    pub animation_type: SwingAnimationType,
    pub duration: i32,
}

impl Default for InteractAnimationImpl {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl InteractAnimationImpl {
    pub const DEFAULT: Self = Self {
        animation_type: SwingAnimationType::Whack,
        duration: 6,
    };

    pub fn read_data(data: &NbtTag) -> Option<Self> {
        let swing = SwingAnimationImpl::read_data(data)?;
        Some(Self {
            animation_type: swing.animation_type,
            duration: swing.duration,
        })
    }
}

impl DataComponentImpl for InteractAnimationImpl {
    fn write_data(&self) -> NbtTag {
        let mut compound = NbtCompound::new();
        compound.put_string("type", self.animation_type.to_name().to_string());
        compound.put_int("duration", self.duration);
        NbtTag::Compound(compound)
    }

    default_impl!(InteractAnimation);
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Default)]
pub struct AdditionalTradeCostImpl {
    pub cost: i32,
}
impl AdditionalTradeCostImpl {
    pub fn read_data(data: &NbtTag) -> Option<Self> {
        Some(Self {
            cost: data.extract_int().unwrap_or(0),
        })
    }
}
impl DataComponentImpl for AdditionalTradeCostImpl {
    fn write_data(&self) -> NbtTag {
        NbtTag::Int(self.cost)
    }
    fn get_hash(&self) -> i32 {
        get_i32_hash(self.cost) as i32
    }
    default_impl!(AdditionalTradeCost);
}

/// Scales how far the listed mobs notice the wearer, vanilla `MobVisibility`. Mob heads.
#[derive(Clone, Debug, PartialEq)]
pub struct MobVisibilityImpl {
    pub targeting_entity_types: IDSet<EntityType>,
    pub visibility: f32,
}
impl Hash for MobVisibilityImpl {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        get_idset_hash(&self.targeting_entity_types).hash(state);
        self.visibility.to_bits().hash(state);
    }
}
impl MobVisibilityImpl {
    pub const MAX_VISIBILITY: f32 = 10.0;

    /// Vanilla `floatRange(0, MAX_VISIBILITY)`, which also rejects NaN.
    #[must_use]
    pub fn is_valid_visibility(visibility: f32) -> bool {
        (0.0..=Self::MAX_VISIBILITY).contains(&visibility)
    }

    pub fn read_data(data: &NbtTag) -> Option<Self> {
        let compound = data.extract_compound()?;
        let targeting_entity_types =
            IDSet::<EntityType>::read(compound.get("targeting_entity_types")?)?;
        let visibility = compound.get_float("visibility")?;
        Self::is_valid_visibility(visibility).then_some(Self {
            targeting_entity_types,
            visibility,
        })
    }
}
impl DataComponentImpl for MobVisibilityImpl {
    fn write_data(&self) -> NbtTag {
        let mut compound = NbtCompound::new();
        self.targeting_entity_types
            .write(&mut compound, "targeting_entity_types");
        compound.put_float("visibility", self.visibility);
        NbtTag::Compound(compound)
    }
    fn get_hash(&self) -> i32 {
        let mut digest = Digest::new();
        digest.update(&get_idset_hash(&self.targeting_entity_types).to_le_bytes());
        digest.update(&get_f32_hash(self.visibility).to_le_bytes());
        digest.finalize() as i32
    }
    default_impl!(MobVisibility);
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct StoredEnchantmentsImpl {
    pub enchantment: Cow<'static, [(&'static Enchantment, i32)]>,
}
impl StoredEnchantmentsImpl {
    pub fn read_data(data: &NbtTag) -> Option<Self> {
        Some(Self {
            enchantment: Cow::from(read_enchantment_levels(data)?),
        })
    }
}
impl DataComponentImpl for StoredEnchantmentsImpl {
    fn write_data(&self) -> NbtTag {
        let mut data = NbtCompound::new();
        for (enc, level) in self.enchantment.iter() {
            data.put_int(enc.name, *level);
        }
        NbtTag::Compound(data)
    }
    fn get_hash(&self) -> i32 {
        let mut digest = Digest::new();
        digest.update(&[2u8]);
        for (enc, level) in self.enchantment.iter() {
            digest.update(&get_str_hash(enc.name).to_le_bytes());
            digest.update(&get_i32_hash(*level).to_le_bytes());
        }
        digest.update(&[3u8]);
        digest.finalize() as i32
    }
    default_impl!(StoredEnchantments);
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct OminousBottleAmplifierImpl {
    pub amplifier: i32,
}
impl OminousBottleAmplifierImpl {
    pub fn read_data(data: &NbtTag) -> Option<Self> {
        data.extract_int().map(|amplifier| Self { amplifier })
    }
}
impl DataComponentImpl for OminousBottleAmplifierImpl {
    fn write_data(&self) -> NbtTag {
        NbtTag::Int(self.amplifier)
    }
    fn get_hash(&self) -> i32 {
        get_i32_hash(self.amplifier) as i32
    }
    default_impl!(OminousBottleAmplifier);
}

/// An armor trim's material and pattern, kept as their raw NBT (each a registry
/// id or an inline definition) since Pumpkin does not yet model trim registries.
// TODO: replace `material`/`pattern` with typed trim material/pattern once those registries are modelled.
#[derive(Clone, Debug, PartialEq)]
pub struct TrimImpl {
    pub material: NbtTag,
    pub pattern: NbtTag,
}
impl TrimImpl {
    pub fn read_data(data: &NbtTag) -> Option<Self> {
        let compound = data.extract_compound()?;
        Some(Self {
            material: compound.get("material")?.clone(),
            pattern: compound.get("pattern")?.clone(),
        })
    }

    #[must_use]
    pub fn new(
        material: crate::trim_material::TrimMaterial,
        pattern: crate::trim_pattern::TrimPattern,
    ) -> Self {
        Self {
            material: NbtTag::String(material.asset_id().into()),
            pattern: NbtTag::String(pattern.asset_id().into()),
        }
    }

    #[must_use]
    pub fn material_enum(&self) -> Option<crate::trim_material::TrimMaterial> {
        self.material
            .extract_string()
            .and_then(crate::trim_material::TrimMaterial::from_name)
    }

    #[must_use]
    pub fn pattern_enum(&self) -> Option<crate::trim_pattern::TrimPattern> {
        self.pattern
            .extract_string()
            .and_then(crate::trim_pattern::TrimPattern::from_name)
    }
}
impl DataComponentImpl for TrimImpl {
    fn write_data(&self) -> NbtTag {
        let mut compound = NbtCompound::new();
        compound.put("material", self.material.clone());
        compound.put("pattern", self.pattern.clone());
        NbtTag::Compound(compound)
    }
    default_impl!(Trim);
}

#[derive(Clone, Debug, PartialEq)]
pub struct MinimumAttackChargeImpl {
    pub charge: f32,
}
impl MinimumAttackChargeImpl {
    pub fn read_data(data: &NbtTag) -> Option<Self> {
        data.extract_float().map(|charge| Self { charge })
    }
}
impl DataComponentImpl for MinimumAttackChargeImpl {
    fn write_data(&self) -> NbtTag {
        NbtTag::Float(self.charge)
    }
    fn get_hash(&self) -> i32 {
        get_f32_hash(self.charge) as i32
    }
    default_impl!(MinimumAttackCharge);
}
impl Hash for MinimumAttackChargeImpl {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.charge.to_bits().hash(state);
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DamageTypeImpl {
    pub damage_type: DamageType,
}
impl DamageTypeImpl {
    pub fn read_data(data: &NbtTag) -> Option<Self> {
        let name = data.extract_string()?;
        let damage_type = DamageType::from_name(name.strip_prefix("minecraft:").unwrap_or(name))?;
        Some(Self { damage_type })
    }
}
impl DataComponentImpl for DamageTypeImpl {
    fn write_data(&self) -> NbtTag {
        NbtTag::String(format!("minecraft:{}", self.damage_type.registry_key()).into())
    }
    fn get_hash(&self) -> i32 {
        get_str_hash(self.damage_type.registry_key()) as i32
    }
    default_impl!(DamageType);
}
impl Hash for DamageTypeImpl {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.damage_type.id.hash(state);
    }
}
