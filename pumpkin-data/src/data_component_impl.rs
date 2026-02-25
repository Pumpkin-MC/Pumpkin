#![allow(dead_code)]

use crate::attributes::Attributes;
use crate::data_component::DataComponent;
use crate::data_component::DataComponent::{
    AttributeModifiers, BlocksAttacks, Consumable, CustomData, CustomName, Damage, DamageResistant,
    DeathProtection, Enchantments, Equippable, FireworkExplosion, Fireworks, Food, ItemModel,
    ItemName, JukeboxPlayable, MaxDamage, MaxStackSize, PotionContents, Tool, Unbreakable,
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
        ItemModel => Some(ItemModelImpl::read_data(data)?.to_dyn()),
        CustomName => Some(CustomNameImpl::read_data(data)?.to_dyn()),
        Enchantments => Some(EnchantmentsImpl::read_data(data)?.to_dyn()),
        Damage => Some(DamageImpl::read_data(data)?.to_dyn()),
        Unbreakable => Some(UnbreakableImpl::read_data(data)?.to_dyn()),
        DamageResistant => Some(DamageResistantImpl::read_data(data)?.to_dyn()),
        PotionContents => Some(PotionContentsImpl::read_data(data)?.to_dyn()),
        Fireworks => Some(FireworksImpl::read_data(data)?.to_dyn()),
        FireworkExplosion => Some(FireworkExplosionImpl::read_data(data)?.to_dyn()),
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
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct CustomNameImpl {
    pub name: String,
}
impl CustomNameImpl {
    fn read_data(data: &NbtTag) -> Option<Self> {
        data.extract_string()
            .map(|s| Self {
                name: s.to_string(),
            })
    }
}
impl DataComponentImpl for CustomNameImpl {
    fn write_data(&self) -> NbtTag {
        NbtTag::String(self.name.clone())
    }
    fn get_hash(&self) -> i32 {
        get_str_hash(&self.name) as i32
    }
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
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ItemModelImpl {
    pub model: String,
}
impl ItemModelImpl {
    fn read_data(data: &NbtTag) -> Option<Self> {
        data.extract_string().map(|s| Self { model: s.to_string() })
    }
}
impl DataComponentImpl for ItemModelImpl {
    fn write_data(&self) -> NbtTag {
        NbtTag::String(self.model.clone())
    }
    fn get_hash(&self) -> i32 {
        get_str_hash(&self.model) as i32
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_model_read_data_from_string_tag() {
        let tag = NbtTag::String("minecraft:diamond_sword".to_string());
        let result = ItemModelImpl::read_data(&tag).unwrap();
        assert_eq!(result.model, "minecraft:diamond_sword");
    }

    #[test]
    fn item_model_read_data_from_non_string_returns_none() {
        let tag = NbtTag::Int(42);
        assert!(ItemModelImpl::read_data(&tag).is_none());
    }

    #[test]
    fn item_model_write_data_roundtrip() {
        let original = ItemModelImpl {
            model: "minecraft:stick".to_string(),
        };
        let tag = original.write_data();
        let restored = ItemModelImpl::read_data(&tag).unwrap();
        assert_eq!(original, restored);
    }

    #[test]
    fn item_model_get_hash_matches_str_hash() {
        let model = ItemModelImpl {
            model: "minecraft:diamond_sword".to_string(),
        };
        let expected = get_str_hash("minecraft:diamond_sword") as i32;
        assert_eq!(model.get_hash(), expected);
    }

    #[test]
    fn custom_name_read_data_from_string_tag() {
        let tag = NbtTag::String("My Custom Sword".to_string());
        let result = CustomNameImpl::read_data(&tag).unwrap();
        assert_eq!(result.name, "My Custom Sword");
    }

    #[test]
    fn custom_name_read_data_from_non_string_returns_none() {
        let tag = NbtTag::Int(42);
        assert!(CustomNameImpl::read_data(&tag).is_none());
    }

    #[test]
    fn custom_name_write_data_roundtrip() {
        let original = CustomNameImpl {
            name: "Excalibur".to_string(),
        };
        let tag = original.write_data();
        let restored = CustomNameImpl::read_data(&tag).unwrap();
        assert_eq!(original, restored);
    }

    #[test]
    fn custom_name_get_hash_matches_str_hash() {
        let name = CustomNameImpl {
            name: "Excalibur".to_string(),
        };
        let expected = get_str_hash("Excalibur") as i32;
        assert_eq!(name.get_hash(), expected);
    }

    #[test]
    fn read_data_dispatches_item_model() {
        let tag = NbtTag::String("minecraft:bow".to_string());
        let result = read_data(DataComponent::ItemModel, &tag);
        assert!(result.is_some());
        let component = result.unwrap();
        assert_eq!(component.get_self_enum(), DataComponent::ItemModel);
        assert_eq!(
            component.write_data(),
            NbtTag::String("minecraft:bow".to_string())
        );
    }

    #[test]
    fn read_data_dispatches_custom_name() {
        let tag = NbtTag::String("Named Item".to_string());
        let result = read_data(DataComponent::CustomName, &tag);
        assert!(result.is_some());
        let component = result.unwrap();
        assert_eq!(component.get_self_enum(), DataComponent::CustomName);
        assert_eq!(
            component.write_data(),
            NbtTag::String("Named Item".to_string())
        );
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

        let custom_effects = compound
            .get_list("custom_effects")
            .map(|list| {
                list.iter()
                    .filter_map(|item| {
                        // Try to get the compound for this specific effect
                        let effect_tag = item.extract_compound()?;

                        // Try to get the ID
                        let id = effect_tag.get_int("id")?;

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
