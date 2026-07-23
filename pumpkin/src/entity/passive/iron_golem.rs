use std::sync::{Arc, Weak};

use pumpkin_data::entity::{EntityType, MobCategory};
use pumpkin_nbt::compound::NbtCompound;

use crate::entity::{
    Entity, NBTStorage, NbtFuture,
    ai::goal::{
        active_target::ActiveTargetGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, melee_attack::MeleeAttackGoal, revenge::RevengeGoal,
        wander_around::WanderAroundGoal,
    },
    ai::pathfinder::node::PathType,
    mob::{Mob, MobEntity},
};

/// Iron golem — protects villagers; attacks most hostile monsters (not creepers).
///
/// Vanilla attack (`IronGolem.doHurtTarget`) is handled in `MobEntity::try_attack`:
/// entity status `START_ATTACKING` (4) for both-arms raise, random damage, sound.
///
/// Pathfinding: water is treated as **impassable** so golems never walk into rivers
/// / ponds when a target is knocked into water (they stay on the bank).
///
/// Wiki: <https://minecraft.wiki/w/Iron_Golem>
pub struct IronGolemEntity {
    pub mob_entity: MobEntity,
}

/// Vanilla iron golem never acquires creepers as active targets.
/// Wardens **are** valid targets in vanilla (Monster category).
const GOLEM_EXCLUDE: &[&EntityType] = &[&EntityType::CREEPER];

impl IronGolemEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);

        // Never path through water (malus < 0 = blocked in WalkNodeEvaluator).
        {
            let mut nav = mob_entity.navigator.lock().unwrap();
            nav.set_pathfinding_malus(PathType::Water, -1.0);
            nav.set_pathfinding_malus(PathType::WaterBorder, -1.0);
        }

        let iron_golem = Self { mob_entity };
        let mob_arc = Arc::new(iron_golem);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();
            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();

            // pause_when_mob_idle=true: if path fails (e.g. target mid-lake), stop
            // thrashing instead of direct-walking into the water.
            goal_selector.add_goal(1, Box::new(MeleeAttackGoal::new(1.0, true)));
            goal_selector.add_goal(6, Box::new(WanderAroundGoal::new(0.6)));
            goal_selector.add_goal(
                7,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(8, Box::new(RandomLookAroundGoal::default()));

            target_selector.add_goal(1, Box::new(RevengeGoal::new(true)));
            target_selector.add_goal(
                2,
                ActiveTargetGoal::for_category(
                    &mob_arc.mob_entity,
                    &MobCategory::MONSTER,
                    GOLEM_EXCLUDE,
                    5,
                    false,
                ),
            );
        };

        mob_arc
    }
}

impl NBTStorage for IronGolemEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        self.mob_entity.living_entity.write_nbt(nbt)
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        self.mob_entity.living_entity.read_nbt_non_mut(nbt)
    }
}

impl Mob for IronGolemEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}
