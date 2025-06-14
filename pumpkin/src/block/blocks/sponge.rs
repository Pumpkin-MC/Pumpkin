use crate::block::pumpkin_block::PumpkinBlock;
use crate::world::World;
use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::world::WorldEvent;
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::{BlockStateId, world::BlockFlags};
use std::sync::Arc;

// Sponge block that can absorb water
#[pumpkin_block("minecraft:sponge")]
pub struct SpongeBlock;

#[async_trait]
impl PumpkinBlock for SpongeBlock {
    async fn placed(
        &self,
        world: &Arc<World>,
        _block: &Block,
        _state_id: BlockStateId,
        block_pos: &BlockPos,
        _old_state_id: BlockStateId,
        _notify: bool,
    ) {
        // When a dry sponge is placed, check if it should absorb water
        if let Err(e) = self.absorb_water(world, *block_pos).await {
            log::warn!("Failed to absorb water when placing sponge at {block_pos:?}: {e}");
        }
    }
}

impl SpongeBlock {
    const ABSORPTION_RADIUS: i32 = 6;
    const MAX_ABSORBED_BLOCKS: usize = 65;

    // Absorbs water in a radius around the sponge block
    pub async fn absorb_water(
        &self,
        world: &Arc<World>,
        sponge_pos: BlockPos,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut water_blocks = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();

        queue.push_back(sponge_pos);
        visited.insert(sponge_pos);

        while let Some(current_pos) = queue.pop_front() {
            if water_blocks.len() >= Self::MAX_ABSORBED_BLOCKS {
                break;
            }

            let dx = (current_pos.0.x - sponge_pos.0.x).abs();
            let dy = (current_pos.0.y - sponge_pos.0.y).abs();
            let dz = (current_pos.0.z - sponge_pos.0.z).abs();

            if dx > Self::ABSORPTION_RADIUS
                || dy > Self::ABSORPTION_RADIUS
                || dz > Self::ABSORPTION_RADIUS
            {
                continue;
            }
            let block = world.get_block(&current_pos).await;
            if Self::is_water_block(&block) {
                water_blocks.push(current_pos);
            }

            for dx in -1..=1 {
                for dy in -1..=1 {
                    for dz in -1..=1 {
                        if dx == 0 && dy == 0 && dz == 0 {
                            continue;
                        }

                        let adjacent_pos = BlockPos::new(
                            current_pos.0.x + dx,
                            current_pos.0.y + dy,
                            current_pos.0.z + dz,
                        );

                        if !visited.contains(&adjacent_pos) {
                            visited.insert(adjacent_pos);
                            queue.push_back(adjacent_pos);
                        }
                    }
                }
            }
        }
        if !water_blocks.is_empty() {
            for water_pos in water_blocks {
                world
                    .set_block_state(
                        &water_pos,
                        Block::AIR.default_state_id,
                        BlockFlags::NOTIFY_LISTENERS,
                    )
                    .await;
                world.update_neighbors(&water_pos, None).await;
            }

            world
                .set_block_state(
                    &sponge_pos,
                    Block::WET_SPONGE.default_state_id,
                    BlockFlags::NOTIFY_LISTENERS,
                )
                .await;
            self.play_absorption_sound(world, sponge_pos).await;
        }

        Ok(())
    }
    // Checks if a block represents water
    fn is_water_block(block: &Block) -> bool {
        block == &Block::WATER
    }

    // Plays the sponge absorption sound effect
    async fn play_absorption_sound(&self, world: &Arc<World>, pos: BlockPos) {
        let sound_pos = Vector3::new(
            f64::from(pos.0.x) + 0.5,
            f64::from(pos.0.y) + 0.5,
            f64::from(pos.0.z) + 0.5,
        );

        world
            .play_sound(Sound::BlockSpongeAbsorb, SoundCategory::Blocks, &sound_pos)
            .await;
    }
}

// Wet sponge block that can be dried
#[pumpkin_block("minecraft:wet_sponge")]
pub struct WetSpongeBlock;

#[async_trait]
impl PumpkinBlock for WetSpongeBlock {
    async fn placed(
        &self,
        world: &Arc<World>,
        _block: &Block,
        _state_id: BlockStateId,
        block_pos: &BlockPos,
        _old_state_id: BlockStateId,
        _notify: bool,
    ) {
        // When a wet sponge is placed, check if it should immediately dry out
        if let Err(e) = self.tick(world, *block_pos).await {
            log::warn!("Failed to check wet sponge drying conditions at {block_pos:?}: {e}");
        }
    }
}

// WetSpongeBlock implementation for drying the sponge
impl WetSpongeBlock {
    pub async fn dry_sponge(
        &self,
        world: &Arc<World>,
        pos: BlockPos,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        world
            .set_block_state(
                &pos,
                Block::SPONGE.default_state_id,
                BlockFlags::NOTIFY_LISTENERS,
            )
            .await;

        // Trigger the WET_SPONGE_DRIES_OUT world event
        world
            .sync_world_event(WorldEvent::WetSpongeDriesOut, pos, 0)
            .await;

        let sound_pos = Vector3::new(
            f64::from(pos.0.x) + 0.5,
            f64::from(pos.0.y) + 0.5,
            f64::from(pos.0.z) + 0.5,
        );

        world
            .play_sound(
                Sound::BlockFireExtinguish,
                SoundCategory::Blocks,
                &sound_pos,
            )
            .await;

        Ok(())
    }

    /// Check if this wet sponge should dry out due to environmental conditions
    pub async fn should_dry_out(
        &self,
        world: &Arc<World>,
        pos: BlockPos,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        // Check for fire nearby (within 2 blocks)
        for dx in -2..=2 {
            for dy in -2..=2 {
                for dz in -2..=2 {
                    let check_pos = BlockPos::new(pos.0.x + dx, pos.0.y + dy, pos.0.z + dz);
                    let block = world.get_block(&check_pos).await;

                    // Dry out if there's fire or lava nearby
                    if block == Block::FIRE || block == Block::LAVA {
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }

    /// Tick function to check if wet sponge should dry out
    pub async fn tick(
        &self,
        world: &Arc<World>,
        pos: BlockPos,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.should_dry_out(world, pos).await? {
            self.dry_sponge(world, pos).await?;
        }
        Ok(())
    }
}
