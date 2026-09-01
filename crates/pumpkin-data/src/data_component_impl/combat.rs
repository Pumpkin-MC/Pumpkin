use crate::Block;
use crate::Enchantment;
use crate::attributes::Attributes;
use crate::data_component_impl::basic::SoundEvent;
use crate::data_component_impl::{
    DataComponentImpl, EquipmentSlot, IDSet, IdOr, get_i32_hash, get_idor, get_idor_hash,
    get_idset_hash, get_str_hash, put_idor,
};
use crate::entity_type::EntityType;
use crate::item::Item;
use crate::item_stack::ItemStack;
use crate::sound::Sound;
use crate::tag::Taggable;
use crc_fast::CrcAlgorithm::Crc32Iscsi;
use crc_fast::Digest;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use std::borrow::Cow;
use std::hash::Hash;

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
    pub slot: crate::AttributeModifierSlot,
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

#[derive(Clone, Hash, PartialEq, Eq, Default)]
pub struct EnchantmentsImpl {
    pub enchantment: Cow<'static, [(&'static Enchantment, i32)]>,
}
impl EnchantmentsImpl {
    pub fn read_data(data: &NbtTag) -> Option<Self> {
        let compound = data.extract_compound()?;
        let data = if let Some(NbtTag::Compound(levels)) = compound.child_tags.get("levels") {
            &levels.child_tags
        } else {
            &compound.child_tags
        };
        let mut enc = Vec::with_capacity(data.len());
        for (name, level) in data {
            let enchantment = Enchantment::from_name(name.as_ref())
                .or_else(|| Enchantment::from_name(&format!("minecraft:{name}")))?;
            enc.push((enchantment, level.extract_int()?));
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

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum DamageResistantType {
    AlwaysHurtsEnderDragons,
    AlwaysKillsArmorStands,
    AlwaysMostSignificantFall,
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
    Explosion,
    Fall,
    Fire,
    Freezing,
    Lightning,
    PlayerAttack,
    Projectile,
    MaceSmash,
    NoAnger,
    NoImpact,
    NoKnockback,
    PanicCauses,
    PanicEnvironmentalCauses,
    WitchResistantTo,
    WitherImmuneTo,
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
    pub fn read_data(data: &NbtTag) -> Option<Self> {
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

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct WeaponImpl {
    pub item_damage_per_attack: u32,
}
impl WeaponImpl {
    pub fn read_data(data: &NbtTag) -> Option<Self> {
        let compound = data.extract_compound()?;
        let item_damage_per_attack = compound
            .get_int("item_damage_per_attack")
            .unwrap_or(1)
            .max(0) as u32;
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

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct AttackRangeImpl;
impl DataComponentImpl for AttackRangeImpl {
    default_impl!(AttackRange);
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
pub struct PiercingWeaponImpl;
impl DataComponentImpl for PiercingWeaponImpl {
    default_impl!(PiercingWeapon);
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct KineticWeaponImpl;
impl DataComponentImpl for KineticWeaponImpl {
    default_impl!(KineticWeapon);
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

    default_impl!(SwingAnimation);
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct AdditionalTradeCostImpl;
impl DataComponentImpl for AdditionalTradeCostImpl {
    default_impl!(AdditionalTradeCost);
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct StoredEnchantmentsImpl {
    pub enchantment: Cow<'static, [(&'static Enchantment, i32)]>,
}
impl StoredEnchantmentsImpl {
    pub fn read_data(data: &NbtTag) -> Option<Self> {
        let compound = data.extract_compound()?;
        let data = if let Some(NbtTag::Compound(levels)) = compound.child_tags.get("levels") {
            &levels.child_tags
        } else {
            &compound.child_tags
        };
        let mut enc = Vec::with_capacity(data.len());
        for (name, level) in data {
            let enchantment = Enchantment::from_name(name.as_ref())
                .or_else(|| Enchantment::from_name(&format!("minecraft:{name}")))?;
            enc.push((enchantment, level.extract_int()?));
        }
        Some(Self {
            enchantment: Cow::from(enc),
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
        let mut digest = Digest::new(Crc32Iscsi);
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

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct MinimumAttackChargeImpl;
impl DataComponentImpl for MinimumAttackChargeImpl {
    default_impl!(MinimumAttackCharge);
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct DamageTypeImpl;
impl DataComponentImpl for DamageTypeImpl {
    default_impl!(DamageType);
}
