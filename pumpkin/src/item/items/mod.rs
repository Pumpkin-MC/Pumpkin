mod egg;
mod hoe;
mod snowball;
mod sword;
mod trident;
mod shovel;
mod axe;
mod honeycomb;

use std::sync::Arc;
use egg::EggItem;
use hoe::HoeItem;
use shovel::ShovelItem;
use snowball::SnowBallItem;
use sword::SwordItem;
use trident::TridentItem;
use axe::AxeItem;
use honeycomb::HoneyCombItem;

use super::registry::ItemRegistry;
#[must_use]
pub fn default_registry() -> Arc<ItemRegistry> {
    let mut manager = ItemRegistry::default();

    manager.register(SnowBallItem);
    manager.register(HoeItem);
    manager.register(EggItem);
    manager.register(SwordItem);
    manager.register(TridentItem);
    manager.register(ShovelItem);
    manager.register(AxeItem);
    manager.register(HoneyCombItem);

    Arc::new(manager)
}
