use std::sync::{Arc, Weak};

use pumpkin_data::{
    data_component_impl::EquipmentSlot, entity::EntityType, item::Item, item_stack::ItemStack,
};

use crate::entity::{
    Entity, NBTStorage, NbtFuture,
    ai::goal::{
        active_target::ActiveTargetGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, melee_attack::MeleeAttackGoal,
        ranged_bow_attack::RangedBowAttackGoal, revenge::RevengeGoal, swim::SwimGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};
use pumpkin_nbt::compound::NbtCompound;

pub mod bogged;
pub mod parched;
#[allow(clippy::module_inception)]
pub mod skeleton;
pub mod stray;
pub mod wither;

pub struct SkeletonEntityBase {
    pub mob_entity: MobEntity,
}

impl SkeletonEntityBase {
    pub fn new(entity: Entity) -> Arc<Self> {
        let uses_bow = entity.entity_type != &EntityType::WITHER_SKELETON;
        let mob_entity = MobEntity::new(entity);
        let mob = Self { mob_entity };
        let mob_arc = Arc::new(mob);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };
        {
            // Vanilla `AbstractSkeleton#populateDefaultEquipmentSlots` equips a bow;
            // WitherSkeleton overrides that hook with a stone sword.
            let main_hand = if uses_bow {
                &Item::BOW
            } else {
                &Item::STONE_SWORD
            };
            mob_arc
                .mob_entity
                .living_entity
                .entity_equipment
                .try_lock()
                .expect("new skeleton equipment is uncontended")
                .equipment
                .insert(
                    EquipmentSlot::MAIN_HAND,
                    Arc::new(tokio::sync::Mutex::new(ItemStack::new(1, main_hand))),
                );

            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();
            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            if uses_bow {
                // Vanilla `AbstractSkeleton#reassessWeaponGoal` selects this at priority 4.
                goal_selector.add_goal(4, Box::new(RangedBowAttackGoal::new(20, 15.0)));
            } else {
                goal_selector.add_goal(4, Box::new(MeleeAttackGoal::new(1.2, false)));
            }
            goal_selector.add_goal(7, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                8,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(8, Box::new(RandomLookAroundGoal::default()));

            target_selector.add_goal(1, Box::new(RevengeGoal::new(true)));
            target_selector.add_goal(
                2,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PLAYER, true),
            );
        };

        mob_arc
    }
}

impl NBTStorage for SkeletonEntityBase {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        self.mob_entity.living_entity.write_nbt(nbt)
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        self.mob_entity.living_entity.read_nbt_non_mut(nbt)
    }
}

impl Mob for SkeletonEntityBase {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}
