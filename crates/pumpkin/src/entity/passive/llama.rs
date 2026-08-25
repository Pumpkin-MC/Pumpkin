use std::sync::atomic::Ordering;
use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;
use pumpkin_data::sound::{Sound, SoundCategory};

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture,
    ai::goal::{
        active_target::ActiveTargetGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, ranged_attack::RangedAttackGoal, revenge::RevengeGoal,
        swim::SwimGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity, RangedAttackMob},
    projectile::llama_spit::LlamaSpitEntity,
};

/// Represents a Llama, a neutral mob that can be used for carrying items and spits at enemies.
///
/// Wiki: <https://minecraft.wiki/w/Llama>
pub struct LlamaEntity {
    pub mob_entity: MobEntity,
}

impl LlamaEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let llama = Self { mob_entity };
        let mob_arc = Arc::new(llama);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };
        let ranged_weak: Weak<dyn RangedAttackMob> = {
            let ranged_arc: Arc<dyn RangedAttackMob> = mob_arc.clone();
            Arc::downgrade(&ranged_arc)
        };

        {
            let mut goal_selector = mob_arc
                .mob_entity
                .goals_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let mut target_selector = mob_arc
                .mob_entity
                .target_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(
                3,
                Box::new(RangedAttackGoal::new(ranged_weak, 1.25, 40, 20.0)),
            );
            goal_selector.add_goal(7, Box::new(WanderAroundGoal::new(0.7)));
            goal_selector.add_goal(
                8,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(9, Box::new(RandomLookAroundGoal::default()));

            target_selector.add_goal(1, Box::new(RevengeGoal::new(true)));
            target_selector.add_goal(
                2,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::WOLF, true),
            );
        };

        mob_arc
    }

    pub async fn spit(&self, target: &Arc<dyn EntityBase>) {
        let entity = self.get_entity();
        let world = entity.world.load();

        let spit_entity = Entity::new(world.clone(), entity.pos.load(), &EntityType::LLAMA_SPIT);
        let spit = LlamaSpitEntity::new_shot(spit_entity, entity);

        let mob_pos = entity.pos.load();
        let target_entity = target.get_entity();
        let target_pos = target_entity.pos.load();
        let target_height = f64::from(target_entity.entity_dimension.load().height);

        let dx = target_pos.x - mob_pos.x;
        let dy = (target_pos.y + target_height / 3.0) - spit.get_entity().pos.load().y;
        let dz = target_pos.z - mob_pos.z;
        let horizontal_distance = dx.hypot(dz);
        let yo = horizontal_distance * 0.2;

        spit.thrown.set_velocity(dx, dy + yo, dz, 1.5, 10.0);

        if !entity.silent.load(Ordering::Relaxed) {
            world.play_sound(Sound::EntityLlamaSpit, SoundCategory::Neutral, &mob_pos);
        }

        let spit_arc: Arc<dyn EntityBase> = Arc::new(spit);
        world.spawn_entity(spit_arc).await;
    }
}

impl Mob for LlamaEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}

impl RangedAttackMob for LlamaEntity {
    fn perform_ranged_attack<'a>(
        &'a self,
        target: &'a Arc<dyn EntityBase>,
        _power: f32,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            self.spit(target).await;
        })
    }
}
