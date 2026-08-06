use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Weak};

use pumpkin_data::attributes::Attributes;
use pumpkin_data::entity::EntityType;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::java::client::play::Metadata;

use crate::entity::{
    Entity, EntityBaseFuture, NBTStorage, NbtFuture,
    ai::goal::{
        active_target::ActiveTargetGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, melee_attack::MeleeAttackGoal, swim::SwimGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

/// `Zoglin#finalizeSpawn`: a fifth of newly spawned zoglins are babies.
const BABY_SPAWN_CHANCE: f32 = 0.2;
/// `Zoglin#BABY_ATTACK_DAMAGE`, applied by `Zoglin#setBaby`.
const BABY_ATTACK_DAMAGE: f64 = 0.5;

pub struct ZoglinEntity {
    pub mob_entity: MobEntity,
    baby: AtomicBool,
}

impl ZoglinEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let zoglin = Self {
            mob_entity,
            baby: AtomicBool::new(false),
        };
        let mob_arc = Arc::new(zoglin);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(4, Box::new(MeleeAttackGoal::new(1.0, true)));
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak.clone(), &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(7, Box::new(RandomLookAroundGoal::default()));

            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();
            target_selector.add_goal(
                1,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PLAYER, true),
            );
        };

        // Vanilla rolls this in `finalizeSpawn`; Pumpkin has no separate finalize step, and
        // `read_nbt_non_mut` overwrites the result for zoglins restored from disk.
        if rand::random::<f32>() < BABY_SPAWN_CHANCE {
            mob_arc.set_baby(true);
        }

        mob_arc
    }

    #[must_use]
    pub fn is_baby(&self) -> bool {
        self.baby.load(Ordering::Relaxed)
    }

    /// `Zoglin#setBaby`. Vanilla only ever lowers `ATTACK_DAMAGE`, because a zoglin is
    /// created as an adult and never grows up; Pumpkin rolls the baby state before NBT is
    /// read back, so the adult base value is restored here as well.
    pub fn set_baby(&self, baby: bool) {
        if self.baby.swap(baby, Ordering::Relaxed) == baby {
            return;
        }

        let entity = &self.mob_entity.living_entity.entity;
        let attack_damage = if baby {
            BABY_ATTACK_DAMAGE
        } else {
            entity
                .entity_type
                .attributes
                .iter()
                .find(|(attribute, _)| attribute.id == Attributes::ATTACK_DAMAGE.id)
                .map_or(BABY_ATTACK_DAMAGE, |(_, base)| *base)
        };
        self.mob_entity
            .living_entity
            .update_attribute(&Attributes::ATTACK_DAMAGE, |instance| {
                instance.base_value = attack_damage;
            });

        entity.send_meta_data(
            &[Metadata::new(
                TrackedData::BABY_ID,
                MetaDataType::BOOLEAN,
                baby,
            )],
            None,
        );
    }
}

impl NBTStorage for ZoglinEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            nbt.put_bool("IsBaby", self.is_baby());
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.set_baby(nbt.get_bool("IsBaby").unwrap_or(false));
        })
    }
}

impl Mob for ZoglinEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            if self.is_baby() {
                self.mob_entity.living_entity.entity.send_meta_data(
                    &[Metadata::new(
                        TrackedData::BABY_ID,
                        MetaDataType::BOOLEAN,
                        true,
                    )],
                    None,
                );
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{BABY_ATTACK_DAMAGE, BABY_SPAWN_CHANCE};
    use pumpkin_data::attributes::Attributes;
    use pumpkin_data::entity::EntityType;

    #[test]
    fn matches_vanilla_zoglin_constants() {
        assert!((BABY_SPAWN_CHANCE - 0.2).abs() < f32::EPSILON);
        assert!((BABY_ATTACK_DAMAGE - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn adult_attack_damage_comes_from_the_generated_attribute_table() {
        let adult = EntityType::ZOGLIN
            .attributes
            .iter()
            .find(|(attribute, _)| attribute.id == Attributes::ATTACK_DAMAGE.id)
            .expect("zoglin has an attack damage attribute")
            .1;
        assert!((adult - 6.0).abs() < f64::EPSILON);
        assert!(adult > BABY_ATTACK_DAMAGE);
    }
}
