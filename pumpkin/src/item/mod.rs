use items::{egg::EggItem, snowball::SnowBallItem};
use registry::ItemRegistry;

use crate::item::items::food::{
    AppleItem, BakedPotatoItem, BeefItem, BeetrootItem, BeetrootSoupItem, BreadItem, CarrotItem,
    ChickenItem, ChorusFruitItem, CodItem, CookedBeefItem, CookedChickenItem, CookedCodItem,
    CookedMuttonItem, CookedPorkchopItem, CookedRabbitItem, CookedSalmonItem, CookieItem,
    DriedKelpItem, EnchantedGoldenAppleItem, GlowBerriesItem, GoldenAppleItem, GoldenCarrotItem,
    HoneyBottleItem, MelonSliceItem, MushroomStewItem, MuttonItem, PoisonousPotatoItem,
    PorkchopItem, PotatoItem, PufferfishItem, PumpkinPieItem, RabbitItem, RabbitStewItem,
    RottenFleshItem, SalmonItem, SpiderEyeItem, SuspiciousStewItem, SweetBerriesItem,
    TropicalFishItem,
};
use std::sync::Arc;

pub mod items;
pub mod pumpkin_item;
pub mod registry;

#[must_use]
pub fn default_registry() -> Arc<ItemRegistry> {
    let mut manager = ItemRegistry::default();

    manager.register(SnowBallItem);
    manager.register(EggItem);

    // Register food items
    manager.register(AppleItem);
    manager.register(BreadItem);
    manager.register(CookedBeefItem);
    manager.register(CookedChickenItem);
    manager.register(CookedChickenItem);
    manager.register(CookedCodItem);
    manager.register(CookedMuttonItem);
    manager.register(CookedPorkchopItem);
    manager.register(CookedRabbitItem);
    manager.register(CookedSalmonItem);
    manager.register(BeefItem);
    manager.register(ChickenItem);
    manager.register(CodItem);
    manager.register(MuttonItem);
    manager.register(PorkchopItem);
    manager.register(RabbitItem);
    manager.register(SalmonItem);
    manager.register(GoldenAppleItem);
    manager.register(EnchantedGoldenAppleItem);
    manager.register(GoldenCarrotItem);
    manager.register(RottenFleshItem);
    manager.register(SpiderEyeItem);
    manager.register(ChorusFruitItem);
    manager.register(SuspiciousStewItem);
    manager.register(DriedKelpItem);
    manager.register(SweetBerriesItem);
    manager.register(HoneyBottleItem);
    manager.register(CookieItem);
    manager.register(MelonSliceItem);
    manager.register(BeetrootItem);
    manager.register(BeetrootSoupItem);
    manager.register(MushroomStewItem);
    manager.register(RabbitStewItem);
    manager.register(CarrotItem);
    manager.register(PotatoItem);
    manager.register(BakedPotatoItem);
    manager.register(PoisonousPotatoItem);
    manager.register(PumpkinPieItem);
    manager.register(TropicalFishItem);
    manager.register(PufferfishItem);
    manager.register(GlowBerriesItem);

    Arc::new(manager)
}
