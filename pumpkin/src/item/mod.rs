use items::{egg::EggItem, snowball::SnowBallItem, splash_potion::SplashPotionItem};
use registry::ItemRegistry;

use std::sync::Arc;

mod items;
pub mod pumpkin_item;
pub mod registry;

#[must_use]
pub fn default_registry() -> Arc<ItemRegistry> {
    let mut manager = ItemRegistry::default();

    manager.register(SnowBallItem);
    manager.register(EggItem);
    manager.register(SplashPotionItem);

    Arc::new(manager)
}
