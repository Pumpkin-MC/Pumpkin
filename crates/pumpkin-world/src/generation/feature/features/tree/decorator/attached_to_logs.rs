use pumpkin_data::BlockDirection;
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};

use crate::generation::proto_chunk::GenerationCache;
use crate::{generation::block_state_provider::BlockStateProvider, world::WorldPortalExt};

pub struct AttachedToLogsTreeDecorator {
    pub probability: f32,
    pub block_provider: BlockStateProvider,
    pub directions: Vec<BlockDirection>,
}

impl AttachedToLogsTreeDecorator {
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        block_registry: &dyn WorldPortalExt,
        random: &mut RandomGenerator,
        log_positions: &[BlockPos],
    ) {
        let mut shuffled = vanilla_hash_set_order(log_positions);
        for i in (1..shuffled.len()).rev() {
            let j = random.next_bounded_i32(i as i32 + 1) as usize;
            shuffled.swap(i, j);
        }
        for pos in shuffled {
            let direction =
                self.directions[random.next_bounded_i32(self.directions.len() as i32) as usize];
            let pos = pos.offset(direction.to_offset());
            if random.next_f32() > self.probability
                || !GenerationCache::get_block_state(chunk, &pos.0)
                    .to_state()
                    .is_air()
            {
                continue;
            }
            chunk.set_block_state(
                &pos.0,
                self.block_provider.get(random, pos, chunk, block_registry),
            );
        }
    }
}

/// Reproduces the iteration order of Java's default `HashSet<BlockPos>`.
/// Vanilla builds the decorator context from that set before sorting by Y.
fn vanilla_hash_set_order(positions: &[BlockPos]) -> Vec<BlockPos> {
    let mut capacity = 16usize;
    let mut threshold = capacity * 3 / 4;
    let mut buckets = vec![Vec::new(); capacity];

    for &pos in positions {
        if buckets.iter().map(Vec::len).sum::<usize>() + 1 > threshold {
            capacity *= 2;
            threshold = capacity * 3 / 4;
            let mut resized = vec![Vec::new(); capacity];
            for bucket in buckets {
                for entry in bucket {
                    let index = java_hash(entry) as usize & (capacity - 1);
                    resized[index].push(entry);
                }
            }
            buckets = resized;
        }

        let index = java_hash(pos) as usize & (capacity - 1);
        buckets[index].push(pos);
    }

    buckets.into_iter().flatten().collect()
}

const fn java_hash(pos: BlockPos) -> u32 {
    let hash = pos
        .0
        .y
        .wrapping_add(pos.0.z.wrapping_mul(31))
        .wrapping_mul(31)
        .wrapping_add(pos.0.x);
    hash as u32 ^ (hash as u32 >> 16)
}
