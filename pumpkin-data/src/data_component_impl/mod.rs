#![allow(dead_code)]

use crate::Block;
use crate::BlockId;
use crate::data_component::DataComponent;
use crate::effect::StatusEffect;
use crate::entity_type::EntityType;
use crate::sound::Sound;
use crate::tag::Taggable;
use crc_fast::CrcAlgorithm::Crc32Iscsi;
use crc_fast::Digest;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use std::any::Any;
use std::borrow::Cow;

pub trait DataComponentImpl: Send + Sync {
    fn write_data(&self) -> NbtTag {
        NbtTag::End
    }
    fn get_hash(&self) -> i32 {
        0
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

macro_rules! default_impl {
    ($t: ident) => {
        fn equal(&self, other: &dyn crate::data_component_impl::DataComponentImpl) -> bool {
            self == crate::data_component_impl::get::<Self>(other)
        }
        #[inline]
        fn get_enum() -> crate::data_component::DataComponent
        where
            Self: Sized,
        {
            crate::data_component::DataComponent::$t
        }
        fn get_self_enum(&self) -> crate::data_component::DataComponent {
            crate::data_component::DataComponent::$t
        }
        fn to_dyn(self) -> Box<dyn crate::data_component_impl::DataComponentImpl> {
            Box::new(self)
        }
        fn clone_dyn(&self) -> Box<dyn crate::data_component_impl::DataComponentImpl> {
            Box::new(self.clone())
        }
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn as_mut_any(&mut self) -> &mut dyn std::any::Any {
            self
        }
    };
}

pub(crate) use default_impl;

pub fn get_str_hash(val: &str) -> u32 {
    let mut digest = Digest::new(Crc32Iscsi);
    digest.update(&[12u8]);
    digest.update(&(val.len() as u32).to_le_bytes());
    let byte = val.as_bytes();
    for i in byte {
        digest.update(&[*i, 0u8]);
    }
    digest.finalize() as u32
}

pub fn get_i32_hash(val: i32) -> u32 {
    let mut digest = Digest::new(Crc32Iscsi);
    digest.update(&[8u8]);
    digest.update(&val.to_le_bytes());
    digest.finalize() as u32
}

pub fn get_f32_hash(val: f32) -> u32 {
    let mut digest = Digest::new(Crc32Iscsi);
    digest.update(&[7u8]);
    digest.update(&val.to_bits().to_le_bytes());
    digest.finalize() as u32
}

pub fn get_idor_hash(val: &IdOr<basic::SoundEvent>) -> u32 {
    let mut digest = Digest::new(Crc32Iscsi);
    digest.update(&[6u8]);
    match val {
        IdOr::Id(sound) => {
            digest.update(&[1u8]);
            digest.update(&get_str_hash(sound.to_name()).to_le_bytes());
        }
        IdOr::Value(sound) => {
            digest.update(&[2u8]);
            digest.update(&get_str_hash(sound.sound_name.as_str()).to_le_bytes());
            if let Some(range) = sound.range {
                digest.update(&[1u8]);
                digest.update(&get_f32_hash(range).to_le_bytes());
            } else {
                digest.update(&[0u8]);
            }
        }
    }
    digest.finalize() as u32
}

pub fn put_idor(nbt: &mut NbtCompound, key: &str, val: &IdOr<basic::SoundEvent>) {
    match val {
        IdOr::Id(id) => nbt.put_string(key, format!("minecraft:{}", id.to_name())),
        IdOr::Value(sound) => {
            let mut sound_compound = NbtCompound::new();

            sound_compound.put_string("sound_id", sound.sound_name.clone());
            if let Some(range) = sound.range {
                sound_compound.put_float("range", range);
            }
            nbt.put(key, NbtTag::Compound(sound_compound));
        }
    }
}

pub fn get_idor(nbt: &NbtCompound, key: &str, default: Sound) -> IdOr<basic::SoundEvent> {
    if let Some(sound) = nbt.get_string(key) {
        let sound = sound.strip_prefix("minecraft:").unwrap_or(sound);
        IdOr::Id(Sound::from_name(sound).unwrap_or(default))
    } else if let Some(sound_compound) = nbt.get_compound(key) {
        let sound_name = sound_compound
            .get_string("sound_id")
            .expect("SoundEvent compound must have a 'sound_id' field");
        let range = sound_compound.get_float("range");
        IdOr::Value(basic::SoundEvent {
            sound_name: sound_name.to_string(),
            range,
        })
    } else {
        IdOr::Id(default)
    }
}

pub fn get_idset_hash<T: IDSetContent>(val: &IDSet<T>) -> u32 {
    let mut digest = Digest::new(Crc32Iscsi);
    match val {
        IDSet::Tag(tag) => {
            digest.update(&[1u8]);
            digest.update(&get_str_hash(tag).to_le_bytes());
        }
        IDSet::IDs(ids) => {
            digest.update(&[2u8]);
            for id in ids.iter() {
                digest.update(&[3u8]);
                digest.update(&get_i32_hash(id.registry_id() as i32).to_le_bytes());
            }
        }
    }
    digest.finalize() as u32
}

pub trait IDSetContent {
    fn registry_id(&self) -> u16;
    fn to_string(&self) -> String;
    fn from_id(id: u16) -> Option<&'static Self>;
    fn from_str(name: &str) -> Option<&'static Self>;
}

impl IDSetContent for Block {
    fn registry_id(&self) -> u16 {
        Taggable::registry_id(self)
    }

    fn from_id(id: u16) -> Option<&'static Self> {
        BlockId::new(id).map(Self::from_id)
    }

    fn from_str(name: &str) -> Option<&'static Self> {
        Block::from_name(name)
    }

    fn to_string(&self) -> String {
        self.name.to_string()
    }
}

#[derive(Clone, Hash, PartialEq, Debug)]
pub enum IDSet<T: IDSetContent + 'static> {
    Tag(Cow<'static, str>),
    IDs(Cow<'static, [&'static T]>),
}

impl<T: IDSetContent + 'static> IDSet<T> {
    pub fn read(data: &NbtTag) -> Option<Self> {
        match data.clone() {
            NbtTag::String(tag) => {
                if tag.starts_with("#") {
                    Some(Self::Tag(Cow::Owned(tag.strip_prefix("#")?.to_string())))
                } else {
                    Some(Self::IDs(Cow::Owned([T::from_str(tag.as_ref())?].to_vec())))
                }
            }
            NbtTag::List(nbt_tags) => {
                let mut ids = Vec::<&T>::new();
                for nbt in nbt_tags {
                    if let NbtTag::String(id) = nbt
                        && let Some(instance) = T::from_str(id.as_ref())
                    {
                        ids.push(instance);
                    }
                }
                Some(Self::IDs(Cow::Owned(ids)))
            }
            _ => None,
        }
    }

    pub fn write(&self, compound: &mut NbtCompound, key: &str) {
        match self {
            Self::Tag(cow) => {
                let mut tag = cow.to_string();
                tag.insert(0, '#');
                compound.put_string(key, tag);
            }
            Self::IDs(arr) => {
                let id_vec: Vec<NbtTag> = arr
                    .iter()
                    .map(|x| NbtTag::String(x.to_string().into()))
                    .collect();
                compound.put_list(key, id_vec);
            }
        }
    }
}

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
    pub const fn get_slot_index(&self) -> i32 {
        match self {
            Self::MainHand(data) => data.index,
            Self::OffHand(data) => data.index,
            Self::Feet(data) => data.index,
            Self::Legs(data) => data.index,
            Self::Chest(data) => data.index,
            Self::Head(data) => data.index,
            Self::Body(data) => data.index,
            Self::Saddle(data) => data.index,
        }
    }

    #[must_use]
    pub fn from_slot_index(name: i32) -> Option<&'static Self> {
        match name {
            0 => Some(&Self::MAIN_HAND),
            1 => Some(&Self::FEET),
            2 => Some(&Self::LEGS),
            3 => Some(&Self::CHEST),
            4 => Some(&Self::HEAD),
            5 => Some(&Self::OFF_HAND),
            6 => Some(&Self::BODY),
            7 => Some(&Self::SADDLE),
            _ => None,
        }
    }

    #[must_use]
    pub fn get_from_name(name: &str) -> Option<&'static Self> {
        match name {
            "mainhand" => Some(&Self::MAIN_HAND),
            "offhand" => Some(&Self::OFF_HAND),
            "feet" => Some(&Self::FEET),
            "legs" => Some(&Self::LEGS),
            "chest" => Some(&Self::CHEST),
            "head" => Some(&Self::HEAD),
            "body" => Some(&Self::BODY),
            "saddle" => Some(&Self::SADDLE),
            _ => None,
        }
    }

    #[must_use]
    pub fn to_name(&self) -> &Cow<'static, str> {
        match self {
            Self::MainHand(data) => &data.name,
            Self::OffHand(data) => &data.name,
            Self::Feet(data) => &data.name,
            Self::Legs(data) => &data.name,
            Self::Chest(data) => &data.name,
            Self::Head(data) => &data.name,
            Self::Body(data) => &data.name,
            Self::Saddle(data) => &data.name,
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
    Tag(&'static crate::tag::Tag),
    Single(&'static EntityType),
}

impl std::hash::Hash for EntityTypeOrTag {
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
pub enum IdOr<T> {
    Id(Sound),
    Value(T),
}

pub mod basic;
pub mod block_entity;
pub mod book;
pub mod combat;
pub mod entity_variant;
pub mod food;
pub mod utility;

// Re-export all component implementations
pub use basic::*;
pub use block_entity::*;
pub use book::*;
pub use combat::*;
pub use entity_variant::*;
pub use food::*;
pub use utility::*;

#[must_use]
pub fn read_data(id: DataComponent, data: &NbtTag) -> Option<Box<dyn DataComponentImpl>> {
    match id {
        DataComponent::MaxStackSize => Some(MaxStackSizeImpl::read_data(data)?.to_dyn()),
        DataComponent::CustomData => Some(CustomDataImpl::read_data(data)?.to_dyn()),
        DataComponent::Enchantments => Some(EnchantmentsImpl::read_data(data)?.to_dyn()),
        DataComponent::Damage => Some(DamageImpl::read_data(data)?.to_dyn()),
        DataComponent::Unbreakable => Some(UnbreakableImpl::read_data(data)?.to_dyn()),
        DataComponent::DamageResistant => Some(DamageResistantImpl::read_data(data)?.to_dyn()),
        DataComponent::PotionContents => Some(PotionContentsImpl::read_data(data)?.to_dyn()),
        DataComponent::Fireworks => Some(FireworksImpl::read_data(data)?.to_dyn()),
        DataComponent::FireworkExplosion => Some(FireworkExplosionImpl::read_data(data)?.to_dyn()),
        DataComponent::CustomName => Some(CustomNameImpl::read_data(data)?.to_dyn()),
        DataComponent::ItemModel => Some(ItemModelImpl::read_data(data)?.to_dyn()),
        DataComponent::Consumable => Some(ConsumableImpl::read_data(data)?.to_dyn()),
        DataComponent::Equippable => Some(EquippableImpl::read_data(data)?.to_dyn()),
        DataComponent::StoredEnchantments => {
            Some(StoredEnchantmentsImpl::read_data(data)?.to_dyn())
        }
        DataComponent::UseCooldown => Some(UseCooldownImpl::read_data(data)?.to_dyn()),
        DataComponent::MapId => Some(MapIdImpl::read_data(data)?.to_dyn()),
        DataComponent::ChargedProjectiles => {
            Some(ChargedProjectilesImpl::read_data(data)?.to_dyn())
        }
        DataComponent::BlockEntityData => Some(BlockEntityDataImpl::read_data(data)?.to_dyn()),
        DataComponent::BundleContents => Some(BundleContentsImpl::read_data(data)?.to_dyn()),
        DataComponent::Container => Some(ContainerImpl::read_data(data)?.to_dyn()),
        DataComponent::WrittenBookContent => {
            Some(WrittenBookContentImpl::read_data(data)?.to_dyn())
        }
        DataComponent::WritableBookContent => {
            Some(WritableBookContentImpl::read_data(data)?.to_dyn())
        }
        DataComponent::OminousBottleAmplifier => {
            Some(OminousBottleAmplifierImpl::read_data(data)?.to_dyn())
        }
        DataComponent::BlockState => Some(BlockStateImpl::read_data(data)?.to_dyn()),
        DataComponent::Profile => Some(ProfileImpl::read_data(data)?.to_dyn()),
        DataComponent::ChickenVariant => Some(ChickenVariantImpl::read_data(data)?.to_dyn()),
        DataComponent::VillagerVariant => Some(VillagerVariantImpl::read_data(data)?.to_dyn()),
        DataComponent::WolfVariant => Some(WolfVariantImpl::read_data(data)?.to_dyn()),
        DataComponent::WolfSoundVariant => Some(WolfSoundVariantImpl::read_data(data)?.to_dyn()),
        DataComponent::WolfCollar => Some(WolfCollarImpl::read_data(data)?.to_dyn()),
        DataComponent::FoxVariant => Some(FoxVariantImpl::read_data(data)?.to_dyn()),
        DataComponent::SalmonSize => Some(SalmonSizeImpl::read_data(data)?.to_dyn()),
        DataComponent::ParrotVariant => Some(ParrotVariantImpl::read_data(data)?.to_dyn()),
        DataComponent::TropicalFishPattern => {
            Some(TropicalFishPatternImpl::read_data(data)?.to_dyn())
        }
        DataComponent::TropicalFishBaseColor => {
            Some(TropicalFishBaseColorImpl::read_data(data)?.to_dyn())
        }
        DataComponent::TropicalFishPatternColor => {
            Some(TropicalFishPatternColorImpl::read_data(data)?.to_dyn())
        }
        DataComponent::MooshroomVariant => Some(MooshroomVariantImpl::read_data(data)?.to_dyn()),
        DataComponent::RabbitVariant => Some(RabbitVariantImpl::read_data(data)?.to_dyn()),
        DataComponent::PigVariant => Some(PigVariantImpl::read_data(data)?.to_dyn()),
        DataComponent::PigSoundVariant => Some(PigSoundVariantImpl::read_data(data)?.to_dyn()),
        DataComponent::CowVariant => Some(CowVariantImpl::read_data(data)?.to_dyn()),
        DataComponent::CowSoundVariant => Some(CowSoundVariantImpl::read_data(data)?.to_dyn()),
        DataComponent::ChickenSoundVariant => {
            Some(ChickenSoundVariantImpl::read_data(data)?.to_dyn())
        }
        DataComponent::ZombieNautilusVariant => {
            Some(ZombieNautilusVariantImpl::read_data(data)?.to_dyn())
        }
        DataComponent::FrogVariant => Some(FrogVariantImpl::read_data(data)?.to_dyn()),
        DataComponent::HorseVariant => Some(HorseVariantImpl::read_data(data)?.to_dyn()),
        DataComponent::PaintingVariant => Some(PaintingVariantImpl::read_data(data)?.to_dyn()),
        DataComponent::LlamaVariant => Some(LlamaVariantImpl::read_data(data)?.to_dyn()),
        DataComponent::AxolotlVariant => Some(AxolotlVariantImpl::read_data(data)?.to_dyn()),
        DataComponent::CatVariant => Some(CatVariantImpl::read_data(data)?.to_dyn()),
        DataComponent::CatSoundVariant => Some(CatSoundVariantImpl::read_data(data)?.to_dyn()),
        DataComponent::CatCollar => Some(CatCollarImpl::read_data(data)?.to_dyn()),
        DataComponent::SheepColor => Some(SheepColorImpl::read_data(data)?.to_dyn()),
        DataComponent::ShulkerColor => Some(ShulkerColorImpl::read_data(data)?.to_dyn()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash() {
        assert_eq!(get_str_hash("minecraft:sharpness"), 2734053906u32);
        assert_eq!(get_i32_hash(3), 3795317917u32);
        assert_eq!(
            EnchantmentsImpl {
                enchantment: Cow::Borrowed(&[(&crate::Enchantment::SHARPNESS, 2)]),
            }
            .get_hash(),
            -1580618251i32
        );
        assert_eq!(MaxStackSizeImpl { size: 99 }.get_hash(), -1632321551i32);
        assert_eq!(MapIdImpl { id: 10 }.get_hash(), -919192125i32);
    }
}
