use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::{
        active_target::ActiveTargetGoal, escape_danger::EscapeDangerGoal,
        leap_at_target::LeapAtTargetGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, melee_attack::MeleeAttackGoal, swim::SwimGoal,
        tempt::TemptGoal, wander_around::WanderAroundGoal,
    },
    ai::pathfinder::node::PathType,
    mob::{Mob, MobEntity},
};

const TEMPT_ITEMS: &[&Item] = &[&Item::TROPICAL_FISH_BUCKET];

/// Axolotl — **Brain** in vanilla (`AxolotlAi`); GoalSelector hunt/tempt stand-in (play-dead TODO).
pub struct AxolotlEntity {
    pub mob_entity: MobEntity,
}

impl AxolotlEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        {
            let mut nav = mob_entity.navigator.lock().unwrap();
            nav.set_pathfinding_malus(PathType::Water, 0.0);
            nav.set_pathfinding_malus(PathType::WaterBorder, 0.0);
        }
        let axolotl = Self { mob_entity };
        let mob_arc = Arc::new(axolotl);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();
            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            // Play-dead not ported — panic when hurt.
            goal_selector.add_goal(1, EscapeDangerGoal::new(1.0));
            goal_selector.add_goal(2, Box::new(TemptGoal::new(0.9, TEMPT_ITEMS)));
            goal_selector.add_goal(2, Box::new(LeapAtTargetGoal::new(0.3)));
            goal_selector.add_goal(3, Box::new(MeleeAttackGoal::new(1.0, true)));
            goal_selector.add_goal(4, Box::new(WanderAroundGoal::new(0.8)));
            goal_selector.add_goal(
                5,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(6, Box::new(RandomLookAroundGoal::default()));

            // Hunt drowned, fish, squid, guardians (vanilla aquatic prey set).
            for (prio, ty) in [
                (1, &EntityType::DROWNED),
                (1, &EntityType::GUARDIAN),
                (1, &EntityType::ELDER_GUARDIAN),
                (2, &EntityType::SQUID),
                (2, &EntityType::GLOW_SQUID),
                (2, &EntityType::COD),
                (2, &EntityType::SALMON),
                (2, &EntityType::TROPICAL_FISH),
                (2, &EntityType::PUFFERFISH),
                (2, &EntityType::TADPOLE),
            ] {
                target_selector.add_goal(
                    prio,
                    ActiveTargetGoal::with_default(&mob_arc.mob_entity, ty, true),
                );
            }
        };

        mob_arc
    }
}

impl NBTStorage for AxolotlEntity {
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

impl Mob for AxolotlEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}
