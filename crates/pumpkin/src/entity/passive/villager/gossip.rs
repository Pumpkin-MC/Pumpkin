use std::collections::{HashMap, HashSet};

use rand::{Rng, RngExt};
use uuid::Uuid;

use super::data::GossipType;

pub struct GossipTypeData {
    pub weight: i32,
    pub max: i32,
    pub decay_per_day: i32,
    pub decay_per_transfer: i32,
}

impl GossipType {
    /// `GossipType.java:7-11`: (id, weight, max, decayPerDay, decayPerTransfer).
    #[must_use]
    pub const fn data(self) -> GossipTypeData {
        match self {
            Self::MajorNegative => GossipTypeData {
                weight: -5,
                max: 100,
                decay_per_day: 10,
                decay_per_transfer: 10,
            },
            Self::MinorNegative => GossipTypeData {
                weight: -1,
                max: 200,
                decay_per_day: 20,
                decay_per_transfer: 20,
            },
            Self::MinorPositive => GossipTypeData {
                weight: 1,
                max: 25,
                decay_per_day: 1,
                decay_per_transfer: 5,
            },
            Self::MajorPositive => GossipTypeData {
                weight: 5,
                max: 20,
                decay_per_day: 0,
                decay_per_transfer: 20,
            },
            Self::Trading => GossipTypeData {
                weight: 1,
                max: 25,
                decay_per_day: 2,
                decay_per_transfer: 20,
            },
        }
    }
}

/// `GossipContainer.DISCARD_THRESHOLD` (`GossipContainer.java:32`).
pub const DISCARD_THRESHOLD: i32 = 2;

/// Rust port of vanilla's `GossipContainer` (`GossipContainer.java`): per-target-UUID,
/// per-type reputation entries with decay and a weighted-random transfer mechanic.
#[derive(Default)]
pub struct GossipContainer {
    entries: HashMap<Uuid, HashMap<GossipType, i32>>,
}

impl GossipContainer {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub const fn from_raw(entries: HashMap<Uuid, HashMap<GossipType, i32>>) -> Self {
        Self { entries }
    }

    #[must_use]
    pub const fn raw(&self) -> &HashMap<Uuid, HashMap<GossipType, i32>> {
        &self.entries
    }

    /// `GossipContainer::add` (`GossipContainer.java:117-124`), inlining
    /// `mergeValuesForAddition` and `makeSureValueIsntTooLowOrTooHigh`.
    pub fn add(&mut self, target: Uuid, gossip_type: GossipType, amount_to_add: i32) {
        let max = gossip_type.data().max;
        let entity_entries = self.entries.entry(target).or_default();
        let old_value = entity_entries.get(&gossip_type).copied().unwrap_or(0);
        let sum = old_value + amount_to_add;
        let merged = if sum > max { max.max(old_value) } else { sum };

        if merged > max {
            entity_entries.insert(gossip_type, max);
        } else if merged < DISCARD_THRESHOLD {
            entity_entries.remove(&gossip_type);
        } else {
            entity_entries.insert(gossip_type, merged);
        }

        if entity_entries.is_empty() {
            self.entries.remove(&target);
        }
    }

    /// `GossipContainer::getReputation` (`GossipContainer.java:108-111`), weighted sum
    /// over the entries matching `include`.
    #[must_use]
    pub fn get_reputation(&self, target: Uuid, mut include: impl FnMut(GossipType) -> bool) -> i32 {
        self.entries.get(&target).map_or(0, |types| {
            types
                .iter()
                .filter(|(t, _)| include(**t))
                .map(|(t, v)| v * t.data().weight)
                .sum()
        })
    }

    /// `EntityGossips::decay` (`GossipContainer.java:191-203`).
    pub fn decay(&mut self) {
        self.entries.retain(|_, types| {
            types.retain(|t, v| {
                *v -= t.data().decay_per_day;
                *v >= DISCARD_THRESHOLD
            });
            !types.is_empty()
        });
    }

    /// `GossipContainer::transferFrom` (`GossipContainer.java:98-106`) plus
    /// `selectGossipsForTransfer` (`GossipContainer.java:68-92`): picks `max_count`
    /// weighted-random entries from `source` (weight = `|value * type.weight|`) and merges
    /// their decayed value into `self` via `max(old, new)`. Not currently invoked anywhere
    /// (see module doc) -- Pumpkin has no brain/sensor system driving villagers to meet
    /// and gossip periodically, so this is implemented and unit-tested but dormant.
    pub fn transfer_from(&mut self, source: &Self, rng: &mut impl Rng, max_count: usize) {
        let flat: Vec<(Uuid, GossipType, i32)> = source
            .entries
            .iter()
            .flat_map(|(uuid, types)| types.iter().map(|(t, v)| (*uuid, *t, *v)))
            .collect();
        if flat.is_empty() {
            return;
        }

        let mut cumulative: i64 = 0;
        let mut ranges = Vec::with_capacity(flat.len());
        for (_, t, v) in &flat {
            cumulative += i64::from((v * t.data().weight).abs());
            ranges.push(cumulative);
        }
        if cumulative == 0 {
            return;
        }

        let mut chosen = HashSet::new();
        for _ in 0..max_count {
            let pick = rng.random_range(0..cumulative);
            let idx = ranges.partition_point(|&r| r <= pick);
            chosen.insert(idx.min(flat.len() - 1));
        }

        for idx in chosen {
            let (uuid, gossip_type, value) = flat[idx];
            let decayed = value - gossip_type.data().decay_per_transfer;
            if decayed >= DISCARD_THRESHOLD {
                let existing = self.entries.entry(uuid).or_default();
                let merged = existing
                    .get(&gossip_type)
                    .copied()
                    .unwrap_or(0)
                    .max(decayed);
                existing.insert(gossip_type, merged);
            }
        }
    }
}

/// `Villager::updateSpecialPrices` (`Villager.java:442-448`), i.e.
/// `addToSpecialPriceDiff(-Mth.floor(reputation * offer.getPriceMultiplier()))`.
///
/// Hero of the Village's additional discount (`Villager.java:450-458`) is not applied here.
#[must_use]
pub fn reputation_price_discount(reputation: i32, price_multiplier: f32) -> i32 {
    #[allow(clippy::cast_possible_truncation)]
    -((reputation as f32 * price_multiplier).floor() as i32)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn gossip_type_data_matches_vanilla() {
        let major_negative = GossipType::MajorNegative.data();
        assert_eq!(major_negative.weight, -5);
        assert_eq!(major_negative.max, 100);
        assert_eq!(major_negative.decay_per_day, 10);
        assert_eq!(major_negative.decay_per_transfer, 10);

        let trading = GossipType::Trading.data();
        assert_eq!(trading.weight, 1);
        assert_eq!(trading.max, 25);
        assert_eq!(trading.decay_per_day, 2);
        assert_eq!(trading.decay_per_transfer, 20);
    }

    #[test]
    fn add_accumulates_until_max_then_clamps() {
        let mut container = GossipContainer::new();
        let target = Uuid::new_v4();
        container.add(target, GossipType::Trading, 2);
        container.add(target, GossipType::Trading, 2);
        assert_eq!(container.get_reputation(target, |_| true), 4);

        container.add(target, GossipType::Trading, 100);
        assert_eq!(container.get_reputation(target, |_| true), 25);
    }

    #[test]
    fn add_below_discard_threshold_removes_entry() {
        let mut container = GossipContainer::new();
        let target = Uuid::new_v4();
        container.add(target, GossipType::MinorNegative, 5);
        container.add(target, GossipType::MinorNegative, -4);
        assert!(container.raw().is_empty());
    }

    #[test]
    fn get_reputation_is_weighted_sum() {
        let mut container = GossipContainer::new();
        let target = Uuid::new_v4();
        container.add(target, GossipType::MinorNegative, 25);
        container.add(target, GossipType::Trading, 2);
        assert_eq!(container.get_reputation(target, |_| true), -25 + 2);
    }

    #[test]
    fn decay_reduces_and_evicts_stale_entries() {
        let mut container = GossipContainer::new();
        let target = Uuid::new_v4();
        container.add(target, GossipType::Trading, 4);
        container.decay();
        assert_eq!(container.get_reputation(target, |_| true), 2);
        container.decay();
        assert!(container.raw().is_empty());
    }

    #[test]
    fn reputation_price_discount_matches_formula() {
        assert_eq!(reputation_price_discount(20, 0.05), -1);
        assert_eq!(reputation_price_discount(-20, 0.05), 1);
        assert_eq!(reputation_price_discount(0, 0.05), 0);
    }

    #[test]
    fn transfer_from_moves_decayed_entries() {
        let mut source = GossipContainer::new();
        let target = Uuid::new_v4();
        source.add(target, GossipType::MinorPositive, 25);

        let mut dest = GossipContainer::new();
        let mut rng = rand::rng();
        dest.transfer_from(&source, &mut rng, 10);

        assert_eq!(dest.get_reputation(target, |_| true), 25 - 5);
    }
}
