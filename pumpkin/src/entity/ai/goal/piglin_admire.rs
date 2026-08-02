//! Simplified port of vanilla `PiglinAi`'s gold-admiring/bartering core behaviors.
//!
//! Ports `StartAdmiringItemIfSeen`, `StopHoldingItemIfNoLongerAdmiring` and
//! `PiglinAi.stopHoldingOffHandItem`. Pumpkin has no Brain/Memory/Activity system,
//! so this is a standalone `Goal` rather than a Core-activity behavior.
//!
//! Scope reduction vs. vanilla: admiring only ever starts via a player directly
//! handing the piglin a gold ingot (`PiglinEntity::mob_interact`), never via a
//! ground-item pickup (`isLovedItem`/`GoToWantedItem`), since Pumpkin has no generic
//! mob item-pickup subsystem. Consequently the offhand item is always exactly the
//! bartering item and the "equip if armor" branch of `stopHoldingOffHandItem` is
//! unreachable here.

use std::sync::{Arc, atomic::Ordering};

use pumpkin_data::{
    data_component::DataComponent,
    data_component_impl::{EquipmentSlot, PotionContentsImpl},
    entity::EntityType,
    item::Item,
    item_stack::ItemStack,
    potion::Potion,
};
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

use super::{Controls, Goal, GoalFuture};
use crate::entity::{Entity, EntityBase, item::ItemEntity, mob::Mob};

/// `PiglinAi.ADMIRE_DURATION` (PiglinAi.java:85), used both for the direct-gift path
/// (`mobInteract` -> `admireGoldItem`, line 544) and the ground-pickup path we don't
/// implement.
pub const ADMIRE_DURATION_TICKS: i32 = 119;

/// A weighted entry mirroring one `minecraft:item` pool entry from
/// `data/minecraft/loot_table/gameplay/piglin_bartering.json`. The table is a single
/// roll against these 19 weighted entries (weights sum to 469, see `total_weight`
/// test). `enchant_randomly` (book, iron boots) is not applied -- it needs an
/// enchantment-level-rolling helper this codebase doesn't have; those two entries are
/// bartered as plain, unenchanted items.
struct BarterEntry {
    item: &'static Item,
    weight: u32,
    min_count: u8,
    max_count: u8,
    /// Registry name for `minecraft:set_potion`, applied via `PotionContentsImpl`.
    potion: Option<&'static str>,
}

const BARTER_TABLE: &[BarterEntry] = &[
    BarterEntry {
        item: &Item::BOOK,
        weight: 5,
        min_count: 1,
        max_count: 1,
        potion: None,
    },
    BarterEntry {
        item: &Item::IRON_BOOTS,
        weight: 8,
        min_count: 1,
        max_count: 1,
        potion: None,
    },
    BarterEntry {
        item: &Item::POTION,
        weight: 8,
        min_count: 1,
        max_count: 1,
        potion: Some("fire_resistance"),
    },
    BarterEntry {
        item: &Item::SPLASH_POTION,
        weight: 8,
        min_count: 1,
        max_count: 1,
        potion: Some("fire_resistance"),
    },
    BarterEntry {
        item: &Item::POTION,
        weight: 10,
        min_count: 1,
        max_count: 1,
        potion: Some("water"),
    },
    BarterEntry {
        item: &Item::IRON_NUGGET,
        weight: 10,
        min_count: 10,
        max_count: 36,
        potion: None,
    },
    BarterEntry {
        item: &Item::ENDER_PEARL,
        weight: 10,
        min_count: 2,
        max_count: 4,
        potion: None,
    },
    BarterEntry {
        item: &Item::DRIED_GHAST,
        weight: 10,
        min_count: 1,
        max_count: 1,
        potion: None,
    },
    BarterEntry {
        item: &Item::STRING,
        weight: 20,
        min_count: 3,
        max_count: 9,
        potion: None,
    },
    BarterEntry {
        item: &Item::QUARTZ,
        weight: 20,
        min_count: 5,
        max_count: 12,
        potion: None,
    },
    BarterEntry {
        item: &Item::OBSIDIAN,
        weight: 40,
        min_count: 1,
        max_count: 1,
        potion: None,
    },
    BarterEntry {
        item: &Item::CRYING_OBSIDIAN,
        weight: 40,
        min_count: 1,
        max_count: 3,
        potion: None,
    },
    BarterEntry {
        item: &Item::FIRE_CHARGE,
        weight: 40,
        min_count: 1,
        max_count: 1,
        potion: None,
    },
    BarterEntry {
        item: &Item::LEATHER,
        weight: 40,
        min_count: 2,
        max_count: 4,
        potion: None,
    },
    BarterEntry {
        item: &Item::SOUL_SAND,
        weight: 40,
        min_count: 2,
        max_count: 8,
        potion: None,
    },
    BarterEntry {
        item: &Item::NETHER_BRICK,
        weight: 40,
        min_count: 2,
        max_count: 8,
        potion: None,
    },
    BarterEntry {
        item: &Item::SPECTRAL_ARROW,
        weight: 40,
        min_count: 6,
        max_count: 12,
        potion: None,
    },
    BarterEntry {
        item: &Item::GRAVEL,
        weight: 40,
        min_count: 8,
        max_count: 16,
        potion: None,
    },
    BarterEntry {
        item: &Item::BLACKSTONE,
        weight: 40,
        min_count: 8,
        max_count: 16,
        potion: None,
    },
];

fn total_weight() -> u32 {
    BARTER_TABLE.iter().map(|entry| entry.weight).sum()
}

fn pick_entry(mut roll: u32) -> &'static BarterEntry {
    for entry in BARTER_TABLE {
        if roll < entry.weight {
            return entry;
        }
        roll -= entry.weight;
    }
    // Unreachable as long as `roll < total_weight()`, kept as a safe fallback.
    &BARTER_TABLE[BARTER_TABLE.len() - 1]
}

fn roll_barter_loot() -> ItemStack {
    let roll = rand::rng().random_range(0..total_weight());
    let entry = pick_entry(roll);
    let count = if entry.max_count > entry.min_count {
        rand::rng().random_range(entry.min_count..=entry.max_count)
    } else {
        entry.min_count
    };

    let mut stack = ItemStack::new(count, entry.item);
    if let Some(potion_name) = entry.potion
        && let Some(potion) = Potion::from_name(potion_name)
    {
        stack.patch.push((
            DataComponent::PotionContents,
            Some(Box::new(PotionContentsImpl {
                potion_id: Some(potion.id as i32),
                custom_color: None,
                custom_effects: Vec::new(),
                custom_name: None,
            })),
        ));
    }
    stack
}

/// `BehaviorUtils.throwItem` (BehaviorUtils.java:89-105): spawns the item at the
/// thrower's eye height minus 0.3, with velocity `normalize(targetPos - throwerPos) *
/// (0.3, 0.3, 0.3)`. `stopHoldingOffHandItem`/`throwItems` (PiglinAi.java:413-437)
/// throw toward the nearest visible player if any, else a nearby position; we throw
/// toward the piglin's own position (i.e. just up) when no player is around, which is
/// a simplification of vanilla's `getRandomNearbyPos`.
async fn throw_barter_item(mob: &dyn Mob, stack: ItemStack) {
    let entity = &mob.get_mob_entity().living_entity.entity;
    let world = entity.world.load_full();
    let pos = entity.pos.load();

    let target_pos = world.get_closest_player(pos, 16.0).map_or(pos, |player| {
        player.get_entity().pos.load() + Vector3::new(0.0, 1.0, 0.0)
    });

    let throw_vector = target_pos - pos;
    let velocity = if throw_vector.length_squared() > 1.0e-6 {
        throw_vector.normalize() * 0.3
    } else {
        Vector3::new(0.0, 0.3, 0.0)
    };

    let spawn_pos = Vector3::new(pos.x, entity.get_eye_y() - 0.3, pos.z);
    let item_entity = Entity::new(world.clone(), spawn_pos, &EntityType::ITEM);
    let item_entity = ItemEntity::new_with_velocity(item_entity, stack, velocity, 10);
    world.spawn_entity(Arc::new(item_entity)).await;

    mob.get_mob_entity().living_entity.swing_hand().await;
}

/// Runs while a piglin is staring at a gold ingot in its offhand.
///
/// Started externally by `PiglinEntity::mob_interact` (which sets `admiring_ticks`
/// and equips the offhand item); this goal only owns the countdown and the barter
/// payout. Shares `admiring_ticks` with the owning `PiglinEntity` so both sides can
/// read/write it without downcasting `&dyn Mob` back to the concrete entity type.
pub struct PiglinAdmireGoal {
    admiring_ticks: Arc<std::sync::atomic::AtomicI32>,
}

impl PiglinAdmireGoal {
    #[must_use]
    pub fn new(admiring_ticks: Arc<std::sync::atomic::AtomicI32>) -> Box<Self> {
        Box::new(Self { admiring_ticks })
    }
}

impl Goal for PiglinAdmireGoal {
    fn can_start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move { self.admiring_ticks.load(Ordering::Relaxed) > 0 })
    }

    fn should_continue<'a>(&'a self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move { self.admiring_ticks.load(Ordering::Relaxed) > 0 })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let ticks = &self.admiring_ticks;
            let remaining = ticks.fetch_sub(1, Ordering::Relaxed) - 1;
            if remaining > 0 {
                return;
            }
            ticks.store(0, Ordering::Relaxed);

            // `PiglinAi.stopHoldingOffHandItem` (PiglinAi.java:373-399), adult branch.
            let equipment = mob
                .get_mob_entity()
                .living_entity
                .entity_equipment
                .lock()
                .await;
            let offhand = equipment.get(&EquipmentSlot::OFF_HAND);
            let taken = {
                let mut stack = offhand.lock().await;
                std::mem::replace(&mut *stack, ItemStack::EMPTY.clone())
            };
            drop(equipment);
            mob.get_mob_entity()
                .living_entity
                .send_equipment_changes(&[(EquipmentSlot::OFF_HAND, ItemStack::EMPTY.clone())]);

            if taken.is_empty() {
                return;
            }
            if taken.item.id == Item::GOLD_INGOT.id {
                throw_barter_item(mob, roll_barter_loot()).await;
            }
            // Non-gold-ingot branch (equip-if-armor / put-in-inventory) intentionally
            // omitted: see module doc, this branch is unreachable in our
            // interact-only trigger.
        })
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn total_weight_matches_vanilla_table() {
        assert_eq!(total_weight(), 469);
    }

    #[test]
    fn every_weight_bucket_resolves_to_an_entry() {
        let total = total_weight();
        for roll in 0..total {
            // Must not panic and must stay within the table.
            let _entry = pick_entry(roll);
        }
    }

    #[test]
    fn pick_entry_boundaries_land_on_expected_entries() {
        assert_eq!(pick_entry(0).item.id, BARTER_TABLE[0].item.id);
        assert_eq!(pick_entry(4).item.id, BARTER_TABLE[0].item.id);
        assert_eq!(pick_entry(5).item.id, BARTER_TABLE[1].item.id);
        let last = &BARTER_TABLE[BARTER_TABLE.len() - 1];
        assert_eq!(pick_entry(468).item.id, last.item.id);
        assert_eq!(pick_entry(468).weight, last.weight);
    }
}
