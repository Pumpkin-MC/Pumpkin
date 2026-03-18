use pumpkin_data::{Block, block_properties::is_air};
use pumpkin_util::{
    math::{int_provider::IntProvider, position::BlockPos},
    random::{RandomGenerator, RandomImpl},
};

use crate::{generation::proto_chunk::GenerationCache, world::BlockRegistryExt};

const LAVA_SEA_LEVEL: i32 = 32; // TODO: use getSeaLevel() instead of hardcoding this
const CLUSTERED_REACH: i32 = 5;
const UNCLUSTERED_REACH: i32 = 8;
const CLUSTERED_SIZE: i32 = 50;
const UNCLUSTERED_SIZE: i32 = 15;

pub struct BasaltColumnsFeature {
    pub height: IntProvider,
    pub reach: IntProvider,
}

impl BasaltColumnsFeature {
    #[expect(clippy::too_many_arguments)]
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        _block_registry: &dyn BlockRegistryExt,
        _min_y: i8,
        _height_limit: u16,
        _feature: &str,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        if !can_place_at(chunk, pos) {
            return false;
        }

        let column_height = self.height.get(random);
        let clustered = random.next_f32() < 0.9;
        let spread = column_height.min(if clustered {
            CLUSTERED_REACH
        } else {
            UNCLUSTERED_REACH
        });
        let count = if clustered {
            CLUSTERED_SIZE
        } else {
            UNCLUSTERED_SIZE
        };
        let mut placed = false;

        for _ in 0..count {
            let x = random.next_inbetween_i32(pos.0.x - spread, pos.0.x + spread);
            let z = random.next_inbetween_i32(pos.0.z - spread, pos.0.z + spread);
            let iter_pos = BlockPos::new(x, pos.0.y, z);
            let blocks_to_place_y = column_height - iter_pos.manhattan_distance(pos);
            if blocks_to_place_y >= 0 {
                let inner_reach = self.reach.get(random);
                placed |= place_column(chunk, iter_pos, blocks_to_place_y, inner_reach);
            }
        }

        placed
    }
}

fn place_column<T: GenerationCache>(
    chunk: &mut T,
    origin: BlockPos,
    column_height: i32,
    reach: i32,
) -> bool {
    let mut placed_any = false;

    for x in (origin.0.x - reach)..=(origin.0.x + reach) {
        for z in (origin.0.z - reach)..=(origin.0.z + reach) {
            let pos = BlockPos::new(x, origin.0.y, z);
            let step_limit = pos.manhattan_distance(origin);

            let column_pos = if is_air_or_lava_ocean(chunk, pos) {
                find_surface(chunk, pos, step_limit)
            } else {
                find_air(chunk, pos, step_limit)
            };

            if let Some(mut cursor) = column_pos {
                let mut blocks_y = column_height - step_limit / 2;
                while blocks_y >= 0 {
                    if is_air_or_lava_ocean(chunk, cursor) {
                        chunk.set_block_state(&cursor.0, Block::BASALT.default_state);
                        cursor = cursor.up();
                        placed_any = true;
                    } else {
                        let block_id =
                            GenerationCache::get_block_state(chunk, &cursor.0).to_block_id();
                        if block_id != Block::BASALT.id {
                            break;
                        }
                        cursor = cursor.up();
                    }
                    blocks_y -= 1;
                }
            }
        }
    }

    placed_any
}

fn find_surface<T: GenerationCache>(chunk: &T, pos: BlockPos, mut limit: i32) -> Option<BlockPos> {
    let mut cursor = pos;
    while cursor.0.y > chunk.bottom_y() as i32 + 1 && limit > 0 {
        limit -= 1;
        if can_place_at(chunk, cursor) {
            return Some(cursor);
        }
        cursor = cursor.down();
    }
    None
}

fn can_place_at<T: GenerationCache>(chunk: &T, pos: BlockPos) -> bool {
    if !is_air_or_lava_ocean(chunk, pos) {
        return false;
    }
    let below = GenerationCache::get_block_state(chunk, &pos.down().0);
    !is_air(below.0) && !is_cannot_place_on(below.to_block_id())
}

fn find_air<T: GenerationCache>(chunk: &T, pos: BlockPos, mut limit: i32) -> Option<BlockPos> {
    let mut cursor = pos;
    while cursor.0.y < chunk.top_y() as i32 && limit > 0 {
        limit -= 1;
        let state = GenerationCache::get_block_state(chunk, &cursor.0);
        if is_cannot_place_on(state.to_block_id()) {
            return None;
        }
        if is_air(state.0) {
            return Some(cursor);
        }
        cursor = cursor.up();
    }
    None
}

fn is_air_or_lava_ocean<T: GenerationCache>(chunk: &T, pos: BlockPos) -> bool {
    let state = GenerationCache::get_block_state(chunk, &pos.0);
    is_air(state.0) || (state.to_block_id() == Block::LAVA.id && pos.0.y <= LAVA_SEA_LEVEL)
}

fn is_cannot_place_on(block_id: u16) -> bool {
    block_id == Block::LAVA.id
        || block_id == Block::BEDROCK.id
        || block_id == Block::MAGMA_BLOCK.id
        || block_id == Block::SOUL_SAND.id
        || block_id == Block::NETHER_BRICKS.id
        || block_id == Block::NETHER_BRICK_FENCE.id
        || block_id == Block::NETHER_BRICK_STAIRS.id
        || block_id == Block::NETHER_WART.id
        || block_id == Block::CHEST.id
        || block_id == Block::SPAWNER.id
}
