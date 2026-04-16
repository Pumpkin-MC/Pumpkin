use pumpkin_data::{
    Block, BlockDirection,
    block_properties::{
        BlockProperties, BrownMushroomBlockLikeProperties, ChorusFlowerLikeProperties, Integer0To5,
    },
    tag,
};
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};

use crate::generation::proto_chunk::GenerationCache;

pub struct ChorusPlantFeature {}

impl ChorusPlantFeature {
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        _min_y: i8,
        _height: u16,
        _feature: &str,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let origin = pos.0;

        // Must be an empty block placed on a supports_chorus_plant block (end_stone)
        if !chunk.is_air(&origin) {
            return false;
        }
        let below = pos.down().0;
        let below_id = GenerationCache::get_block_state(chunk, &below).to_block_id();
        if !tag::Block::MINECRAFT_SUPPORTS_CHORUS_PLANT
            .1
            .contains(&below_id)
        {
            return false;
        }

        // Place the base chorus plant with connections, then grow the tree
        set_chorus_plant(chunk, &pos);
        grow_tree_recursive(chunk, random, &pos, &pos, 8, 0);
        true
    }
}

/// Returns `true` if all four horizontal neighbors of `pos` are air,
/// optionally ignoring one direction (`ignore`).
fn all_neighbors_empty<T: GenerationCache>(
    chunk: &T,
    pos: &BlockPos,
    ignore: Option<BlockDirection>,
) -> bool {
    for dir in BlockDirection::horizontal() {
        if Some(dir) == ignore {
            continue;
        }
        if !chunk.is_air(&pos.offset(dir.to_offset()).0) {
            return false;
        }
    }
    true
}

/// Computes chorus-plant connection state at `pos` and writes it to the chunk.
fn set_chorus_plant<T: GenerationCache>(chunk: &mut T, pos: &BlockPos) {
    let down_id = GenerationCache::get_block_state(chunk, &pos.down().0).to_block_id();
    let up_id = GenerationCache::get_block_state(chunk, &pos.up().0).to_block_id();
    let north_id =
        GenerationCache::get_block_state(chunk, &pos.offset(BlockDirection::North.to_offset()).0)
            .to_block_id();
    let east_id =
        GenerationCache::get_block_state(chunk, &pos.offset(BlockDirection::East.to_offset()).0)
            .to_block_id();
    let south_id =
        GenerationCache::get_block_state(chunk, &pos.offset(BlockDirection::South.to_offset()).0)
            .to_block_id();
    let west_id =
        GenerationCache::get_block_state(chunk, &pos.offset(BlockDirection::West.to_offset()).0)
            .to_block_id();

    let plant_id = Block::CHORUS_PLANT.id;
    let flower_id = Block::CHORUS_FLOWER.id;
    let supports = tag::Block::MINECRAFT_SUPPORTS_CHORUS_PLANT.1;

    let props = BrownMushroomBlockLikeProperties {
        down: down_id == plant_id || down_id == flower_id || supports.contains(&down_id),
        up: up_id == plant_id || up_id == flower_id,
        north: north_id == plant_id || north_id == flower_id,
        east: east_id == plant_id || east_id == flower_id,
        south: south_id == plant_id || south_id == flower_id,
        west: west_id == plant_id || west_id == flower_id,
    };

    let state_id = props.to_state_id(&Block::CHORUS_PLANT);
    chunk.set_block_state(&pos.0, pumpkin_data::BlockState::from_id(state_id));
}

/// Places a dead chorus flower (age 5) at `pos`.
fn place_dead_flower<T: GenerationCache>(chunk: &mut T, pos: &BlockPos) {
    let props = ChorusFlowerLikeProperties {
        age: Integer0To5::L5,
    };
    let state_id = props.to_state_id(&Block::CHORUS_FLOWER);
    chunk.set_block_state(&pos.0, pumpkin_data::BlockState::from_id(state_id));
}

/// Recursively grows the chorus plant tree upward and laterally.
fn grow_tree_recursive<T: GenerationCache>(
    chunk: &mut T,
    random: &mut RandomGenerator,
    current: &BlockPos,
    start_pos: &BlockPos,
    max_horizontal_spread: i32,
    depth: i32,
) {
    let mut height = random.next_bounded_i32(4) + 1;
    if depth == 0 {
        height += 1;
    }

    // Grow vertically: place chorus plant for `height` blocks above current
    for i in 0..height {
        let target = current.up_height(i + 1);
        if !all_neighbors_empty(chunk, &target, None) {
            return;
        }
        set_chorus_plant(chunk, &target);
        // Re-bake the block below so it gains the UP connection
        set_chorus_plant(chunk, &target.down());
    }

    let top = current.up_height(height);
    let mut placed_stem = false;

    if depth < 4 {
        let mut stems = random.next_bounded_i32(4);
        if depth == 0 {
            stems += 1;
        }

        // The 4 horizontal directions
        const HORIZ: [BlockDirection; 4] = BlockDirection::horizontal();

        for _ in 0..stems {
            // Pick a random horizontal direction
            let dir = HORIZ[random.next_bounded_i32(4) as usize];
            let target = top.offset(dir.to_offset());
            let target_below = target.down();

            let dx = (target.0.x - start_pos.0.x).abs();
            let dz = (target.0.z - start_pos.0.z).abs();

            if dx < max_horizontal_spread
                && dz < max_horizontal_spread
                && chunk.is_air(&target.0)
                && chunk.is_air(&target_below.0)
                && all_neighbors_empty(chunk, &target, Some(dir.opposite()))
            {
                placed_stem = true;
                set_chorus_plant(chunk, &target);
                // Re-bake the stem apex so it gains the connection face toward the new branch.
                set_chorus_plant(chunk, &top);
                grow_tree_recursive(
                    chunk,
                    random,
                    &target,
                    start_pos,
                    max_horizontal_spread,
                    depth + 1,
                );
            }
        }
    }

    if !placed_stem {
        // Cap the stem with a dead chorus flower
        place_dead_flower(chunk, &top);
    }
}
