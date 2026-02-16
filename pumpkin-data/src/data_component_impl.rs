#![allow(dead_code)]

use crate::attributes::Attributes;
use crate::data_component::DataComponent;
use crate::data_component::DataComponent::{
    AttributeModifiers, BlocksAttacks, Consumable, CustomData, CustomName, Damage, DeathProtection,
    Enchantments, Equippable, FireworkExplosion, Fireworks, Food, ItemName, JukeboxPlayable,
    MaxDamage, MaxStackSize, PotionContents, Tool, Unbreakable,
};
use crate::entity_type::EntityType;
use crate::tag::{Tag, Taggable};
use crate::{AttributeModifierSlot, Block, Enchantment};
use crc_fast::CrcAlgorithm::Crc32Iscsi;
use crc_fast::Digest;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::registry::RegistryEntryList;
use pumpkin_util::text::TextComponent;
use serde::de::SeqAccess;
use serde::ser::SerializeSeq;
use serde::{Deserialize, Serialize, de};
use std::any::Any;
use std::borrow::Cow;
use std::fmt::Debug;
use std::hash::Hash;

pub trait DataComponentImpl: Send + Sync {
    fn write_data(&self) -> NbtTag {
        todo!()
    }
    fn get_hash(&self) -> i32 {
        todo!()
    }
    /// make sure other is the same type component, or it will panic
    fn equal(&self, other: &dyn DataComponentImpl) -> bool;
    fn get_enum() -> DataComponent
    where
        Self: Sized;
    fn get_self_enum(&self) -> DataComponent; // only for debugging
    fn to_dyn(self) -> Box<dyn DataComponentImpl>;
    fn clone_dyn(&self) -> Box<dyn DataComponentImpl>;
    fn as_any(&self) -> &dyn Any;
    fn as_mut_any(&mut self) -> &mut dyn Any;
}
#[must_use]
pub fn read_data(id: DataComponent, data: &NbtTag) -> Option<Box<dyn DataComponentImpl>> {
    match id {
        MaxStackSize => Some(MaxStackSizeImpl::read_data(data)?.to_dyn()),
        Enchantments => Some(EnchantmentsImpl::read_data(data)?.to_dyn()),
        Damage => Some(DamageImpl::read_data(data)?.to_dyn()),
        Unbreakable => Some(UnbreakableImpl::read_data(data)?.to_dyn()),
        PotionContents => Some(PotionContentsImpl::read_data(data)?.to_dyn()),
        _ => None,
    }
}
// Also Pumpkin\pumpkin-protocol\src\codec\data_component.rs

macro_rules! default_impl {
    ($t: ident) => {
        fn equal(&self, other: &dyn DataComponentImpl) -> bool {
            self == get::<Self>(other)
        }
        #[inline]
        fn get_enum() -> DataComponent
        where
            Self: Sized,
        {
            $t
        }
        fn get_self_enum(&self) -> DataComponent {
            $t
        }
        fn to_dyn(self) -> Box<dyn DataComponentImpl> {
            Box::new(self)
        }
        fn clone_dyn(&self) -> Box<dyn DataComponentImpl> {
            Box::new(self.clone())
        }
        fn as_any(&self) -> &dyn Any {
            self
        }
        fn as_mut_any(&mut self) -> &mut dyn Any {
            self
        }
    };
}

impl Clone for Box<dyn DataComponentImpl> {
    fn clone(&self) -> Self {
        self.clone_dyn()
    }
}

#[inline]
pub fn get<T: DataComponentImpl + 'static>(value: &dyn DataComponentImpl) -> &T {
    value.as_any().downcast_ref::<T>().unwrap_or_else(|| {
        panic!(
            "you are trying to cast {} to {}",
            value.get_self_enum().to_name(),
            T::get_enum().to_name()
        )
    })
}
#[inline]
pub fn get_mut<T: DataComponentImpl + 'static>(value: &mut dyn DataComponentImpl) -> &mut T {
    value.as_mut_any().downcast_mut::<T>().unwrap()
}
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct CustomDataImpl;
impl DataComponentImpl for CustomDataImpl {
    default_impl!(CustomData);
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct MaxStackSizeImpl {
    pub size: u8,
}
impl MaxStackSizeImpl {
    fn read_data(data: &NbtTag) -> Option<Self> {
        data.extract_int().map(|size| Self { size: size as u8 })
    }
}
impl DataComponentImpl for MaxStackSizeImpl {
    fn write_data(&self) -> NbtTag {
        NbtTag::Int(i32::from(self.size))
    }
    fn get_hash(&self) -> i32 {
        get_i32_hash(i32::from(self.size)) as i32
    }

    default_impl!(MaxStackSize);
}
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct MaxDamageImpl {
    pub max_damage: i32,
}
impl DataComponentImpl for MaxDamageImpl {
    default_impl!(MaxDamage);
}
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct DamageImpl {
    pub damage: i32,
}
impl DamageImpl {
    fn read_data(data: &NbtTag) -> Option<Self> {
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
#[derive(Clone, Hash, PartialEq, Eq)]
pub struct UnbreakableImpl;
impl UnbreakableImpl {
    const fn read_data(_data: &NbtTag) -> Option<Self> {
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
#[derive(Clone, Hash, PartialEq, Eq)]
pub struct CustomNameImpl {
    // TODO make TextComponent const
    pub name: &'static str,
}
impl DataComponentImpl for CustomNameImpl {
    default_impl!(CustomName);
}
#[derive(Clone, Hash, PartialEq, Eq)]
pub struct ItemNameImpl {
    // TODO make TextComponent const
    pub name: &'static str,
}
impl DataComponentImpl for ItemNameImpl {
    default_impl!(ItemName);
}
#[derive(Clone, Hash, PartialEq, Eq)]
pub struct ItemModelImpl;
#[derive(Clone, Hash, PartialEq, Eq)]
pub struct LoreImpl;
#[derive(Clone, Hash, PartialEq, Eq)]
pub struct RarityImpl;
#[derive(Clone, Hash, PartialEq, Eq)]
pub struct EnchantmentsImpl {
    pub enchantment: Cow<'static, [(&'static Enchantment, i32)]>,
}
impl EnchantmentsImpl {
    fn read_data(data: &NbtTag) -> Option<Self> {
        let data = &data.extract_compound()?.child_tags;
        let mut enc = Vec::with_capacity(data.len());
        for (name, level) in data {
            enc.push((Enchantment::from_name(name.as_str())?, level.extract_int()?));
        }
        Some(Self {
            enchantment: Cow::from(enc),
        })
    }
}

fn get_str_hash(val: &str) -> u32 {
    let mut digest = Digest::new(Crc32Iscsi);
    digest.update(&[12u8]);
    digest.update(&(val.len() as u32).to_le_bytes());
    let byte = val.as_bytes();
    for i in byte {
        digest.update(&[*i, 0u8]);
    }
    digest.finalize() as u32
}

fn get_i32_hash(val: i32) -> u32 {
    let mut digest = Digest::new(Crc32Iscsi);
    digest.update(&[8u8]);
    digest.update(&val.to_le_bytes());
    digest.finalize() as u32
}

#[test]
fn hash() {
    assert_eq!(get_str_hash("minecraft:sharpness"), 2734053906u32);
    assert_eq!(get_i32_hash(3), 3795317917u32);
    assert_eq!(
        EnchantmentsImpl {
            enchantment: Cow::Borrowed(&[(&Enchantment::SHARPNESS, 2)]),
        }
        .get_hash(),
        -1580618251i32
    );
    assert_eq!(MaxStackSizeImpl { size: 99 }.get_hash(), -1632321551i32);
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
        let mut digest = Digest::new(Crc32Iscsi);
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
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct CanPlaceOnImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct CanBreakImpl;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Operation {
    AddValue,
    AddMultipliedBase,
    AddMultipliedTotal,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Modifier {
    pub r#type: &'static Attributes,
    pub id: &'static str,
    pub amount: f64,
    pub operation: Operation,
    pub slot: AttributeModifierSlot,
}
impl Hash for Modifier {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.r#type.hash(state);
        self.id.hash(state);
        unsafe { (*(&raw const self.amount).cast::<u64>()).hash(state) };
        self.operation.hash(state);
        self.slot.hash(state);
    }
}
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct AttributeModifiersImpl {
    pub attribute_modifiers: Cow<'static, [Modifier]>,
}
impl DataComponentImpl for AttributeModifiersImpl {
    default_impl!(AttributeModifiers);
}
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct CustomModelDataImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct TooltipDisplayImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct RepairCostImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct CreativeSlotLockImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct EnchantmentGlintOverrideImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct IntangibleProjectileImpl;
#[derive(Clone, Debug, PartialEq)]
pub struct FoodImpl {
    pub nutrition: i32,
    pub saturation: f32,
    pub can_always_eat: bool,
}
impl DataComponentImpl for FoodImpl {
    default_impl!(Food);
}
impl Hash for FoodImpl {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.nutrition.hash(state);
        unsafe { (*(&raw const self.saturation).cast::<u32>()).hash(state) };
        self.can_always_eat.hash(state);
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct ConsumableImpl {
    pub consume_seconds: f32,
    // TODO: more
}

impl ConsumableImpl {
    #[must_use]
    pub fn consume_ticks(&self) -> i32 {
        (self.consume_seconds * 20.0) as i32
    }
}

impl DataComponentImpl for ConsumableImpl {
    default_impl!(Consumable);
}
impl Hash for ConsumableImpl {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe { (*(&raw const self.consume_seconds).cast::<u32>()).hash(state) };
    }
}
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct UseRemainderImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct UseCooldownImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct DamageResistantImpl;

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum IDSet {
    Tag(&'static Tag),
    Blocks(Cow<'static, [&'static Block]>),
}

#[derive(Clone, PartialEq)]
pub struct ToolRule {
    pub blocks: IDSet,
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
impl DataComponentImpl for ToolImpl {
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
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct WeaponImpl;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum EquipmentType {
    Hand,
    HumanoidArmor,
    AnimalArmor,
    Saddle,
}

#[derive(Clone, Hash, Eq, PartialEq)]
pub struct EquipmentSlotData {
    pub slot_type: EquipmentType,
    pub entity_id: i32,
    pub max_count: i32,
    pub index: i32,
    pub name: Cow<'static, str>,
}

#[derive(Clone, Hash, Eq, PartialEq)]
#[repr(i8)]
pub enum EquipmentSlot {
    MainHand(EquipmentSlotData),
    OffHand(EquipmentSlotData),
    Feet(EquipmentSlotData),
    Legs(EquipmentSlotData),
    Chest(EquipmentSlotData),
    Head(EquipmentSlotData),
    Body(EquipmentSlotData),
    Saddle(EquipmentSlotData),
}

impl EquipmentSlot {
    pub const MAIN_HAND: Self = Self::MainHand(EquipmentSlotData {
        slot_type: EquipmentType::Hand,
        entity_id: 0,
        index: 0,
        max_count: 0,
        name: Cow::Borrowed("mainhand"),
    });
    pub const OFF_HAND: Self = Self::OffHand(EquipmentSlotData {
        slot_type: EquipmentType::Hand,
        entity_id: 1,
        index: 5,
        max_count: 0,
        name: Cow::Borrowed("offhand"),
    });
    pub const FEET: Self = Self::Feet(EquipmentSlotData {
        slot_type: EquipmentType::HumanoidArmor,
        entity_id: 0,
        index: 1,
        max_count: 1,
        name: Cow::Borrowed("feet"),
    });
    pub const LEGS: Self = Self::Legs(EquipmentSlotData {
        slot_type: EquipmentType::HumanoidArmor,
        entity_id: 1,
        index: 2,
        max_count: 1,
        name: Cow::Borrowed("legs"),
    });
    pub const CHEST: Self = Self::Chest(EquipmentSlotData {
        slot_type: EquipmentType::HumanoidArmor,
        entity_id: 2,
        index: 3,
        max_count: 1,
        name: Cow::Borrowed("chest"),
    });
    pub const HEAD: Self = Self::Head(EquipmentSlotData {
        slot_type: EquipmentType::HumanoidArmor,
        entity_id: 3,
        index: 4,
        max_count: 1,
        name: Cow::Borrowed("head"),
    });
    pub const BODY: Self = Self::Body(EquipmentSlotData {
        slot_type: EquipmentType::AnimalArmor,
        entity_id: 0,
        index: 6,
        max_count: 1,
        name: Cow::Borrowed("body"),
    });
    pub const SADDLE: Self = Self::Saddle(EquipmentSlotData {
        slot_type: EquipmentType::Saddle,
        entity_id: 0,
        index: 7,
        max_count: 1,
        name: Cow::Borrowed("saddle"),
    });

    #[must_use]
    pub const fn get_entity_slot_id(&self) -> i32 {
        match self {
            Self::MainHand(data) => data.entity_id,
            Self::OffHand(data) => data.entity_id,
            Self::Feet(data) => data.entity_id,
            Self::Legs(data) => data.entity_id,
            Self::Chest(data) => data.entity_id,
            Self::Head(data) => data.entity_id,
            Self::Body(data) => data.entity_id,
            Self::Saddle(data) => data.entity_id,
        }
    }

    #[must_use]
    pub fn get_from_name(name: &str) -> Option<Self> {
        match name {
            "mainhand" => Some(Self::MAIN_HAND),
            "offhand" => Some(Self::OFF_HAND),
            "feet" => Some(Self::FEET),
            "legs" => Some(Self::LEGS),
            "chest" => Some(Self::CHEST),
            "head" => Some(Self::HEAD),
            "body" => Some(Self::BODY),
            "saddle" => Some(Self::SADDLE),
            _ => None,
        }
    }

    #[must_use]
    pub const fn get_offset_entity_slot_id(&self, offset: i32) -> i32 {
        self.get_entity_slot_id() + offset
    }

    #[must_use]
    pub const fn slot_type(&self) -> EquipmentType {
        match self {
            Self::MainHand(data) => data.slot_type,
            Self::OffHand(data) => data.slot_type,
            Self::Feet(data) => data.slot_type,
            Self::Legs(data) => data.slot_type,
            Self::Chest(data) => data.slot_type,
            Self::Head(data) => data.slot_type,
            Self::Body(data) => data.slot_type,
            Self::Saddle(data) => data.slot_type,
        }
    }

    #[must_use]
    pub const fn is_armor_slot(&self) -> bool {
        matches!(
            self.slot_type(),
            EquipmentType::HumanoidArmor | EquipmentType::AnimalArmor
        )
    }

    #[must_use]
    pub const fn discriminant(&self) -> i8 {
        match self {
            Self::MainHand(_) => 0,
            Self::OffHand(_) => 1,
            Self::Feet(_) => 2,
            Self::Legs(_) => 3,
            Self::Chest(_) => 4,
            Self::Head(_) => 5,
            Self::Body(_) => 6,
            Self::Saddle(_) => 7,
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum EntityTypeOrTag {
    Tag(&'static Tag),
    Single(&'static EntityType),
}

impl Hash for EntityTypeOrTag {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Tag(tag) => {
                for x in tag.0 {
                    x.hash(state);
                }
            }
            Self::Single(entity_type) => {
                entity_type.id.hash(state);
            }
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct EnchantableImpl;
#[derive(Clone, Hash, PartialEq, Eq)]
pub struct EquippableImpl {
    pub slot: &'static EquipmentSlot,
    pub equip_sound: &'static str,
    pub asset_id: Option<&'static str>,
    pub camera_overlay: Option<&'static str>,
    pub allowed_entities: Option<&'static [EntityTypeOrTag]>,
    pub dispensable: bool,
    pub swappable: bool,
    pub damage_on_hurt: bool,
    pub equip_on_interact: bool,
    pub can_be_sheared: bool,
    pub shearing_sound: Option<&'static str>,
}
impl DataComponentImpl for EquippableImpl {
    default_impl!(Equippable);
}
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct RepairableImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct GliderImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct TooltipStyleImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct DeathProtectionImpl;
impl DataComponentImpl for DeathProtectionImpl {
    default_impl!(DeathProtection);
}
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct BlocksAttacksImpl;

impl DataComponentImpl for BlocksAttacksImpl {
    default_impl!(BlocksAttacks);
}
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct StoredEnchantmentsImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct DyedColorImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct MapColorImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct MapIdImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct MapDecorationsImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct MapPostProcessingImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ChargedProjectilesImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct BundleContentsImpl;
/// Status effect instance for potion contents
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct StatusEffectInstance {
    pub effect_id: i32,
    pub amplifier: i32,
    pub duration: i32,
    pub ambient: bool,
    pub show_particles: bool,
    pub show_icon: bool,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct PotionContentsImpl {
    pub potion_id: Option<i32>,
    pub custom_color: Option<i32>,
    pub custom_effects: Vec<StatusEffectInstance>,
    pub custom_name: Option<String>,
}

impl PotionContentsImpl {
    pub fn read_data(tag: &NbtTag) -> Option<Self> {
        let compound = tag.extract_compound()?;
        let potion_id = if let Some(id) = compound.get_int("potion") {
            Some(id)
        } else if let Some(name) = compound.get_string("potion") {
            // Handle "minecraft:swiftness" -> "swiftness"
            let name = name.strip_prefix("minecraft:").unwrap_or(name);
            crate::potion::Potion::from_name(name).map(|p| p.id as i32)
        } else {
            None
        };

        let custom_color = compound.get_int("custom_color");
        let custom_name = compound.get_string("custom_name").map(|s| s.to_string());

        let mut custom_effects = Vec::new();
        if let Some(list) = compound.get_list("custom_effects") {
            for item in list {
                let effect_tag = item.extract_compound()?;
                let id = effect_tag.get_int("id")?;
                let amplifier = effect_tag.get_int("amplifier").unwrap_or(0);
                let duration = effect_tag.get_int("duration").unwrap_or(0);
                let ambient = effect_tag.get_byte("ambient").unwrap_or(0) != 0;
                let show_particles = effect_tag.get_byte("show_particles").unwrap_or(1) != 0;
                let show_icon = effect_tag.get_byte("show_icon").unwrap_or(1) != 0;

                custom_effects.push(StatusEffectInstance {
                    effect_id: id,
                    amplifier,
                    duration,
                    ambient,
                    show_particles,
                    show_icon,
                });
            }
        }

        Some(Self {
            potion_id,
            custom_color,
            custom_effects,
            custom_name,
        })
    }
}

impl DataComponentImpl for PotionContentsImpl {
    fn write_data(&self) -> NbtTag {
        let mut compound = NbtCompound::new();

        if let Some(potion_id) = self.potion_id {
            compound.put_int("potion", potion_id);
        }

        if let Some(color) = self.custom_color {
            compound.put_int("custom_color", color);
        }

        if !self.custom_effects.is_empty() {
            let mut effects_list = Vec::new();
            for effect in &self.custom_effects {
                let mut effect_compound = NbtCompound::new();
                effect_compound.put_int("id", effect.effect_id);
                effect_compound.put_int("amplifier", effect.amplifier);
                effect_compound.put_int("duration", effect.duration);
                effect_compound.put_byte("ambient", effect.ambient as i8);
                effect_compound.put_byte("show_particles", effect.show_particles as i8);
                effect_compound.put_byte("show_icon", effect.show_icon as i8);
                effects_list.push(NbtTag::Compound(effect_compound));
            }
            compound.put("custom_effects", NbtTag::List(effects_list));
        }

        if let Some(name) = &self.custom_name {
            compound.put_string("custom_name", name.clone());
        }

        NbtTag::Compound(compound)
    }

    fn get_hash(&self) -> i32 {
        let mut digest = Digest::new(Crc32Iscsi);
        
        if let Some(id) = self.potion_id {
            digest.update(&[1u8]);
            digest.update(&get_i32_hash(id).to_le_bytes());
        }

        if let Some(color) = self.custom_color {
            digest.update(&[2u8]);
            digest.update(&get_i32_hash(color).to_le_bytes());
        }

        if let Some(name) = &self.custom_name {
            digest.update(&[3u8]);
            digest.update(&get_str_hash(name).to_le_bytes());
        }

        if !self.custom_effects.is_empty() {
             digest.update(&[4u8]);
             for effect in &self.custom_effects {
                 digest.update(&get_i32_hash(effect.effect_id).to_le_bytes());
                 digest.update(&get_i32_hash(effect.amplifier).to_le_bytes());
                 digest.update(&get_i32_hash(effect.duration).to_le_bytes());
                 digest.update(&[effect.ambient as u8]);
                 digest.update(&[effect.show_particles as u8]);
                 digest.update(&[effect.show_icon as u8]);
             }
        }
        
        digest.finalize() as i32
    }

    default_impl!(PotionContents);
}
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct PotionDurationScaleImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct SuspiciousStewEffectsImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct WritableBookContentImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct WrittenBookContentImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct TrimImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct DebugStickStateImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct EntityDataImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct BucketEntityDataImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct BlockEntityDataImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct InstrumentImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ProvidesTrimMaterialImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct OminousBottleAmplifierImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct JukeboxPlayableImpl {
    pub song: &'static str,
}
impl DataComponentImpl for JukeboxPlayableImpl {
    default_impl!(JukeboxPlayable);
}
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ProvidesBannerPatternsImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct RecipesImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct LodestoneTrackerImpl;
/// Firework explosion shape types
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FireworkExplosionShape {
    SmallBall = 0,
    LargeBall = 1,
    Star = 2,
    Creeper = 3,
    Burst = 4,
}

impl FireworkExplosionShape {
    pub fn from_id(id: i32) -> Option<Self> {
        match id {
            0 => Some(Self::SmallBall),
            1 => Some(Self::LargeBall),
            2 => Some(Self::Star),
            3 => Some(Self::Creeper),
            4 => Some(Self::Burst),
            _ => None,
        }
    }

    pub fn to_id(&self) -> i32 {
        *self as i32
    }

    pub fn to_name(&self) -> &str {
        match self {
            Self::SmallBall => "small_ball",
            Self::LargeBall => "large_ball",
            Self::Star => "star",
            Self::Creeper => "creeper",
            Self::Burst => "burst",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "small_ball" => Some(Self::SmallBall),
            "large_ball" => Some(Self::LargeBall),
            "star" => Some(Self::Star),
            "creeper" => Some(Self::Creeper),
            "burst" => Some(Self::Burst),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct FireworkExplosionImpl {
    pub shape: FireworkExplosionShape,
    pub colors: Vec<i32>,
    pub fade_colors: Vec<i32>,
    pub has_trail: bool,
    pub has_twinkle: bool,
}

impl FireworkExplosionImpl {
    pub fn new(
        shape: FireworkExplosionShape,
        colors: Vec<i32>,
        fade_colors: Vec<i32>,
        has_trail: bool,
        has_twinkle: bool,
    ) -> Self {
        Self {
            shape,
            colors,
            fade_colors,
            has_trail,
            has_twinkle,
        }
    }
}

impl DataComponentImpl for FireworkExplosionImpl {
    fn write_data(&self) -> NbtTag {
        let mut compound = NbtCompound::new();
        compound.put_string("shape", self.shape.to_name().to_string());
        compound.put("colors", NbtTag::IntArray(self.colors.clone()));
        compound.put("fade_colors", NbtTag::IntArray(self.fade_colors.clone()));
        compound.put_bool("has_trail", self.has_trail);
        compound.put_bool("has_twinkle", self.has_twinkle);
        NbtTag::Compound(compound)
    }

    fn get_hash(&self) -> i32 {
        let mut digest = Digest::new(Crc32Iscsi);
        digest.update(&[2u8]);
        digest.update(&[self.shape.to_id() as u8]);
        for color in &self.colors {
            digest.update(&get_i32_hash(*color).to_le_bytes());
        }
        digest.update(&[3u8]);
        for color in &self.fade_colors {
            digest.update(&get_i32_hash(*color).to_le_bytes());
        }
        digest.update(&[4u8]);
        digest.update(&[self.has_trail as u8]);
        digest.update(&[self.has_twinkle as u8]);
        digest.finalize() as i32
    }

    default_impl!(FireworkExplosion);
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct FireworksImpl {
    pub flight_duration: i32,
    pub explosions: Vec<FireworkExplosionImpl>,
}

impl FireworksImpl {
    pub fn new(flight_duration: i32, explosions: Vec<FireworkExplosionImpl>) -> Self {
        Self {
            flight_duration,
            explosions,
        }
    }
}

impl DataComponentImpl for FireworksImpl {
    fn write_data(&self) -> NbtTag {
        let mut compound = NbtCompound::new();
        compound.put_int("flight_duration", self.flight_duration);
        let explosions_list: Vec<NbtTag> = self.explosions.iter().map(|e| e.write_data()).collect();
        compound.put_list("explosions", explosions_list);
        NbtTag::Compound(compound)
    }

    fn get_hash(&self) -> i32 {
        let mut digest = Digest::new(Crc32Iscsi);
        digest.update(&[2u8]);
        digest.update(&get_i32_hash(self.flight_duration).to_le_bytes());
        for explosion in &self.explosions {
            digest.update(&get_i32_hash(explosion.get_hash()).to_le_bytes());
        }
        digest.update(&[3u8]);
        digest.finalize() as i32
    }

    default_impl!(Fireworks);
}
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ProfileImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct NoteBlockSoundImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct BannerPatternsImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct BaseColorImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct PotDecorationsImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ContainerImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct BlockStateImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct BeesImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct LockImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ContainerLootImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct BreakSoundImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct VillagerVariantImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct WolfVariantImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct WolfSoundVariantImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct WolfCollarImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct FoxVariantImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct SalmonSizeImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ParrotVariantImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct TropicalFishPatternImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct TropicalFishBaseColorImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct TropicalFishPatternColorImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct MooshroomVariantImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct RabbitVariantImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct PigVariantImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct CowVariantImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ChickenVariantImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct FrogVariantImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct HorseVariantImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct PaintingVariantImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct LlamaVariantImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct AxolotlVariantImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct CatVariantImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct CatCollarImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct SheepColorImpl;
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ShulkerColorImpl;
