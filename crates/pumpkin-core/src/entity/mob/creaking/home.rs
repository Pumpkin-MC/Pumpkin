use std::sync::{Arc, PoisonError};

use pumpkin_util::math::{position::BlockPos, vector2::Vector2};
use pumpkin_world::{chunk_system::ChunkLoading, level::Level};

use super::CreakingEntity;
use crate::{
    block::entities::creaking_heart::CreakingHeartBlockEntity,
    entity::EntityBase,
    world::{BlockEntityLookup, World},
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum HeartHome {
    Current(BlockPos),
    Legacy(BlockPos),
}

impl HeartHome {
    pub(super) const fn pos(self) -> BlockPos {
        match self {
            Self::Current(pos) | Self::Legacy(pos) => pos,
        }
    }
}

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

    fn request_home_chunk(&self, world: &World, home: BlockPos) {
        let chunk = home.chunk_position();
        let mut ticket = self
            .home_chunk_ticket
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        if self.get_entity().is_removed() || self.get_home_pos() != Some(home) {
            return;
        }
        if ticket
            .as_ref()
            .is_none_or(|ticket| ticket.chunk != chunk || !Arc::ptr_eq(&ticket.level, &world.level))
        {
            *ticket = Some(HeartChunkTicket::new(world.level.clone(), chunk));
        }
    }

    pub(super) fn check_home(&self) -> bool {
        let Some(link) = self.home_pos.load() else {
            return true;
        };
        let home = link.pos();
        let living = &self.mob_entity.living_entity;
        if living.is_dead_or_dying() {
            return true;
        }
        let entity = self.get_entity();
        let world = entity.world.load_full();
        let BlockEntityLookup::Loaded(block_entity) = world.get_block_entity_if_chunk_loaded(&home)
        else {
            self.request_home_chunk(&world, home);
            return false;
        };
        if self.home_pos.load() != Some(link) {
            return false;
        }
        let legacy = matches!(link, HeartHome::Legacy(_));
        let heart = block_entity
            .as_deref()
            .and_then(|block_entity| {
                block_entity
                    .as_any()
                    .downcast_ref::<CreakingHeartBlockEntity>()
            })
            .filter(|heart| {
                heart.is_protector(entity.entity_uuid)
                    || legacy && heart.try_adopt_legacy_protector(&world, self)
            });
        let Some(heart) = heart else {
            living.set_health(0.0);
            self.pending_home_feedback.store(None);
            self.release_home_ticket();
            return true;
        };
        if legacy {
            let _ = self
                .home_pos
                .compare_exchange(Some(link), Some(HeartHome::Current(home)));
        }
        if let Some((pending_home, hit_by_player)) = self.pending_home_feedback.load()
            && pending_home == home
            && self
                .pending_home_feedback
                .compare_exchange(Some((pending_home, hit_by_player)), None)
                .is_ok()
        {
            self.play_heart_feedback(&world, heart, hit_by_player);
        }
        !entity.is_removed()
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
    use std::sync::atomic::Ordering::Relaxed;

    use pumpkin_data::{Block, entity::EntityType};
    use pumpkin_nbt::compound::NbtCompound;
    use pumpkin_util::math::vector3::Vector3;
    use pumpkin_world::{chunk::ChunkData, world::BlockFlags};

    use super::*;
    use crate::{
        block::entities::BlockEntity,
        entity::{
            Entity,
            ai::{brain::memory::codec::block_pos_from_tag, test_support::world},
            mob::Mob,
            passive::iron_golem::IronGolemEntity,
        },
    };

    #[tokio::test]
    async fn protector_replays_an_unloaded_home_hit_during_its_next_eligible_tick()
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
        let attacker = IronGolemEntity::new(Entity::new(
            world.clone(),
            Vector3::new(1.5, 64.0, 0.5),
            &EntityType::IRON_GOLEM,
        ));
        assert!(attacker.do_hurt_target(creaking.as_ref()));
        assert!(creaking.pending_home_feedback.load().is_some());
        assert!(
            creaking
                .home_chunk_ticket
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .is_none()
        );
        let ambient = &creaking.mob_entity.ambient_sound_time;
        assert_eq!(ambient.load(Relaxed), 0);
        assert!(!creaking.prepare_mob_tick());
        assert!(creaking.mob_entity.living_entity.is_alive());
        assert!(creaking.pending_home_feedback.load().is_some());

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
        assert!(creaking.prepare_mob_tick());
        assert!(creaking.pending_home_feedback.load().is_none());
        assert_eq!(
            ambient.load(Relaxed),
            -creaking.get_ambient_sound_interval()
        );
        ambient.store(17, Relaxed);
        assert!(creaking.prepare_mob_tick());
        assert_eq!(ambient.load(Relaxed), 17);
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

    #[tokio::test]
    async fn legacy_home_preserves_its_format_until_the_heart_can_adopt()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let creaking = CreakingEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &EntityType::CREAKING,
        ));
        let home = BlockPos::new(32, 64, 0);
        let mut coords = NbtCompound::new();
        coords.put_int("x", 32);
        coords.put_int("y", 64);
        coords.put_int("z", 0);
        let mut legacy = NbtCompound::new();
        legacy.put_compound("home_pos", coords);
        creaking.mob_read_nbt(&legacy);
        assert_eq!(creaking.get_home_pos(), Some(home));
        assert!(!creaking.prepare_mob_tick());

        let mut before_home_load = NbtCompound::new();
        creaking.mob_write_nbt(&mut before_home_load);
        assert!(before_home_load.get_compound("home_pos").is_some());
        creaking.mob_read_nbt(&before_home_load);

        let chunk_pos = home.chunk_position();
        let chunk = ChunkData::empty_sync(chunk_pos.x, chunk_pos.y);
        world.level.loaded_chunks.insert(chunk_pos, chunk);
        world.set_block_state(
            &home,
            Block::CREAKING_HEART.default_state.id,
            BlockFlags::NOTIFY_ALL,
        );
        assert!(creaking.prepare_mob_tick());
        assert!(creaking.mob_entity.living_entity.is_alive());
        assert!(
            world
                .get_block_entity(&home)
                .as_deref()
                .is_some_and(|heart| creaking.linked_heart(heart).is_some())
        );

        let mut migrated = NbtCompound::new();
        creaking.mob_write_nbt(&mut migrated);
        assert_eq!(
            migrated.get("home_pos").and_then(block_pos_from_tag),
            Some(home),
        );
        world.level.shutdown().await;
        Ok(())
    }
}
