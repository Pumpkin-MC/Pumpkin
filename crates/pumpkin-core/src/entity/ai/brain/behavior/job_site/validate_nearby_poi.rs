use pumpkin_data::block_properties::WhiteBedLikeProperties as BedProperties;
use pumpkin_data::entity::EntityPose;
use pumpkin_data::poi_type::PoiType;
use pumpkin_data::tag::{Block as BlockTag, Taggable};
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

use super::super::super::memory::{GlobalPos, MemoryModuleType, MemoryStatus};
use super::super::OneShot;
use super::super::utils::world_for_dimension;
use crate::entity::EntityBase;
use crate::entity::mob::Mob;
use crate::entity::passive::villager::VillagerEntity;
use crate::world::World;

const MAX_DISTANCE: i32 = 16;

#[must_use]
pub fn create(
    poi_type: impl Fn(PoiType) -> bool + Send + Sync + 'static,
    memory_type: MemoryModuleType<GlobalPos>,
) -> OneShot {
    OneShot::new(
        "ValidateNearbyPoi",
        vec![(memory_type.id(), MemoryStatus::ValuePresent)],
        move |tick| {
            let Some(global_pos) = tick.state.get(memory_type).copied() else {
                return false;
            };
            let pos = global_pos.pos;
            if tick.world.dimension.id != global_pos.dimension.id
                || !closer_to_center_than(
                    pos,
                    tick.mob.get_entity().pos.load(),
                    f64::from(MAX_DISTANCE),
                )
            {
                return false;
            }
            let Some(poi_level) = world_for_dimension(tick.world, global_pos.dimension) else {
                tick.state.erase(memory_type.id());
                return true;
            };
            let exists = {
                let mut pois = poi_level
                    .portal_poi
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                pois.exists(&pos, &poi_type)
            };
            if !exists {
                tick.state.erase(memory_type.id());
            } else if bed_is_occupied(&poi_level, pos, tick.mob) {
                tick.state.erase(memory_type.id());
                if !bed_is_occupied_by_villager(&poi_level, pos) {
                    poi_level
                        .portal_poi
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .release(&pos);
                }
            }
            true
        },
    )
}

pub(super) fn closer_to_center_than(pos: BlockPos, point: Vector3<f64>, distance: f64) -> bool {
    let dx = f64::from(pos.0.x) + 0.5 - point.x;
    let dy = f64::from(pos.0.y) + 0.5 - point.y;
    let dz = f64::from(pos.0.z) + 0.5 - point.z;
    dx * dx + dy * dy + dz * dz < distance * distance
}

fn bed_is_occupied(poi_level: &World, poi_pos: BlockPos, body: &dyn Mob) -> bool {
    let (block, state) = poi_level.get_block_and_state(&poi_pos);
    block.has_tag(&BlockTag::MINECRAFT_VILLAGERS_CAN_SLEEP_ON_BED)
        && BedProperties::from_state_id(state.id).occupied
        && body.get_entity().pose.load() != EntityPose::Sleeping
}

fn bed_is_occupied_by_villager(poi_level: &World, poi_pos: BlockPos) -> bool {
    poi_level
        .get_entities_at_box(&BoundingBox::from_block(&poi_pos))
        .iter()
        .filter_map(|entity| entity.cast_any().downcast_ref::<VillagerEntity>())
        .any(|villager| villager.get_entity().pose.load() == EntityPose::Sleeping)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closer_to_center_uses_euclidean_distance_from_the_block_center() {
        let pos = BlockPos::new(0, 0, 0);
        assert!(closer_to_center_than(
            pos,
            Vector3::new(16.0, 0.5, 0.5),
            16.0
        ));
        assert!(!closer_to_center_than(
            pos,
            Vector3::new(16.5, 0.5, 0.5),
            16.0
        ));
        assert!(closer_to_center_than(
            pos,
            Vector3::new(0.5, 0.5, 0.5),
            0.001
        ));
        assert!(!closer_to_center_than(
            pos,
            Vector3::new(12.0, 12.0, 0.5),
            16.0
        ));
    }
}
