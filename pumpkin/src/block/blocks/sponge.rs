use crate::block::pumpkin_block::PumpkinBlock;
use crate::world::World;
use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_data::fluid::Fluid;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::world::WorldEvent;
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
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

    // Helper function to remove waterlogged property (not async)
    fn remove_waterlogged_property(block: &Block, state_id: BlockStateId) -> Option<BlockStateId> {
        if let Some(properties) = block.properties(state_id) {
            let original_props = properties.to_props();
            let is_waterlogged = original_props
                .iter()
                .any(|(key, value)| key == "waterlogged" && value == "true");

            if is_waterlogged {
                let mut props_vec: Vec<(&str, String)> = Vec::with_capacity(original_props.len());

                for (key, value) in &original_props {
                    if key == "waterlogged" {
                        props_vec.push((key.as_str(), "false".to_string()));
                    } else {
                        props_vec.push((key.as_str(), value.clone()));
                    }
                }

                // Convert to the format expected by from_properties
                let props_refs: Vec<(&str, &str)> =
                    props_vec.iter().map(|(k, v)| (*k, v.as_str())).collect();

                if let Some(new_props) = block.from_properties(props_refs) {
                    return Some(new_props.to_state_id(block));
                }
            }
        }
        None
    }

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

            if Self::is_water_block(world, &current_pos).await {
                water_blocks.push(current_pos);
            }

            // Add adjacent blocks to the queue
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

        water_blocks.sort_by_key(|pos| {
            let dx = (pos.0.x - sponge_pos.0.x).abs();
            let dy = (pos.0.y - sponge_pos.0.y).abs();
            let dz = (pos.0.z - sponge_pos.0.z).abs();
            dx + dy + dz
        });

        if !water_blocks.is_empty() {
            // Remove water blocks starting from closest to sponge
            for water_pos in water_blocks {
                Self::remove_water_at_position(world, &water_pos).await;
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
    async fn remove_water_at_position(world: &Arc<World>, pos: &BlockPos) {
        let block = world.get_block(pos).await;
        let state_id = world.get_block_state_id(pos).await;

        // Try to remove waterlogged property first
        if let Some(new_state_id) = Self::remove_waterlogged_property(&block, state_id) {
            world
                .set_block_state(pos, new_state_id, BlockFlags::NOTIFY_LISTENERS)
                .await;
            return;
        }

        world
            .set_block_state(
                pos,
                Block::AIR.default_state_id,
                BlockFlags::NOTIFY_LISTENERS,
            )
            .await;
    }

    fn is_waterlogged(block: &Block, state_id: BlockStateId) -> bool {
        block.properties(state_id).is_some_and(|properties| {
            properties
                .to_props()
                .iter()
                .any(|(key, value)| key == "waterlogged" && value == "true")
        })
    }

    async fn is_water_block(world: &Arc<World>, pos: &BlockPos) -> bool {
        let block = world.get_block(pos).await;
        let state_id = world.get_block_state_id(pos).await;

        if block == Block::WATER {
            return true;
        }

        if let Some(fluid) = Fluid::from_state_id(state_id) {
            if fluid.name.contains("water") {
                return true;
            }
        }
        Self::is_waterlogged(&block, state_id)
    }

    async fn play_absorption_sound(&self, world: &Arc<World>, pos: BlockPos) {
        world
            .play_block_sound(Sound::BlockSpongeAbsorb, SoundCategory::Blocks, pos)
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

        world
            .play_block_sound(Sound::BlockFireExtinguish, SoundCategory::Blocks, pos)
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

                    if block == Block::FIRE || block == Block::LAVA {
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }

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
