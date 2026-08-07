use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Weak};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::{EntityStatus, EntityType};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture,
    ai::goal::{
        defend_village_target::DefendVillageTargetGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, melee_attack::MeleeAttackGoal,
        nearest_hostile_target::NearestHostileTargetGoal, offer_flower::OfferFlowerGoal,
        revenge::RevengeGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
    player::Player,
};
use pumpkin_nbt::compound::NbtCompound;

/// `IronGolem.IRON_INGOT_HEAL_AMOUNT` (`IronGolem.java:54`).
const IRON_INGOT_HEAL_AMOUNT: f32 = 25.0;

/// `Crackiness.Level` (`Crackiness.java:36-40`), as produced by `Crackiness.GOLEM`
/// (`Crackiness.java:6`, thresholds `0.75 / 0.5 / 0.25`) in `Crackiness.byFraction`
/// (`Crackiness.java:18-25`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Crackiness {
    None,
    Low,
    Medium,
    High,
}

#[must_use]
fn golem_crackiness(health: f32, max_health: f32) -> Crackiness {
    let fraction = health / max_health;
    if fraction < 0.25 {
        Crackiness::High
    } else if fraction < 0.5 {
        Crackiness::Medium
    } else if fraction < 0.75 {
        Crackiness::Low
    } else {
        Crackiness::None
    }
}

/// `IronGolem.doHurtTarget` damage roll (`IronGolem.java:192`):
/// `attackDamage / 2 + random.nextInt((int)attackDamage)`, or flat `attackDamage` when the
/// truncated attribute is not positive.
#[must_use]
fn attack_damage_roll(base_attack_damage: f32, rand_int: i32) -> f32 {
    if base_attack_damage as i32 > 0 {
        base_attack_damage / 2.0 + rand_int as f32
    } else {
        base_attack_damage
    }
}

/// Represents an Iron Golem, a powerful neutral mob that protects villagers and players.
///
/// Wiki: <https://minecraft.wiki/w/Iron_Golem>
pub struct IronGolemEntity {
    pub mob_entity: MobEntity,
    /// Vanilla `IronGolem.DATA_PLAYER_CREATED_ID`/`isPlayerCreated` (`IronGolem.java:287-291`,
    /// persisted as `"PlayerCreated"` at line 147/154). Set by `CarvedPumpkinBlock` when a
    /// player assembles a golem out of iron blocks; village-spawned golems (e.g.
    /// `Villager::spawnGolemIfNeeded`) leave it `false`. Gates `DefendVillageTargetGoal` and
    /// `canAttack` (`IronGolem.java:136-141`): a player-created golem never attacks players,
    /// regardless of reputation.
    pub player_created: AtomicBool,
    /// Health snapshot taken in `pre_damage`, so `on_damage` can compare the crackiness tier
    /// before and after the hit the way `IronGolem.hurtServer` (`IronGolem.java:206-215`) does
    /// around its `super.hurtServer` call. Re-seeded on every damage attempt, so it cannot go
    /// stale across NBT loads or regeneration.
    health_before_damage: AtomicCell<f32>,
}

impl IronGolemEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let iron_golem = Self {
            mob_entity,
            player_created: AtomicBool::new(false),
            health_before_damage: AtomicCell::new(0.0),
        };
        let mob_arc = Arc::new(iron_golem);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();
            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();

            goal_selector.add_goal(1, Box::new(MeleeAttackGoal::new(1.0, true)));
            goal_selector.add_goal(5, OfferFlowerGoal::new());
            goal_selector.add_goal(6, Box::new(WanderAroundGoal::new(0.6)));
            goal_selector.add_goal(
                7,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(8, Box::new(RandomLookAroundGoal::default()));

            // Vanilla `targetSelector.addGoal(1, new DefendVillageTargetGoal(this))`
            // (`IronGolem.java:75`): attack a player any nearby villager holds reputation
            // -100 or lower against. See `defend_village_target.rs` for full citation.
            target_selector.add_goal(1, DefendVillageTargetGoal::new());
            // Vanilla priority 2: `HurtByTargetGoal(this)`.
            target_selector.add_goal(2, Box::new(RevengeGoal::new(true)));
            // Vanilla targets players through `NearestAttackableTargetGoal<>(..., this::isAngryAt)`,
            // so a golem only goes after a player it already holds a grudge against. We have no
            // per player anger state yet, so we leave players to `RevengeGoal` instead of
            // attacking every player on sight.
            //
            // Vanilla priority 3: `NearestAttackableTargetGoal<Mob>(this, Mob.class, 5, false,
            // false, (target, level) -> target instanceof Enemy && !(target instanceof Creeper))`
            // -- attacks the nearest hostile mob (excluding creepers) within follow range. This
            // has no village-proximity condition in vanilla; see `nearest_hostile_target.rs`.
            target_selector.add_goal(3, NearestHostileTargetGoal::new(&mob_arc.mob_entity));
        };

        mob_arc
    }
}

impl NBTStorage for IronGolemEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.mob_entity.living_entity.write_nbt(nbt).await;
            // `IronGolem.java:147`.
            nbt.put_bool("PlayerCreated", self.player_created.load(Ordering::Relaxed));
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.mob_entity.living_entity.read_nbt_non_mut(nbt).await;
            // `IronGolem.java:154`: `getBooleanOr("PlayerCreated", false)`.
            self.player_created.store(
                nbt.get_bool("PlayerCreated").unwrap_or(false),
                Ordering::Relaxed,
            );
        })
    }
}

impl Mob for IronGolemEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    /// Vanilla `IronGolem.canAttack` (`IronGolem.java:135-142`): a player-created golem never
    /// attacks players, and no golem ever attacks a creeper (which is why one that gets caught
    /// in a creeper blast does not retaliate).
    fn can_attack(&self, target: &Entity) -> bool {
        let target_type = target.entity_type;
        if self.player_created.load(Ordering::Relaxed) && target_type == &EntityType::PLAYER {
            return false;
        }
        target_type != &EntityType::CREEPER
    }

    /// Vanilla `IronGolem.doHurtTarget` (`IronGolem.java:187-204`): a randomized damage roll
    /// plus a straight-up fling scaled by the target's knockback resistance, and the attack
    /// sound/animation event, replacing the generic flat-damage melee path.
    fn try_attack<'a>(&'a self, target: &'a dyn EntityBase) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            let living = &self.mob_entity.living_entity;
            if living.dead.load(Ordering::Relaxed) {
                return false;
            }

            let entity = &living.entity;
            let world = entity.world.load();
            world.send_entity_status(entity, EntityStatus::StartAttacking);

            let base_attack_damage = living.get_attribute_value(&Attributes::ATTACK_DAMAGE) as f32;
            let rand_int = if base_attack_damage as i32 > 0 {
                rand::rng().random_range(0..base_attack_damage as i32)
            } else {
                0
            };
            let damage = attack_damage_roll(base_attack_damage, rand_int);

            let caller = world.get_entity_by_id(entity.entity_id);
            let damaged = target
                .damage_with_context(
                    target,
                    damage,
                    DamageType::MOB_ATTACK,
                    Some(entity.pos.load()),
                    caller.as_deref(),
                    caller.as_deref(),
                )
                .await;

            if damaged {
                let resistance = target.get_living_entity().map_or(0.0, |target_living| {
                    target_living.get_attribute_value(&Attributes::KNOCKBACK_RESISTANCE)
                });
                let scale = (1.0 - resistance).max(0.0);
                target
                    .get_entity()
                    .add_velocity(Vector3::new(0.0, 0.4 * scale, 0.0));
            }

            // `IronGolem.java:202`: played whether or not the hit landed.
            world.play_sound(
                Sound::EntityIronGolemAttack,
                SoundCategory::Neutral,
                &entity.pos.load(),
            );

            damaged
        })
    }

    /// `IronGolem.hurtServer` (`IronGolem.java:206-215`) reads the crackiness tier before the
    /// hit; this snapshot lets `on_damage` do the same comparison afterwards.
    fn pre_damage<'a>(
        &'a self,
        _damage_type: DamageType,
        _source: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            self.health_before_damage
                .store(self.mob_entity.living_entity.health.load());
            true
        })
    }

    /// `IronGolem.hurtServer` (`IronGolem.java:210-212`): crossing a crackiness threshold plays
    /// the golem's cracking sound on top of the normal hurt sound.
    fn on_damage<'a>(
        &'a self,
        _damage_type: DamageType,
        _source: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let living = &self.mob_entity.living_entity;
            let max_health = living.get_max_health();
            let before = golem_crackiness(self.health_before_damage.load(), max_health);
            let after = golem_crackiness(living.health.load(), max_health);
            if before != after {
                living.entity.play_sound(Sound::EntityIronGolemDamage);
            }
        })
    }

    /// Vanilla `IronGolem.mobInteract` (`IronGolem.java:259-276`): an iron ingot repairs the
    /// golem for 25 health. A golem already at full health consumes nothing and makes no sound.
    fn mob_interact<'a>(
        &'a self,
        player: &'a Arc<Player>,
        item_stack: &'a mut ItemStack,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            // Vanilla runs `Mob.checkAndHandleImportantInteractions` (lead/nametag) before
            // `mobInteract`, so an unleash always wins over the ingot branch.
            if self
                .get_mob_entity()
                .mob_interact(player, item_stack, self.can_be_leashed())
                .await
            {
                return true;
            }

            if item_stack.item.id == Item::IRON_INGOT.id {
                let living = &self.mob_entity.living_entity;
                let health_before = living.health.load();
                living.heal(IRON_INGOT_HEAL_AMOUNT);
                if living.health.load() == health_before {
                    return false;
                }

                let mut rng = rand::rng();
                let pitch = 1.0 + (rng.random::<f32>() - rng.random::<f32>()) * 0.2;
                let entity = &living.entity;
                entity.world.load().play_sound_fine(
                    Sound::EntityIronGolemRepair,
                    SoundCategory::Neutral,
                    &entity.pos.load(),
                    1.0,
                    pitch,
                );
                item_stack.decrement_unless_creative(player.gamemode.load(), 1);
                return true;
            }

            false
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Crackiness, attack_damage_roll, golem_crackiness};

    #[test]
    fn crackiness_tiers_match_golem_thresholds() {
        assert_eq!(golem_crackiness(100.0, 100.0), Crackiness::None);
        assert_eq!(golem_crackiness(75.0, 100.0), Crackiness::None);
        assert_eq!(golem_crackiness(74.0, 100.0), Crackiness::Low);
        assert_eq!(golem_crackiness(50.0, 100.0), Crackiness::Low);
        assert_eq!(golem_crackiness(49.0, 100.0), Crackiness::Medium);
        assert_eq!(golem_crackiness(25.0, 100.0), Crackiness::Medium);
        assert_eq!(golem_crackiness(24.0, 100.0), Crackiness::High);
    }

    #[test]
    fn damage_roll_adds_half_base_to_the_roll() {
        assert_eq!(attack_damage_roll(15.0, 0), 7.5);
        assert_eq!(attack_damage_roll(15.0, 14), 21.5);
    }

    #[test]
    fn non_positive_base_damage_is_flat() {
        assert_eq!(attack_damage_roll(0.0, 0), 0.0);
        assert_eq!(attack_damage_roll(0.5, 3), 0.5);
    }
}
