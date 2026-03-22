use serde::Deserialize;

use crate::random::{RandomGenerator, RandomImpl};

/// Represents a weighted selection pool for random sampling.
#[derive(Deserialize, Clone, Debug)]
pub struct Pool;

impl Pool {
    /// Selects an element from a weighted distribution using the provided random generator.
    ///
    /// # Arguments
    /// * `distribution` – A slice of weighted entries to select from.
    /// * `random` – The random number generator to use for selection.
    ///
    /// # Returns
    /// An `Option<E>` representing the selected element, or `None` if the distribution is empty.
    pub fn get<E: Clone>(distribution: &[Weighted<E>], random: &mut RandomGenerator) -> Option<E> {
        let mut total_weight = 0;
        for dist in distribution {
            total_weight += dist.weight;
        }
        let mut index = random.next_bounded_i32(total_weight);
        if total_weight < 64 {
            return Some(FlattenedContent::get(index, distribution, total_weight));
        }
        // WrappedContent
        for dist in distribution {
            index -= dist.weight;
            if index >= 0 {
                continue;
            }
            return Some(dist.data.clone());
        }
        None
    }
}

/// A weighted entry in a pool.
#[derive(Deserialize, Clone, Debug)]
pub struct Weighted<E> {
    /// The element stored in this entry.
    pub data: E,
    /// The weight of this entry for random selection.
    pub weight: i32,
}

/// Helper struct for flattened weighted selection.
struct FlattenedContent;

impl FlattenedContent {
    /// Selects an element from a flattened representation of the weighted entries.
    ///
    /// # Arguments
    /// * `index` – The target index to select.
    /// * `entries` – The weighted entries to flatten.
    /// * `total_weight` – The total weight of all entries.
    ///
    /// # Returns
    /// The element corresponding to the given index.
    pub fn get<E: Clone>(index: i32, entries: &[Weighted<E>], total_weight: i32) -> E {
        let mut final_entries = Vec::with_capacity(total_weight as usize);
        let mut cur_index = 0;
        for entry in entries {
            let weight = entry.weight;
            for i in cur_index..cur_index + weight {
                final_entries.insert(i as usize, entry.data.clone());
            }
            cur_index += weight;
        }
        final_entries[index as usize].clone()
    }
}
