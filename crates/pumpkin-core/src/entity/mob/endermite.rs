use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Weak};

use pumpkin_data::BlockState;
use pumpkin_data::entity::EntityType;
use pumpkin_data::sound::Sound;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use rand::RngExt;

use crate::entity::ai::behavior::hunting::{self, Hunt};
use crate::entity::ai::goal::revenge::RevengeGoal;
use crate::entity::ai::target_match::TargetMatch;
use crate::entity::{
    Entity, EntityBase,
    ai::goal::{
        climb_on_top_of_powder_snow::ClimbOnTopOfPowderSnowGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, melee_attack::MeleeAttackGoal, swim::SwimGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

const ENDERMITE_HUNTS: &[Hunt] = &[Hunt::new(2, TargetMatch::Type(&EntityType::PLAYER), true)];
const MAX_LIFETIME: i32 = 2400;

pub struct EndermiteEntity {
    pub mob_entity: MobEntity,
    pub lifetime: AtomicI32,
}

impl EndermiteEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let endermite = Self {
            mob_entity,
            lifetime: AtomicI32::new(0),
        };
        let mob_arc = Arc::new(endermite);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let goal_selector = &mob_arc.mob_entity.goals_selector;

            goal_selector.add_goal(1, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, Box::new(ClimbOnTopOfPowderSnowGoal));
            goal_selector.add_goal(2, Box::new(MeleeAttackGoal::new(1.0, false)));
            goal_selector.add_goal(3, Box::new(WanderAroundGoal::water_avoiding(1.0)));
            goal_selector.add_goal(
                7,
                LookAtEntityGoal::with_default(mob_weak.clone(), &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(8, Box::new(RandomLookAroundGoal::default()));

            let target_selector = &mob_arc.mob_entity.target_selector;
            target_selector.add_goal(1, Box::new(RevengeGoal::new(true).alerting_others()));
            hunting::apply(target_selector, &mob_arc.mob_entity, ENDERMITE_HUNTS);
        };

        mob_arc
    }
}

impl Mob for EndermiteEntity {
    fn mob_play_step_sound(&self, _pos: &BlockPos, _state: &BlockState) {
        self.play_entity_sound(Sound::EntityEndermiteStep, 0.15, 1.0);
    }

    fn get_ambient_sound(&self) -> Option<Sound> {
        Some(Sound::EntityEndermiteAmbient)
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_int("Lifetime", self.lifetime.load(Ordering::Relaxed));
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        if let Some(life) = nbt.get_int("Lifetime") {
            self.lifetime.store(life, Ordering::Relaxed);
        }
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        let entity = &self.mob_entity.living_entity.entity;
        if !entity.is_alive() {
            return;
        }

        entity.body_yaw.store(entity.yaw.load());
    }

    fn post_tick(&self) {
        let entity = &self.mob_entity.living_entity.entity;
        if !entity.is_alive() {
            return;
        }

        if !self.mob_entity.persistence_required.load(Ordering::Relaxed) {
            self.lifetime.fetch_add(1, Ordering::Relaxed);
        }
        if self.lifetime.load(Ordering::Relaxed) >= MAX_LIFETIME {
            entity.remove();
            return;
        }

        let dimensions = entity.entity_dimension.load();
        let position = entity.pos.load();
        let mut rng = self.get_random();
        for _ in 0..2 {
            let particle_position = pumpkin_util::math::vector3::Vector3::new(
                position.x + (rng.random::<f64>() - 0.5) * f64::from(dimensions.width),
                position.y + rng.random::<f64>() * f64::from(dimensions.height),
                position.z + (rng.random::<f64>() - 0.5) * f64::from(dimensions.width),
            );
            let velocity = pumpkin_util::math::vector3::Vector3::new(
                (rng.random::<f64>() - 0.5) * 2.0,
                -rng.random::<f64>(),
                (rng.random::<f64>() - 0.5) * 2.0,
            );
            entity.world.load().spawn_particle(
                particle_position,
                velocity.to_f32_lossy(),
                1.0,
                0,
                pumpkin_data::particle::Particle::Portal,
            );
        }
    }
}
