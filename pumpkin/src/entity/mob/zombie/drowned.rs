use std::sync::Arc;

use pumpkin_data::entity::EntityType;

use crate::entity::ai::goal::active_target::ActiveTargetGoal;
use crate::entity::ai::pathfinder::node::PathType;
use crate::entity::mob::zombie::ZombieEntityBase;
use crate::entity::{
    Entity, NBTStorage,
    mob::{Mob, MobEntity},
};

/// Drowned — vanilla addBehaviourGoals: water/trident/beach + axolotl target.
/// Inherits shared zombie goals then adds axolotl (decompile Drowned.java).
pub struct DrownedEntity {
    entity: Arc<ZombieEntityBase>,
}

impl DrownedEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let entity = ZombieEntityBase::new(entity);
        {
            let mut nav = entity.mob_entity.navigator.lock().unwrap();
            nav.set_pathfinding_malus(PathType::Water, 0.0);
            nav.set_pathfinding_malus(PathType::WaterBorder, 0.0);
            nav.set_pathfinding_malus(PathType::Open, 2.0);
        }
        // NearestAttackableTargetGoal(Axolotl) priority 3
        {
            let mut target_selector = entity.mob_entity.target_selector.lock().unwrap();
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(&entity.mob_entity, &EntityType::AXOLOTL, true),
            );
        }
        Arc::new(Self { entity })
    }
}

impl NBTStorage for DrownedEntity {
    fn write_nbt<'a>(
        &'a self,
        nbt: &'a mut pumpkin_nbt::compound::NbtCompound,
    ) -> crate::entity::NbtFuture<'a, ()> {
        self.entity.write_nbt(nbt)
    }

    fn read_nbt_non_mut<'a>(
        &'a self,
        nbt: &'a pumpkin_nbt::compound::NbtCompound,
    ) -> crate::entity::NbtFuture<'a, ()> {
        self.entity.read_nbt_non_mut(nbt)
    }
}

impl Mob for DrownedEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity.mob_entity
    }
}
