use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::sync::{Arc, Weak};

use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_nbt::compound::NbtCompound;
use tokio::sync::Mutex;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture,
    ai::goal::{
        active_target::ActiveTargetGoal,
        break_door::{self, BreakDoorGoal},
        interact_with_door::InteractWithDoorGoal,
        johnny_attack::JohnnyAttackGoal,
        look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal,
        melee_attack::MeleeAttackGoal,
        revenge::RevengeGoal,
        swim::SwimGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

pub struct VindicatorEntity {
    pub mob_entity: MobEntity,
    /// Vanilla: `Vindicator.isJohnny`. One-way latch set (via `mob_tick`) once the vindicator is
    /// custom-named "Johnny".
    is_johnny: AtomicBool,
}

impl VindicatorEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let vindicator = Self {
            mob_entity,
            is_johnny: AtomicBool::new(false),
        };
        let mob_arc = Arc::new(vindicator);
        // Vanilla `populateDefaultEquipmentSlots`: only gives the axe `if getCurrentRaid() ==
        // null`; raid-spawned vindicators get their axe from `apply_raid_buffs` instead, called
        // by `Raid::spawn_group` right after construction, which overwrites this unconditionally.
        mob_arc
            .mob_entity
            .living_entity
            .entity_equipment
            .try_lock()
            .expect("new vindicator equipment is uncontended")
            .equipment
            .insert(
                EquipmentSlot::MAIN_HAND,
                Arc::new(Mutex::new(ItemStack::new(1, &Item::IRON_AXE))),
            );
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(2, Box::new(BreakDoorGoal::new(break_door::normal_or_hard)));
            goal_selector.add_goal(
                3,
                Box::new(InteractWithDoorGoal::new(false).raid_gated(true)),
            );
            goal_selector.add_goal(5, Box::new(MeleeAttackGoal::new(1.0, true)));
            goal_selector.add_goal(6, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                7,
                LookAtEntityGoal::with_default(mob_weak.clone(), &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(8, Box::new(RandomLookAroundGoal::default()));

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
                2,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::IRON_GOLEM, true),
            );
            target_selector.add_goal(3, Box::new(JohnnyAttackGoal::new(&mob_arc.mob_entity)));
        };

        mob_arc
    }

    #[must_use]
    pub fn is_johnny(&self) -> bool {
        self.is_johnny.load(Relaxed)
    }
}

impl NBTStorage for VindicatorEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.mob_entity.living_entity.write_nbt(nbt).await;
            if self.is_johnny() {
                nbt.put_bool("Johnny", true);
            }
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.mob_entity.living_entity.read_nbt_non_mut(nbt).await;
            if nbt.get_bool("Johnny") == Some(true) {
                self.is_johnny.store(true, Relaxed);
            }
        })
    }
}

impl Mob for VindicatorEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    /// Vanilla: `Vindicator.setCustomName`'s one-way "Johnny" latch. Pumpkin has no per-mob
    /// custom-name-changed hook, so this lazily checks-and-latches every tick instead (up to one
    /// tick of latency versus vanilla's setter-time latch, unobservable to a player).
    /// `customServerAiStep`'s `getNavigation().setCanOpenDoors(level.isRaided(...))` is
    /// approximated by `InteractWithDoorGoal::raid_gated`/`BreakDoorGoal` each independently
    /// checking `has_active_raid()` rather than a shared navigator flag, since Pumpkin's
    /// navigator has no `can_open_doors` concept to set.
    fn mob_tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            if !self.is_johnny.load(Relaxed)
                && let Some(name) = &**self.mob_entity.living_entity.entity.custom_name.load()
                && name.clone().get_text() == "Johnny"
            {
                self.is_johnny.store(true, Relaxed);
            }
        })
    }

    /// Vanilla: `Vindicator.applyRaidBuffs`. Builds a fresh iron axe and sets it unconditionally;
    /// the enchant roll is skipped since Pumpkin has no `VanillaEnchantmentProviders`/
    /// `EnchantmentHelper.enchantItemFromProvider` equivalent yet (confirmed absent by a
    /// repo-wide search), so `RAID_VINDICATOR`/`RAID_VINDICATOR_POST_WAVE_5` cannot be applied.
    fn apply_raid_buffs(&self, _wave: i32, _is_captain: bool) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            self.mob_entity
                .living_entity
                .entity_equipment
                .lock()
                .await
                .put(
                    &EquipmentSlot::MAIN_HAND,
                    ItemStack::new(1, &Item::IRON_AXE),
                )
                .await;
        })
    }
}
