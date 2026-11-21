use pumpkin_data::HorizontalFacingExt;
use pumpkin_data::block_properties::WhiteBedLikeProperties;
use pumpkin_data::entity::EntityPose;
use pumpkin_data::tag::{Block as BlockTag, Taggable};
use pumpkin_data::tracked_data;
use pumpkin_data::{Block, BlockState};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::math::wrap_degrees;

use crate::block::blocks::bed::BedBlock;
use crate::entity::EntityBase;
use crate::entity::ai::brain::memory::types;

use super::VillagerEntity;

const ADJUST_HEIGHT: f64 = 0.125;
const STAND_UP_HEIGHT: f64 = 0.1;

impl VillagerEntity {
    #[must_use]
    pub fn is_sleeping(&self) -> bool {
        self.get_entity().pose.load() == EntityPose::Sleeping
    }

    pub fn start_sleeping(&self, bed_pos: BlockPos) -> bool {
        let world = self.get_entity().world.load();
        let (block, state) = world.get_block_and_state(&bed_pos);
        if !block.has_tag(&BlockTag::MINECRAFT_BEDS) {
            return false;
        }
        let Some(sleep_height) = get_sleep_height(state) else {
            return false;
        };

        self.get_entity().set_pos(
            bed_pos
                .to_f64()
                .add_raw(0.0, sleep_height + ADJUST_HEIGHT, 0.0),
        );
        BedBlock::set_occupied(true, &world, block, &bed_pos, state.id);
        self.get_entity().set_pose(EntityPose::Sleeping);
        self.get_entity()
            .set_synced_data(tracked_data::villager::SLEEPING_POS_ID, Some(bed_pos));
        self.get_entity().set_velocity(Vector3::default());
        *self
            .sleeping_pos
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(bed_pos);
        true
    }

    pub fn stop_sleeping(&self) {
        let world = self.get_entity().world.load();
        let bed_pos = self
            .sleeping_pos
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        if let Some(bed_pos) = bed_pos
            && let Some(state_id) = world.get_block_state_id_if_loaded(&bed_pos)
        {
            let block = Block::from_state_id(state_id);
            if block.has_tag(&BlockTag::MINECRAFT_BEDS) {
                BedBlock::set_occupied(false, &world, block, &bed_pos, state_id);

                let facing = WhiteBedLikeProperties::from_state_id(state_id)
                    .facing
                    .to_block_direction();
                let stand_up = BedBlock::find_stand_up_position(
                    self.get_entity().entity_type,
                    &world,
                    &bed_pos,
                    facing,
                    self.get_entity().yaw.load(),
                )
                .unwrap_or_else(|| bed_pos.up().to_f64().add_raw(0.0, STAND_UP_HEIGHT, 0.0));
                let look_direction = bed_pos.to_f64().sub(&stand_up).normalize();
                let yaw = wrap_degrees(
                    look_direction.z.atan2(look_direction.x).to_degrees() as f32 - 90.0,
                );
                self.get_entity().set_pos(stand_up);
                self.get_entity().set_rotation(yaw, 0.0);
            }
        }

        let position = self.get_entity().pos.load();
        self.get_entity().set_pose(EntityPose::Standing);
        self.get_entity().set_pos(position);
        self.get_entity()
            .set_synced_data(tracked_data::villager::SLEEPING_POS_ID, None::<BlockPos>);
    }

    pub fn wake_up(&self) {
        self.stop_sleeping();
        let time = self
            .get_entity()
            .world
            .load()
            .level_time
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .world_age;
        self.mob_entity
            .with_brain(self, |brain| brain.set(types::LAST_WOKEN, time));
    }

    #[must_use]
    pub fn check_bed_exists(&self) -> bool {
        if !self.is_sleeping() {
            return false;
        }
        let Some(bed_pos) = *self
            .sleeping_pos
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
        else {
            return false;
        };
        let world = self.get_entity().world.load();
        let (block, state) = world.get_block_and_state(&bed_pos);
        block.has_tag(&BlockTag::MINECRAFT_BEDS) && get_sleep_height(state).is_some()
    }
}

fn get_sleep_height(state: &BlockState) -> Option<f64> {
    state
        .get_block_collision_shapes()
        .map(|shape| shape.max.y)
        .reduce(f64::max)
}
