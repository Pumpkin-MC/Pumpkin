use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::vector3::Vector3;
use std::collections::{HashSet, VecDeque};
use std::sync::Arc;

use crate::block::{BlockBehaviour, BlockFuture, OnNeighborUpdateArgs, PlacedArgs};
use pumpkin_data::Block;
use pumpkin_data::dimension::Dimension;
use pumpkin_data::particle::Particle;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;

#[pumpkin_block("minecraft:sponge")]
pub struct SpongeBlock;

impl SpongeBlock {
    pub async fn absorb_water(world: &Arc<crate::world::World>, position: &BlockPos) -> bool {
        let mut water_blocks = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        // Start from the sponge position
        queue.push_back(*position);
        visited.insert(*position);

        while let Some(current_pos) = queue.pop_front() {
            for direction in &[
                (1, 0, 0),
                (-1, 0, 0),
                (0, 1, 0),
                (0, -1, 0),
                (0, 0, 1),
                (0, 0, -1),
            ] {
                let next_pos = BlockPos::new(
                    current_pos.0.x + direction.0,
                    current_pos.0.y + direction.1,
                    current_pos.0.z + direction.2,
                );

                if visited.contains(&next_pos) {
                    continue;
                }

                let taxicab_dist = (next_pos.0.x - position.0.x).abs()
                    + (next_pos.0.y - position.0.y).abs()
                    + (next_pos.0.z - position.0.z).abs();

                // Wiki standard: distance 7, max 118 blocks
                if taxicab_dist > 7 || water_blocks.len() >= 118 {
                    continue;
                }

                visited.insert(next_pos);
                let (block, _state) = world.get_block_and_state(&next_pos).await;

                // Only add to queue if it's water.
                // This prevents "jumping" through air or solid blocks.
                if block.id == Block::WATER.id {
                    water_blocks.push(next_pos);
                    queue.push_back(next_pos);
                }
            }
        }

        if water_blocks.is_empty() {
            false
        } else {
            for water_pos in &water_blocks {
                world
                    .set_block_state(water_pos, 0, BlockFlags::NOTIFY_ALL)
                    .await;
            }
            world
                .set_block_state(
                    position,
                    Block::WET_SPONGE.default_state.id,
                    BlockFlags::NOTIFY_ALL,
                )
                .await;

            world
                .play_block_sound(Sound::BlockSpongeAbsorb, SoundCategory::Blocks, *position)
                .await;

            true
        }
    }
}

impl BlockBehaviour for SpongeBlock {
    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            // Attempt to absorb water on placement
            Self::absorb_water(args.world, args.position).await;
        })
    }

    fn on_neighbor_update<'a>(&'a self, args: OnNeighborUpdateArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            // If a neighboring block changed and it's water, attempt to absorb.
            if args.source_block.id == Block::WATER.id {
                Self::absorb_water(args.world, args.position).await;
            }
        })
    }
}

#[pumpkin_block("minecraft:wet_sponge")]
pub struct WetSpongeBlock;

impl BlockBehaviour for WetSpongeBlock {
    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            // Check if placed in Nether, if so, dry out
            if args.world.dimension == Dimension::THE_NETHER {
                args.world
                    .set_block_state(
                        args.position,
                        Block::SPONGE.default_state.id,
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;

                // Play dry sound and spawn smoke particles
                args.world
                    .play_block_sound(
                        Sound::BlockWetSpongeDries,
                        SoundCategory::Blocks,
                        *args.position,
                    )
                    .await;

                args.world
                    .spawn_particle(
                        Vector3::new(
                            args.position.0.x as f64 + 0.5,
                            args.position.0.y as f64 + 1.0,
                            args.position.0.z as f64 + 0.5,
                        ),
                        Vector3::new(0.25, 0.0, 0.25),
                        0.01,
                        16,
                        Particle::Cloud,
                    )
                    .await;
            }
        })
    }
}
