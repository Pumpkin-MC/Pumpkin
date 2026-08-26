use std::sync::Arc;

use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, tag};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;
use rand::RngExt;

use crate::block::{BlockBehaviour, BonemealArgs, RandomTickArgs};
use crate::world::World;

#[pumpkin_block_from_tag("minecraft:nylium")]
pub struct NyliumBlock;

impl NyliumBlock {
    fn can_be_nylium(world: &World, pos: &BlockPos) -> bool {
        let above_pos = pos.up();
        if !world.is_loaded(&above_pos) {
            return true;
        }
        let above_state = world.get_block_state(&above_pos);
        above_state.opacity < 15
    }
}

impl BlockBehaviour for NyliumBlock {
    fn random_tick(&self, args: RandomTickArgs<'_>) {
        if !Self::can_be_nylium(args.world, args.position) {
            args.world.set_block_state(
                args.position,
                Block::NETHERRACK.default_state.id,
                BlockFlags::NOTIFY_ALL,
            );
        }
    }

    fn is_valid_bonemeal_target(&self, args: BonemealArgs<'_>) -> bool {
        let above = args.position.up();
        args.world.is_in_height_limit(above.0.y)
            && args.world.is_loaded(&above)
            && args.world.get_block_state(&above).is_air()
    }

    fn is_bonemeal_success(&self, _args: BonemealArgs<'_>) -> bool {
        true
    }

    fn perform_bonemeal(&self, args: BonemealArgs<'_>) {
        let world = args.world;
        let block = args.block;
        let above_pos = args.position.up();

        if block == &Block::CRIMSON_NYLIUM {
            place_crimson_vegetation(world, &above_pos);
        } else if block == &Block::WARPED_NYLIUM {
            place_warped_vegetation(world, &above_pos);
            place_nether_sprouts(world, &above_pos);
            if rand::rng().random_range(0..8) == 0 {
                place_twisting_vines(world, &above_pos);
            }
        }
    }
}

fn place_crimson_vegetation(world: &Arc<World>, origin: &BlockPos) {
    for _ in 0..9 {
        let dx = rand::rng().random_range(0..3) - rand::rng().random_range(0..3);
        let dy = rand::rng().random_range(0..1) - rand::rng().random_range(0..1);
        let dz = rand::rng().random_range(0..3) - rand::rng().random_range(0..3);
        let target_pos = origin.add(dx, dy, dz);

        if !world.is_in_height_limit(target_pos.0.y)
            || !world.is_loaded(&target_pos)
            || !world.get_block_state(&target_pos).is_air()
        {
            continue;
        }

        let below_pos = target_pos.down();
        if !world.is_loaded(&below_pos) {
            continue;
        }
        let below_block = world.get_block(&below_pos);
        if !below_block.has_tag(&tag::Block::MINECRAFT_NYLIUM) {
            continue;
        }

        let roll = rand::rng().random_range(0..99);
        let placed_block = if roll < 87 {
            &Block::CRIMSON_ROOTS
        } else if roll < 98 {
            &Block::CRIMSON_FUNGUS
        } else {
            &Block::WARPED_FUNGUS
        };

        let state = placed_block.default_state;
        if !world.block_registry.can_place_at(
            None,
            Some(world),
            world.as_ref(),
            None,
            placed_block,
            state,
            &target_pos,
            None,
            None,
        ) {
            continue;
        }

        world.set_block_state(&target_pos, state.id, BlockFlags::NOTIFY_ALL);
    }
}

fn place_warped_vegetation(world: &Arc<World>, origin: &BlockPos) {
    for _ in 0..9 {
        let dx = rand::rng().random_range(0..3) - rand::rng().random_range(0..3);
        let dy = rand::rng().random_range(0..1) - rand::rng().random_range(0..1);
        let dz = rand::rng().random_range(0..3) - rand::rng().random_range(0..3);
        let target_pos = origin.add(dx, dy, dz);

        if !world.is_in_height_limit(target_pos.0.y)
            || !world.is_loaded(&target_pos)
            || !world.get_block_state(&target_pos).is_air()
        {
            continue;
        }

        let below_pos = target_pos.down();
        if !world.is_loaded(&below_pos) {
            continue;
        }
        let below_block = world.get_block(&below_pos);
        if !below_block.has_tag(&tag::Block::MINECRAFT_NYLIUM) {
            continue;
        }

        let roll = rand::rng().random_range(0..100);
        let placed_block = if roll < 85 {
            &Block::WARPED_ROOTS
        } else if roll < 86 {
            &Block::CRIMSON_ROOTS
        } else if roll < 99 {
            &Block::WARPED_FUNGUS
        } else {
            &Block::CRIMSON_FUNGUS
        };

        let state = placed_block.default_state;
        if !world.block_registry.can_place_at(
            None,
            Some(world),
            world.as_ref(),
            None,
            placed_block,
            state,
            &target_pos,
            None,
            None,
        ) {
            continue;
        }

        world.set_block_state(&target_pos, state.id, BlockFlags::NOTIFY_ALL);
    }
}

fn place_nether_sprouts(world: &Arc<World>, origin: &BlockPos) {
    for _ in 0..9 {
        let dx = rand::rng().random_range(0..3) - rand::rng().random_range(0..3);
        let dy = rand::rng().random_range(0..1) - rand::rng().random_range(0..1);
        let dz = rand::rng().random_range(0..3) - rand::rng().random_range(0..3);
        let target_pos = origin.add(dx, dy, dz);

        if !world.is_in_height_limit(target_pos.0.y)
            || !world.is_loaded(&target_pos)
            || !world.get_block_state(&target_pos).is_air()
        {
            continue;
        }

        let below_pos = target_pos.down();
        if !world.is_loaded(&below_pos) {
            continue;
        }
        let below_block = world.get_block(&below_pos);
        if !below_block.has_tag(&tag::Block::MINECRAFT_NYLIUM) {
            continue;
        }

        let placed_block = &Block::NETHER_SPROUTS;
        let state = placed_block.default_state;
        if !world.block_registry.can_place_at(
            None,
            Some(world),
            world.as_ref(),
            None,
            placed_block,
            state,
            &target_pos,
            None,
            None,
        ) {
            continue;
        }

        world.set_block_state(&target_pos, state.id, BlockFlags::NOTIFY_ALL);
    }
}

fn place_twisting_vines(world: &Arc<World>, origin: &BlockPos) {
    for _ in 0..9 {
        let dx = rand::rng().random_range(0..3) - rand::rng().random_range(0..3);
        let dy = rand::rng().random_range(0..1) - rand::rng().random_range(0..1);
        let dz = rand::rng().random_range(0..3) - rand::rng().random_range(0..3);
        let mut target_pos = origin.add(dx, dy, dz);

        if !find_twisting_vines_target_y(world, &mut target_pos) {
            continue;
        }

        let mut height = rand::rng().random_range(0..2) + 1;
        if rand::rng().random_range(0..6) == 0 {
            height *= 2;
        }
        if rand::rng().random_range(0..10) == 0 {
            height = 1;
        }

        let mut current_pos = target_pos;
        for i in 0..height {
            if !world.is_in_height_limit(current_pos.0.y)
                || !world.is_loaded(&current_pos)
                || !world.get_block_state(&current_pos).is_air()
            {
                break;
            }

            let is_top = i == height - 1
                || !world.is_in_height_limit(current_pos.up().0.y)
                || !world.is_loaded(&current_pos.up())
                || !world.get_block_state(&current_pos.up()).is_air();

            if is_top {
                world.set_block_state(
                    &current_pos,
                    Block::TWISTING_VINES.default_state.id,
                    BlockFlags::NOTIFY_ALL,
                );
                break;
            }
            world.set_block_state(
                &current_pos,
                Block::TWISTING_VINES_PLANT.default_state.id,
                BlockFlags::NOTIFY_ALL,
            );
            current_pos = current_pos.up();
        }
    }
}

fn find_twisting_vines_target_y(world: &World, pos: &mut BlockPos) -> bool {
    let mut current = *pos;
    for _ in 0..1 {
        if world.is_loaded(&current) && world.get_block_state(&current).is_air() {
            let below = current.down();
            if world.is_loaded(&below) {
                let block_below = world.get_block(&below);
                if block_below == &Block::WARPED_NYLIUM
                    || block_below == &Block::WARPED_WART_BLOCK
                    || block_below == &Block::TWISTING_VINES
                    || block_below == &Block::TWISTING_VINES_PLANT
                {
                    *pos = current;
                    return true;
                }
            }
        }
        current = current.down();
    }
    false
}
