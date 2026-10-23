use std::sync::{Arc, Weak};

use super::{
    abstract_fish::{AbstractFish, FishData},
    water_animal::WaterAnimal,
};
use crate::entity::EntityBase;
use pumpkin_data::{entity::EntityType, sound::Sound};
use pumpkin_nbt::compound::NbtCompound;

use crate::entity::{
    Entity,
    ai::goal::{
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal, swim::SwimGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

pub struct CodEntity {
    pub mob_entity: MobEntity,
    fish_data: FishData,
}

impl CodEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let cod = Self {
            mob_entity,
            fish_data: FishData::default(),
        };
        let mob_arc = Arc::new(cod);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let goal_selector = &mob_arc.mob_entity.goals_selector;

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                2,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(3, Box::new(RandomLookAroundGoal::default()));
        };

        mob_arc
    }
}

impl Mob for CodEntity {
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
