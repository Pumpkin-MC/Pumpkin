use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage,
    ai::goal::{
        active_target::ActiveTargetGoal,
        evoker_spell::{
            EvokerAttackSpellGoal, EvokerCastingSpellGoal, EvokerSummonSpellGoal,
            EvokerWololoSpellGoal,
        },
        look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal,
        revenge::RevengeGoal,
        spellcaster::SpellcasterState,
        swim::SwimGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

pub struct EvokerEntity {
    pub mob_entity: MobEntity,
    /// Vanilla: `SpellcasterIllager.spellCastingTickCount` / `currentSpell`.
    pub spellcaster: SpellcasterState,
    /// Vanilla: `Evoker.wololoTarget`.
    pub wololo_target: tokio::sync::Mutex<Option<Arc<dyn EntityBase>>>,
}

impl EvokerEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let evoker = Self {
            mob_entity,
            spellcaster: SpellcasterState::new(),
            wololo_target: tokio::sync::Mutex::new(None),
        };
        let mob_arc = Arc::new(evoker);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };
        let evoker_weak = Arc::downgrade(&mob_arc);

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(
                1,
                Box::new(EvokerCastingSpellGoal::new(evoker_weak.clone())),
            );
            // Scope reduction: vanilla also registers `AvoidEntityGoal<Player>` (priority 2) and
            // `AvoidEntityGoal<Creaking>` (priority 3) here; that's an unrelated pre-existing gap,
            // not part of evoker spellcasting.
            goal_selector.add_goal(4, Box::new(EvokerSummonSpellGoal::new(evoker_weak.clone())));
            goal_selector.add_goal(5, Box::new(EvokerAttackSpellGoal::new(evoker_weak.clone())));
            goal_selector.add_goal(6, Box::new(EvokerWololoSpellGoal::new(evoker_weak)));
            goal_selector.add_goal(8, Box::new(WanderAroundGoal::new(1.0)));
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

impl NBTStorage for EvokerEntity {}

impl Mob for EvokerEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    /// Vanilla: `SpellcasterIllager.customServerAiStep`.
    fn mob_tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            self.spellcaster.tick();
        })
    }
}
