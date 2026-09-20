//! Choosing which enchantments an item receives, and at which levels.
//!
//! This is the shared core behind every route that enchants something without a player
//! spending levels at a table: enchantment providers, raid equipment, mob spawn gear and
//! trial spawner loot all end up here. Mirrors vanilla's
//! `EnchantmentHelper.selectEnchantment` and `getAvailableEnchantmentResults`.

use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Enchantment, data_component_impl::EnchantableImpl, item::Item};
use rand::{Rng, RngExt};

/// An enchantment picked for an item, at the level it was picked at.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct EnchantmentInstance {
    pub enchantment: &'static Enchantment,
    pub level: i32,
}

/// The lowest level any enchantment can be offered at. Vanilla's `Enchantment.getMinLevel`
/// is a constant rather than part of the enchantment definition.
const MIN_ENCHANTMENT_LEVEL: i32 = 1;

/// Whether `enchantment` is offered on `item`.
///
/// `supported_items` is the set the enchantment may be applied to at all; `primary_items`,
/// where an enchantment declares one, narrows that to what it is offered on. Sharpness
/// supports every sharp weapon but is only offered on melee ones, so an axe can carry it
/// from an anvil while the table never proposes it for one. Mirrors
/// `Enchantment.isPrimaryItem`.
#[must_use]
pub fn is_primary_item(enchantment: &Enchantment, item: &Item) -> bool {
    item.has_tag(enchantment.supported_items)
        && enchantment
            .primary_items
            .is_none_or(|primary| item.has_tag(primary))
}

/// The enchantments from `candidates` that `cost` can buy for `stack`, each at the highest
/// level that cost reaches.
///
/// Mirrors `EnchantmentHelper.getAvailableEnchantmentResults`: levels are tried from the
/// maximum downwards and the first one whose cost range contains `cost` wins, so a higher
/// level is always preferred over a lower one.
#[must_use]
pub fn available_enchantment_results(
    cost: i32,
    stack: &ItemStack,
    candidates: &[&'static Enchantment],
) -> Vec<EnchantmentInstance> {
    // A book takes any enchantment regardless of what the enchantment is offered on.
    let is_book = stack.item.id == Item::BOOK.id;

    candidates
        .iter()
        .filter(|enchantment| is_book || is_primary_item(enchantment, stack.item))
        .filter_map(|enchantment| {
            (MIN_ENCHANTMENT_LEVEL..=enchantment.max_level)
                .rev()
                .find(|&level| {
                    cost >= enchantment.min_cost.calculate(level)
                        && cost <= enchantment.max_cost.calculate(level)
                })
                .map(|level| EnchantmentInstance { enchantment, level })
        })
        .collect()
}

/// Picks one entry, weighted by each enchantment's `weight`.
///
/// [`None`] when the list is empty or every weight is zero, matching vanilla's
/// `WeightedRandom.getRandomItem`.
fn pick_weighted<R: Rng>(
    rng: &mut R,
    entries: &[EnchantmentInstance],
) -> Option<EnchantmentInstance> {
    let total: i32 = entries.iter().map(|entry| entry.enchantment.weight).sum();
    if total <= 0 {
        return None;
    }

    let mut remaining = rng.random_range(0..total);
    entries
        .iter()
        .find(|entry| {
            remaining -= entry.enchantment.weight;
            remaining < 0
        })
        .copied()
}

/// Drops everything from `entries` that cannot sit alongside `picked`.
///
/// Mirrors `EnchantmentHelper.filterCompatibleEnchantments`: an enchantment is incompatible
/// with itself and with anything in its exclusive set.
fn retain_compatible(entries: &mut Vec<EnchantmentInstance>, picked: &EnchantmentInstance) {
    entries.retain(|entry| {
        entry.enchantment.id != picked.enchantment.id
            && picked
                .enchantment
                .exclusive_set
                .is_none_or(|set| !set.1.contains(&u16::from(entry.enchantment.id)))
            && entry
                .enchantment
                .exclusive_set
                .is_none_or(|set| !set.1.contains(&u16::from(picked.enchantment.id)))
    });
}

/// The enchantments `stack` receives for an enchanting cost of `cost`.
///
/// Mirrors `EnchantmentHelper.selectEnchantment`, in the same order, because the order is
/// what the result is: the cost is first pushed around by the item's own enchantability and
/// then by a small random span, one enchantment is drawn by weight, and further ones are
/// drawn while the halving cost keeps winning a roll against 50.
///
/// An item with no `enchantable` component takes nothing.
#[must_use]
pub fn select_enchantment<R: Rng>(
    rng: &mut R,
    stack: &ItemStack,
    cost: i32,
    candidates: &[&'static Enchantment],
) -> Vec<EnchantmentInstance> {
    let Some(enchantable) = stack.get_data_component::<EnchantableImpl>() else {
        return Vec::new();
    };

    let quarter = enchantable.value / 4 + 1;
    let mut cost = cost + 1 + rng.random_range(0..quarter) + rng.random_range(0..quarter);
    let span = (rng.random::<f32>() + rng.random::<f32>() - 1.0) * 0.15;
    #[allow(clippy::cast_possible_truncation)]
    let rounded = (cost as f32).mul_add(span, cost as f32).round() as i32;
    cost = rounded.max(1);

    let mut available = available_enchantment_results(cost, stack, candidates);
    let mut picked = Vec::new();
    let Some(first) = pick_weighted(rng, &available) else {
        return picked;
    };
    picked.push(first);

    while rng.random_range(0..50) <= cost {
        if let Some(last) = picked.last() {
            retain_compatible(&mut available, last);
        }
        if available.is_empty() {
            break;
        }
        let Some(next) = pick_weighted(rng, &available) else {
            break;
        };
        picked.push(next);
        cost /= 2;
    }

    picked
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    use super::{
        EnchantmentInstance, available_enchantment_results, is_primary_item, retain_compatible,
        select_enchantment,
    };
    use pumpkin_data::item_stack::ItemStack;
    use pumpkin_data::tag::Taggable;
    use pumpkin_data::{Enchantment, item::Item};

    #[test]
    fn an_axe_supports_sharpness_without_being_offered_it() {
        // This is the whole point of `primary_items` being separate from `supported_items`:
        // an axe accepts sharpness from an anvil, but nothing ever proposes it for one.
        assert!(
            Item::DIAMOND_AXE.has_tag(Enchantment::SHARPNESS.supported_items),
            "an axe is still a supported item"
        );
        assert!(!is_primary_item(
            &Enchantment::SHARPNESS,
            &Item::DIAMOND_AXE
        ));
        assert!(is_primary_item(
            &Enchantment::SHARPNESS,
            &Item::DIAMOND_SWORD
        ));
    }

    #[test]
    fn a_cost_buys_the_highest_level_it_reaches() {
        // Sharpness costs `1 + 11*(level-1)` to `21 + 11*(level-1)`, so the bands overlap
        // and a cost inside several of them has to resolve to the highest, not the first.
        let sword = ItemStack::new(1, &Item::DIAMOND_SWORD);
        let candidates = [&Enchantment::SHARPNESS];

        for (cost, expected_level) in [(5, 1), (12, 2), (23, 3), (45, 5)] {
            let results = available_enchantment_results(cost, &sword, &candidates);
            assert_eq!(
                results.len(),
                1,
                "cost {cost} should buy sharpness exactly once"
            );
            assert_eq!(
                results[0].level, expected_level,
                "cost {cost} should reach sharpness {expected_level}"
            );
        }
    }

    #[test]
    fn a_cost_below_every_band_buys_nothing() {
        let sword = ItemStack::new(1, &Item::DIAMOND_SWORD);
        let results = available_enchantment_results(0, &sword, &[&Enchantment::SHARPNESS]);
        assert!(results.is_empty(), "sharpness starts at cost 1");
    }

    #[test]
    fn picking_one_of_an_exclusive_set_drops_the_others() {
        // Protection and fire protection share `exclusive_set/armor`; unbreaking is in no
        // set at all and has to survive.
        let mut available = vec![
            EnchantmentInstance {
                enchantment: &Enchantment::FIRE_PROTECTION,
                level: 1,
            },
            EnchantmentInstance {
                enchantment: &Enchantment::UNBREAKING,
                level: 1,
            },
            EnchantmentInstance {
                enchantment: &Enchantment::PROTECTION,
                level: 1,
            },
        ];
        retain_compatible(
            &mut available,
            &EnchantmentInstance {
                enchantment: &Enchantment::PROTECTION,
                level: 1,
            },
        );

        let left: Vec<&str> = available
            .iter()
            .map(|entry| entry.enchantment.name)
            .collect();
        assert_eq!(left, ["minecraft:unbreaking"], "left with {left:?}");
    }

    #[test]
    fn an_item_that_cannot_be_enchanted_takes_nothing() {
        let mut rng = StdRng::seed_from_u64(1);
        let stone = ItemStack::new(1, &Item::STONE);
        let results = select_enchantment(&mut rng, &stone, 30, &[&Enchantment::SHARPNESS]);
        assert!(results.is_empty());
    }

    #[test]
    fn a_selection_never_contains_two_that_exclude_each_other() {
        // Whatever the roll, the result has to be a set that could coexist on one item.
        let armour = ItemStack::new(1, &Item::DIAMOND_CHESTPLATE);
        let candidates = [
            &Enchantment::PROTECTION,
            &Enchantment::FIRE_PROTECTION,
            &Enchantment::BLAST_PROTECTION,
            &Enchantment::PROJECTILE_PROTECTION,
            &Enchantment::UNBREAKING,
        ];

        for seed in 0..64 {
            let mut rng = StdRng::seed_from_u64(seed);
            let picked = select_enchantment(&mut rng, &armour, 30, &candidates);
            for (index, entry) in picked.iter().enumerate() {
                for other in &picked[index + 1..] {
                    assert_ne!(
                        entry.enchantment.id, other.enchantment.id,
                        "seed {seed} picked {} twice",
                        entry.enchantment.name
                    );
                    if let Some(set) = entry.enchantment.exclusive_set {
                        assert!(
                            !set.1.contains(&u16::from(other.enchantment.id)),
                            "seed {seed} picked {} alongside {}",
                            entry.enchantment.name,
                            other.enchantment.name
                        );
                    }
                }
                assert!(
                    (1..=entry.enchantment.max_level).contains(&entry.level),
                    "seed {seed} gave {} an out-of-range level {}",
                    entry.enchantment.name,
                    entry.level
                );
            }
        }
    }
}
