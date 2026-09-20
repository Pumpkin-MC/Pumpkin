//! Applying an enchantment provider to an item.
//!
//! A provider is how something other than a player at a table decides what an item should
//! carry: raid equipment, mob spawn gear, the enderman's silk touch. Vanilla ships them as
//! datapack entries and applies them through `EnchantmentHelper.enchantItemFromProvider`.

use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::{Enchantment, EnchantmentProvider};
use rand::{Rng, RngExt};

use super::selection::select_enchantment;

/// Puts `provider`'s enchantments on `stack`.
///
/// `difficulty` is the regional difficulty's special multiplier, which is what widens the
/// cost band of a `ByCostWithDifficulty` provider: at 0.0 it buys exactly `min_cost`, and
/// at 1.0 it can reach `min_cost + max_cost_span`.
///
/// Levels are upgraded rather than replaced, matching `ItemEnchantments.Mutable.upgrade`:
/// an item that already carries a higher level of the same enchantment keeps it.
pub fn enchant_from_provider<R: Rng>(
    rng: &mut R,
    stack: &mut ItemStack,
    provider: &EnchantmentProvider,
    difficulty: f32,
) {
    match provider {
        EnchantmentProvider::Single { enchantment, level } => {
            // Vanilla clamps the level into the enchantment's own range rather than
            // trusting the provider, so a datapack cannot ask for level 7 sharpness.
            let level = (*level).clamp(1, enchantment.max_level);
            stack.enchant(enchantment, level);
        }
        EnchantmentProvider::ByCostWithDifficulty {
            enchantments,
            min_cost,
            max_cost_span,
        } => {
            #[allow(clippy::cast_possible_truncation)]
            let span = (difficulty * *max_cost_span as f32) as i32;
            let cost = rng.random_range(*min_cost..=*min_cost + span);

            let candidates: Vec<&'static Enchantment> = enchantments
                .1
                .iter()
                .filter_map(|id| Enchantment::from_id(*id as u8))
                .collect();

            for picked in select_enchantment(rng, stack, cost, &candidates) {
                stack.enchant(picked.enchantment, picked.level);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    use super::enchant_from_provider;
    use pumpkin_data::data_component_impl::EnchantmentsImpl;
    use pumpkin_data::item_stack::ItemStack;
    use pumpkin_data::tag::Taggable;
    use pumpkin_data::{Enchantment, EnchantmentProvider, item::Item};

    fn levels(stack: &ItemStack) -> Vec<(&'static str, i32)> {
        stack
            .get_data_component::<EnchantmentsImpl>()
            .map(|data| {
                data.enchantment
                    .iter()
                    .map(|(enchantment, level)| (enchantment.name, *level))
                    .collect()
            })
            .unwrap_or_default()
    }

    #[test]
    fn a_single_provider_applies_its_enchantment() {
        let mut rng = StdRng::seed_from_u64(0);
        let mut axe = ItemStack::new(1, &Item::IRON_AXE);
        enchant_from_provider(
            &mut rng,
            &mut axe,
            &EnchantmentProvider::RAID_VINDICATOR_POST_WAVE_5,
            0.0,
        );
        assert_eq!(levels(&axe), [("minecraft:sharpness", 2)]);
    }

    #[test]
    fn a_weaker_provider_does_not_undo_a_stronger_one() {
        // Both raid vindicator providers give sharpness; the post-wave-5 one gives level 2.
        // A wave that applies the level 1 provider afterwards must not take that away.
        let mut rng = StdRng::seed_from_u64(0);
        let mut axe = ItemStack::new(1, &Item::IRON_AXE);
        enchant_from_provider(
            &mut rng,
            &mut axe,
            &EnchantmentProvider::RAID_VINDICATOR_POST_WAVE_5,
            0.0,
        );
        enchant_from_provider(
            &mut rng,
            &mut axe,
            &EnchantmentProvider::RAID_VINDICATOR,
            0.0,
        );
        assert_eq!(levels(&axe), [("minecraft:sharpness", 2)]);
    }

    #[test]
    fn a_by_cost_provider_only_draws_from_its_own_tag() {
        let mut given = 0;
        for seed in 0..64 {
            let mut rng = StdRng::seed_from_u64(seed);
            let mut helmet = ItemStack::new(1, &Item::IRON_HELMET);
            enchant_from_provider(
                &mut rng,
                &mut helmet,
                &EnchantmentProvider::MOB_SPAWN_EQUIPMENT,
                1.0,
            );
            for (name, level) in levels(&helmet) {
                given += 1;
                let enchantment =
                    Enchantment::from_name(name).expect("the stack reported a real enchantment");
                assert!(
                    enchantment
                        .has_tag(&pumpkin_data::tag::Enchantment::MINECRAFT_ON_MOB_SPAWN_EQUIPMENT),
                    "seed {seed} gave {name}, which is not on the provider's tag"
                );
                assert!(
                    (1..=enchantment.max_level).contains(&level),
                    "seed {seed} gave {name} an out-of-range level {level}"
                );
            }
        }
        assert!(
            given > 0,
            "64 seeds at full difficulty should enchant something at least once"
        );
    }
}
