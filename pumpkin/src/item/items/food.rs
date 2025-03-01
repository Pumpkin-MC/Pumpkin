// pumpkin/src/item/items/food.rs

use crate::item::items::pumpkin_food::PumpkinFood;
use async_trait::async_trait;
use pumpkin_macros::pumpkin_item;

// Basic food items
#[pumpkin_item("apple")]
pub struct AppleItem;

#[async_trait]
impl PumpkinFood for AppleItem {}

#[pumpkin_item("bread")]
pub struct BreadItem;

#[async_trait]
impl PumpkinFood for BreadItem {}

#[pumpkin_item("cooked_beef")]
pub struct CookedBeefItem;

#[async_trait]
impl PumpkinFood for CookedBeefItem {}

#[pumpkin_item("cooked_chicken")]
pub struct CookedChickenItem;

#[async_trait]
impl PumpkinFood for CookedChickenItem {}

#[pumpkin_item("cooked_cod")]
pub struct CookedCodItem;

#[async_trait]
impl PumpkinFood for CookedCodItem {}

#[pumpkin_item("cooked_mutton")]
pub struct CookedMuttonItem;

#[async_trait]
impl PumpkinFood for CookedMuttonItem {}

#[pumpkin_item("cooked_porkchop")]
pub struct CookedPorkchopItem;

#[async_trait]
impl PumpkinFood for CookedPorkchopItem {}

#[pumpkin_item("cooked_rabbit")]
pub struct CookedRabbitItem;

#[async_trait]
impl PumpkinFood for CookedRabbitItem {}

#[pumpkin_item("cooked_salmon")]
pub struct CookedSalmonItem;

#[async_trait]
impl PumpkinFood for CookedSalmonItem {}

// Raw foods
#[pumpkin_item("beef")]
pub struct BeefItem;

#[async_trait]
impl PumpkinFood for BeefItem {}

#[pumpkin_item("chicken")]
pub struct ChickenItem;

#[async_trait]
impl PumpkinFood for ChickenItem {}

#[pumpkin_item("cod")]
pub struct CodItem;

#[async_trait]
impl PumpkinFood for CodItem {}

#[pumpkin_item("mutton")]
pub struct MuttonItem;

#[async_trait]
impl PumpkinFood for MuttonItem {}

#[pumpkin_item("porkchop")]
pub struct PorkchopItem;

#[async_trait]
impl PumpkinFood for PorkchopItem {}

#[pumpkin_item("rabbit")]
pub struct RabbitItem;

#[async_trait]
impl PumpkinFood for RabbitItem {}

#[pumpkin_item("salmon")]
pub struct SalmonItem;

#[async_trait]
impl PumpkinFood for SalmonItem {}

// Special Foods
#[pumpkin_item("golden_apple")]
pub struct GoldenAppleItem;

#[async_trait]
impl PumpkinFood for GoldenAppleItem {}

#[pumpkin_item("enchanted_golden_apple")]
pub struct EnchantedGoldenAppleItem;

#[async_trait]
impl PumpkinFood for EnchantedGoldenAppleItem {}

#[pumpkin_item("golden_carrot")]
pub struct GoldenCarrotItem;

#[async_trait]
impl PumpkinFood for GoldenCarrotItem {}

#[pumpkin_item("rotten_flesh")]
pub struct RottenFleshItem;

#[async_trait]
impl PumpkinFood for RottenFleshItem {}

#[pumpkin_item("spider_eye")]
pub struct SpiderEyeItem;

#[async_trait]
impl PumpkinFood for SpiderEyeItem {}

#[pumpkin_item("chorus_fruit")]
pub struct ChorusFruitItem;

#[async_trait]
impl PumpkinFood for ChorusFruitItem {
    // Chorus fruit has its teleport handled through the effects system
}

#[pumpkin_item("suspicious_stew")]
pub struct SuspiciousStewItem;

#[async_trait]
impl PumpkinFood for SuspiciousStewItem {}

#[pumpkin_item("dried_kelp")]
pub struct DriedKelpItem;

#[async_trait]
impl PumpkinFood for DriedKelpItem {}

#[pumpkin_item("sweet_berries")]
pub struct SweetBerriesItem;

#[async_trait]
impl PumpkinFood for SweetBerriesItem {}

#[pumpkin_item("honey_bottle")]
pub struct HoneyBottleItem;

#[async_trait]
impl PumpkinFood for HoneyBottleItem {}

// Additional foods
#[pumpkin_item("cookie")]
pub struct CookieItem;

#[async_trait]
impl PumpkinFood for CookieItem {}

#[pumpkin_item("melon_slice")]
pub struct MelonSliceItem;

#[async_trait]
impl PumpkinFood for MelonSliceItem {}

#[pumpkin_item("beetroot")]
pub struct BeetrootItem;

#[async_trait]
impl PumpkinFood for BeetrootItem {}

#[pumpkin_item("beetroot_soup")]
pub struct BeetrootSoupItem;

#[async_trait]
impl PumpkinFood for BeetrootSoupItem {}

#[pumpkin_item("mushroom_stew")]
pub struct MushroomStewItem;

#[async_trait]
impl PumpkinFood for MushroomStewItem {}

#[pumpkin_item("rabbit_stew")]
pub struct RabbitStewItem;

#[async_trait]
impl PumpkinFood for RabbitStewItem {}

#[pumpkin_item("carrot")]
pub struct CarrotItem;

#[async_trait]
impl PumpkinFood for CarrotItem {}

#[pumpkin_item("potato")]
pub struct PotatoItem;

#[async_trait]
impl PumpkinFood for PotatoItem {}

#[pumpkin_item("baked_potato")]
pub struct BakedPotatoItem;

#[async_trait]
impl PumpkinFood for BakedPotatoItem {}

#[pumpkin_item("poisonous_potato")]
pub struct PoisonousPotatoItem;

#[async_trait]
impl PumpkinFood for PoisonousPotatoItem {}

#[pumpkin_item("pumpkin_pie")]
pub struct PumpkinPieItem;

#[async_trait]
impl PumpkinFood for PumpkinPieItem {}

#[pumpkin_item("tropical_fish")]
pub struct TropicalFishItem;

#[async_trait]
impl PumpkinFood for TropicalFishItem {}

#[pumpkin_item("pufferfish")]
pub struct PufferfishItem;

#[async_trait]
impl PumpkinFood for PufferfishItem {}

#[pumpkin_item("glow_berries")]
pub struct GlowBerriesItem;

#[async_trait]
impl PumpkinFood for GlowBerriesItem {}
