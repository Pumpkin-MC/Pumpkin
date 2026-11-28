use pumpkin_data::environment_attribute::Activity;

use super::super::super::BrainTick;
use super::super::super::memory::types;
use super::super::OneShot;
use crate::entity::passive::villager::VillagerEntity;

#[must_use]
pub fn create() -> OneShot {
    OneShot::new("WakeUp", Vec::new(), |tick| {
        if !tick.state.is_active(Activity::Rest) && is_sleeping(tick) {
            defer_wake_up(tick);
            true
        } else {
            false
        }
    })
}

pub(super) fn is_sleeping(tick: &BrainTick<'_>) -> bool {
    tick.mob
        .cast_any()
        .downcast_ref::<VillagerEntity>()
        .is_some_and(VillagerEntity::is_sleeping)
}

pub(super) fn defer_wake_up(tick: &mut BrainTick<'_>) {
    let time = tick.time;
    tick.defer_with_update(
        |mob| {
            mob.cast_any()
                .downcast_ref::<VillagerEntity>()
                .is_some_and(|villager| {
                    villager.stop_sleeping();
                    true
                })
        },
        move |tick, woke| {
            if woke {
                tick.state.set(types::LAST_WOKEN, time);
            }
        },
    );
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use pumpkin_data::entity::{EntityPose, EntityType};
    use pumpkin_data::tag::Taggable;
    use pumpkin_data::{Block, block_properties::WhiteBedLikeProperties, dimension::Dimension};
    use pumpkin_util::math::{position::BlockPos, vector2::Vector2, vector3::Vector3};
    use pumpkin_world::{chunk::ChunkData, world::BlockFlags};

    use super::*;
    use crate::entity::ai::brain::{Brain, activity::ActivityData};
    use crate::entity::ai::test_support::server;
    use crate::entity::{Entity, EntityBase};
    use crate::plugin::{
        BoxFuture, EventRwHandler,
        api::events::{EventPriority, entity::entity_pose_change::EntityPoseChangeEvent},
    };
    use crate::server::Server;

    struct VetoSleepingPose;

    impl EventRwHandler<EntityPoseChangeEvent> for VetoSleepingPose {
        fn handle<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut EntityPoseChangeEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                if event.pose == (EntityPose::Sleeping as u8).to_string() {
                    event.cancelled = true;
                }
            })
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn wake_clears_the_bed_after_a_vetoed_sleeping_pose()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let server = server(&folder).await?;
        let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
        world
            .level
            .loaded_chunks
            .insert(Vector2::new(0, 0), ChunkData::empty_sync(0, 0));
        let bed = BlockPos(Vector3::new(8, 64, 8));
        world.set_block_state(&bed, Block::WHITE_BED.default_state.id, BlockFlags::empty());
        assert!(
            world
                .get_block_and_state(&bed)
                .0
                .has_tag(&pumpkin_data::tag::Block::MINECRAFT_BEDS)
        );
        let villager = VillagerEntity::new(Entity::new(
            world.clone(),
            bed.to_f64(),
            &EntityType::VILLAGER,
        ));
        server.plugin_manager.register_blocking(
            Arc::new(VetoSleepingPose),
            EventPriority::Normal,
            None,
        );
        assert!(villager.start_sleeping(bed));
        assert_eq!(villager.get_entity().pose.load(), EntityPose::Standing);
        assert!(WhiteBedLikeProperties::from_state_id(world.get_block_state(&bed).id).occupied);

        let mut brain = Brain::default();
        brain.register_memory(types::LAST_WOKEN.id());
        brain.add_activity(ActivityData::with_priority_start(
            Activity::Core,
            0,
            vec![Box::new(create())],
        ));
        brain.tick(&world, villager.as_ref(), 42);

        assert_eq!(brain.get(types::LAST_WOKEN), Some(&42));
        assert!(!WhiteBedLikeProperties::from_state_id(world.get_block_state(&bed).id).occupied);
        server.shutdown().await;
        Ok(())
    }
}
