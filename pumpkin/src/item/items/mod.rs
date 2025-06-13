mod axe;
mod bucket;
mod egg;
mod ender_eye;
mod flint_and_steel;
mod hoe;
mod honeycomb;
mod shovel;
mod snowball;
mod sword;
mod trident;
mod fire_charge;

use axe::AxeItem;
use bucket::{EmptyBucketItem, FilledBucketItem};
use egg::EggItem;
use ender_eye::EnderEyeItem;
use flint_and_steel::FlintAndSteelItem;
use hoe::HoeItem;
use honeycomb::HoneyCombItem;
use shovel::ShovelItem;
use snowball::SnowBallItem;
use std::sync::Arc;
use sword::SwordItem;
use trident::TridentItem;
use fire_charge::FireChargeItem;
use super::registry::ItemRegistry;
#[must_use]
pub fn default_registry() -> Arc<ItemRegistry> {
    let mut manager = ItemRegistry::default();

    manager.register(SnowBallItem);
    manager.register(HoeItem);
    manager.register(EggItem);
    manager.register(FlintAndSteelItem);
    manager.register(SwordItem);
    manager.register(TridentItem);
    manager.register(EmptyBucketItem);
    manager.register(FilledBucketItem);
    manager.register(ShovelItem);
    manager.register(AxeItem);
    manager.register(HoneyCombItem);
    manager.register(EnderEyeItem);
    manager.register(FireChargeItem);

    Arc::new(manager)
}
