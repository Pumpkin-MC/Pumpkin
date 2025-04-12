mod egg;
mod hoe;
mod snowball;
mod sword;
mod trident;
mod bucket;

use std::sync::Arc;

use egg::EggItem;
use hoe::HoeItem;
use snowball::SnowBallItem;
use sword::SwordItem;
use trident::TridentItem;
use bucket::BucketItem;

use super::registry::ItemRegistry;
#[must_use]
pub fn default_registry() -> Arc<ItemRegistry> {
    let mut manager = ItemRegistry::default();

    manager.register(SnowBallItem);
    manager.register(HoeItem);
    manager.register(EggItem);
    manager.register(SwordItem);
    manager.register(TridentItem);
    manager.register(BucketItem);

    Arc::new(manager)
}
