use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::{
        avoid_entity::AvoidEntityGoal, escape_danger::EscapeDangerGoal,
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal, swim::SwimGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

/// Wandering trader — panics and flees undead (vanilla AvoidEntity goals),
/// drinks invisibility at dusk and milk at dawn.
pub struct WanderingTraderEntity {
    pub mob_entity: MobEntity,
    /// Throttles the day/night potion check to once a second.
    invisibility_check_cooldown: std::sync::atomic::AtomicU8,
}

impl WanderingTraderEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let trader = Self {
            mob_entity,
            invisibility_check_cooldown: std::sync::atomic::AtomicU8::new(0),
        };
        let mob_arc = Arc::new(trader);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, EscapeDangerGoal::new(0.5));
            // Vanilla: avoid zombies / husks / drowned / zombie villagers / zoglins.
            goal_selector.add_goal(
                1,
                Box::new(AvoidEntityGoal::new(&EntityType::ZOMBIE, 8.0, 0.5, 0.5)),
            );
            goal_selector.add_goal(
                1,
                Box::new(AvoidEntityGoal::new(&EntityType::HUSK, 8.0, 0.5, 0.5)),
            );
            goal_selector.add_goal(
                1,
                Box::new(AvoidEntityGoal::new(&EntityType::DROWNED, 8.0, 0.5, 0.5)),
            );
            goal_selector.add_goal(
                1,
                Box::new(AvoidEntityGoal::new(
                    &EntityType::ZOMBIE_VILLAGER,
                    8.0,
                    0.5,
                    0.5,
                )),
            );
            goal_selector.add_goal(
                1,
                Box::new(AvoidEntityGoal::new(&EntityType::ZOGLIN, 10.0, 0.5, 0.5)),
            );
            goal_selector.add_goal(
                1,
                Box::new(AvoidEntityGoal::new(
                    &EntityType::ILLUSIONER,
                    12.0,
                    0.5,
                    0.5,
                )),
            );
            goal_selector.add_goal(
                1,
                Box::new(AvoidEntityGoal::new(&EntityType::VINDICATOR, 8.0, 0.5, 0.5)),
            );
            goal_selector.add_goal(
                1,
                Box::new(AvoidEntityGoal::new(&EntityType::EVOKER, 12.0, 0.5, 0.5)),
            );
            goal_selector.add_goal(
                1,
                Box::new(AvoidEntityGoal::new(&EntityType::PILLAGER, 15.0, 0.5, 0.5)),
            );
            goal_selector.add_goal(
                1,
                Box::new(AvoidEntityGoal::new(&EntityType::VEX, 8.0, 0.5, 0.5)),
            );
            goal_selector.add_goal(2, Box::new(WanderAroundGoal::new(0.35)));
            goal_selector.add_goal(
                3,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 3.0),
            );
            goal_selector.add_goal(3, Box::new(RandomLookAroundGoal::default()));
        };

        mob_arc
    }
}

impl NBTStorage for WanderingTraderEntity {
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

impl Mob for WanderingTraderEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    /// Vanilla WanderingTrader UseItemGoal pair: drink an invisibility potion
    /// at nightfall and milk at dawn (simplified to direct effect toggling with
    /// the drink sounds).
    fn mob_tick<'a>(
        &'a self,
        _caller: &'a Arc<dyn crate::entity::EntityBase>,
    ) -> crate::entity::EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            use std::sync::atomic::Ordering;
            let cooldown = self
                .invisibility_check_cooldown
                .fetch_add(1, Ordering::Relaxed);
            if cooldown < 20 {
                return;
            }
            self.invisibility_check_cooldown.store(0, Ordering::Relaxed);

            let living = &self.mob_entity.living_entity;
            let world = living.entity.world.load_full();
            // Vanilla night window for the invisibility drink.
            let time = world.get_time_of_day().await % 24000;
            let is_night = (13000..23000).contains(&time);
            let invisible = living
                .has_effect(&pumpkin_data::effect::StatusEffect::INVISIBILITY)
                .await;

            if is_night && !invisible {
                living
                    .add_effect(pumpkin_data::potion::Effect {
                        effect_type: &pumpkin_data::effect::StatusEffect::INVISIBILITY,
                        // Covers the night; milk clears it at dawn.
                        duration: 11000,
                        amplifier: 0,
                        ambient: false,
                        show_particles: true,
                        show_icon: true,
                        blend: false,
                    })
                    .await;
                world.play_sound_fine(
                    pumpkin_data::sound::Sound::EntityWanderingTraderDrinkPotion,
                    pumpkin_data::sound::SoundCategory::Neutral,
                    &living.entity.pos.load(),
                    1.0,
                    1.0,
                );
            } else if !is_night && invisible {
                living
                    .remove_effect(&pumpkin_data::effect::StatusEffect::INVISIBILITY)
                    .await;
                world.play_sound_fine(
                    pumpkin_data::sound::Sound::EntityWanderingTraderDrinkMilk,
                    pumpkin_data::sound::SoundCategory::Neutral,
                    &living.entity.pos.load(),
                    1.0,
                    1.0,
                );
            }
        })
    }
}
