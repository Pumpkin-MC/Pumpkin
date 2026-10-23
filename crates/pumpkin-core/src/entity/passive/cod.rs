use std::sync::Arc;

use super::{
    abstract_fish::{AbstractFish, FishData},
    abstract_schooling_fish::{AbstractSchoolingFish, SchoolingFishData},
    water_animal::WaterAnimal,
};
use crate::entity::{EntityBase, bucketable::Bucketable};
use pumpkin_data::sound::Sound;
use pumpkin_nbt::compound::NbtCompound;
use std::sync::atomic::Ordering::Relaxed;

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
    fn as_bucketable(&self) -> Option<&dyn Bucketable> {
        Some(self)
    }
    fn mob_interact_with_hand(
        &self,
        player: &Arc<crate::entity::player::Player>,
        stack: &mut pumpkin_data::item_stack::ItemStack,
        _hand: pumpkin_util::Hand,
    ) -> Option<bool> {
        crate::entity::bucketable::bucket_mob_pickup(player, stack, self)
    }

    fn as_schooling_fish(&self) -> Option<&dyn AbstractSchoolingFish> {
        Some(self)
    }
    fn post_tick(&self) {
        self.schooling_tick();
    }
    fn finalize_spawn(
        &self,
        _world: &Arc<crate::world::World>,
        group_data: Option<crate::entity::mob::spawn::SpawnGroupData>,
    ) -> Option<crate::entity::mob::spawn::SpawnGroupData> {
        self.finalize_school_spawn(group_data)
    }

    fn before_mob_ai_step(&self) {
        self.fish_ai_step();
    }
    fn travel_in_water(&self, caller: &dyn EntityBase, _gravity: f64, _falling: bool) {
        self.fish_travel_in_water(caller);
    }

    fn as_fish(&self) -> Option<&dyn AbstractFish> {
        Some(self)
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
    fn remove_when_far_away(&self, _distance_sq: f64) -> bool {
        !self.is_from_bucket() && self.get_entity().custom_name.load().is_none()
    }
    fn requires_custom_persistence(&self) -> bool {
        self.is_from_bucket()
            || self
                .get_entity()
                .vehicle
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .is_some()
            || self
                .get_entity()
                .leashed_to
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .is_some()
    }
    fn check_spawn_obstruction(&self, world: &crate::world::World) -> bool {
        self.water_animal_spawn_obstruction(world)
    }
    fn mob_is_pushed_by_fluids(&self) -> bool {
        false
    }
    fn can_be_leashed(&self) -> bool {
        false
    }
    fn get_base_experience_reward(&self) -> u32 {
        self.water_animal_experience_reward()
    }
    fn base_tick(&self) {
        self.fish_base_tick(self);
    }
    fn as_custom_sound(&self) -> Option<&dyn crate::entity::custom_sound::CustomSound> {
        Some(self)
    }
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}

impl WaterAnimal for CodEntity {}

impl AbstractFish for CodEntity {
    fn flop_sound(&self) -> Sound {
        Sound::EntityCodFlop
    }
    fn fish_data(&self) -> &FishData {
        &self.fish_data
    }
    fn ambient_sound(&self) -> Sound {
        Sound::EntityCodAmbient
    }
}

impl crate::entity::custom_sound::CustomSound for CodEntity {
    fn death_sound(&self) -> Option<Sound> {
        self.get_entity().entity_type.death_sound
    }
    fn hurt_sound(&self) -> Option<Sound> {
        self.get_entity().entity_type.hurt_sound
    }
    fn sound_volume(&self) -> f32 {
        if self.get_entity().is_silent() {
            0.0
        } else {
            1.0
        }
    }
    fn sound_category(&self) -> pumpkin_data::sound::SoundCategory {
        pumpkin_data::sound::SoundCategory::Neutral
    }
    fn on_hurt_sound(&self) {
        self.reset_ambient_sound_time();
    }
}

impl AbstractSchoolingFish for CodEntity {
    fn schooling_data(&self) -> &SchoolingFishData {
        &self.schooling_data
    }
}

impl Bucketable for CodEntity {
    fn is_from_bucket(&self) -> bool {
        self.fish_data.from_bucket.load(Relaxed)
    }
    fn set_from_bucket(&self, from_bucket: bool) {
        self.fish_data.from_bucket.store(from_bucket, Relaxed);
        self.get_entity().set_synced_data(
            pumpkin_data::tracked_data::abstract_fish::FROM_BUCKET,
            from_bucket,
        );
    }
    fn bucket_item(&self) -> &'static pumpkin_data::item::Item {
        &pumpkin_data::item::Item::COD_BUCKET
    }
    fn pickup_sound(&self) -> Sound {
        Sound::ItemBucketFillFish
    }
}
