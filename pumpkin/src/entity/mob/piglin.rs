use std::sync::{
    Arc, Weak,
    atomic::{AtomicI32, Ordering},
};

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage,
    ai::goal::{
        active_target::ActiveTargetGoal,
        look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal,
        melee_attack::MeleeAttackGoal,
        piglin_admire::{ADMIRE_DURATION_TICKS, PiglinAdmireGoal},
        swim::SwimGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
    player::Player,
};
use pumpkin_data::{
    damage::DamageType, data_component_impl::EquipmentSlot, entity::EntityType, item::Item,
    item_stack::ItemStack,
};

/// `PiglinAi.ADMIRING_DISABLED` lockout: `wasHurtBy` (PiglinAi.java:567) sets a
/// 400-tick expiry when the attacker is a player, blocking `canAdmire` for that long.
const ADMIRING_DISABLED_TICKS: i32 = 400;

pub struct PiglinEntity {
    pub mob_entity: MobEntity,
    /// Ticks left staring at a gold ingot given by a player, shared with
    /// `PiglinAdmireGoal`. See `pumpkin/src/entity/ai/goal/piglin_admire.rs`.
    pub admiring_ticks: Arc<AtomicI32>,
    /// Mirrors vanilla's `ADMIRING_DISABLED` memory expiry (see `ADMIRING_DISABLED_TICKS`).
    admiring_disabled_ticks: AtomicI32,
}

impl PiglinEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let admiring_ticks = Arc::new(AtomicI32::new(0));
        let piglin = Self {
            mob_entity,
            admiring_ticks: admiring_ticks.clone(),
            admiring_disabled_ticks: AtomicI32::new(0),
        };
        let mob_arc = Arc::new(piglin);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, PiglinAdmireGoal::new(admiring_ticks));
            // Piglins use crossbows or swords, but for now we give them melee
            goal_selector.add_goal(2, Box::new(MeleeAttackGoal::new(1.0, true)));
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak.clone(), &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(7, Box::new(RandomLookAroundGoal::default()));

            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();
            target_selector.add_goal(
                1,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PLAYER, true),
            );
            target_selector.add_goal(
                2,
                ActiveTargetGoal::with_default(
                    &mob_arc.mob_entity,
                    &EntityType::WITHER_SKELETON,
                    true,
                ),
            );
            target_selector.add_goal(
                2,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::WITHER, true),
            );
        };

        mob_arc
    }

    fn is_adult(&self) -> bool {
        self.mob_entity
            .living_entity
            .entity
            .age
            .load(Ordering::Relaxed)
            >= 0
    }
}

impl NBTStorage for PiglinEntity {}

impl Mob for PiglinEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    /// `PiglinAi.mobInteract`/`canAdmire` (PiglinAi.java:539-554): a player directly
    /// handing an adult, non-admiring, non-locked-out piglin a gold ingot starts
    /// admiring. Ground-item pickup (`isLovedItem`/broader `piglin_loved` tag) is not
    /// implemented -- see `piglin_admire.rs` module doc.
    fn mob_interact<'a>(
        &'a self,
        player: &'a Arc<Player>,
        item_stack: &'a mut ItemStack,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            let is_gold_ingot = item_stack.item.id == Item::GOLD_INGOT.id;
            let admiring_disabled = self.admiring_disabled_ticks.load(Ordering::Relaxed) > 0;
            let already_admiring = self.admiring_ticks.load(Ordering::Relaxed) > 0;

            if !is_gold_ingot || admiring_disabled || already_admiring || !self.is_adult() {
                return self.mob_entity.mob_interact(player, item_stack).await;
            }

            let taken = ItemStack::new(1, &Item::GOLD_INGOT);
            item_stack.decrement_unless_creative(player.gamemode.load(), 1);

            let mut equipment = self.mob_entity.living_entity.entity_equipment.lock().await;
            let previous = equipment.put(&EquipmentSlot::OFF_HAND, taken.clone()).await;
            drop(equipment);
            if !previous.is_empty() {
                // `holdInOffhand` (PiglinAi.java:353-359): drop whatever was already
                // held in the offhand before replacing it.
                let pos = self.mob_entity.living_entity.entity.block_pos.load();
                self.mob_entity
                    .living_entity
                    .entity
                    .world
                    .load()
                    .drop_stack(&pos, previous)
                    .await;
            }
            self.mob_entity
                .living_entity
                .send_equipment_changes(&[(EquipmentSlot::OFF_HAND, taken)]);

            self.admiring_ticks
                .store(ADMIRE_DURATION_TICKS, Ordering::Relaxed);
            true
        })
    }

    /// `PiglinAi.wasHurtBy` (PiglinAi.java:556-587), simplified: cancels admiring and
    /// applies the player-attacker lockout. The anger/avoid/retaliation broadcasting
    /// half of `wasHurtBy` is skipped -- it needs a shared anger-propagation subsystem
    /// Pumpkin doesn't have (see task note on zombified-piglin anger propagation).
    fn on_damage<'a>(
        &'a self,
        _damage_type: DamageType,
        source: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            if let Some(source) = source
                && source.get_entity().entity_type.id == EntityType::PIGLIN.id
            {
                // vanilla: `if (!(attacker instanceof Piglin))` guards the whole method.
                return;
            }

            self.admiring_ticks.store(0, Ordering::Relaxed);

            // `cancelAdmiring` (PiglinAi.java:401-406): drop the offhand item instead
            // of silently voiding it.
            let equipment = self.mob_entity.living_entity.entity_equipment.lock().await;
            let offhand = equipment.get(&EquipmentSlot::OFF_HAND);
            let dropped = {
                let mut stack = offhand.lock().await;
                std::mem::replace(&mut *stack, ItemStack::EMPTY.clone())
            };
            drop(equipment);
            if !dropped.is_empty() {
                let pos = self.mob_entity.living_entity.entity.block_pos.load();
                self.mob_entity
                    .living_entity
                    .entity
                    .world
                    .load()
                    .drop_stack(&pos, dropped)
                    .await;
                self.mob_entity
                    .living_entity
                    .send_equipment_changes(&[(EquipmentSlot::OFF_HAND, ItemStack::EMPTY.clone())]);
            }

            let attacker_is_player =
                source.is_some_and(|s| s.get_entity().entity_type.id == EntityType::PLAYER.id);
            if attacker_is_player {
                self.admiring_disabled_ticks
                    .store(ADMIRING_DISABLED_TICKS, Ordering::Relaxed);
            }
        })
    }

    /// Decrements the admiring-disabled lockout every tick, matching the passive
    /// expiry of vanilla's `ADMIRING_DISABLED` memory. Must run independently of
    /// `PiglinAdmireGoal` since the goal isn't running while admiring is disabled.
    fn mob_tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let remaining = self.admiring_disabled_ticks.load(Ordering::Relaxed);
            if remaining > 0 {
                self.admiring_disabled_ticks
                    .store(remaining - 1, Ordering::Relaxed);
            }
        })
    }
}
