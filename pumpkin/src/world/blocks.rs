use crate::block::entities::{BlockEntity, block_entity_from_nbt};
use crate::world::World;
use pumpkin_config::lighting::LightingEngineConfig;
use pumpkin_data::chunk::Biome;
use pumpkin_data::fluid::{Fluid, FluidState};
use pumpkin_data::{Block, BlockState, BlockStateId};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::chunk::io::Dirtiable;
use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;

impl World {
    pub async fn get_block_state_id_async(&self, position: &BlockPos) -> BlockStateId {
        if !self.is_in_build_limit(*position) {
            return Block::AIR.default_state.id;
        }

        let (chunk_coordinate, relative) = position.chunk_and_chunk_relative_position();
        self.level
            .get_or_fetch_chunk(chunk_coordinate, |chunk| {
                chunk
                    .section
                    .get_block_absolute_y(relative.x as usize, relative.y, relative.z as usize)
                    .unwrap_or(Block::AIR.default_state.id)
            })
            .await
    }

    pub async fn get_block_state_async(&self, position: &BlockPos) -> &'static BlockState {
        let id = self.get_block_state_id_async(position).await;
        BlockState::from_id(id)
    }

    /// Vanilla `Level.calculateAmbientDarkness` / sky darken (0–11).
    ///
    /// Used by monster spawn light tests so open sky is dark at night.
    #[must_use]
    pub fn calculate_sky_darken(time_of_day: i64, rain_level: f32, thunder_level: f32) -> u8 {
        let d = 1.0 - f64::from(rain_level) * 5.0 / 16.0;
        let e = 1.0 - f64::from(thunder_level) * 5.0 / 16.0;
        // Sky angle from vanilla DimensionType.getSkyAngle
        let day_frac = {
            let t = (time_of_day as f64) / 24000.0 - 0.25;
            t - t.floor()
        };
        let angle = {
            let cos_part = 0.5 - (day_frac * std::f64::consts::PI).cos() / 2.0;
            (day_frac * 2.0 + cos_part) / 3.0
        };
        let f = {
            let cos = (angle * std::f64::consts::TAU).cos();
            0.5 + 2.0 * cos.clamp(-0.25, 0.25)
        };
        ((1.0 - f * d * e) * 11.0).round().clamp(0.0, 11.0) as u8
    }

    /// Effective light at a position after subtracting ambient sky darken
    /// (vanilla `Level.getLightLevel(pos, ambientDarkness)`).
    #[must_use]
    pub fn get_light_level_with_darken(&self, pos: &BlockPos, ambient_darkness: u8) -> u8 {
        let sky = self.get_sky_light_level(pos);
        let block = self.get_block_light_level(pos).unwrap_or(0);
        let sky_after = sky.saturating_sub(ambient_darkness);
        sky_after.max(block)
    }

    pub fn get_lighting_config(&self) -> LightingEngineConfig {
        self.server
            .upgrade()
            .map(|s| s.advanced_config.world.lighting)
            .unwrap_or_default()
    }

    /// Light for monster/general checks — includes sky darken (night/rain).
    pub fn get_max_local_raw_brightness(&self, pos: &BlockPos) -> u8 {
        let ambient = self.sky_darken.load(Relaxed);
        self.get_light_level_with_darken(pos, ambient)
    }

    /// Vanilla animal spawn light: raw block/sky max with ambient 0 (no sky darken).
    pub fn get_raw_brightness_no_darken(&self, pos: &BlockPos) -> u8 {
        self.get_light_level_with_darken(pos, 0)
    }

    pub fn get_block_light_level(&self, position: &BlockPos) -> Option<u8> {
        self.level
            .light_engine
            .get_block_light_level(&self.level, position)
    }

    pub fn get_sky_light_level(&self, position: &BlockPos) -> u8 {
        self.level
            .light_engine
            .get_sky_light_level(&self.level, position)
    }

    pub fn set_block_light_level(&self, position: &BlockPos, light_level: u8) {
        let _ = self
            .level
            .light_engine
            .set_block_light_level(&self.level, position, light_level);
    }

    pub fn set_sky_light_level(&self, position: &BlockPos, light_level: u8) {
        let _ = self
            .level
            .light_engine
            .set_sky_light_level(&self.level, position, light_level);
    }

    pub fn get_biome(&self, position: &BlockPos) -> &'static Biome {
        let chunk_pos = position.chunk_position();
        if let Some(chunk) = self.level.loaded_chunks.get(&chunk_pos) {
            let id = chunk
                .section
                .get_rough_biome_absolute_y(
                    (position.0.x & 15) as usize,
                    position.0.y,
                    (position.0.z & 15) as usize,
                )
                .unwrap_or(0);
            Biome::from_id(id).unwrap_or(&Biome::PLAINS)
        } else {
            &Biome::PLAINS
        }
    }

    #[must_use]
    pub fn is_valid(dest: BlockPos) -> bool {
        Self::is_valid_horizontally(dest) && Self::is_valid_vertically(dest.0.y)
    }

    #[must_use]
    pub fn is_valid_horizontally(dest: BlockPos) -> bool {
        // Note: 30_000_000 is not valid, but -30_000_000 is.
        (-30_000_000..30_000_000).contains(&dest.0.x)
            && (-30_000_000..30_000_000).contains(&dest.0.z)
    }

    #[must_use]
    pub fn is_valid_vertically(y: i32) -> bool {
        // Note: 20_000_000 is not valid, but -20_000_000 is.
        (-20_000_000..20_000_000).contains(&y)
    }

    #[must_use]
    pub fn is_in_build_limit(&self, dest: BlockPos) -> bool {
        self.is_in_height_limit(dest.0.y) && Self::is_valid_horizontally(dest)
    }

    #[must_use]
    pub fn is_in_height_limit(&self, y: i32) -> bool {
        (self.get_bottom_y()..=self.get_top_y()).contains(&y)
    }

    pub const fn get_bottom_y(&self) -> i32 {
        self.dimension.min_y
    }

    pub const fn get_top_y(&self) -> i32 {
        self.dimension.min_y + self.dimension.height - 1
    }

    /// Gets a `Block` from the block registry. Returns `Block::AIR` if the block was not found.
    pub fn get_block(&self, position: &BlockPos) -> &'static Block {
        self.get_block_state_id_if_loaded(position)
            .map_or(&Block::AIR, Block::from_state_id)
    }

    #[must_use]
    pub fn get_block_state_id_if_loaded(&self, position: &BlockPos) -> Option<BlockStateId> {
        if !self.is_in_build_limit(*position) {
            return None;
        }

        let (chunk_coordinate, relative) = position.chunk_and_chunk_relative_position();
        self.level.read_chunk_sync(&chunk_coordinate, |chunk| {
            chunk
                .section
                .get_block_absolute_y(relative.x as usize, relative.y, relative.z as usize)
        })?
    }

    #[must_use]
    pub fn get_block_state_if_loaded(&self, position: &BlockPos) -> Option<&'static BlockState> {
        self.get_block_state_id_if_loaded(position)
            .map(BlockState::from_id)
    }

    #[must_use]
    pub fn is_loaded(&self, position: &BlockPos) -> bool {
        self.get_block_state_id_if_loaded(position).is_some()
    }

    pub fn get_fluid(&self, position: &BlockPos) -> &'static pumpkin_data::fluid::Fluid {
        let id = self.get_block_state_id(position);
        let fluid = Fluid::from_state_id(id).ok_or(&Fluid::EMPTY);
        if let Ok(fluid) = fluid {
            return fluid.to_flowing();
        }
        let block = Block::from_state_id(id);
        block
            .properties(id)
            .and_then(|props| {
                props
                    .to_props()
                    .into_iter()
                    .find(|p| p.0 == "waterlogged")
                    .map(|(_, value)| {
                        if value == "true" {
                            &Fluid::FLOWING_WATER
                        } else {
                            &Fluid::EMPTY
                        }
                    })
            })
            .unwrap_or(&Fluid::EMPTY)
    }

    pub fn get_block_and_fluid(
        &self,
        position: &BlockPos,
    ) -> (
        &'static pumpkin_data::Block,
        &'static pumpkin_data::fluid::Fluid,
    ) {
        let id = self.get_block_state_id(position);
        let block = Block::from_state_id(id);

        let fluid = Fluid::from_state_id(id)
            .map(Fluid::to_flowing)
            .ok_or(&Fluid::EMPTY)
            .unwrap_or_else(|_| {
                block
                    .properties(id)
                    .and_then(|props| {
                        props
                            .to_props()
                            .into_iter()
                            .find(|p| p.0 == "waterlogged")
                            .map(|(_, value)| {
                                if value == "true" {
                                    &Fluid::FLOWING_WATER
                                } else {
                                    &Fluid::EMPTY
                                }
                            })
                    })
                    .unwrap_or(&Fluid::EMPTY)
            });
        (block, fluid)
    }

    pub fn get_fluid_and_fluid_state(
        &self,
        position: &BlockPos,
    ) -> (&'static Fluid, &'static FluidState) {
        let id = self.get_block_state_id(position);

        let Some(raw_fluid) = Fluid::from_state_id(id) else {
            let block = Block::from_state_id(id);
            if let Some(properties) = block.properties(id) {
                for (name, value) in properties.to_props() {
                    if name == "waterlogged" {
                        if value == "true" {
                            let state = &Fluid::FLOWING_WATER.states[0];
                            return (&Fluid::FLOWING_WATER, state);
                        }

                        break;
                    }
                }
            }

            let state = &Fluid::EMPTY.states[0];
            return (&Fluid::EMPTY, state);
        };

        let fluid = raw_fluid.to_flowing();
        let state = &fluid.states[0];

        (fluid, state)
    }

    pub fn get_block_state_id(&self, position: &BlockPos) -> BlockStateId {
        self.get_block_state_id_if_loaded(position)
            .unwrap_or(Block::AIR.default_state.id)
    }

    /// Gets the `BlockState` from the block registry. Returns Air if the block state was not found.
    pub fn get_block_state(&self, position: &BlockPos) -> &'static BlockState {
        let id = self.get_block_state_id(position);
        BlockState::from_id(id)
    }

    /// Gets the Block + Block state from the Block Registry, Returns Air if the Block state has not been found
    pub fn get_block_and_state(
        &self,
        position: &BlockPos,
    ) -> (&'static Block, &'static BlockState) {
        let id = self.get_block_state_id(position);
        BlockState::from_id_with_block(id)
    }

    /// Gets the Block + state id from the Block Registry, Returns Air if the Block state has not been found
    pub fn get_block_and_state_id(&self, position: &BlockPos) -> (&'static Block, BlockStateId) {
        let id = self.get_block_state_id(position);
        (Block::from_state_id(id), id)
    }

    pub fn get_block_entity(&self, block_pos: &BlockPos) -> Option<Arc<dyn BlockEntity>> {
        let chunk_pos = block_pos.chunk_position();
        if let Some(chunk_block_entities) = self.block_entities.get(&chunk_pos)
            && let Some(entity) = chunk_block_entities.get(block_pos)
        {
            return Some(entity.clone());
        }

        let nbt = self
            .level
            .read_chunk_sync(&chunk_pos, |chunk| {
                chunk
                    .pending_block_entities
                    .lock()
                    .unwrap()
                    .remove(block_pos)
            })
            .flatten()?;
        let entity = block_entity_from_nbt(&nbt)?;
        self.block_entities
            .entry(chunk_pos)
            .or_default()
            .insert(*block_pos, entity.clone());
        Some(entity)
    }

    pub fn add_block_entity(&self, block_entity: Arc<dyn BlockEntity>) {
        let block_pos = block_entity.get_position();
        let chunk_pos = block_pos.chunk_position();

        {
            let id = block_entity.resource_location();
            if id == crate::block::entities::sculk_sensor::SculkSensorBlockEntity::ID
                || id == crate::block::entities::calibrated_sculk_sensor::CalibratedSculkSensorBlockEntity::ID
            {
                self.has_sculk_sensors
                    .store(true, std::sync::atomic::Ordering::Relaxed);
            }
        }

        self.block_entities
            .entry(chunk_pos)
            .or_default()
            .insert(block_pos, block_entity);
        self.unsent_block_entity_updates
            .lock()
            .unwrap()
            .insert(block_pos);
        self.level.read_chunk_sync(&chunk_pos, |chunk| {
            chunk.mark_dirty(true);
        });
    }

    pub fn add_block_entity_nbt(&self, block_pos: BlockPos, nbt: &NbtCompound) {
        self.level
            .read_chunk_sync(&block_pos.chunk_position(), |chunk| {
                chunk
                    .pending_block_entities
                    .lock()
                    .unwrap()
                    .insert(block_pos, nbt.clone());
                chunk.mark_dirty(true);
            });
    }

    pub fn remove_block_entity(&self, block_pos: &BlockPos) {
        let chunk_pos = block_pos.chunk_position();
        let removed =
            self.block_entities
                .get_mut(&chunk_pos)
                .is_some_and(|mut chunk_block_entities| {
                    chunk_block_entities.remove(block_pos).is_some()
                });
        if removed {
            // Drop the chunk's map once its last block entity is gone.
            self.block_entities
                .remove_if(&chunk_pos, |_, entities| entities.is_empty());
            self.level.read_chunk_sync(&chunk_pos, |chunk| {
                chunk.mark_dirty(true);
            });
        }
    }

    pub fn update_block_entity(&self, block_entity: &Arc<dyn BlockEntity>) {
        let block_pos = block_entity.get_position();
        let chunk_pos = block_pos.chunk_position();
        self.unsent_block_entity_updates
            .lock()
            .unwrap()
            .insert(block_pos);
        self.level.read_chunk_sync(&chunk_pos, |chunk| {
            chunk.mark_dirty(true);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_util::math::vector3::Vector3;

    #[test]
    fn sky_darken_matches_vanilla_reference_points() {
        // Noon, clear weather: no darken.
        assert_eq!(World::calculate_sky_darken(6000, 0.0, 0.0), 0);
        // Midnight, clear weather: full darken.
        assert_eq!(World::calculate_sky_darken(18000, 0.0, 0.0), 11);
        // Noon with full rain: (1 - 11/16) * 11 = 3.4375 -> 3.
        assert_eq!(World::calculate_sky_darken(6000, 1.0, 0.0), 3);
        // Noon with full rain and thunder: (1 - (11/16)^2) * 11 = 5.8 -> 6.
        assert_eq!(World::calculate_sky_darken(6000, 1.0, 1.0), 6);
    }

    #[test]
    fn position_validity_bounds_match_vanilla() {
        // Horizontal limit: -30_000_000 inclusive, 30_000_000 exclusive.
        assert!(World::is_valid_horizontally(BlockPos(Vector3::new(
            29_999_999,
            0,
            -30_000_000
        ))));
        assert!(!World::is_valid_horizontally(BlockPos(Vector3::new(
            30_000_000, 0, 0
        ))));
        assert!(!World::is_valid_horizontally(BlockPos(Vector3::new(
            0,
            0,
            -30_000_001
        ))));
        // Vertical limit: -20_000_000 inclusive, 20_000_000 exclusive.
        assert!(World::is_valid_vertically(19_999_999));
        assert!(World::is_valid_vertically(-20_000_000));
        assert!(!World::is_valid_vertically(20_000_000));
        // Combined check uses both limits.
        assert!(World::is_valid(BlockPos(Vector3::new(0, 0, 0))));
        assert!(!World::is_valid(BlockPos(Vector3::new(0, 20_000_000, 0))));
    }
}
