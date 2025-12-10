#![allow(dead_code)]

use crate::attributes::Attributes;
use crate::data_component::DataComponent;
use crate::data_component::DataComponent::*;
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
pub fn read_data(id: DataComponent, data: &NbtTag) -> Option<Box<dyn DataComponentImpl>> {
    match id {
        MaxStackSize => Some(MaxStackSizeImpl::read_data(data)?.to_dyn()),
        Enchantments => Some(EnchantmentsImpl::read_data(data)?.to_dyn()),
        Damage => Some(DamageImpl::read_data(data)?.to_dyn()),
        _ => todo!(),
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
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct CustomDataImpl;
impl DataComponentImpl for CustomDataImpl {
    default_impl!(CustomData);
}

#[derive(Clone, Debug, Hash, PartialEq)]
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
    fn get_hash(&self) -> i32 {
        get_i32_hash(self.size as i32) as i32
    }

    default_impl!(MaxStackSize);
}
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct MaxDamageImpl {
    pub max_damage: i32,
}
impl DataComponentImpl for MaxDamageImpl {
    default_impl!(MaxDamage);
}
#[derive(Clone, Debug, Hash, PartialEq)]
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
#[derive(Clone, Hash, PartialEq)]
pub struct UnbreakableImpl;
#[derive(Clone, Hash, PartialEq)]
pub struct CustomNameImpl {
    // TODO make TextComponent const
    pub name: &'static str,
}
impl DataComponentImpl for CustomNameImpl {
    default_impl!(CustomName);
}
#[derive(Clone, Hash, PartialEq)]
pub struct ItemNameImpl {
    // TODO make TextComponent const
    pub name: &'static str,
}
impl DataComponentImpl for ItemNameImpl {
    default_impl!(ItemName);
}
#[derive(Clone, Hash, PartialEq)]
pub struct ItemModelImpl;
#[derive(Clone, Hash, PartialEq)]
pub struct LoreImpl;
#[derive(Clone, Hash, PartialEq)]
pub struct RarityImpl;
#[derive(Clone, Hash, PartialEq)]
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
fn test_hash() {
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
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct CanPlaceOnImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct CanBreakImpl;

#[derive(Clone, Copy, Debug, PartialEq, Hash)]
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
        unsafe { (*(&self.amount as *const f64 as *const u64)).hash(state) };
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
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct CustomModelDataImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct TooltipDisplayImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct RepairCostImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct CreativeSlotLockImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct EnchantmentGlintOverrideImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
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
        unsafe { (*(&self.saturation as *const f32 as *const u32)).hash(state) };
        self.can_always_eat.hash(state);
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct ConsumableImpl {
    pub consume_seconds: f32,
    // TODO: more
}

impl ConsumableImpl {
    pub fn consume_ticks(&self) -> i32 {
        (self.consume_seconds * 20.0) as i32
    }
}

impl DataComponentImpl for ConsumableImpl {
    default_impl!(Consumable);
}
impl Hash for ConsumableImpl {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe { (*(&self.consume_seconds as *const f32 as *const u32)).hash(state) };
    }
}
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct UseRemainderImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct UseCooldownImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct DamageResistantImpl;

#[derive(Clone, Hash, PartialEq)]
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
            unsafe { (*(&val as *const f32 as *const u32)).hash(state) };
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
        unsafe { (*(&self.default_mining_speed as *const f32 as *const u32)).hash(state) };
        self.damage_per_block.hash(state);
        self.can_destroy_blocks_in_creative.hash(state);
    }
}
#[derive(Clone, Debug, Hash, PartialEq)]
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

#[derive(Clone, PartialEq)]
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

#[derive(Clone, Debug, Hash, PartialEq)]
pub struct EnchantableImpl;
#[derive(Clone, Hash, PartialEq)]
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
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct RepairableImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct GliderImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct TooltipStyleImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct DeathProtectionImpl;
impl DataComponentImpl for DeathProtectionImpl {
    default_impl!(DeathProtection);
}
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct BlocksAttacksImpl;

impl DataComponentImpl for BlocksAttacksImpl {
    default_impl!(BlocksAttacks);
}
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct StoredEnchantmentsImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct DyedColorImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct MapColorImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct MapIdImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct MapDecorationsImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct MapPostProcessingImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct ChargedProjectilesImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct BundleContentsImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct PotionContentsImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct PotionDurationScaleImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct SuspiciousStewEffectsImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct WritableBookContentImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct WrittenBookContentImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct TrimImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct DebugStickStateImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct EntityDataImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct BucketEntityDataImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct BlockEntityDataImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct InstrumentImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct ProvidesTrimMaterialImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct OminousBottleAmplifierImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct JukeboxPlayableImpl {
    pub song: &'static str,
}
impl DataComponentImpl for JukeboxPlayableImpl {
    default_impl!(JukeboxPlayable);
}
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct ProvidesBannerPatternsImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct RecipesImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct LodestoneTrackerImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct FireworkExplosionImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct FireworksImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct ProfileImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct NoteBlockSoundImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct BannerPatternsImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct BaseColorImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct PotDecorationsImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct ContainerImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct BlockStateImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct BeesImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct LockImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct ContainerLootImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct BreakSoundImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct VillagerVariantImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct WolfVariantImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct WolfSoundVariantImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct WolfCollarImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct FoxVariantImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct SalmonSizeImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct ParrotVariantImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct TropicalFishPatternImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct TropicalFishBaseColorImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct TropicalFishPatternColorImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct MooshroomVariantImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct RabbitVariantImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct PigVariantImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct CowVariantImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct ChickenVariantImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct FrogVariantImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct HorseVariantImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct PaintingVariantImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct LlamaVariantImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct AxolotlVariantImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct CatVariantImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct CatCollarImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct SheepColorImpl;
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct ShulkerColorImpl;
