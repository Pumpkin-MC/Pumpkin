use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering::Relaxed};

use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_data::tag::{self, Taggable};

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage,
    ai::goal::{
        active_target::ActiveTargetGoal, climb_on_top_of_powder_snow::ClimbOnTopOfPowderSnowGoal,
        melee_attack::MeleeAttackGoal, revenge::RevengeGoal,
        silverfish_merge_with_stone::SilverfishMergeWithStoneGoal,
        silverfish_wake_up_friends::SilverfishWakeUpFriendsGoal, swim::SwimGoal,
    },
    mob::{Mob, MobEntity},
};

/// Vanilla `Silverfish`. Source: `net/minecraft/world/entity/monster/Silverfish.java`.
pub struct SilverfishEntity {
    entity: Arc<MobEntity>,
    /// Vanilla `Silverfish.friendsGoal` countdown (`SilverfishWakeUpFriendsGoal.lookForFriends`),
    /// armed by `notify_hurt` and consumed by `SilverfishWakeUpFriendsGoal`.
    pub wake_up_friends_timer: AtomicI32,
}

impl SilverfishEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let entity = Arc::new(MobEntity::new(entity));
        let silverfish = Self {
            entity,
            wake_up_friends_timer: AtomicI32::new(0),
        };
        let mob_arc = Arc::new(silverfish);

        {
            let mut goal_selector = mob_arc.entity.goals_selector.lock().unwrap();
            let mut target_selector = mob_arc.entity.target_selector.lock().unwrap();
            target_selector.add_goal(1, Box::new(RevengeGoal::new(true)));

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, ClimbOnTopOfPowderSnowGoal::new());
            goal_selector.add_goal(
                3,
                Box::new(SilverfishWakeUpFriendsGoal::new(mob_arc.clone())),
            );
            goal_selector.add_goal(4, Box::new(MeleeAttackGoal::new(1.0, false)));
            goal_selector.add_goal(5, Box::new(SilverfishMergeWithStoneGoal::new()));

            target_selector.add_goal(
                2,
                ActiveTargetGoal::with_default(&mob_arc.entity, &EntityType::PLAYER, true),
            );
        };

        mob_arc
    }

    /// Vanilla `Silverfish.SilverfishWakeUpFriendsGoal.notifyHurt`.
    fn notify_hurt(&self) {
        if self.wake_up_friends_timer.load(Relaxed) == 0 {
            self.wake_up_friends_timer
                .store(crate::entity::ai::goal::to_goal_ticks(20), Relaxed);
        }
    }
}

impl NBTStorage for SilverfishEntity {}

impl Mob for SilverfishEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity
    }

    /// Vanilla `Silverfish.hurtServer`: only alerts nearby infested blocks when the damage came
    /// from an entity, or is tagged `#minecraft:always_triggers_silverfish`.
    fn on_damage<'a>(
        &'a self,
        damage_type: DamageType,
        source: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            if source.is_some()
                || damage_type.has_tag(&tag::DamageType::MINECRAFT_ALWAYS_TRIGGERS_SILVERFISH)
            {
                self.notify_hurt();
            }
        })
    }
}
