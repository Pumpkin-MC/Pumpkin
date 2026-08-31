use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::vector3::Vector3;
use std::collections::{HashSet, VecDeque};
use std::sync::Arc;

use crate::block::{BlockBehaviour, OnNeighborUpdateArgs, PlacedArgs};
use pumpkin_data::dimension::Dimension;
use pumpkin_data::particle::Particle;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::{Block, BlockStateId};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;

#[pumpkin_block("minecraft:sponge")]
pub struct SpongeBlock;

impl SpongeBlock {
    pub fn absorb_water(world: &Arc<crate::world::World>, position: &BlockPos) -> bool {
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
                let (block, _state) = world.get_block_and_state(&next_pos);

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
            let mut event =
                crate::plugin::api::events::block::sponge_absorb::SpongeAbsorbEvent::new(*position);
            if let Some(server) = world.server.upgrade() {
                server.plugin_manager.fire_blocking(&server, &mut event);
            }
            if event.cancelled {
                return false;
            }

            for water_pos in &water_blocks {
                world.set_block_state(water_pos, BlockStateId::AIR, BlockFlags::NOTIFY_ALL);
            }
            world.set_block_state(
                position,
                Block::WET_SPONGE.default_state.id,
                BlockFlags::NOTIFY_ALL,
            );

            world.play_block_sound(Sound::BlockSpongeAbsorb, SoundCategory::Blocks, *position);

            true
        }
    }
}

impl BlockBehaviour for SpongeBlock {
    fn placed(&self, args: PlacedArgs<'_>) {
        // Attempt to absorb water on placement
        Self::absorb_water(args.world, args.position);
    }

    fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        // If a neighboring block changed and it's water, attempt to absorb.
        if args.source_block.id == Block::WATER.id {
            Self::absorb_water(args.world, args.position);
        }
    }
}

#[pumpkin_block("minecraft:wet_sponge")]
pub struct WetSpongeBlock;

impl BlockBehaviour for WetSpongeBlock {
    fn placed(&self, args: PlacedArgs<'_>) {
        {
            // Check if placed in Nether, if so, dry out
            if args.world.dimension == Dimension::THE_NETHER {
                args.world.set_block_state(
                    args.position,
                    Block::SPONGE.default_state.id,
                    BlockFlags::NOTIFY_ALL,
                );

                // Play dry sound and spawn smoke particles
                args.world.play_block_sound(
                    Sound::BlockWetSpongeDries,
                    SoundCategory::Blocks,
                    *args.position,
                );

                args.world.spawn_particle(
                    Vector3::new(
                        args.position.0.x as f64 + 0.5,
                        args.position.0.y as f64 + 1.0,
                        args.position.0.z as f64 + 0.5,
                    ),
                    Vector3::new(0.25, 0.0, 0.25),
                    0.01,
                    16,
                    Particle::Cloud,
                );
            }
        }
    }
}
