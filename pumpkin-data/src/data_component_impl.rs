#![allow(dead_code)]

use crate::attributes::Attributes;
use crate::data_component::DataComponent;
use crate::data_component::DataComponent::*;
use crate::entity_type::EntityType;
use crate::tag::Tag;
use crate::{AttributeModifierSlot, Block, Enchantment};
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

pub trait DataComponentImpl: Send + Sync + Debug {
    fn write_data(&self) -> NbtTag {
        todo!()
    }
    fn get_enum() -> DataComponent
    where
        Self: Sized;
    fn to_dyn(self) -> Box<dyn DataComponentImpl>;
    fn clone_dyn(&self) -> Box<dyn DataComponentImpl>;
    fn as_any(&self) -> &dyn Any;
    fn as_mut_any(&mut self) -> &mut dyn Any;
}
pub fn read_data(id: DataComponent, data: &NbtTag) -> Option<Box<dyn DataComponentImpl>> {
    match id {
        MaxStackSize => Some(MaxStackSizeImpl::read_data(data)?.to_dyn()),
        Enchantments => Some(EnchantmentsImpl::read_data(data)?.to_dyn()),
        _ => todo!(),
    }
}

impl Clone for Box<dyn DataComponentImpl> {
    fn clone(&self) -> Self {
        self.clone_dyn()
    }
}

#[inline]
pub fn get<T: DataComponentImpl + 'static>(value: &dyn DataComponentImpl) -> &T {
    value.as_any().downcast_ref::<T>().unwrap()
}
#[inline]
pub fn get_mut<T: DataComponentImpl + 'static>(value: &mut dyn DataComponentImpl) -> &mut T {
    value.as_mut_any().downcast_mut::<T>().unwrap()
}
#[derive(Clone, Debug, Hash)]
pub struct CustomDataImpl;
impl DataComponentImpl for CustomDataImpl {
    #[inline]
    fn get_enum() -> DataComponent
    where
        Self: Sized,
    {
        CustomData
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
}

#[derive(Clone, Debug, Hash)]
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
        NbtTag::Int(self.size as i32)
    }
    #[inline]
    fn get_enum() -> DataComponent
    where
        Self: Sized,
    {
        MaxStackSize
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
}
#[derive(Clone, Debug, Hash)]
pub struct MaxDamageImpl {
    pub max_damage: i32,
}
impl DataComponentImpl for MaxDamageImpl {
    #[inline]
    fn get_enum() -> DataComponent
    where
        Self: Sized,
    {
        MaxDamage
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
}
#[derive(Clone, Debug, Hash)]
pub struct DamageImpl {
    pub damage: i32,
}
impl DataComponentImpl for DamageImpl {
    #[inline]
    fn get_enum() -> DataComponent
    where
        Self: Sized,
    {
        Damage
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
}
#[derive(Clone, Debug, Hash)]
pub struct UnbreakableImpl;
#[derive(Clone, Debug, Hash)]
pub struct CustomNameImpl;
#[derive(Clone, Debug, Hash)]
pub struct ItemNameImpl {
    // TODO make TextComponent const
    pub name: &'static str,
}
impl DataComponentImpl for ItemNameImpl {
    #[inline]
    fn get_enum() -> DataComponent
    where
        Self: Sized,
    {
        ItemName
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
}
#[derive(Clone, Debug, Hash)]
pub struct ItemModelImpl;
#[derive(Clone, Debug, Hash)]
pub struct LoreImpl;
#[derive(Clone, Debug, Hash)]
pub struct RarityImpl;
#[derive(Clone, Debug, Hash)]
pub struct EnchantmentsImpl {
    pub enchantment: Cow<'static, [(&'static Enchantment, i32)]>,
}
impl EnchantmentsImpl {
    fn read_data(data: &NbtTag) -> Option<Self> {
        let data = &data.extract_compound()?.child_tags;
        let mut enc = Vec::with_capacity(data.len());
        for (name, level) in data {
            enc.push((Enchantment::from_name(name.as_str())?, level.extract_int()?))
        }
        Some(Self {
            enchantment: Cow::from(enc),
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
    #[inline]
    fn get_enum() -> DataComponent
    where
        Self: Sized,
    {
        Enchantments
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
}
#[derive(Clone, Debug, Hash)]
pub struct CanPlaceOnImpl;
#[derive(Clone, Debug, Hash)]
pub struct CanBreakImpl;

#[derive(Clone, Copy, Debug, PartialEq, Hash)]
pub enum Operation {
    AddValue,
    AddMultipliedBase,
    AddMultipliedTotal,
}
#[derive(Clone, Debug)]
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
        unsafe { (*(&self.amount as *const f64 as *const u64)).hash(state) };
        self.operation.hash(state);
        self.slot.hash(state);
    }
}
#[derive(Clone, Debug, Hash)]
pub struct AttributeModifiersImpl {
    pub attribute_modifiers: Cow<'static, [Modifier]>,
}
impl DataComponentImpl for AttributeModifiersImpl {
    #[inline]
    fn get_enum() -> DataComponent
    where
        Self: Sized,
    {
        AttributeModifiers
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
}
#[derive(Clone, Debug, Hash)]
pub struct CustomModelDataImpl;
#[derive(Clone, Debug, Hash)]
pub struct TooltipDisplayImpl;
#[derive(Clone, Debug, Hash)]
pub struct RepairCostImpl;
#[derive(Clone, Debug, Hash)]
pub struct CreativeSlotLockImpl;
#[derive(Clone, Debug, Hash)]
pub struct EnchantmentGlintOverrideImpl;
#[derive(Clone, Debug, Hash)]
pub struct IntangibleProjectileImpl;
#[derive(Clone, Debug)]
pub struct FoodImpl {
    pub nutrition: i32,
    pub saturation: f32,
    pub can_always_eat: bool,
}
impl DataComponentImpl for FoodImpl {
    #[inline]
    fn get_enum() -> DataComponent
    where
        Self: Sized,
    {
        Food
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
}
impl Hash for FoodImpl {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.nutrition.hash(state);
        unsafe { (*(&self.saturation as *const f32 as *const u32)).hash(state) };
        self.can_always_eat.hash(state);
    }
}
#[derive(Clone, Debug, Hash)]
pub struct ConsumableImpl;
#[derive(Clone, Debug, Hash)]
pub struct UseRemainderImpl;
#[derive(Clone, Debug, Hash)]
pub struct UseCooldownImpl;
#[derive(Clone, Debug, Hash)]
pub struct DamageResistantImpl;

#[derive(Clone, Debug, Hash)]
pub enum IDSet {
    Tag(&'static Tag),
    Blocks(Cow<'static, [&'static Block]>),
}

#[derive(Clone, Debug)]
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
            unsafe { (*(&val as *const f32 as *const u32)).hash(state) };
        } else {
            false.hash(state);
        }
        self.correct_for_drops.hash(state);
    }
}
#[derive(Clone, Debug)]
pub struct ToolImpl {
    pub rules: Cow<'static, [ToolRule]>,
    pub default_mining_speed: f32,
    pub damage_per_block: u32,
    pub can_destroy_blocks_in_creative: bool,
}
impl DataComponentImpl for ToolImpl {
    #[inline]
    fn get_enum() -> DataComponent
    where
        Self: Sized,
    {
        Tool
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
}
impl Hash for ToolImpl {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.rules.hash(state);
        unsafe { (*(&self.default_mining_speed as *const f32 as *const u32)).hash(state) };
        self.damage_per_block.hash(state);
        self.can_destroy_blocks_in_creative.hash(state);
    }
}
#[derive(Clone, Debug, Hash)]
pub struct WeaponImpl;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum EquipmentType {
    Hand,
    HumanoidArmor,
    AnimalArmor,
    Saddle,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct EquipmentSlotData {
    pub slot_type: EquipmentType,
    pub entity_id: i32,
    pub max_count: i32,
    pub index: i32,
    pub name: Cow<'static, str>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
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

    pub fn get_entity_slot_id(&self) -> i32 {
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

    pub fn get_offset_entity_slot_id(&self, offset: i32) -> i32 {
        self.get_entity_slot_id() + offset
    }

    pub fn slot_type(&self) -> EquipmentType {
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

    pub fn is_armor_slot(&self) -> bool {
        matches!(
            self.slot_type(),
            EquipmentType::HumanoidArmor | EquipmentType::AnimalArmor
        )
    }

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

#[derive(Clone, Debug)]
pub enum EntityTypeOrTag {
    Tag(&'static Tag),
    Single(&'static EntityType),
}

impl Hash for EntityTypeOrTag {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            EntityTypeOrTag::Tag(tag) => {
                for x in tag.0 {
                    x.hash(state);
                }
            }
            EntityTypeOrTag::Single(entity_type) => {
                entity_type.id.hash(state);
            }
        }
    }
}

#[derive(Clone, Debug, Hash)]
pub struct EnchantableImpl;
#[derive(Clone, Debug, Hash)]
pub struct EquippableImpl {
    pub slot: &'static EquipmentSlot,
    pub equip_sound: &'static str,
    pub asset_id: Option<&'static str>,
    pub camera_overlay: Option<&'static str>,
    // pub allowed_entities: Option<&'static [&'static str]>,
    pub allowed_entities: Option<&'static [EntityTypeOrTag]>,
    pub dispensable: bool,
    pub swappable: bool,
    pub damage_on_hurt: bool,
    pub equip_on_interact: bool,
    pub can_be_sheared: bool,
    pub shearing_sound: Option<&'static str>,
}
impl DataComponentImpl for EquippableImpl {
    #[inline]
    fn get_enum() -> DataComponent
    where
        Self: Sized,
    {
        Equippable
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
}
#[derive(Clone, Debug, Hash)]
pub struct RepairableImpl;
#[derive(Clone, Debug, Hash)]
pub struct GliderImpl;
#[derive(Clone, Debug, Hash)]
pub struct TooltipStyleImpl;
#[derive(Clone, Debug, Hash)]
pub struct DeathProtectionImpl;
#[derive(Clone, Debug, Hash)]
pub struct BlocksAttacksImpl;
#[derive(Clone, Debug, Hash)]
pub struct StoredEnchantmentsImpl;
#[derive(Clone, Debug, Hash)]
pub struct DyedColorImpl;
#[derive(Clone, Debug, Hash)]
pub struct MapColorImpl;
#[derive(Clone, Debug, Hash)]
pub struct MapIdImpl;
#[derive(Clone, Debug, Hash)]
pub struct MapDecorationsImpl;
#[derive(Clone, Debug, Hash)]
pub struct MapPostProcessingImpl;
#[derive(Clone, Debug, Hash)]
pub struct ChargedProjectilesImpl;
#[derive(Clone, Debug, Hash)]
pub struct BundleContentsImpl;
#[derive(Clone, Debug, Hash)]
pub struct PotionContentsImpl;
#[derive(Clone, Debug, Hash)]
pub struct PotionDurationScaleImpl;
#[derive(Clone, Debug, Hash)]
pub struct SuspiciousStewEffectsImpl;
#[derive(Clone, Debug, Hash)]
pub struct WritableBookContentImpl;
#[derive(Clone, Debug, Hash)]
pub struct WrittenBookContentImpl;
#[derive(Clone, Debug, Hash)]
pub struct TrimImpl;
#[derive(Clone, Debug, Hash)]
pub struct DebugStickStateImpl;
#[derive(Clone, Debug, Hash)]
pub struct EntityDataImpl;
#[derive(Clone, Debug, Hash)]
pub struct BucketEntityDataImpl;
#[derive(Clone, Debug, Hash)]
pub struct BlockEntityDataImpl;
#[derive(Clone, Debug, Hash)]
pub struct InstrumentImpl;
#[derive(Clone, Debug, Hash)]
pub struct ProvidesTrimMaterialImpl;
#[derive(Clone, Debug, Hash)]
pub struct OminousBottleAmplifierImpl;
#[derive(Clone, Debug, Hash)]
pub struct JukeboxPlayableImpl {
    pub song: &'static str,
}
impl DataComponentImpl for JukeboxPlayableImpl {
    #[inline]
    fn get_enum() -> DataComponent
    where
        Self: Sized,
    {
        JukeboxPlayable
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
}
#[derive(Clone, Debug, Hash)]
pub struct ProvidesBannerPatternsImpl;
#[derive(Clone, Debug, Hash)]
pub struct RecipesImpl;
#[derive(Clone, Debug, Hash)]
pub struct LodestoneTrackerImpl;
#[derive(Clone, Debug, Hash)]
pub struct FireworkExplosionImpl;
#[derive(Clone, Debug, Hash)]
pub struct FireworksImpl;
#[derive(Clone, Debug, Hash)]
pub struct ProfileImpl;
#[derive(Clone, Debug, Hash)]
pub struct NoteBlockSoundImpl;
#[derive(Clone, Debug, Hash)]
pub struct BannerPatternsImpl;
#[derive(Clone, Debug, Hash)]
pub struct BaseColorImpl;
#[derive(Clone, Debug, Hash)]
pub struct PotDecorationsImpl;
#[derive(Clone, Debug, Hash)]
pub struct ContainerImpl;
#[derive(Clone, Debug, Hash)]
pub struct BlockStateImpl;
#[derive(Clone, Debug, Hash)]
pub struct BeesImpl;
#[derive(Clone, Debug, Hash)]
pub struct LockImpl;
#[derive(Clone, Debug, Hash)]
pub struct ContainerLootImpl;
#[derive(Clone, Debug, Hash)]
pub struct BreakSoundImpl;
#[derive(Clone, Debug, Hash)]
pub struct VillagerVariantImpl;
#[derive(Clone, Debug, Hash)]
pub struct WolfVariantImpl;
#[derive(Clone, Debug, Hash)]
pub struct WolfSoundVariantImpl;
#[derive(Clone, Debug, Hash)]
pub struct WolfCollarImpl;
#[derive(Clone, Debug, Hash)]
pub struct FoxVariantImpl;
#[derive(Clone, Debug, Hash)]
pub struct SalmonSizeImpl;
#[derive(Clone, Debug, Hash)]
pub struct ParrotVariantImpl;
#[derive(Clone, Debug, Hash)]
pub struct TropicalFishPatternImpl;
#[derive(Clone, Debug, Hash)]
pub struct TropicalFishBaseColorImpl;
#[derive(Clone, Debug, Hash)]
pub struct TropicalFishPatternColorImpl;
#[derive(Clone, Debug, Hash)]
pub struct MooshroomVariantImpl;
#[derive(Clone, Debug, Hash)]
pub struct RabbitVariantImpl;
#[derive(Clone, Debug, Hash)]
pub struct PigVariantImpl;
#[derive(Clone, Debug, Hash)]
pub struct CowVariantImpl;
#[derive(Clone, Debug, Hash)]
pub struct ChickenVariantImpl;
#[derive(Clone, Debug, Hash)]
pub struct FrogVariantImpl;
#[derive(Clone, Debug, Hash)]
pub struct HorseVariantImpl;
#[derive(Clone, Debug, Hash)]
pub struct PaintingVariantImpl;
#[derive(Clone, Debug, Hash)]
pub struct LlamaVariantImpl;
#[derive(Clone, Debug, Hash)]
pub struct AxolotlVariantImpl;
#[derive(Clone, Debug, Hash)]
pub struct CatVariantImpl;
#[derive(Clone, Debug, Hash)]
pub struct CatCollarImpl;
#[derive(Clone, Debug, Hash)]
pub struct SheepColorImpl;
#[derive(Clone, Debug, Hash)]
pub struct ShulkerColorImpl;
