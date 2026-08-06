use std::sync::{Arc, Weak};

use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use tokio::sync::Mutex;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage,
    ai::goal::{
        active_target::ActiveTargetGoal,
        avoid_entity::AvoidEntityGoal,
        illusioner_spell::{
            IllusionerBlindnessSpellGoal, IllusionerCastingSpellGoal, IllusionerMirrorSpellGoal,
        },
        look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal,
        ranged_bow_attack::RangedBowAttackGoal,
        revenge::RevengeGoal,
        spellcaster::SpellcasterState,
        swim::SwimGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

pub struct IllusionerEntity {
    pub mob_entity: MobEntity,
    /// Vanilla: `SpellcasterIllager.spellCastingTickCount` / `currentSpell`.
    pub spellcaster: SpellcasterState,
}

impl IllusionerEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let illusioner = Self {
            mob_entity,
            spellcaster: SpellcasterState::new(),
        };
        let mob_arc = Arc::new(illusioner);
        mob_arc
            .mob_entity
            .living_entity
            .entity_equipment
            .try_lock()
            .expect("new illusioner equipment is uncontended")
            .equipment
            .insert(
                EquipmentSlot::MAIN_HAND,
                Arc::new(Mutex::new(ItemStack::new(1, &Item::BOW))),
            );
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };
        let illusioner_weak = Arc::downgrade(&mob_arc);

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(
                1,
                Box::new(IllusionerCastingSpellGoal::new(illusioner_weak.clone())),
            );
            goal_selector.add_goal(
                3,
                Box::new(AvoidEntityGoal::new(&EntityType::CREAKING, 8.0, 1.0, 1.2)),
            );
            goal_selector.add_goal(
                4,
                Box::new(IllusionerMirrorSpellGoal::new(illusioner_weak.clone())),
            );
            goal_selector.add_goal(
                5,
                Box::new(IllusionerBlindnessSpellGoal::new(illusioner_weak)),
            );
            goal_selector.add_goal(6, Box::new(RangedBowAttackGoal::new(20, 15.0)));
            goal_selector.add_goal(8, Box::new(WanderAroundGoal::new(0.6)));
            goal_selector.add_goal(
                9,
                LookAtEntityGoal::with_default(mob_weak.clone(), &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(10, Box::new(RandomLookAroundGoal::default()));

            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();
            target_selector.add_goal(1, Box::new(RevengeGoal::new(true)));
            target_selector.add_goal(
                1,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PLAYER, true),
            );
            target_selector.add_goal(
                2,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::VILLAGER, true),
            );
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::IRON_GOLEM, true),
            );
        };

        mob_arc
    }
}

impl NBTStorage for IllusionerEntity {}

impl Mob for IllusionerEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    /// Vanilla: `SpellcasterIllager.customServerAiStep`.
    fn mob_tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            self.spellcaster.tick();
        })
    }

    // Vanilla `Illusioner.applyRaidBuffs` is an empty override; the `Mob` trait's no-op default
    // is already correct parity.
}
