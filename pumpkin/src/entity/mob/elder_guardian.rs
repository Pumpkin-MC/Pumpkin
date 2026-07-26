use std::sync::atomic::Ordering::Relaxed;
use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;
use pumpkin_data::potion::Effect;
use pumpkin_data::{effect::StatusEffect, sound::Sound, sound::SoundCategory};

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage,
    ai::goal::{
        active_target::ActiveTargetGoal, guardian_laser::GuardianLaserGoal,
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal, revenge::RevengeGoal,
        swim::SwimGoal, wander_around::WanderAroundGoal,
    },
    ai::pathfinder::node::PathType,
    mob::{Mob, MobEntity},
};

/// Elder guardian — laser + periodic Mining Fatigue III pulse (vanilla curse).
pub struct ElderGuardianEntity {
    pub mob_entity: MobEntity,
}

impl ElderGuardianEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        {
            let mut nav = mob_entity.navigator.lock().unwrap();
            nav.set_pathfinding_malus(PathType::Water, 0.0);
            nav.set_pathfinding_malus(PathType::WaterBorder, 0.0);
        }
        let guardian = Self { mob_entity };
        let mob_arc = Arc::new(guardian);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();
            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(4, GuardianLaserGoal::new(1.0));
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                8,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(9, Box::new(RandomLookAroundGoal::default()));

            target_selector.add_goal(1, Box::new(RevengeGoal::new(true)));
            target_selector.add_goal(
                2,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PLAYER, true),
            );
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::SQUID, true),
            );
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::GLOW_SQUID, true),
            );
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::AXOLOTL, true),
            );
        };

        mob_arc
    }

    /// Vanilla: every ~60s apply Mining Fatigue III to players in 50-block range.
    async fn mining_fatigue_pulse(&self) {
        let living = &self.mob_entity.living_entity;
        if !living.is_alive() {
            return;
        }
        let world = living.entity.world.load();
        let pos = living.entity.pos.load();
        world.play_sound(
            Sound::EntityElderGuardianCurse,
            SoundCategory::Hostile,
            &pos,
        );
        for player in world.get_nearby_players(pos, 50.0) {
            if player.is_spectator() || player.is_creative() {
                continue;
            }
            player
                .add_effect(Effect {
                    effect_type: &StatusEffect::MINING_FATIGUE,
                    duration: 6000, // 5 minutes
                    amplifier: 2,   // III
                    ambient: false,
                    show_particles: true,
                    show_icon: true,
                    blend: false,
                })
                .await;
        }
    }
}

impl NBTStorage for ElderGuardianEntity {
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

impl Mob for ElderGuardianEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            // ~once per 1200 ticks (60s).
            let age = self.mob_entity.living_entity.entity.age.load(Relaxed);
            if age > 0 && age % 1200 == 0 {
                self.mining_fatigue_pulse().await;
            }
        })
    }
}
