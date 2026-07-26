use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;

use crate::entity::ai::goal::door_interact::OpenDoorGoal;
use crate::entity::{
    Entity, NBTStorage,
    ai::goal::{
        active_target::ActiveTargetGoal, join_anger::JoinAngerGoal,
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal,
        melee_attack::MeleeAttackGoal, revenge::RevengeGoal, swim::SwimGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

/// Piglin brute — always hostile melee (bastion brain TODO).
pub struct PiglinBruteEntity {
    pub mob_entity: MobEntity,
}

impl PiglinBruteEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let piglin = Self { mob_entity };
        let mob_arc = Arc::new(piglin);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            // Vanilla AbstractPiglin.applyOpenDoorsAbility.
            let mut navigator = mob_arc.mob_entity.navigator.lock().unwrap();
            navigator.set_can_open_doors(true);
        }

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();
            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            // Brain InteractWithDoor stand-in: open doors on the way through.
            goal_selector.add_goal(2, Box::new(OpenDoorGoal::new(false)));
            // Vanilla: brutes do NOT flee zombified piglins (unlike regular piglins).
            goal_selector.add_goal(1, Box::new(MeleeAttackGoal::new(1.0, true)));
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(0.6)));
            goal_selector.add_goal(
                8,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(8, Box::new(RandomLookAroundGoal::default()));

            target_selector.add_goal(1, Box::new(RevengeGoal::new(true)));
            // Bastion pack: join nearby brute anger.
            target_selector.add_goal(2, JoinAngerGoal::new(&EntityType::PIGLIN_BRUTE));
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PLAYER, true),
            );
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(
                    &mob_arc.mob_entity,
                    &EntityType::WITHER_SKELETON,
                    true,
                ),
            );
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::WITHER, true),
            );
        };

        mob_arc
    }
}

impl NBTStorage for PiglinBruteEntity {
    fn write_nbt<'a>(
        &'a self,
        nbt: &'a mut pumpkin_nbt::compound::NbtCompound,
    ) -> crate::entity::NbtFuture<'a, ()> {
        self.get_mob_entity().living_entity.write_nbt(nbt)
    }

    fn read_nbt_non_mut<'a>(
        &'a self,
        nbt: &'a pumpkin_nbt::compound::NbtCompound,
    ) -> crate::entity::NbtFuture<'a, ()> {
        self.get_mob_entity().living_entity.read_nbt_non_mut(nbt)
    }
}

impl Mob for PiglinBruteEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}
