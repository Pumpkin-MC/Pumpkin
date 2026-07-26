pub use pumpkin_data::statistic::{CustomStatistic, StatisticCategory};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use std::collections::HashMap;

#[derive(Default)]
pub struct Statistics {
    /// (Category ID, Statistic ID) -> Value
    pub stats: HashMap<(i32, i32), i32>,
}

impl Statistics {
    pub fn increment(&mut self, category: StatisticCategory, stat: i32, amount: i32) {
        let entry = self.stats.entry((category as i32, stat)).or_insert(0);
        *entry = entry.saturating_add(amount);
    }

    pub fn increment_custom(&mut self, stat: CustomStatistic, amount: i32) {
        self.increment(StatisticCategory::Custom, stat as i32, amount);
    }

    pub fn set(&mut self, category: StatisticCategory, stat: i32, value: i32) {
        self.stats.insert((category as i32, stat), value);
    }

    #[must_use]
    pub fn get(&self, category: StatisticCategory, stat: i32) -> i32 {
        *self.stats.get(&(category as i32, stat)).unwrap_or(&0)
    }

    pub fn write_nbt(&self, nbt: &mut NbtCompound) {
        let mut stats_compound = NbtCompound::new();
        for ((category, stat), value) in &self.stats {
            stats_compound.put_int(&format!("{category}:{stat}"), *value);
        }
        nbt.put_compound("Statistics", stats_compound);
    }

    pub fn read_nbt(&mut self, nbt: &NbtCompound) {
        if let Some(stats_compound) = nbt.get_compound("Statistics") {
            for (key, tag) in &stats_compound.child_tags {
                let parts: Vec<&str> = key.split(':').collect();
                if let (NbtTag::Int(value), [cat_str, stat_str]) = (tag, parts.as_slice())
                    && let (Ok(category), Ok(stat)) =
                        (cat_str.parse::<i32>(), stat_str.parse::<i32>())
                {
                    self.stats.insert((category, stat), *value);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn increment_accumulates_and_saturates() {
        let mut stats = Statistics::default();
        assert_eq!(stats.get(StatisticCategory::Mined, 7), 0);
        stats.increment(StatisticCategory::Mined, 7, 2);
        stats.increment(StatisticCategory::Mined, 7, 3);
        assert_eq!(stats.get(StatisticCategory::Mined, 7), 5);
        stats.increment(StatisticCategory::Mined, 7, i32::MAX);
        assert_eq!(stats.get(StatisticCategory::Mined, 7), i32::MAX);
    }

    #[test]
    fn set_overwrites_and_categories_stay_independent() {
        let mut stats = Statistics::default();
        stats.set(StatisticCategory::Used, 1, 10);
        stats.set(StatisticCategory::Used, 1, 4);
        assert_eq!(stats.get(StatisticCategory::Used, 1), 4);
        assert_eq!(stats.get(StatisticCategory::Broken, 1), 0);
    }

    #[test]
    fn custom_statistics_land_in_the_custom_category() {
        let mut stats = Statistics::default();
        stats.increment_custom(CustomStatistic::Jump, 3);
        assert_eq!(
            stats.get(StatisticCategory::Custom, CustomStatistic::Jump as i32),
            3
        );
    }
}
