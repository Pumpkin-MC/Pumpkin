use std::sync::{Arc, Weak};

use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::{
        active_target::ActiveTargetGoal, avoid_entity::AvoidEntityGoal, bow_attack::BowAttackGoal,
        join_anger::JoinAngerGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, revenge::RevengeGoal, swim::SwimGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

/// Pillager — crossbow stand-in via bow AI + raid targets.
pub struct PillagerEntity {
    pub mob_entity: MobEntity,
}

impl PillagerEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let pillager = Self { mob_entity };
        let mob_arc = Arc::new(pillager);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();
            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();

            // Vanilla 26.2: RangedCrossbowAttackGoal — use bow goal as stand-in.
            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(
                1,
                Box::new(AvoidEntityGoal::new(&EntityType::CREAKING, 8.0, 1.0, 1.2)),
            );
            goal_selector.add_goal(3, BowAttackGoal::new(1.0, 20));
            goal_selector.add_goal(8, Box::new(WanderAroundGoal::new(0.6)));
            goal_selector.add_goal(
                9,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 15.0),
            );
            goal_selector.add_goal(10, Box::new(RandomLookAroundGoal::default()));

            target_selector.add_goal(1, Box::new(RevengeGoal::new(true)));
            // Raid pack anger (vanilla HurtByTarget setAlertOthers illagers).
            target_selector.add_goal(2, JoinAngerGoal::new(&EntityType::PILLAGER));
            target_selector.add_goal(2, JoinAngerGoal::new(&EntityType::VINDICATOR));
            target_selector.add_goal(2, JoinAngerGoal::new(&EntityType::EVOKER));
            target_selector.add_goal(2, JoinAngerGoal::new(&EntityType::ILLUSIONER));
            target_selector.add_goal(2, JoinAngerGoal::new(&EntityType::RAVAGER));
            target_selector.add_goal(2, JoinAngerGoal::new(&EntityType::WITCH));
            target_selector.add_goal(
                2,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PLAYER, true),
            );
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::VILLAGER, false),
            );
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::IRON_GOLEM, true),
            );
        };

        mob_arc
    }

    async fn equip_crossbow(&self) {
        if crate::entity::mob::equipment::EQUIPMENT_REGISTRY
            .contains_key(self.get_entity().entity_type.resource_name)
        {
            return;
        }

        let living = &self.mob_entity.living_entity;
        let stack = ItemStack::new(1, &Item::CROSSBOW);
        living
            .entity_equipment
            .lock()
            .await
            .put(&EquipmentSlot::MAIN_HAND, stack.clone())
            .await;
        living.send_equipment_changes(&[(EquipmentSlot::MAIN_HAND, stack)]);
    }
}

impl NBTStorage for PillagerEntity {}

impl Mob for PillagerEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_init_data_tracker(&self) -> crate::entity::EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            self.equip_crossbow().await;
        })
    }
}
