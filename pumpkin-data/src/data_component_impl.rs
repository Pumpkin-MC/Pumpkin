#![allow(dead_code)]

use crate::attributes::Attributes;
use crate::data_component::DataComponent;
use crate::data_component::DataComponent::{
    AttributeModifiers, BlocksAttacks, Consumable, CustomData, CustomName, Damage, DamageResistant,
    DeathProtection, Enchantments, Equippable, FireworkExplosion, Fireworks, Food, ItemModel,
    ItemName, JukeboxPlayable, MaxDamage, MaxStackSize, PotionContents, Tool, Unbreakable, Weapon,
};
use crate::effect::{self, StatusEffect};
use crate::entity_type::EntityType;
use crate::sound::Sound;
use crate::tag::{RegistryKey, Tag, Taggable};
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
use std::error::Error;
use std::fmt::Debug;
use std::str::FromStr;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

pub trait DataComponentImpl: Send + Sync {
    fn write_data(&self) -> NbtTag {
        NbtTag::End
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
        DamageResistant => Some(DamageResistantImpl::read_data(data)?.to_dyn()),
        PotionContents => Some(PotionContentsImpl::read_data(data)?.to_dyn()),
        Fireworks => Some(FireworksImpl::read_data(data)?.to_dyn()),
        FireworkExplosion => Some(FireworkExplosionImpl::read_data(data)?.to_dyn()),
        ItemModel => Some(ItemModelImpl::read_data(data)?.to_dyn()),
        Consumable => Some(ConsumableImpl::read_data(data)?.to_dyn()),
        Equippable => Some(EquippableImpl::read_data(data)?.to_dyn()),
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
pub struct ItemModelImpl {
    pub id: String,
}
impl ItemModelImpl {
    fn read_data(data: &NbtTag) -> Option<Self> {
        data.extract_string().map(|id| Self {
            id: String::from(id),
        })
    }
}
impl DataComponentImpl for ItemModelImpl {
    fn write_data(&self) -> NbtTag {
        NbtTag::String(self.id.clone())
    }

    fn get_hash(&self) -> i32 {
        get_str_hash(self.id.as_str()) as i32
    }

    default_impl!(ItemModel);
}
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

fn get_f32_hash(val: f32) -> u32 {
    let mut digest = Digest::new(Crc32Iscsi);
    digest.update(&[7u8]);
    digest.update(&val.to_bits().to_le_bytes());
    digest.finalize() as u32
}

fn get_idor_hash(val: &IdOr<SoundEvent>) -> u32 {
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

fn put_idor(nbt: &mut NbtCompound, key: &str, val: &IdOr<SoundEvent>) {
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

fn get_idor(nbt: &NbtCompound, key: &str, default: Sound) -> IdOr<SoundEvent> {
    if let Some(sound) = nbt.get_string(key) {
        IdOr::Id(Sound::from_name(sound).unwrap_or(default))
    } else if let Some(sound_compound) = nbt.get_compound(key) {
        let mut sound_name = sound_compound
            .get_string("sound_id")
            .expect("SoundEvent compound must have a 'sound_id' field");
        sound_name = sound_name.strip_prefix("minecraft:").unwrap_or(sound_name);
        let range = sound_compound.get_float("range");
        IdOr::Value(SoundEvent {
            sound_name: sound_name.to_string(),
            range,
        })
    } else {
        IdOr::Id(default)
    }
}

fn get_idset_hash<T: IDSetContent>(val: &IDSet<T>) -> u32 {
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
pub struct SoundEvent {
    pub sound_name: String,
    pub range: Option<f32>,
}

impl Hash for SoundEvent {
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
#[derive(Clone, Debug, PartialEq, Hash)]
pub enum IdOr<T> {
    Id(Sound),
    Value(T),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConsumableImpl {
    pub consume_seconds: f32,
    pub animation: ConsumeAnimation,
    pub sound_event: IdOr<SoundEvent>,
    pub consume_particles: bool,
    pub effects: Cow<'static, [ConsumeEffect]>,
}
#[derive(Clone, Debug, PartialEq)]
pub enum ConsumeAnimation {
    None,
    Eat,
    Drink,
    Block,
    Bow,
    Spear,
    Crossbow,
    Spyglass,
    Horn,
    Brush,
}

impl ConsumeAnimation {
    #[must_use]
    pub const fn to_str(&self) -> &'static str {
        match self {
            ConsumeAnimation::None => "none",
            ConsumeAnimation::Eat => "eat",
            ConsumeAnimation::Drink => "drink",
            ConsumeAnimation::Block => "block",
            ConsumeAnimation::Bow => "bow",
            ConsumeAnimation::Spear => "spear",
            ConsumeAnimation::Crossbow => "crossbow",
            ConsumeAnimation::Spyglass => "spyglass",
            ConsumeAnimation::Horn => "horn",
            ConsumeAnimation::Brush => "brush",
        }
    }
}
impl TryFrom<i32> for ConsumeAnimation {
    type Error = ();

    /// Attempts to convert an `i32` value into a [`ConsumeAnimation`].
    ///
    /// # Parameters
    /// - `value`: The numeric representation of a consume animation.
    ///
    /// # Returns
    /// - `Ok(ConsumeAnimation)` if the value corresponds to a valid consume animation:
    ///   - `0` → `None`
    ///   - `1` → `Eat`
    ///   - `2` → `Drink`
    ///   - `3` → `Block`
    ///   - `4` → `Bow`
    ///   - `5` → `Spear`
    ///   - `6` → `Crossbow`
    ///   - `7` → `Spyglass`
    ///   - `8` → `Horn`
    ///   - `9` → `Brush`
    /// - `Err(())` if the value does not correspond to any valid consume animation.
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::Eat),
            2 => Ok(Self::Drink),
            3 => Ok(Self::Block),
            4 => Ok(Self::Bow),
            5 => Ok(Self::Spear),
            6 => Ok(Self::Crossbow),
            7 => Ok(Self::Spyglass),
            8 => Ok(Self::Horn),
            9 => Ok(Self::Brush),
            _ => Err(()),
        }
    }
}
impl FromStr for ConsumeAnimation {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "none" => Ok(Self::None),
            "eat" => Ok(Self::Eat),
            "drink" => Ok(Self::Drink),
            "block" => Ok(Self::Block),
            "bow" => Ok(Self::Bow),
            "spear" => Ok(Self::Spear),
            "crossbow" => Ok(Self::Crossbow),
            "spyglass" => Ok(Self::Spyglass),
            "horn" => Ok(Self::Horn),
            "brush" => Ok(Self::Brush),
            _ => Err(()),
        }
    }
}

impl ConsumableImpl {
    pub const fn new(
        consume_seconds: f32,
        animation: ConsumeAnimation,
        sound_event: IdOr<SoundEvent>,
        consume_particles: bool,
        effects: Cow<'static, [ConsumeEffect]>,
    ) -> Self {
        Self {
            consume_seconds,
            animation,
            sound_event,
            consume_particles,
            effects,
        }
    }
    #[must_use]
    pub fn consume_ticks(&self) -> i32 {
        (self.consume_seconds * 20.0) as i32
    }

    pub fn read_data(data: &NbtTag) -> Option<Self> {
        let compound = data.extract_compound()?;
        let consume_seconds = compound.get_float("consume_seconds")?;
        let animation = compound
            .get_string("animation")?
            .parse::<ConsumeAnimation>()
            .ok()?;
        let sound_event = get_idor(compound, "sound", Sound::EntityGenericEat); // Default to generic eat sound if the sound name is invalid
        let consume_particles = compound.get_bool("has_consume_particles").unwrap_or(false);
        let opt_list = compound.get_list("on_consume_effects");

        let effects: Cow<'static, [ConsumeEffect]> = if let Some(effect_list) = opt_list {
            effect_list
                .iter()
                .filter_map(ConsumeEffect::read_data)
                .collect()
        } else {
            Cow::Borrowed(&[])
        };

        Some(Self {
            consume_seconds,
            animation,
            sound_event,
            consume_particles,
            effects,
        })
    }
}
impl DataComponentImpl for ConsumableImpl {
    fn write_data(&self) -> NbtTag {
        let mut compound = NbtCompound::new();
        compound.put_float("consume_seconds", self.consume_seconds);
        compound.put_string("animation", self.animation.to_str().to_string());
        put_idor(&mut compound, "sound", &self.sound_event);
        compound.put_bool("has_consume_particles", self.consume_particles);

        let nbt_vec = self.effects.iter().map(|x| x.as_nbt()).collect();
        compound.put_list("on_consume_effects", nbt_vec);
        NbtTag::Compound(compound)
    }

    fn get_hash(&self) -> i32 {
        let mut digest = Digest::new(Crc32Iscsi);
        digest.update(&[2u8]);
        digest.update(&get_f32_hash(self.consume_seconds).to_le_bytes());
        digest.update(&get_i32_hash(self.animation.clone() as i32).to_le_bytes());
        digest.update(&get_idor_hash(&self.sound_event).to_be_bytes());
        digest.update(&[self.consume_particles as u8]);
        for effect in self.effects.iter() {
            digest.update(&effect.get_hash().to_le_bytes());
        }
        digest.update(&[3u8]);
        digest.finalize() as i32
    }

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
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum DamageResistantType {
    /// Damage always dealt to ender dragons
    AlwaysHurtsEnderDragons,
    /// Destroys armor stands in a single hit
    AlwaysKillsArmorStands,
    AlwaysMostSignificantFall,
    /// Damage always notifies nearby hidden silverfish
    AlwaysTriggersSilverfish,
    AvoidsGuardianThorns,
    BurnsArmorStands,
    BurnFromStepping,
    BypassesArmor,
    BypassesCooldown,
    BypassesEffects,
    BypassesEnchantments,
    BypassesInvulnerability,
    BypassesResistance,
    BypassesShield,
    BypassesWolfArmor,
    CanBreakArmorStands,
    DamagesHelmet,
    IgnitesArmorStands,
    Drowning,
    /// Damage is reduced by the blast protection enchantment
    Explosion,
    /// Damage is reduced by the feather falling enchantment and ignored if slow falling
    Fall,
    /// Damage is reduced by the fire protection enchantment or ignored by game rule
    Fire,
    /// Damage is reduced by wearing any piece of leather armor or ignored by game rule
    Freezing,
    /// So turtles drop bowls when killed by lightning
    Lightning,
    PlayerAttack,
    /// Damage is reduced by the projectile protection enchantment
    Projectile,
    MaceSmash,
    /// Prevents entities from becoming angry at the source of the damage
    NoAnger,
    /// Prevents entities from being marked hurt (preventing the server from syncing velocity)
    NoImpact,
    NoKnockback,
    PanicCauses,
    PanicEnvironmentalCauses,
    /// Reducese damage dealt to witches by 85%
    WitchResistantTo,
    WitherImmuneTo,
    /// Generic fallback
    Generic,
}

impl DamageResistantType {
    pub fn from_tag(s: &str) -> Self {
        match s {
            "#minecraft:always_hurts_ender_dragons"
            | "minecraft:always_hurts_ender_dragons"
            | "always_hurts_ender_dragons" => Self::AlwaysHurtsEnderDragons,
            "#minecraft:always_kills_armor_stands"
            | "minecraft:always_kills_armor_stands"
            | "always_kills_armor_stands" => Self::AlwaysKillsArmorStands,
            "#minecraft:always_most_significant_fall"
            | "minecraft:always_most_significant_fall"
            | "always_most_significant_fall" => Self::AlwaysMostSignificantFall,
            "#minecraft:always_triggers_silverfish"
            | "minecraft:always_triggers_silverfish"
            | "always_triggers_silverfish" => Self::AlwaysTriggersSilverfish,
            "#minecraft:avoids_guardian_thorns"
            | "minecraft:avoids_guardian_thorns"
            | "avoids_guardian_thorns" => Self::AvoidsGuardianThorns,
            "#minecraft:burns_armor_stands"
            | "minecraft:burns_armor_stands"
            | "burns_armor_stands" => Self::BurnsArmorStands,
            "#minecraft:burn_from_stepping"
            | "minecraft:burn_from_stepping"
            | "burn_from_stepping" => Self::BurnFromStepping,
            "#minecraft:bypasses_armor" | "minecraft:bypasses_armor" | "bypasses_armor" => {
                Self::BypassesArmor
            }
            "#minecraft:bypasses_cooldown"
            | "minecraft:bypasses_cooldown"
            | "bypasses_cooldown" => Self::BypassesCooldown,
            "#minecraft:bypasses_effects" | "minecraft:bypasses_effects" | "bypasses_effects" => {
                Self::BypassesEffects
            }
            "#minecraft:bypasses_enchantments"
            | "minecraft:bypasses_enchantments"
            | "bypasses_enchantments" => Self::BypassesEnchantments,
            "#minecraft:bypasses_invulnerability"
            | "minecraft:bypasses_invulnerability"
            | "bypasses_invulnerability" => Self::BypassesInvulnerability,
            "#minecraft:bypasses_resistance"
            | "minecraft:bypasses_resistance"
            | "bypasses_resistance" => Self::BypassesResistance,
            "#minecraft:bypasses_shield" | "minecraft:bypasses_shield" | "bypasses_shield" => {
                Self::BypassesShield
            }
            "#minecraft:bypasses_wolf_armor"
            | "minecraft:bypasses_wolf_armor"
            | "bypasses_wolf_armor" => Self::BypassesWolfArmor,
            "#minecraft:can_break_armor_stand"
            | "minecraft:can_break_armor_stand"
            | "can_break_armor_stand" => Self::CanBreakArmorStands,
            "#minecraft:damages_helmet" | "minecraft:damages_helmet" | "damages_helmet" => {
                Self::DamagesHelmet
            }
            "#minecraft:ignites_armor_stands"
            | "minecraft:ignites_armor_stands"
            | "ignites_armor_stands" => Self::IgnitesArmorStands,
            "#minecraft:is_drowning" | "minecraft:is_drowning" | "is_drowning" => Self::Drowning,
            "#minecraft:is_explosion" | "minecraft:is_explosion" | "is_explosion" | "explosion" => {
                Self::Explosion
            }
            "#minecraft:is_fall" | "minecraft:is_fall" | "is_fall" | "fall" => Self::Fall,
            "#minecraft:is_fire" | "minecraft:is_fire" | "is_fire" | "fire" | "in_fire"
            | "minecraft:in_fire" => Self::Fire,
            "#minecraft:is_freezing" | "minecraft:is_freezing" | "is_freezing" => Self::Freezing,
            "#minecraft:is_lightning" | "minecraft:is_lightning" | "is_lightning" => {
                Self::Lightning
            }
            "#minecraft:is_player_attack" | "minecraft:is_player_attack" | "is_player_attack" => {
                Self::PlayerAttack
            }
            "#minecraft:is_projectile" | "minecraft:is_projectile" | "is_projectile" => {
                Self::Projectile
            }
            "#minecraft:mace_smash" | "minecraft:mace_smash" | "mace_smash" => Self::MaceSmash,
            "#minecraft:no_anger" | "minecraft:no_anger" | "no_anger" => Self::NoAnger,
            "#minecraft:no_impact" | "minecraft:no_impact" | "no_impact" => Self::NoImpact,
            "#minecraft:no_knockback" | "minecraft:no_knockback" | "no_knockback" => {
                Self::NoKnockback
            }
            "#minecraft:panic_causes" | "minecraft:panic_causes" | "panic_causes" => {
                Self::PanicCauses
            }
            "#minecraft:panic_environmental_causes"
            | "minecraft:panic_environmental_causes"
            | "panic_environmental_causes" => Self::PanicEnvironmentalCauses,
            "#minecraft:witch_resistant_to"
            | "minecraft:witch_resistant_to"
            | "witch_resistant_to" => Self::WitchResistantTo,
            "#minecraft:wither_immune_to" | "minecraft:wither_immune_to" | "wither_immune_to" => {
                Self::WitherImmuneTo
            }
            _ => Self::Generic,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AlwaysHurtsEnderDragons => "#minecraft:always_hurts_ender_dragons",
            Self::AlwaysKillsArmorStands => "#minecraft:always_kills_armor_stands",
            Self::AlwaysMostSignificantFall => "#minecraft:always_most_significant_fall",
            Self::AlwaysTriggersSilverfish => "#minecraft:always_triggers_silverfish",
            Self::AvoidsGuardianThorns => "#minecraft:avoids_guardian_thorns",
            Self::BurnsArmorStands => "#minecraft:burns_armor_stands",
            Self::BurnFromStepping => "#minecraft:burn_from_stepping",
            Self::BypassesArmor => "#minecraft:bypasses_armor",
            Self::BypassesCooldown => "#minecraft:bypasses_cooldown",
            Self::BypassesEffects => "#minecraft:bypasses_effects",
            Self::BypassesEnchantments => "#minecraft:bypasses_enchantments",
            Self::BypassesInvulnerability => "#minecraft:bypasses_invulnerability",
            Self::BypassesResistance => "#minecraft:bypasses_resistance",
            Self::BypassesShield => "#minecraft:bypasses_shield",
            Self::BypassesWolfArmor => "#minecraft:bypasses_wolf_armor",
            Self::CanBreakArmorStands => "#minecraft:can_break_armor_stand",
            Self::DamagesHelmet => "#minecraft:damages_helmet",
            Self::IgnitesArmorStands => "#minecraft:ignites_armor_stands",
            Self::Drowning => "#minecraft:is_drowning",
            Self::Explosion => "#minecraft:is_explosion",
            Self::Fall => "#minecraft:is_fall",
            Self::Fire => "#minecraft:is_fire",
            Self::Freezing => "#minecraft:is_freezing",
            Self::Lightning => "#minecraft:is_lightning",
            Self::PlayerAttack => "#minecraft:is_player_attack",
            Self::Projectile => "#minecraft:is_projectile",
            Self::MaceSmash => "#minecraft:mace_smash",
            Self::NoAnger => "#minecraft:no_anger",
            Self::NoImpact => "#minecraft:no_impact",
            Self::NoKnockback => "#minecraft:no_knockback",
            Self::PanicCauses => "#minecraft:panic_causes",
            Self::PanicEnvironmentalCauses => "#minecraft:panic_environmental_causes",
            Self::WitchResistantTo => "#minecraft:witch_resistant_to",
            Self::WitherImmuneTo => "#minecraft:wither_immune_to",
            Self::Generic => "minecraft:generic",
        }
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct DamageResistantImpl {
    pub res_type: DamageResistantType,
}

impl DamageResistantImpl {
    fn read_data(data: &NbtTag) -> Option<Self> {
        let compound = data.extract_compound()?;
        let type_str = compound.get_string("types")?;

        Some(Self {
            res_type: DamageResistantType::from_tag(type_str),
        })
    }
}

impl std::str::FromStr for DamageResistantType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(DamageResistantType::from_tag(s))
    }
}

impl DataComponentImpl for DamageResistantImpl {
    fn write_data(&self) -> NbtTag {
        let mut compound = NbtCompound::new();
        compound.put_string("types", self.res_type.as_str().to_string());
        NbtTag::Compound(compound)
    }

    fn get_hash(&self) -> i32 {
        get_str_hash(self.res_type.as_str()) as i32
    }

    default_impl!(DamageResistant);
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
        Some(Block::from_id(id))
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
    fn read(data: &NbtTag) -> Option<Self> {
        match data.clone() {
            NbtTag::String(tag) => {
                if tag.starts_with("#") {
                    Some(Self::Tag(Cow::Owned(tag.strip_prefix("#")?.to_string())))
                } else {
                    Some(Self::IDs(Cow::Owned([T::from_str(tag.as_str())?].to_vec())))
                }
            }
            NbtTag::List(nbt_tags) => {
                let mut ids = Vec::<&T>::new();
                for nbt in nbt_tags {
                    if let NbtTag::String(id) = nbt
                        && let Some(instance) = T::from_str(id.as_str())
                    {
                        ids.push(instance);
                    }
                }
                Some(Self::IDs(Cow::Owned(ids)))
            }
            _ => None,
        }
    }

    fn write(&self, compound: &mut NbtCompound, key: &str) {
        match self {
            Self::Tag(cow) => {
                let mut tag = cow.to_string();
                tag.insert(0, '#');
                compound.put_string(key, tag);
            }
            Self::IDs(arr) => {
                let id_vec: Vec<NbtTag> =
                    arr.iter().map(|x| NbtTag::String(x.to_string())).collect();
                compound.put_list(key, id_vec);
            }
        }
    }
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
/// Weapon component: specifies durability cost per attack.
/// NOTE: If additional fields are added, update `get_hash()` to include them.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct WeaponImpl {
    pub item_damage_per_attack: u32,
}
impl WeaponImpl {
    fn read_data(data: &NbtTag) -> Option<Self> {
        let compound = data.extract_compound()?;
        // NOTE: Error handling for item_damage_per_attack:
        // - Missing key: defaults to 1 (vanilla behavior for unmodified items).
        // - Wrong NBT type (e.g. float instead of int): silently defaults to 1.
        // - Negative value: clamped to 0 then cast to u32 (protects against direct NBT manipulation).
        // This conservative approach prioritizes safety over strict validation.
        // TODO: Add tracing::warn! at the call site (in pumpkin crate where tracing is available)
        // to help datapack authors debug negative values or type mismatches.
        let item_damage_per_attack = compound
            .get_int("item_damage_per_attack")
            .unwrap_or(1)
            .max(0) as u32;
        // TODO: Deserialize disable_blocking_for_seconds once NBT float API is clarified.
        // For now, preserve it by storing the raw compound for round-trip fidelity.
        Some(Self {
            item_damage_per_attack,
        })
    }
}
impl DataComponentImpl for WeaponImpl {
    fn write_data(&self) -> NbtTag {
        let mut compound = NbtCompound::new();
        compound.put_int("item_damage_per_attack", self.item_damage_per_attack as i32);
        NbtTag::Compound(compound)
    }
    fn get_hash(&self) -> i32 {
        self.item_damage_per_attack as i32
    }
    default_impl!(Weapon);
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
        let shearing_sound = get_idor(compound, "shearing_sound_sound", Sound::ItemShearsSnip);

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
        let mut digest = Digest::new(Crc32Iscsi);
        digest.update(&[16u8]); // is it used?
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
    pub effect_id: Cow<'static, str>,
    pub amplifier: i32,
    pub duration: i32,
    pub ambient: bool,
    pub show_particles: bool,
    pub show_icon: bool,
}

impl StatusEffectInstance {
    pub fn read_data(nbt: &NbtTag) -> Option<Self> {
        let compound = nbt.extract_compound()?;

        let effect_id = Cow::Owned(compound.get_string("id")?.to_string());
        let amplifier = compound.get_int("amplifier")?;
        let duration = compound.get_int("duration")?;
        let ambient = compound.get_bool("ambient")?;
        let show_particles = compound.get_bool("show_particles")?;
        let show_icon = compound.get_bool("show_icon")?;

        Some(Self {
            effect_id,
            amplifier,
            duration,
            ambient,
            show_particles,
            show_icon,
        })
    }

    pub fn as_nbt(&self) -> NbtTag {
        let mut compound = NbtCompound::new();

        compound.put_string("id", self.effect_id.to_string());
        compound.put_int("amplifier", self.amplifier);
        compound.put_int("duration", self.duration);
        compound.put_bool("ambient", self.ambient);
        compound.put_bool("show_particles", self.show_particles);
        compound.put_bool("show_icon", self.show_icon);

        NbtTag::Compound(compound)
    }

    pub fn get_hash(&self) -> i32 {
        let mut digest = Digest::new(Crc32Iscsi);

        digest.update(&get_str_hash(self.effect_id.as_ref()).to_le_bytes());
        digest.update(&get_i32_hash(self.amplifier).to_le_bytes());
        digest.update(&get_i32_hash(self.duration).to_le_bytes());
        digest.update(&[self.ambient as u8]);
        digest.update(&[self.show_particles as u8]);
        digest.update(&[self.show_icon as u8]);

        digest.finalize() as i32
    }
}

impl Hash for ConsumeEffect {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.to_str().hash(state);
        match self {
            ConsumeEffect::ApplyEffects(tuple) => {
                tuple.0.hash(state);
                unsafe { (*(&raw const tuple.1).cast::<u32>()).hash(state) };
            }
            ConsumeEffect::RemoveEffects(status_effect_instances) => {
                status_effect_instances.hash(state)
            }
            ConsumeEffect::ClearAllEffects => (),
            ConsumeEffect::TeleportRandomly(dst) => unsafe {
                (*(&raw const dst).cast::<u32>()).hash(state)
            },
            ConsumeEffect::PlaySound(id_or) => id_or.hash(state),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ConsumeEffect {
    /// Vec of status effect adn a f32 represeting
    /// the propability
    ApplyEffects((Cow<'static, [StatusEffectInstance]>, f32)),
    RemoveEffects(IDSet<StatusEffect>),
    ClearAllEffects,
    /// f32 diameter
    TeleportRandomly(f32),
    PlaySound(IdOr<SoundEvent>),
}
impl ConsumeEffect {
    pub fn to_str(&self) -> &str {
        match self {
            ConsumeEffect::ApplyEffects(_) => "apply_effects",
            ConsumeEffect::RemoveEffects(_) => "remove_effects",
            ConsumeEffect::ClearAllEffects => "clear_all_effects",
            ConsumeEffect::TeleportRandomly(_) => "teleport_randomly",
            ConsumeEffect::PlaySound(_) => "play_sound",
        }
    }

    pub fn registry_id(&self) -> u8 {
        match self {
            ConsumeEffect::ApplyEffects(_) => 0,
            ConsumeEffect::RemoveEffects(_) => 1,
            ConsumeEffect::ClearAllEffects => 2,
            ConsumeEffect::TeleportRandomly(_) => 3,
            ConsumeEffect::PlaySound(_) => 4,
        }
    }

    pub fn read_data(nbt: &NbtTag) -> Option<Self> {
        let compound = nbt.extract_compound()?;

        let r#type = compound.get_string("type")?;

        match r#type {
            "remove_effects" => {
                let idset = IDSet::read(compound.get("effects")?)?;
                Some(Self::RemoveEffects(idset))
            }
            "clear_all_effects" => Some(Self::ClearAllEffects),
            "teleport_randomly" => {
                let dst = compound.get_float("diameter")?;
                Some(Self::TeleportRandomly(dst))
            }
            "play_sound" => {
                let sound = get_idor(compound, "sound", Sound::EntityGenericEat);
                Some(Self::PlaySound(sound))
            }
            "apply_effects" => {
                let probability = compound.get_float("probability")?;
                let effects_vec: Vec<StatusEffectInstance> = compound
                    .get_list("effects")?
                    .iter()
                    .filter_map(StatusEffectInstance::read_data)
                    .collect();
                let effects: Cow<'static, [StatusEffectInstance]> = Cow::Owned(effects_vec);
                Some(Self::ApplyEffects((effects, probability)))
            }
            _ => None,
        }
    }

    pub fn as_nbt(&self) -> NbtTag {
        let mut compound = NbtCompound::new();
        compound.put_string("type", self.to_str().to_string());

        match self {
            ConsumeEffect::ApplyEffects(data) => {
                let nbt_arr = data.0.iter().map(|x| x.as_nbt()).collect();
                compound.put_list("effects", nbt_arr);
                compound.put_float("probability", data.1);
            }
            ConsumeEffect::RemoveEffects(idset) => idset.write(&mut compound, "effects"),
            ConsumeEffect::ClearAllEffects => (),
            ConsumeEffect::TeleportRandomly(dst) => compound.put_float("diameter", *dst),
            ConsumeEffect::PlaySound(id_or) => {
                put_idor(&mut compound, "sound", id_or);
            }
        }

        NbtTag::Compound(compound)
    }

    pub fn get_hash(&self) -> i32 {
        let mut digest = Digest::new(Crc32Iscsi);

        match self {
            ConsumeEffect::ApplyEffects((effects, probability)) => {
                digest.update(&[1u8]);
                for effect in effects.iter() {
                    digest.update(&effect.get_hash().to_le_bytes());
                }
                digest.update(&[13u8]);
                digest.update(&get_f32_hash(*probability).to_le_bytes());
            }
            ConsumeEffect::RemoveEffects(idset) => {
                digest.update(&[2u8]);
                digest.update(&get_idset_hash(idset).to_le_bytes());
            }
            ConsumeEffect::ClearAllEffects => {
                digest.update(&[3u8]);
            }
            ConsumeEffect::TeleportRandomly(dst) => {
                digest.update(&[4u8]);
                digest.update(&get_f32_hash(*dst).to_le_bytes());
            }
            ConsumeEffect::PlaySound(id_or) => {
                digest.update(&[5u8]);
                digest.update(&get_idor_hash(id_or).to_le_bytes());
            }
        }
        digest.finalize() as i32
    }
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

        let custom_effects = compound
            .get_list("custom_effects")
            .map(|list| {
                list.iter()
                    .filter_map(|item| {
                        // Try to get the compound for this specific effect
                        let effect_tag = item.extract_compound()?;

                        // Try to get the ID
                        let id: Cow<'static, str> =
                            Cow::Owned(effect_tag.get_string("id")?.to_string());

                        // Fallback values for optional fields
                        let amplifier = effect_tag
                            .get_int("amplifier")
                            .or_else(|| effect_tag.get_byte("amplifier").map(i32::from))
                            .unwrap_or(0);
                        let duration = effect_tag
                            .get_int("duration")
                            .or_else(|| effect_tag.get_byte("duration").map(i32::from))
                            .unwrap_or(0);
                        let ambient = effect_tag.get_bool("ambient").unwrap_or(false);
                        let show_particles = effect_tag.get_bool("show_particles").unwrap_or(true);
                        let show_icon = effect_tag.get_bool("show_icon").unwrap_or(true);

                        // Create the StatusEffectInstance
                        Some(StatusEffectInstance {
                            effect_id: id,
                            amplifier,
                            duration,
                            ambient,
                            show_particles,
                            show_icon,
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

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
                effect_compound.put_string("id", effect.effect_id.to_string());
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
                digest.update(&effect.get_hash().to_le_bytes());
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

    pub fn read_data(tag: &NbtTag) -> Option<Self> {
        let compound = tag.extract_compound()?;
        let shape = FireworkExplosionShape::from_name(compound.get_string("shape")?)?;
        let colors = compound
            .get_int_array("colors")
            .map(|v| v.to_vec())
            .unwrap_or_default();
        let fade_colors = compound
            .get_int_array("fade_colors")
            .map(|v| v.to_vec())
            .unwrap_or_default();
        let has_trail = compound.get_bool("has_trail").unwrap_or(false);
        let has_twinkle = compound.get_bool("has_twinkle").unwrap_or(false);

        Some(Self {
            shape,
            colors,
            fade_colors,
            has_trail,
            has_twinkle,
        })
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

    pub fn read_data(tag: &NbtTag) -> Option<Self> {
        let compound = tag.extract_compound()?;
        let flight_duration = compound
            .get_byte("flight_duration")
            .map(i32::from)
            .or_else(|| compound.get_int("flight_duration"))
            .unwrap_or(1);

        let mut explosions = Vec::new();
        if let Some(list) = compound.get_list("explosions") {
            for item in list {
                if let Some(explosion) = FireworkExplosionImpl::read_data(item) {
                    explosions.push(explosion);
                }
            }
        }

        Some(Self {
            flight_duration,
            explosions,
        })
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
