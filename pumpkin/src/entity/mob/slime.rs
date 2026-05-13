use std::sync::atomic::Ordering;
use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;
use pumpkin_data::sound::Sound;
use pumpkin_nbt::compound::NbtCompound;

use crate::entity::{
    Entity, NBTStorage, NbtFuture,
    ai::goal::{
        active_target::ActiveTargetGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, swim::SwimGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

pub struct SlimeEntity {
    entity: Arc<MobEntity>,
}

impl SlimeEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let slime = Self {
            entity: Arc::new(mob_entity),
        };
        let mob_arc = Arc::new(slime);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.entity.goals_selector.lock().await;
            let mut target_selector = mob_arc.entity.target_selector.lock().await;

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(6, Box::new(RandomLookAroundGoal::default()));

            target_selector.add_goal(
                1,
                ActiveTargetGoal::with_default(&mob_arc.entity, &EntityType::PLAYER, true),
            );
        };

        mob_arc
    }

    pub(crate) const fn hurt_sound_for_size(size: i32) -> Sound {
        if size == 1 {
            Sound::EntitySlimeHurtSmall
        } else {
            Sound::EntitySlimeHurt
        }
    }
}

impl NBTStorage for SlimeEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.entity.living_entity.write_nbt(nbt).await;
            nbt.put_int(
                "Size",
                self.entity
                    .living_entity
                    .entity
                    .data
                    .load(Ordering::Relaxed),
            );
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.entity.living_entity.read_nbt_non_mut(nbt).await;
            self.entity
                .living_entity
                .entity
                .data
                .store(nbt.get_int("Size").unwrap_or(0), Ordering::Relaxed);
        })
    }
}

impl Mob for SlimeEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_small_hurt_sound_only_for_smallest_slimes() {
        assert_eq!(
            SlimeEntity::hurt_sound_for_size(1),
            Sound::EntitySlimeHurtSmall
        );
        assert_eq!(SlimeEntity::hurt_sound_for_size(0), Sound::EntitySlimeHurt);
        assert_eq!(SlimeEntity::hurt_sound_for_size(2), Sound::EntitySlimeHurt);
    }
}
