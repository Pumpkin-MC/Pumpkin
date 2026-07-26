use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::{
        breed::BreedGoal, follow_parent::FollowParentGoal, join_anger::JoinAngerGoal,
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal,
        melee_attack::MeleeAttackGoal, revenge::RevengeGoal, swim::SwimGoal, tempt::TemptGoal,
        wander_around::WanderAroundGoal,
    },
    ai::pathfinder::node::PathType,
    mob::{Mob, MobEntity},
};

// Vanilla ItemTags.BEE_FOOD — common flowers stand-in.
const TEMPT_ITEMS: &[&Item] = &[
    &Item::DANDELION,
    &Item::POPPY,
    &Item::ALLIUM,
    &Item::CORNFLOWER,
    &Item::TORCHFLOWER,
    &Item::SUNFLOWER,
    &Item::LILAC,
    &Item::ROSE_BUSH,
    &Item::PEONY,
];

/// Bee — vanilla 26.2 GoalSelector (hive/pollinate Brain-ish goals TODO).
///
/// Decompile `Bee.registerGoals`: Attack, EnterHive, Breed, Tempt, Pollinate,
/// FollowParent, Wander, Float; HurtBy.setAlertOthers + BecomeAngry.
pub struct BeeEntity {
    pub mob_entity: MobEntity,
}

impl BeeEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        {
            let mut nav = mob_entity.navigator.lock().unwrap();
            nav.set_pathfinding_malus(PathType::Water, 16.0);
            nav.set_pathfinding_malus(PathType::WaterBorder, 8.0);
        }
        let bee = Self { mob_entity };
        let mob_arc = Arc::new(bee);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();
            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();

            // 0 BeeAttackGoal
            goal_selector.add_goal(0, Box::new(MeleeAttackGoal::new(1.4, true)));
            // 1 BeeEnterHiveGoal TODO
            goal_selector.add_goal(2, BreedGoal::new(1.0));
            goal_selector.add_goal(3, Box::new(TemptGoal::new(1.25, TEMPT_ITEMS)));
            // 4 BeePollinateGoal TODO
            goal_selector.add_goal(5, Box::new(FollowParentGoal::new(1.25)));
            // 5–7 hive/flower/crop goals TODO
            goal_selector.add_goal(8, Box::new(WanderAroundGoal::new(1.0)));
            // 9 FloatGoal — last priority in vanilla
            goal_selector.add_goal(9, Box::new(SwimGoal::default()));
            goal_selector.add_goal(
                10,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(10, Box::new(RandomLookAroundGoal::default()));

            // BeeHurtByOther.setAlertOthers + BecomeAngry stand-in
            target_selector.add_goal(1, Box::new(RevengeGoal::new(true)));
            target_selector.add_goal(1, JoinAngerGoal::new(&EntityType::BEE));
        };

        mob_arc
    }
}

impl NBTStorage for BeeEntity {
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

impl Mob for BeeEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn get_mob_gravity(&self) -> f64 {
        0.05 // light float
    }
}
