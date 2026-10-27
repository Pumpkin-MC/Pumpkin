use std::sync::Arc;

use super::{
    abstract_fish::{AbstractFish, FishData},
    abstract_schooling_fish::{AbstractSchoolingFish, SchoolingFishData},
};
use crate::entity::bucketable::Bucketable;
use pumpkin_data::{item::Item, sound::Sound};
use pumpkin_nbt::compound::NbtCompound;

use crate::entity::{
    Entity,
    ai::goal::follow_flock_leader::FollowFlockLeaderGoal,
    mob::{Mob, MobEntity},
};

pub struct CodEntity {
    pub mob_entity: MobEntity,
    fish_data: FishData,
    schooling_data: SchoolingFishData,
}

impl CodEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mut mob_entity = MobEntity::new(entity);
        super::abstract_fish::initialize_fish_movement(&mut mob_entity);
        let mob_arc = Arc::new_cyclic(|weak: &std::sync::Weak<Self>| Self {
            mob_entity,
            fish_data: FishData::default(),
            schooling_data: SchoolingFishData::new(weak.clone()),
        });
        super::abstract_fish::register_fish_goals(&mob_arc.mob_entity);
        mob_arc
            .mob_entity
            .goals_selector
            .add_goal(5, Box::new(FollowFlockLeaderGoal::default()));

        mob_arc
    }
}

impl Mob for CodEntity {
    crate::fish_mob_hooks!();
    crate::schooling_fish_mob_hooks!();

    fn finalize_spawn(
        &self,
        _world: &Arc<crate::world::World>,
        group_data: Option<crate::entity::mob::spawn::SpawnGroupData>,
    ) -> Option<crate::entity::mob::spawn::SpawnGroupData> {
        self.finalize_school_spawn(group_data)
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        self.fish_write_nbt(nbt);
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.fish_read_nbt(nbt);
    }

    fn mob_init_data_tracker(&self) {
        self.set_from_bucket(self.is_from_bucket());
    }
}

crate::impl_abstract_fish!(
    CodEntity,
    bucket: &Item::COD_BUCKET,
    flop: Sound::EntityCodFlop,
    ambient: Some(Sound::EntityCodAmbient)
);
crate::impl_schooling_fish!(CodEntity);
