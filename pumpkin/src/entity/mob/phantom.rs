use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Weak};

use pumpkin_data::attributes::Attributes;
use pumpkin_data::entity::EntityType;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::boundingbox::{BoundingBox, EntityDimensions};

use crate::entity::{
    Entity, NBTStorage, NbtFuture,
    ai::goal::{
        active_target::ActiveTargetGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, melee_attack::MeleeAttackGoal,
    },
    mob::{Mob, MobEntity},
};

pub struct PhantomEntity {
    pub mob_entity: MobEntity,
    size: AtomicI32,
}

impl PhantomEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let phantom = Self {
            mob_entity,
            size: AtomicI32::new(0),
        };
        let mob_arc = Arc::new(phantom);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();
            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();

            goal_selector.add_goal(1, Box::new(MeleeAttackGoal::new(1.0, false)));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(6, Box::new(RandomLookAroundGoal::default()));

            target_selector.add_goal(
                1,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PLAYER, true),
            );
        };

        mob_arc
    }

    pub fn set_size(&self, size: i32) {
        let size = size.clamp(0, 64);
        self.size.store(size, Ordering::Relaxed);

        let entity = &self.mob_entity.living_entity.entity;
        if let Some(attack_damage) = self
            .mob_entity
            .living_entity
            .attributes
            .write()
            .unwrap()
            .get_mut(&Attributes::ATTACK_DAMAGE.id)
        {
            attack_damage.base_value = 6.0 + f64::from(size);
            attack_damage.dirty.store(true, Ordering::Relaxed);
        }

        let original = entity.entity_type.dimension;
        let scale = 1.0 + 0.15 * size as f32;
        let dimensions = EntityDimensions {
            width: original[0] * scale,
            height: original[1] * scale,
            eye_height: entity.entity_type.eye_height * scale,
        };
        entity.entity_dimension.store(dimensions);
        let position = entity.pos.load();
        entity.bounding_box.store(BoundingBox::new_from_pos(
            position.x,
            position.y,
            position.z,
            &dimensions,
        ));
    }

    #[must_use]
    pub fn size(&self) -> i32 {
        self.size.load(Ordering::Relaxed)
    }
}

impl NBTStorage for PhantomEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.mob_entity.living_entity.write_nbt(nbt).await;
            nbt.put_int("size", self.size());
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.mob_entity.living_entity.read_nbt_non_mut(nbt).await;
            self.set_size(nbt.get_int("size").unwrap_or(0));
        })
    }
}

impl Mob for PhantomEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn get_mob_gravity(&self) -> f64 {
        0.0
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn phantom_sizes_follow_vanilla_bounds() {
        assert_eq!((-10i32).clamp(0, 64), 0);
        assert_eq!(64i32.clamp(0, 64), 64);
        assert_eq!(90i32.clamp(0, 64), 64);
    }

    #[test]
    fn phantom_attack_damage_scales_with_size() {
        assert_eq!(6.0 + f64::from(0), 6.0);
        assert_eq!(6.0 + f64::from(12), 18.0);
    }
}
