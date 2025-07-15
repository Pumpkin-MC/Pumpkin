#![allow(dead_code)]

use crate::attributes::Attributes;
use crate::tag::Tag;
use crate::{AttributeModifierSlot, Block};
use pumpkin_util::text::TextComponent;
use std::borrow::Cow;
use std::hash::Hash;

// please don't make the size of the struct too large. use Box<> when necessary
#[derive(Clone, Debug, Hash)]
pub struct CustomDataImpl;
#[derive(Clone, Debug, Hash)]
pub struct MaxStackSizeImpl {
    pub size: u8,
}
#[derive(Clone, Debug, Hash)]
pub struct MaxDamageImpl {
    pub max_damage: i32,
}
#[derive(Clone, Debug, Hash)]
pub struct DamageImpl {
    pub damage: i32,
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

#[derive(Clone, Debug, Hash)]
pub struct ItemModelImpl;
#[derive(Clone, Debug, Hash)]
pub struct LoreImpl;
#[derive(Clone, Debug, Hash)]
pub struct RarityImpl;
#[derive(Clone, Debug, Hash)]
pub struct EnchantmentsImpl;
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
#[derive(Clone, Debug, Hash)]
pub struct EnchantableImpl;
#[derive(Clone, Debug, Hash)]
pub struct EquippableImpl;
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
