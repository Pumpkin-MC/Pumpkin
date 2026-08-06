use std::sync::atomic::{AtomicI32, Ordering::Relaxed};
use std::sync::{Arc, Weak};

use pumpkin_data::attributes::Attributes;
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::{EntityStatus, EntityType};
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::{lerp, position::BlockPos, vector3::Vector3};

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture,
    ai::goal::{
        active_target::ActiveTargetGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, melee_attack::MeleeAttackGoal, revenge::RevengeGoal,
        swim::SwimGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

/// Vanilla: the four `AbstractIllager`-family raiders that a roaring Ravager never damages
/// (`entity instanceof AbstractIllager`). Vex extends `Monster`, not `AbstractIllager`, and is
/// deliberately not in this list.
fn is_illager(entity_type: &'static EntityType) -> bool {
    entity_type == &EntityType::VINDICATOR
        || entity_type == &EntityType::PILLAGER
        || entity_type == &EntityType::ILLUSIONER
        || entity_type == &EntityType::EVOKER
}

pub struct RavagerEntity {
    pub mob_entity: MobEntity,
    /// Vanilla: `Ravager.attackTick`.
    attack_tick: AtomicI32,
    /// Vanilla: `Ravager.stunnedTick`.
    stunned_tick: AtomicI32,
    /// Vanilla: `Ravager.roarTick`.
    roar_tick: AtomicI32,
}

impl RavagerEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let ravager = Self {
            mob_entity,
            attack_tick: AtomicI32::new(0),
            stunned_tick: AtomicI32::new(0),
            roar_tick: AtomicI32::new(0),
        };
        let mob_arc = Arc::new(ravager);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(4, Box::new(MeleeAttackGoal::new(1.0, true)));
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new_water_avoiding(0.4)));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak.clone(), &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(7, Box::new(RandomLookAroundGoal::default()));

            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();
            target_selector.add_goal(2, Box::new(RevengeGoal::new(true)));
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PLAYER, true),
            );
            target_selector.add_goal(
                4,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::VILLAGER, true),
            );
            target_selector.add_goal(
                4,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::IRON_GOLEM, true),
            );
        };

        mob_arc
    }

    /// Vanilla: `Ravager.isImmobile` (`super.isImmobile() || attackTick > 0 || stunnedTick > 0 ||
    /// roarTick > 0`).
    ///
    /// Scope reduction: no generic goal in Pumpkin currently consults a mob-level
    /// "is immobile"/"has line of sight" hook (each ranged/melee goal implements its own local
    /// line-of-sight raycast instead), so these three counters correctly track state and drive
    /// the movement-speed lerp/roar/stun behavior below, but do not yet pause
    /// movement/attacking through a shared override point -- there isn't one to hook into.
    fn is_immobile(&self) -> bool {
        self.attack_tick.load(Relaxed) > 0
            || self.stunned_tick.load(Relaxed) > 0
            || self.roar_tick.load(Relaxed) > 0
    }

    /// Vanilla: `Ravager.aiStep` step 1 -- lerps `MOVEMENT_SPEED`'s base value toward `0.35`
    /// (has target) or `0.3` (idle) at rate `Mth.lerp(0.1, current, max)`, or forces it to `0.0`
    /// while immobile.
    async fn tick_movement_speed(&self) {
        let living = &self.mob_entity.living_entity;
        if self.is_immobile() {
            living.set_attribute_base(&Attributes::MOVEMENT_SPEED, 0.0);
        } else {
            let has_target = self.mob_entity.target.lock().await.is_some();
            let max_speed = if has_target { 0.35 } else { 0.3 };
            let base = living.get_attribute_base(&Attributes::MOVEMENT_SPEED);
            living.set_attribute_base(&Attributes::MOVEMENT_SPEED, lerp(0.1, base, max_speed));
        }
    }

    /// Vanilla: `Ravager.aiStep` step 2 -- destroys leaves in an inflated bounding box while
    /// colliding horizontally, if `MOB_GRIEFING` is on.
    async fn tick_leaf_destroy(&self) {
        let entity = &self.mob_entity.living_entity.entity;
        if !entity.horizontal_collision.load(Relaxed) {
            return;
        }
        let world = entity.world.load_full();
        if !world.level_info.load().game_rules.mob_griefing {
            return;
        }

        let bb = entity.bounding_box.load().expand(0.2, 0.2, 0.2);
        let (min_x, min_y, min_z) = (
            bb.min.x.floor() as i32,
            bb.min.y.floor() as i32,
            bb.min.z.floor() as i32,
        );
        let (max_x, max_y, max_z) = (
            bb.max.x.floor() as i32,
            bb.max.y.floor() as i32,
            bb.max.z.floor() as i32,
        );

        for x in min_x..=max_x {
            for y in min_y..=max_y {
                for z in min_z..=max_z {
                    let pos = BlockPos::new(x, y, z);
                    if world.get_block(&pos).has_tag(&tag::Block::MINECRAFT_LEAVES) {
                        world
                            .break_block(&pos, None, pumpkin_world::world::BlockFlags::empty())
                            .await;
                    }
                }
            }
        }
        // Scope reduction: vanilla additionally `jumpFromGround()`s if nothing was destroyed and
        // the ravager is on the ground; `LivingEntity::jump` is private to `living.rs`, so this
        // small obstacle-hop flavor behavior is not ported.
    }

    /// Vanilla: `Ravager.roar`. Area-of-effect 6-damage hit + strong knockback (except players
    /// and other ravagers; illagers are never damaged) in a 4-block-inflated box.
    async fn roar(&self, caller: &Arc<dyn EntityBase>) {
        if !caller.get_entity().is_alive() {
            return;
        }
        let entity = &self.mob_entity.living_entity.entity;
        let world = entity.world.load_full();
        let mob_griefing = world.level_info.load().game_rules.mob_griefing;
        let bb = entity.bounding_box.load().expand(4.0, 4.0, 4.0);

        for candidate in world.get_entities_at_box(&bb) {
            let candidate_entity = candidate.get_entity();
            if candidate_entity.entity_type == &EntityType::RAVAGER || !candidate_entity.is_alive()
            {
                continue;
            }
            if !mob_griefing && candidate_entity.entity_type == &EntityType::ARMOR_STAND {
                continue;
            }

            if !is_illager(candidate_entity.entity_type) {
                candidate
                    .damage_with_context(
                        candidate.as_ref(),
                        6.0,
                        DamageType::MOB_ATTACK,
                        Some(entity.pos.load()),
                        Some(caller.as_ref()),
                        Some(caller.as_ref()),
                    )
                    .await;
            }

            if candidate.get_player().is_none() {
                strong_knockback(entity, candidate_entity);
            }
        }

        world.send_entity_status(entity, EntityStatus::RavagerRoared);
    }
}

/// Vanilla: `Ravager.strongKnockback` (`entity.push(xd/dd*4.0, 0.2, zd/dd*4.0)`) -- a raw
/// velocity addition, not the normal attack-knockback formula.
fn strong_knockback(source: &Entity, target: &Entity) {
    let source_pos = source.pos.load();
    let target_pos = target.pos.load();
    let xd = target_pos.x - source_pos.x;
    let zd = target_pos.z - source_pos.z;
    let dd = (xd * xd + zd * zd).max(0.001);
    target.add_velocity(Vector3::new(xd / dd * 4.0, 0.2, zd / dd * 4.0));
}

impl NBTStorage for RavagerEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.mob_entity.living_entity.write_nbt(nbt).await;
            nbt.put_int("AttackTick", self.attack_tick.load(Relaxed));
            nbt.put_int("StunTick", self.stunned_tick.load(Relaxed));
            nbt.put_int("RoarTick", self.roar_tick.load(Relaxed));
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.mob_entity.living_entity.read_nbt_non_mut(nbt).await;
            if let Some(v) = nbt.get_int("AttackTick") {
                self.attack_tick.store(v, Relaxed);
            }
            if let Some(v) = nbt.get_int("StunTick") {
                self.stunned_tick.store(v, Relaxed);
            }
            if let Some(v) = nbt.get_int("RoarTick") {
                self.roar_tick.store(v, Relaxed);
            }
        })
    }
}

impl Mob for RavagerEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn can_be_raid_leader(&self) -> bool {
        false
    }

    /// Vanilla: `Ravager.aiStep` (the counter-driven parts of it; movement is handled above via
    /// `MOVEMENT_SPEED` base-value mutation, which `LivingEntity::set_attribute_base` supports).
    fn mob_tick<'a>(&'a self, caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            if !caller.get_entity().is_alive() {
                return;
            }

            self.tick_movement_speed().await;
            self.tick_leaf_destroy().await;

            let roar_tick = self.roar_tick.load(Relaxed);
            if roar_tick > 0 {
                let new_roar_tick = roar_tick - 1;
                self.roar_tick.store(new_roar_tick, Relaxed);
                if new_roar_tick == 10 {
                    self.roar(caller).await;
                }
            }

            let attack_tick = self.attack_tick.load(Relaxed);
            if attack_tick > 0 {
                self.attack_tick.store(attack_tick - 1, Relaxed);
            }

            let stunned_tick = self.stunned_tick.load(Relaxed);
            if stunned_tick > 0 {
                let new_stunned_tick = stunned_tick - 1;
                self.stunned_tick.store(new_stunned_tick, Relaxed);
                // Scope reduction: `stunEffect()`'s tinted particle spawn is a pure client
                // rendering effect and is not ported.
                if new_stunned_tick == 0 {
                    self.mob_entity
                        .living_entity
                        .entity
                        .world
                        .load()
                        .play_sound(
                            Sound::EntityRavagerRoar,
                            SoundCategory::Hostile,
                            &self.mob_entity.living_entity.entity.pos.load(),
                        );
                    self.roar_tick.store(20, Relaxed);
                }
            }
        })
    }

    /// Vanilla: `Ravager.doHurtTarget` (`attackTick = 10`, broadcasts entity event `4`, plays the
    /// attack sound) -- runs after `MobEntity::try_attack` lands a successful hit.
    fn on_successful_attack<'a>(&'a self, _target: &'a dyn EntityBase) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            self.attack_tick.store(10, Relaxed);
            let entity = &self.mob_entity.living_entity.entity;
            let world = entity.world.load();
            world.send_entity_status(entity, EntityStatus::StartAttacking);
            world.play_sound(
                Sound::EntityRavagerAttack,
                SoundCategory::Hostile,
                &entity.pos.load(),
            );
        })
    }

    /// Vanilla: `Ravager.blockedByItem` -- while not roaring, a 50/50 roll either stuns self for
    /// 40 ticks (chaining into a roar once the stun ends, per `aiStep`) or knocks the shield
    /// blocker back.
    fn blocked_by_item<'a>(
        &'a self,
        defender: &'a dyn EntityBase,
        _damage: f32,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            if self.roar_tick.load(Relaxed) != 0 {
                return;
            }
            let entity = &self.mob_entity.living_entity.entity;
            if rand::random::<f64>() < 0.5 {
                self.stunned_tick.store(40, Relaxed);
                entity.world.load().play_sound(
                    Sound::EntityRavagerStunned,
                    SoundCategory::Hostile,
                    &entity.pos.load(),
                );
                entity
                    .world
                    .load()
                    .send_entity_status(entity, EntityStatus::RavagerStunned);
                // Scope reduction: vanilla's `defender.push(this)` is a generic physics
                // separation push (not a knockback formula); not replicated here.
            } else {
                strong_knockback(entity, defender.get_entity());
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::is_illager;
    use pumpkin_data::entity::EntityType;

    #[test]
    fn roar_exempts_illager_family_but_not_vex_or_witch() {
        assert!(is_illager(&EntityType::VINDICATOR));
        assert!(is_illager(&EntityType::PILLAGER));
        assert!(is_illager(&EntityType::ILLUSIONER));
        assert!(is_illager(&EntityType::EVOKER));
        assert!(!is_illager(&EntityType::VEX));
        assert!(!is_illager(&EntityType::WITCH));
        assert!(!is_illager(&EntityType::RAVAGER));
        assert!(!is_illager(&EntityType::PLAYER));
    }
}
