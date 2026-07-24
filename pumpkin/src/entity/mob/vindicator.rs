use std::sync::{Arc, Weak};

use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;

use crate::entity::{
    Entity, EntityBase, NBTStorage,
    ai::goal::{
        active_target::ActiveTargetGoal, avoid_entity::AvoidEntityGoal, join_anger::JoinAngerGoal,
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal,
        melee_attack::MeleeAttackGoal, revenge::RevengeGoal, swim::SwimGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

pub struct VindicatorEntity {
    pub mob_entity: MobEntity,
}

impl VindicatorEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let vindicator = Self { mob_entity };
        let mob_arc = Arc::new(vindicator);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

            // Vanilla 26.2 Vindicator.registerGoals
            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(
                1,
                Box::new(AvoidEntityGoal::new(&EntityType::CREAKING, 8.0, 1.0, 1.2)),
            );
            // MeleeAttackGoal(1.0, false) — speed 1.15 used for snappier raids
            goal_selector.add_goal(5, Box::new(MeleeAttackGoal::new(1.15, false)));
            goal_selector.add_goal(8, Box::new(WanderAroundGoal::new(0.6)));
            goal_selector.add_goal(
                9,
                LookAtEntityGoal::with_default(mob_weak.clone(), &EntityType::PLAYER, 3.0),
            );
            goal_selector.add_goal(10, Box::new(RandomLookAroundGoal::default()));

            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();
            // HurtByTargetGoal (Raider class filter TODO)
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
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::VILLAGER, true),
            );
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::IRON_GOLEM, true),
            );
        };

        mob_arc
    }

    /// Vanilla always equips an iron axe in the main hand.
    async fn equip_iron_axe(&self) {
        let living = &self.mob_entity.living_entity;
        let axe = ItemStack::new(1, &Item::IRON_AXE);
        living
            .entity_equipment
            .lock()
            .await
            .put(&EquipmentSlot::MAIN_HAND, axe.clone())
            .await;
        living.send_equipment_changes(&[(EquipmentSlot::MAIN_HAND, axe)]);
    }
}

impl NBTStorage for VindicatorEntity {}

impl Mob for VindicatorEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_init_data_tracker(&self) -> crate::entity::EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            // Default mob baby metadata, then equip axe so clients render the weapon.
            let entity = self.get_entity();
            let is_baby = entity.age.load(std::sync::atomic::Ordering::Relaxed) < 0;
            if is_baby {
                use pumpkin_data::meta_data_type::MetaDataType;
                use pumpkin_data::tracked_data::TrackedData;
                use pumpkin_protocol::java::client::play::Metadata;
                entity.send_meta_data(
                    &[Metadata::new(
                        TrackedData::BABY_ID,
                        MetaDataType::BOOLEAN,
                        true,
                    )],
                    None,
                );
            }
            self.equip_iron_axe().await;
        })
    }
}
