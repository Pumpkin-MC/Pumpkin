pub mod armor_stand;
pub mod axe;
pub mod bucket;
pub mod dye;
pub mod egg;
pub mod end_crystal;
pub mod ender_eye;
pub mod firework_rocket;
pub mod glowing_ink_sac;
pub mod hoe;
pub mod honeycomb;
pub mod ignite;
pub mod ink_sac;
pub mod mace;
pub mod minecart;
pub mod name_tag;
pub mod shovel;
pub mod snowball;
pub mod spawn_egg;
pub mod swords;
pub mod trident;
pub mod wind_charge;

use crate::item::items::armor_stand::ArmorStandItem;
use crate::item::items::end_crystal::EndCrystalItem;
use crate::item::items::firework_rocket::FireworkRocketItem;
use crate::item::items::minecart::MinecartItem;
use crate::item::items::name_tag::NameTagItem;
use crate::item::items::spawn_egg::SpawnEggItem;
use crate::item::items::wind_charge::WindChargeItem;

use super::registry::ItemRegistry;
use axe::AxeItem;
use bucket::{EmptyBucketItem, FilledBucketItem};
use dye::DyeItem;
use egg::EggItem;
use ender_eye::EnderEyeItem;
use glowing_ink_sac::GlowingInkSacItem;
use hoe::HoeItem;
use honeycomb::HoneyCombItem;
use ignite::fire_charge::FireChargeItem;
use ignite::flint_and_steel::FlintAndSteelItem;
use ink_sac::InkSacItem;
use mace::MaceItem;
use shovel::ShovelItem;
use snowball::SnowBallItem;
use std::sync::Arc;
use swords::SwordItem;
use trident::TridentItem;

#[must_use]
pub fn default_registry() -> Arc<ItemRegistry> {
    let mut manager = ItemRegistry::default();

    manager.register(SnowBallItem);
    manager.register(HoeItem);
    manager.register(EggItem);
    manager.register(FlintAndSteelItem);
    manager.register(SwordItem);
    manager.register(MaceItem);
    manager.register(TridentItem);
    manager.register(EmptyBucketItem);
    manager.register(FilledBucketItem);
    manager.register(ShovelItem);
    manager.register(SpawnEggItem);
    manager.register(AxeItem);
    manager.register(EndCrystalItem);
    manager.register(MinecartItem);
    manager.register(HoneyCombItem);
    manager.register(NameTagItem);
    manager.register(EnderEyeItem);
    manager.register(FireChargeItem);
    manager.register(DyeItem);
    manager.register(FireworkRocketItem);
    manager.register(InkSacItem);
    manager.register(GlowingInkSacItem);
    manager.register(ArmorStandItem);
    manager.register(WindChargeItem);

    Arc::new(manager)
}
