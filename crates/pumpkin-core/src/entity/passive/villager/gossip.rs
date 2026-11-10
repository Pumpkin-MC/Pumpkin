use rand::RngExt;
use rustc_hash::FxHashMap;
use uuid::Uuid;

use super::data::GossipType;

const DISCARD_THRESHOLD: i32 = 2;

#[derive(Default, Clone)]
pub struct GossipContainer {
    gossips: FxHashMap<Uuid, FxHashMap<GossipType, i32>>,
}

#[derive(Clone, Copy)]
struct GossipEntry {
    target: Uuid,
    gossip_type: GossipType,
    value: i32,
}

impl GossipEntry {
    const fn weighted_value(self) -> i32 {
        self.value.wrapping_mul(self.gossip_type.weight())
    }
}

impl GossipContainer {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.gossips.is_empty()
    }

    pub fn decay(&mut self) {
        for gossips in self.gossips.values_mut() {
            gossips.retain(|gossip_type, value| {
                *value = value.wrapping_sub(gossip_type.daily_decay());
                *value >= DISCARD_THRESHOLD
            });
        }
        self.gossips.retain(|_, gossips| !gossips.is_empty());
    }

    #[must_use]
    pub fn get_reputation(&self, target: Uuid, filter: impl Fn(GossipType) -> bool) -> i32 {
        self.gossips.get(&target).map_or(0, |gossips| {
            gossips
                .iter()
                .filter(|(gossip_type, _)| filter(**gossip_type))
                .map(|(gossip_type, value)| value.wrapping_mul(gossip_type.weight()))
                .sum()
        })
    }

    pub fn add(&mut self, target: Uuid, gossip_type: GossipType, value: i32) {
        let empty = {
            let gossips = self.gossips.entry(target).or_default();
            match gossips.get_mut(&gossip_type) {
                Some(old_value) => {
                    *old_value = merge_values_for_addition(gossip_type, *old_value, value);
                }
                None => {
                    gossips.insert(gossip_type, value);
                }
            }
            if let Some(current) = gossips.get_mut(&gossip_type) {
                if *current > gossip_type.max_value() {
                    *current = gossip_type.max_value();
                }
                if *current < DISCARD_THRESHOLD {
                    gossips.remove(&gossip_type);
                }
            }
            gossips.is_empty()
        };
        if empty {
            self.gossips.remove(&target);
        }
    }

    pub fn remove(&mut self, target: Uuid, gossip_type: GossipType, value: i32) {
        self.add(target, gossip_type, value.wrapping_neg());
    }

    pub fn remove_gossip(&mut self, gossip_type: GossipType) {
        for gossips in self.gossips.values_mut() {
            gossips.remove(&gossip_type);
        }
        self.gossips.retain(|_, gossips| !gossips.is_empty());
    }

    pub fn transfer_from(
        &mut self,
        source: &Self,
        random: &mut impl rand::Rng,
        max_gossips_to_transfer: i32,
    ) -> i32 {
        let new_gossips = source.select_gossips_for_transfer(random, max_gossips_to_transfer);
        for new_gossip in &new_gossips {
            let decayed_value = new_gossip
                .value
                .wrapping_sub(new_gossip.gossip_type.decay_per_transfer());
            if decayed_value >= DISCARD_THRESHOLD {
                let gossips = self.gossips.entry(new_gossip.target).or_default();
                match gossips.get_mut(&new_gossip.gossip_type) {
                    Some(old_value) => {
                        *old_value = (*old_value).max(decayed_value);
                    }
                    None => {
                        gossips.insert(new_gossip.gossip_type, decayed_value);
                    }
                }
            }
        }
        new_gossips.len() as i32
    }

    pub fn put_all(&mut self, other: &Self) {
        for (target, gossips) in &other.gossips {
            self.gossips.entry(*target).or_default().extend(
                gossips
                    .iter()
                    .map(|(gossip_type, value)| (*gossip_type, *value)),
            );
        }
    }

    #[must_use]
    pub fn entries(&self) -> Vec<(Uuid, GossipType, i32)> {
        self.unpack()
            .map(|entry| (entry.target, entry.gossip_type, entry.value))
            .collect()
    }

    #[must_use]
    pub fn from_entries(entries: impl IntoIterator<Item = (Uuid, GossipType, i32)>) -> Self {
        let mut container = Self::new();
        for (target, gossip_type, value) in entries {
            container
                .gossips
                .entry(target)
                .or_default()
                .insert(gossip_type, value);
        }
        container
    }

    fn unpack(&self) -> impl Iterator<Item = GossipEntry> + '_ {
        self.gossips.iter().flat_map(|(target, gossips)| {
            let target = *target;
            gossips.iter().map(move |(gossip_type, value)| GossipEntry {
                target,
                gossip_type: *gossip_type,
                value: *value,
            })
        })
    }

    fn select_gossips_for_transfer(
        &self,
        random: &mut impl rand::Rng,
        max_count: i32,
    ) -> Vec<GossipEntry> {
        let entries: Vec<GossipEntry> = self.unpack().collect();
        if entries.is_empty() {
            return Vec::new();
        }

        let mut ranges = vec![0; entries.len()];
        let mut ranges_end: i32 = 0;
        for (index, gossip) in entries.iter().enumerate() {
            ranges_end = ranges_end.wrapping_add(gossip.weighted_value().wrapping_abs());
            ranges[index] = ranges_end.wrapping_sub(1);
        }

        let mut results = Vec::new();
        for _ in 0..max_count {
            let choice = random.random_range(0..ranges_end);
            let selected_index = ranges.partition_point(|range| *range < choice);
            let entry = entries[selected_index];
            if !results.iter().any(|result: &GossipEntry| {
                result.target == entry.target && result.gossip_type == entry.gossip_type
            }) {
                results.push(entry);
            }
        }
        results
    }
}

fn merge_values_for_addition(gossip_type: GossipType, old_value: i32, new_value: i32) -> i32 {
    let sum = old_value.wrapping_add(new_value);
    if sum > gossip_type.max_value() {
        gossip_type.max_value().max(old_value)
    } else {
        sum
    }
}

#[cfg(test)]
mod tests {
    use rand::{SeedableRng, rngs::StdRng};
    use uuid::Uuid;

    use super::{GossipContainer, GossipType};

    #[test]
    fn transfer_from_limits_and_merges_by_max() {
        let target = Uuid::from_u128(1);
        let source = GossipContainer::from_entries([(target, GossipType::MajorPositive, 30)]);

        let mut destination =
            GossipContainer::from_entries([(target, GossipType::MajorPositive, 7)]);
        let mut rng = StdRng::seed_from_u64(42);

        assert_eq!(destination.transfer_from(&source, &mut rng, 0), 0);
        assert_eq!(
            destination.get_reputation(target, |_| true),
            7 * GossipType::MajorPositive.weight()
        );

        assert_eq!(destination.transfer_from(&source, &mut rng, 3), 1);
        assert_eq!(
            destination.get_reputation(target, |_| true),
            10 * GossipType::MajorPositive.weight()
        );

        let mut higher = GossipContainer::from_entries([(target, GossipType::MajorPositive, 15)]);
        assert_eq!(higher.transfer_from(&source, &mut rng, 1), 1);
        assert_eq!(
            higher.get_reputation(target, |_| true),
            15 * GossipType::MajorPositive.weight()
        );
    }

    #[test]
    fn transfer_from_respects_limit() {
        let source = GossipContainer::from_entries([
            (Uuid::from_u128(1), GossipType::MinorPositive, 10),
            (Uuid::from_u128(2), GossipType::MinorPositive, 10),
            (Uuid::from_u128(3), GossipType::MinorPositive, 10),
        ]);
        let mut destination = GossipContainer::new();
        let mut rng = StdRng::seed_from_u64(7);

        assert_eq!(destination.transfer_from(&source, &mut rng, 1), 1);
        assert_eq!(destination.entries().len(), 1);
    }

    #[test]
    fn decay_drops_exhausted_entries_and_empty_targets() {
        let first = Uuid::from_u128(1);
        let second = Uuid::from_u128(2);
        let mut container = GossipContainer::from_entries([
            (first, GossipType::MinorPositive, 2),
            (first, GossipType::MajorPositive, 20),
            (second, GossipType::Trading, 3),
        ]);

        container.decay();

        assert_eq!(
            container.entries(),
            vec![(first, GossipType::MajorPositive, 20)]
        );
        assert_eq!(container.get_reputation(second, |_| true), 0);
    }

    #[test]
    fn add_clamps_at_the_type_max() {
        let target = Uuid::from_u128(1);
        let mut container = GossipContainer::new();

        container.add(target, GossipType::MinorPositive, 100);
        assert_eq!(
            container.get_reputation(target, |_| true),
            GossipType::MinorPositive.max_value() * GossipType::MinorPositive.weight()
        );

        container.add(target, GossipType::MinorPositive, 100);
        assert_eq!(
            container.get_reputation(target, |_| true),
            GossipType::MinorPositive.max_value() * GossipType::MinorPositive.weight()
        );
    }

    #[test]
    fn get_reputation_sums_weighted_values_over_filter() {
        let target = Uuid::from_u128(1);
        let mut container = GossipContainer::new();

        container.add(target, GossipType::MajorPositive, 5);
        container.add(target, GossipType::Trading, 4);

        assert_eq!(container.get_reputation(target, |_| true), 29);
        assert_eq!(
            container.get_reputation(target, |gossip_type| {
                gossip_type == GossipType::MajorPositive
            }),
            25
        );
        assert_eq!(container.get_reputation(Uuid::from_u128(2), |_| true), 0);
    }
}
