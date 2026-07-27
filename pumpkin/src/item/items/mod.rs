pub mod armor_stand;
pub mod arrow;
pub mod axe;
pub mod boat;
pub mod bow;
pub mod bucket;
pub mod bundle;
pub mod crossbow;
pub mod dye;
pub mod egg;
pub mod end_crystal;
pub mod ender_eye;
pub mod ender_pearl;
pub mod firework_rocket;
pub mod fishing_rod;
pub mod glowing_ink_sac;
pub mod hoe;
pub mod honeycomb;
pub mod ignite;
pub mod ink_sac;
pub mod mace;
pub mod map;
pub mod minecart;
pub mod name_tag;
pub mod potions;
pub mod shovel;
pub mod snowball;
pub mod spawn_egg;
pub mod swords;
pub mod trident;
pub mod wind_charge;

pub mod lead;

use crate::item::items::armor_stand::ArmorStandItem;
use crate::item::items::boat::BoatItem;
use crate::item::items::bundle::BundleItem;
use crate::item::items::end_crystal::EndCrystalItem;
use crate::item::items::lead::LeadItem;
use crate::item::items::map::MapItem;

use crate::item::items::minecart::MinecartItem;
use crate::item::items::name_tag::NameTagItem;
use crate::item::items::spawn_egg::SpawnEggItem;
use crate::item::items::wind_charge::WindChargeItem;
use firework_rocket::FireworkRocketItem;
use fishing_rod::FishingRodItem;
use glowing_ink_sac::GlowingInkSacItem;

use super::registry::ItemRegistry;
use crate::item::items::potions::{LingeringPotionItem, PotionItem, SplashPotionItem};
use arrow::ArrowItem;
use axe::AxeItem;
use bow::BowItem;
use bucket::{EmptyBucketItem, FilledBucketItem};
use crossbow::CrossbowItem;
use dye::DyeItem;
use egg::EggItem;
use ender_eye::EnderEyeItem;
use ender_pearl::EnderPearlItem;
use hoe::HoeItem;
use honeycomb::HoneyCombItem;
use ignite::fire_charge::FireChargeItem;
use ignite::flint_and_steel::FlintAndSteelItem;
use ink_sac::InkSacItem;
use mace::MaceItem;
use pumpkin_data::{Block, BlockStateId};
use shovel::ShovelItem;
use snowball::SnowBallItem;
use std::sync::Arc;
use swords::SwordItem;
use trident::TridentItem;

/// Returns the state of `new_block` that carries over every property it shares
/// with `old_block` in `old_state_id`.
///
/// This is the equivalent of vanilla's `Block#withPropertiesOf`, which is what
/// stripping, waxing, scraping and de-oxidizing use so the replacement block
/// keeps its facing, waterlogging, slab type, and so on. Falls back to the
/// default state when either block carries no properties.
#[must_use]
pub fn state_with_properties_of(
    old_block: &Block,
    old_state_id: BlockStateId,
    new_block: &Block,
) -> BlockStateId {
    let default_state_id = new_block.default_state.id;
    // `from_properties` panics for blocks without any property, so only copy
    // when both sides actually carry state.
    if new_block.properties(default_state_id).is_none() {
        return default_state_id;
    }
    old_block
        .properties(old_state_id)
        .map_or(default_state_id, |properties| {
            new_block
                .from_properties(&properties.to_props())
                .to_state_id(new_block)
        })
}

#[must_use]
pub fn default_registry() -> Arc<ItemRegistry> {
    let mut manager = ItemRegistry::default();

    manager.register(ArrowItem);
    manager.register(BowItem);
    manager.register(CrossbowItem);
    manager.register(SnowBallItem);
    manager.register(HoeItem);
    manager.register(EggItem);
    manager.register(FlintAndSteelItem);
    manager.register(SwordItem);
    manager.register(MaceItem);
    manager.register(TridentItem);
    manager.register(FishingRodItem);
    // TODO: Register CrossbowItem with per-shot durability cost.
    // TODO: Register BrushItem with per-stroke durability cost.
    // TODO: Register CarrotOnAStickItem and WarpedFungusOnAStickItem with boost durability costs.
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
    manager.register(EnderPearlItem);
    manager.register(FireChargeItem);
    manager.register(DyeItem);
    manager.register(MapItem);
    manager.register(FireworkRocketItem);
    manager.register(InkSacItem);
    manager.register(GlowingInkSacItem);
    manager.register(ArmorStandItem);
    manager.register(WindChargeItem);
    manager.register(BoatItem);
    manager.register(PotionItem);
    manager.register(SplashPotionItem);
    manager.register(LingeringPotionItem);
    manager.register(BundleItem);
    manager.register(LeadItem);

    Arc::new(manager)
}
