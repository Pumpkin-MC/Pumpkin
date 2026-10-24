use std::sync::{
    Arc, Mutex, Weak,
    atomic::{AtomicI32, Ordering::Relaxed},
};

use rand::RngExt;

use super::abstract_fish::AbstractFish;
use crate::entity::{EntityBase, mob::spawn::SpawnGroupData};

pub const SCHOOL_SEARCH_RANGE: f64 = 8.0;
const LEADER_DISTANCE_SQR: f64 = 121.0;
const SCHOOL_CHECK_INTERVAL: i32 = 200;

pub struct SchoolingFishData {
    pub self_reference: Weak<dyn EntityBase>,
    leader: Mutex<Option<Weak<dyn EntityBase>>>,
    school_size: AtomicI32,
}

impl SchoolingFishData {
    #[must_use]
    pub fn new(self_reference: Weak<dyn EntityBase>) -> Self {
        Self {
            self_reference,
            leader: Mutex::new(None),
            school_size: AtomicI32::new(1),
        }
    }
}

pub trait AbstractSchoolingFish: AbstractFish {
    fn schooling_data(&self) -> &SchoolingFishData;

    fn leader(&self) -> Option<Arc<dyn EntityBase>> {
        self.schooling_data()
            .leader
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .and_then(Weak::upgrade)
    }

    fn is_follower(&self) -> bool {
        self.leader().is_some_and(|leader| {
            leader.get_entity().is_alive()
                && leader
                    .get_living_entity()
                    .is_some_and(|living| !living.is_dead_or_dying())
        })
    }

    fn has_followers(&self) -> bool {
        self.schooling_data().school_size.load(Relaxed) > 1
    }

    fn can_be_followed(&self) -> bool {
        self.has_followers()
            && self.schooling_data().school_size.load(Relaxed)
                < self.get_entity().entity_type.limit_per_chunk
    }

    fn start_following(&self, leader: &Arc<dyn EntityBase>) {
        *self
            .schooling_data()
            .leader
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(Arc::downgrade(leader));
        if let Some(fish) = leader.get_mob().and_then(|mob| mob.as_schooling_fish()) {
            fish.schooling_data().school_size.fetch_add(1, Relaxed);
        }
    }

    fn stop_following(&self) {
        let leader = self
            .schooling_data()
            .leader
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
            .and_then(|leader| leader.upgrade());
        if let Some(fish) = leader
            .as_ref()
            .and_then(|leader| leader.get_mob())
            .and_then(|mob| mob.as_schooling_fish())
        {
            fish.schooling_data().school_size.fetch_sub(1, Relaxed);
        }
    }

    fn add_followers(&self, followers: Vec<Arc<dyn EntityBase>>) {
        let limit = (self.get_entity().entity_type.limit_per_chunk
            - self.schooling_data().school_size.load(Relaxed))
        .max(0) as usize;
        let Some(leader) = self.schooling_data().self_reference.upgrade() else {
            return;
        };
        for follower in followers
            .into_iter()
            .take(limit)
            .filter(|fish| fish.get_entity().entity_id != self.get_entity().entity_id)
        {
            if let Some(fish) = follower.get_mob().and_then(|mob| mob.as_schooling_fish()) {
                fish.start_following(&leader);
            }
        }
    }

    fn in_range_of_leader(&self) -> bool {
        self.leader().is_some_and(|leader| {
            self.get_entity()
                .pos
                .load()
                .squared_distance_to_vec(&leader.get_entity().pos.load())
                <= LEADER_DISTANCE_SQR
        })
    }

    fn path_to_leader(&self) {
        if !self.is_follower() {
            return;
        }
        if let Some(leader) = self.leader() {
            let mob = self.get_mob_entity();
            if let Some(living) = leader.get_living_entity() {
                mob.steered()
                    .navigator()
                    .move_to_entity(living, 1.0, &mob.living_entity);
            }
        }
    }

    fn school_neighbors(&self) -> Vec<Arc<dyn EntityBase>> {
        let entity = self.get_entity();
        entity
            .world
            .load()
            .get_entities_at_box(&entity.bounding_box.load().expand(
                SCHOOL_SEARCH_RANGE,
                SCHOOL_SEARCH_RANGE,
                SCHOOL_SEARCH_RANGE,
            ))
            .into_iter()
            .filter(|other| other.get_entity().entity_type == entity.entity_type)
            .collect()
    }

    fn schooling_tick(&self) {
        if self.has_followers()
            && self.get_random().random_range(0..SCHOOL_CHECK_INTERVAL) == 1
            && self.school_neighbors().len() <= 1
        {
            self.schooling_data().school_size.store(1, Relaxed);
        }
    }

    fn finalize_school_spawn(&self, group: Option<SpawnGroupData>) -> Option<SpawnGroupData> {
        self.get_mob_entity().finalize_spawn_base();
        if let Some(SpawnGroupData::School(leader) | SpawnGroupData::TropicalFish { leader, .. }) =
            &group
        {
            if let Some(leader) = leader.upgrade() {
                self.start_following(&leader);
            }
            group
        } else {
            Some(SpawnGroupData::School(
                self.schooling_data().self_reference.clone(),
            ))
        }
    }
}

/// Generates the `AbstractSchoolingFish` impl. The type needs a `schooling_data` field.
#[macro_export]
macro_rules! impl_schooling_fish {
    ($fish:ty) => {
        impl $crate::entity::passive::abstract_schooling_fish::AbstractSchoolingFish for $fish {
            fn schooling_data(
                &self,
            ) -> &$crate::entity::passive::abstract_schooling_fish::SchoolingFishData {
                &self.schooling_data
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::{ai::test_support::world, r#type::from_type};
    use pumpkin_data::entity::EntityType;
    use pumpkin_util::math::vector3::Vector3;
    use uuid::Uuid;

    #[tokio::test]
    async fn school_limit_precedes_self_filter_and_stop_releases_a_slot()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let leader = from_type(
            &EntityType::COD,
            Vector3::new(0.0, 64.0, 0.0),
            &world,
            Uuid::new_v4(),
        );
        let followers: Vec<_> = (0..8)
            .map(|_| {
                from_type(
                    &EntityType::COD,
                    Vector3::new(2.0, 64.0, 0.0),
                    &world,
                    Uuid::new_v4(),
                )
            })
            .collect();
        let fish = leader
            .get_mob()
            .and_then(|mob| mob.as_schooling_fish())
            .ok_or("missing leader")?;
        let mut candidates = vec![leader.clone()];
        candidates.extend(followers.iter().cloned());
        fish.add_followers(candidates);
        let following = |entity: &Arc<dyn EntityBase>| {
            entity
                .get_mob()
                .and_then(|mob| mob.as_schooling_fish())
                .is_some_and(AbstractSchoolingFish::is_follower)
        };
        assert_eq!(
            followers.iter().filter(|entity| following(entity)).count(),
            6
        );
        fish.add_followers(vec![followers[6].clone()]);
        assert!(!fish.can_be_followed());
        let first = followers[0]
            .get_mob()
            .and_then(|mob| mob.as_schooling_fish())
            .ok_or("missing follower")?;
        leader.get_entity().set_pos(Vector3::new(1000.0, 64.0, 0.0));
        assert!(!first.in_range_of_leader());
        assert!(
            crate::world::entity_islands::links_of(followers[0].as_ref())
                .contains(&leader.get_entity().entity_id)
        );
        first.stop_following();
        assert!(fish.can_be_followed());
        assert!(!first.is_follower());
        world.level.shutdown().await;
        Ok(())
    }
}
