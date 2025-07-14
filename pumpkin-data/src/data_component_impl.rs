#![allow(dead_code)]

use crate::attributes::Attributes;
use crate::tag::Tag;
use crate::{AttributeModifierSlot, Block};
use pumpkin_util::text::TextComponent;

// please don't make the size of the struct too large. use Box<> when necessary
#[derive(Clone, Debug)]
pub struct CustomDataImpl;
#[derive(Clone, Debug)]
pub struct MaxStackSizeImpl {
    pub size: u8,
}
#[derive(Clone, Debug)]
pub struct MaxDamageImpl {
    pub max_damage: i32,
}
#[derive(Clone, Debug)]
pub struct DamageImpl {
    pub damage: i32,
}
#[derive(Clone, Debug)]
pub struct UnbreakableImpl;
#[derive(Clone, Debug)]
pub struct CustomNameImpl;
#[derive(Clone, Debug)]
pub struct ItemNameImpl<'a> {
    // pub name: &'a TextComponent, TODO make TextComponent in compile time
    pub name: &'a str,
}

#[derive(Clone, Debug)]
pub struct ItemModelImpl;
#[derive(Clone, Debug)]
pub struct LoreImpl;
#[derive(Clone, Debug)]
pub struct RarityImpl;
#[derive(Clone, Debug)]
pub struct EnchantmentsImpl;
#[derive(Clone, Debug)]
pub struct CanPlaceOnImpl;
#[derive(Clone, Debug)]
pub struct CanBreakImpl;

#[derive(Clone, Copy, Debug, PartialEq)]
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
#[derive(Clone, Debug)]
pub struct AttributeModifiersImpl<'a> {
    pub attribute_modifiers: &'a [Modifier],
}
#[derive(Clone, Debug)]
pub struct CustomModelDataImpl;
#[derive(Clone, Debug)]
pub struct TooltipDisplayImpl;
#[derive(Clone, Debug)]
pub struct RepairCostImpl;
#[derive(Clone, Debug)]
pub struct CreativeSlotLockImpl;
#[derive(Clone, Debug)]
pub struct EnchantmentGlintOverrideImpl;
#[derive(Clone, Debug)]
pub struct IntangibleProjectileImpl;
#[derive(Clone, Debug)]
pub struct FoodImpl {
    pub nutrition: i32,
    pub saturation: f32,
    pub can_always_eat: bool,
}
#[derive(Clone, Debug)]
pub struct ConsumableImpl;
#[derive(Clone, Debug)]
pub struct UseRemainderImpl;
#[derive(Clone, Debug)]
pub struct UseCooldownImpl;
#[derive(Clone, Debug)]
pub struct DamageResistantImpl;

#[derive(Clone, Debug)]
pub enum IDSet<'a> {
    Tag(&'static Tag),
    Blocks(&'a [&'static Block]),
}

#[derive(Clone, Debug)]
pub struct ToolRule<'a> {
    pub blocks: IDSet<'a>,
    pub speed: Option<f32>,
    pub correct_for_drops: Option<bool>,
}
#[derive(Clone, Debug)]
pub struct ToolImpl<'a> {
    pub rules: &'a [ToolRule<'a>],
    pub default_mining_speed: f32,
    pub damage_per_block: u32,
    pub can_destroy_blocks_in_creative: bool,
}
#[derive(Clone, Debug)]
pub struct WeaponImpl;
#[derive(Clone, Debug)]
pub struct EnchantableImpl;
#[derive(Clone, Debug)]
pub struct EquippableImpl;
#[derive(Clone, Debug)]
pub struct RepairableImpl;
#[derive(Clone, Debug)]
pub struct GliderImpl;
#[derive(Clone, Debug)]
pub struct TooltipStyleImpl;
#[derive(Clone, Debug)]
pub struct DeathProtectionImpl;
#[derive(Clone, Debug)]
pub struct BlocksAttacksImpl;
#[derive(Clone, Debug)]
pub struct StoredEnchantmentsImpl;
#[derive(Clone, Debug)]
pub struct DyedColorImpl;
#[derive(Clone, Debug)]
pub struct MapColorImpl;
#[derive(Clone, Debug)]
pub struct MapIdImpl;
#[derive(Clone, Debug)]
pub struct MapDecorationsImpl;
#[derive(Clone, Debug)]
pub struct MapPostProcessingImpl;
#[derive(Clone, Debug)]
pub struct ChargedProjectilesImpl;
#[derive(Clone, Debug)]
pub struct BundleContentsImpl;
#[derive(Clone, Debug)]
pub struct PotionContentsImpl;
#[derive(Clone, Debug)]
pub struct PotionDurationScaleImpl;
#[derive(Clone, Debug)]
pub struct SuspiciousStewEffectsImpl;
#[derive(Clone, Debug)]
pub struct WritableBookContentImpl;
#[derive(Clone, Debug)]
pub struct WrittenBookContentImpl;
#[derive(Clone, Debug)]
pub struct TrimImpl;
#[derive(Clone, Debug)]
pub struct DebugStickStateImpl;
#[derive(Clone, Debug)]
pub struct EntityDataImpl;
#[derive(Clone, Debug)]
pub struct BucketEntityDataImpl;
#[derive(Clone, Debug)]
pub struct BlockEntityDataImpl;
#[derive(Clone, Debug)]
pub struct InstrumentImpl;
#[derive(Clone, Debug)]
pub struct ProvidesTrimMaterialImpl;
#[derive(Clone, Debug)]
pub struct OminousBottleAmplifierImpl;
#[derive(Clone, Debug)]
pub struct JukeboxPlayableImpl {
    pub song: &'static str,
}
#[derive(Clone, Debug)]
pub struct ProvidesBannerPatternsImpl;
#[derive(Clone, Debug)]
pub struct RecipesImpl;
#[derive(Clone, Debug)]
pub struct LodestoneTrackerImpl;
#[derive(Clone, Debug)]
pub struct FireworkExplosionImpl;
#[derive(Clone, Debug)]
pub struct FireworksImpl;
#[derive(Clone, Debug)]
pub struct ProfileImpl;
#[derive(Clone, Debug)]
pub struct NoteBlockSoundImpl;
#[derive(Clone, Debug)]
pub struct BannerPatternsImpl;
#[derive(Clone, Debug)]
pub struct BaseColorImpl;
#[derive(Clone, Debug)]
pub struct PotDecorationsImpl;
#[derive(Clone, Debug)]
pub struct ContainerImpl;
#[derive(Clone, Debug)]
pub struct BlockStateImpl;
#[derive(Clone, Debug)]
pub struct BeesImpl;
#[derive(Clone, Debug)]
pub struct LockImpl;
#[derive(Clone, Debug)]
pub struct ContainerLootImpl;
#[derive(Clone, Debug)]
pub struct BreakSoundImpl;
#[derive(Clone, Debug)]
pub struct VillagerVariantImpl;
#[derive(Clone, Debug)]
pub struct WolfVariantImpl;
#[derive(Clone, Debug)]
pub struct WolfSoundVariantImpl;
#[derive(Clone, Debug)]
pub struct WolfCollarImpl;
#[derive(Clone, Debug)]
pub struct FoxVariantImpl;
#[derive(Clone, Debug)]
pub struct SalmonSizeImpl;
#[derive(Clone, Debug)]
pub struct ParrotVariantImpl;
#[derive(Clone, Debug)]
pub struct TropicalFishPatternImpl;
#[derive(Clone, Debug)]
pub struct TropicalFishBaseColorImpl;
#[derive(Clone, Debug)]
pub struct TropicalFishPatternColorImpl;
#[derive(Clone, Debug)]
pub struct MooshroomVariantImpl;
#[derive(Clone, Debug)]
pub struct RabbitVariantImpl;
#[derive(Clone, Debug)]
pub struct PigVariantImpl;
#[derive(Clone, Debug)]
pub struct CowVariantImpl;
#[derive(Clone, Debug)]
pub struct ChickenVariantImpl;
#[derive(Clone, Debug)]
pub struct FrogVariantImpl;
#[derive(Clone, Debug)]
pub struct HorseVariantImpl;
#[derive(Clone, Debug)]
pub struct PaintingVariantImpl;
#[derive(Clone, Debug)]
pub struct LlamaVariantImpl;
#[derive(Clone, Debug)]
pub struct AxolotlVariantImpl;
#[derive(Clone, Debug)]
pub struct CatVariantImpl;
#[derive(Clone, Debug)]
pub struct CatCollarImpl;
#[derive(Clone, Debug)]
pub struct SheepColorImpl;
#[derive(Clone, Debug)]
pub struct ShulkerColorImpl;
