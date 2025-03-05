mod egg;
mod snowball;
mod sword;
mod trident;

use std::sync::Arc;

use egg::EggItem;
use snowball::SnowBallItem;
use sword::SwordItem;
use trident::TridentItem;

use super::registry::ItemRegistry;
#[must_use]
pub fn default_registry() -> Arc<ItemRegistry> {
    let mut manager = ItemRegistry::default();

    manager.register(SnowBallItem);
    manager.register(EggItem);
    manager.register(SwordItem);
    manager.register(TridentItem);

    Arc::new(manager)
}
