use items::{egg::EggItem, snowball::SnowBallItem};
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

    Arc::new(manager)
}
