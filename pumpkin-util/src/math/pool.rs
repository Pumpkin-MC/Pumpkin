use serde::Deserialize;

use crate::random::{RandomGenerator, RandomImpl};

#[derive(Deserialize, Clone, Debug)]
pub struct Pool;

impl Pool {
    pub fn get<E: Clone>(
        &self,
        distribution: &[Weighted<E>],
        random: &mut RandomGenerator,
    ) -> Option<E> {
        let mut total_weight = 0;
        for dist in distribution {
            total_weight += dist.weight;
        }
        let index = random.next_bounded_i32(total_weight);
        if total_weight < 64 {
            return Some(FlattenedContent::get(index, distribution, total_weight));
        } else {
            // WrappedContent
            for dist in distribution {
                if index - dist.weight >= 0 {
                    continue;
                }
                return Some(dist.data.clone());
            }
        }
        None
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct Weighted<E> {
    pub data: E,
    pub weight: i32,
}

struct FlattenedContent;

impl FlattenedContent {
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
