use std::sync::{Arc, PoisonError};

use pumpkin_util::math::{position::BlockPos, vector2::Vector2};
use pumpkin_world::{chunk_system::ChunkLoading, level::Level};

use super::CreakingEntity;
use crate::{
    block::entities::creaking_heart::CreakingHeartBlockEntity, entity::EntityBase, world::World,
};

pub(super) struct HeartChunkTicket {
    level: Arc<Level>,
    chunk: Vector2<i32>,
}

impl HeartChunkTicket {
    fn new(level: Arc<Level>, chunk: Vector2<i32>) -> Self {
        let mut loading = level
            .chunk_loading
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        loading.add_ticket(chunk, ChunkLoading::FULL_CHUNK_LEVEL);
        loading.send_change();
        drop(loading);
        Self { level, chunk }
    }
}

impl Drop for HeartChunkTicket {
    fn drop(&mut self) {
        let mut loading = self
            .level
            .chunk_loading
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        loading.remove_ticket(self.chunk, ChunkLoading::FULL_CHUNK_LEVEL);
        loading.send_change();
    }
}

impl CreakingEntity {
    pub(super) fn release_home_ticket(&self) {
        self.home_chunk_ticket
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take();
    }

    pub(super) fn request_unloaded_home(
        &self,
        world: &World,
        home: BlockPos,
        hit_by_player: Option<bool>,
    ) -> bool {
        let chunk = home.chunk_position();
        if !world.is_in_build_limit(home) || world.level.is_chunk_loaded(&chunk) {
            return false;
        }

        let mut ticket = self
            .home_chunk_ticket
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        if self.get_entity().is_removed() || self.get_home_pos() != Some(home) {
            return false;
        }
        if ticket
            .as_ref()
            .is_none_or(|ticket| ticket.chunk != chunk || !Arc::ptr_eq(&ticket.level, &world.level))
        {
            *ticket = Some(HeartChunkTicket::new(world.level.clone(), chunk));
        }
        if let Some(hit_by_player) = hit_by_player
            && self
                .pending_home_feedback
                .swap(Some((home, hit_by_player)))
                .is_none()
        {
            world.defer_creaking_heart_hit(self.get_entity().entity_id);
        }
        true
    }

    pub(super) fn check_home(&self) -> bool {
        let Some(home) = self.get_home_pos() else {
            return true;
        };
        let living = &self.mob_entity.living_entity;
        if living.is_dead_or_dying() {
            return true;
        }
        let entity = self.get_entity();
        let world = entity.world.load();
        if self.request_unloaded_home(&world, home, None) {
            return false;
        }
        let linked = world
            .get_block_entity(&home)
            .as_deref()
            .and_then(|block_entity| self.linked_heart(block_entity))
            .is_some();
        if self.get_home_pos() != Some(home) {
            return false;
        }
        if !linked {
            // Without a ticket a chunk can unload in between the two lookups.
            if self.request_unloaded_home(&world, home, None) {
                return false;
            }
            living.set_health(0.0);
            self.pending_home_feedback.store(None);
            self.release_home_ticket();
        }
        true
    }

    pub(crate) fn finish_pending_heart_hit(&self, world: &Arc<World>, active: bool) -> bool {
        let Some((home, hit_by_player)) = self.pending_home_feedback.load() else {
            return true;
        };
        if !world.level.is_chunk_loaded(&home.chunk_position()) {
            return false;
        }
        if self
            .pending_home_feedback
            .compare_exchange(Some((home, hit_by_player)), None)
            .is_err()
        {
            return self.pending_home_feedback.load().is_none();
        }
        if self.get_home_pos() == Some(home)
            && !self.get_entity().is_removed()
            && !self.mob_entity.living_entity.is_dead_or_dying()
            && let Some(block_entity) = world.get_block_entity(&home)
            && let Some(heart) = self.linked_heart(block_entity.as_ref())
        {
            self.play_heart_feedback(world, heart, hit_by_player);
        }
        if !active {
            let mut ticket = self
                .home_chunk_ticket
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            if self.pending_home_feedback.load().is_none()
                && ticket.as_ref().is_some_and(|ticket| {
                    ticket.chunk == home.chunk_position()
                        && Arc::ptr_eq(&ticket.level, &world.level)
                })
            {
                *ticket = None;
            }
        }
        true
    }

    pub(super) fn linked_heart<'a>(
        &self,
        block_entity: &'a dyn crate::block::entities::BlockEntity,
    ) -> Option<&'a CreakingHeartBlockEntity> {
        block_entity
            .as_any()
            .downcast_ref::<CreakingHeartBlockEntity>()
            .filter(|heart| heart.is_protector(self.get_entity().entity_uuid))
    }
}

#[cfg(test)]
mod tests {
    use pumpkin_data::{Block, entity::EntityType};
    use pumpkin_nbt::compound::NbtCompound;
    use pumpkin_util::math::vector3::Vector3;
    use pumpkin_world::{chunk::ChunkData, world::BlockFlags};

    use super::*;
    use crate::{
        block::entities::BlockEntity,
        entity::{Entity, ai::test_support::world, mob::Mob},
    };

    #[tokio::test]
    async fn protector_waits_for_the_home_chunk_then_checks_its_link()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let creaking = CreakingEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &EntityType::CREAKING,
        ));
        let home = BlockPos::new(32, 64, 0);
        let chunk_pos = home.chunk_position();
        creaking.set_transient(home);
        assert!(!world.level.is_chunk_loaded(&chunk_pos));
        assert!(!creaking.prepare_mob_tick());
        assert!(creaking.mob_entity.living_entity.is_alive());
        assert!(creaking.request_unloaded_home(&world, home, Some(false)));
        assert!(!creaking.finish_pending_heart_hit(&world, false));

        let chunk = ChunkData::empty_sync(chunk_pos.x, chunk_pos.y);
        world.level.loaded_chunks.insert(chunk_pos, chunk);
        world.set_block_state(
            &home,
            Block::CREAKING_HEART.default_state.id,
            BlockFlags::NOTIFY_ALL,
        );
        let mut nbt = NbtCompound::new();
        nbt.put_uuid("creaking", creaking.get_entity().entity_uuid);
        world.add_block_entity(Arc::new(CreakingHeartBlockEntity::from_nbt(&nbt, home)));
        assert!(creaking.finish_pending_heart_hit(&world, false));
        assert!(creaking.pending_home_feedback.load().is_none());
        assert!(
            creaking
                .home_chunk_ticket
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .is_none()
        );
        assert!(creaking.prepare_mob_tick());
        assert!(creaking.mob_entity.living_entity.is_alive());

        world.set_block_state(&home, Block::AIR.default_state.id, BlockFlags::NOTIFY_ALL);
        assert!(creaking.prepare_mob_tick());
        assert!(creaking.mob_entity.living_entity.is_dead_or_dying());
        assert!(
            creaking
                .home_chunk_ticket
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .is_none()
        );
        world.level.shutdown().await;
        Ok(())
    }
}
