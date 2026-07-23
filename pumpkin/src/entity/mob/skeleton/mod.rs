use std::sync::{Arc, Weak};

use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::java::client::play::Metadata;

use crate::entity::{
    Entity, EntityBase, NBTStorage, NbtFuture,
    ai::goal::{
        active_target::ActiveTargetGoal, avoid_entity::AvoidEntityGoal, bow_attack::BowAttackGoal,
        flee_sun::FleeSunGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, melee_attack::MeleeAttackGoal, revenge::RevengeGoal,
        swim::SwimGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

pub mod bogged;
pub mod parched;
#[allow(clippy::module_inception)]
pub mod skeleton;
pub mod stray;
pub mod wither;

/// Shared skeleton AI base.
///
/// * `ranged = true` → bow AI (skeleton / stray / bogged / parched)
/// * `ranged = false` → melee only (wither skeleton)
pub struct SkeletonEntityBase {
    pub mob_entity: MobEntity,
    ranged: bool,
}

impl SkeletonEntityBase {
    pub fn new(entity: Entity) -> Arc<Self> {
        Self::with_combat(entity, true)
    }

    pub fn with_combat(entity: Entity, ranged: bool) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let mob = Self { mob_entity, ranged };
        let mob_arc = Arc::new(mob);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };
        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();
            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();

            // Vanilla 26.2 AbstractSkeleton.registerGoals
            // (RestrictSun not ported — FleeSun covers daylight shelter.)
            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(3, Box::new(FleeSunGoal::new(1.0)));
            goal_selector.add_goal(
                3,
                Box::new(AvoidEntityGoal::new(&EntityType::WOLF, 6.0, 1.0, 1.2)),
            );
            if ranged {
                // Vanilla RangedBowAttackGoal priority 4, speed 1.0, interval 20.
                goal_selector.add_goal(4, BowAttackGoal::new(1.0, 20));
            } else {
                goal_selector.add_goal(4, Box::new(MeleeAttackGoal::new(1.2, false)));
            }
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(6, Box::new(RandomLookAroundGoal::default()));

            target_selector.add_goal(1, Box::new(RevengeGoal::new(true)));
            target_selector.add_goal(
                2,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PLAYER, true),
            );
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::IRON_GOLEM, true),
            );
            // NearestAttackableTargetGoal(Turtle, 10, true, false, BABY_ON_LAND)
            // — baby filter TODO; still target turtles like undead.
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::TURTLE, true),
            );
        };

        mob_arc
    }

    async fn equip_default_weapon(&self) {
        let living = &self.mob_entity.living_entity;
        let item = if self.ranged {
            &Item::BOW
        } else {
            &Item::STONE_SWORD
        };
        let stack = ItemStack::new(1, item);
        living
            .entity_equipment
            .lock()
            .await
            .put(&EquipmentSlot::MAIN_HAND, stack.clone())
            .await;
        // Broadcast to nearby players; also covers re-equip after NBT load with empty hand.
        living.send_equipment_changes(&[(EquipmentSlot::MAIN_HAND, stack)]);
    }

    /// True if this skeleton uses a bow (not wither melee).
    #[must_use]
    pub fn is_ranged(&self) -> bool {
        self.ranged
    }
}

impl NBTStorage for SkeletonEntityBase {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        self.mob_entity.living_entity.write_nbt(nbt)
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        self.mob_entity.living_entity.read_nbt_non_mut(nbt)
    }
}

impl Mob for SkeletonEntityBase {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_init_data_tracker(&self) -> crate::entity::EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            let entity = self.get_entity();
            let is_baby = entity.age.load(std::sync::atomic::Ordering::Relaxed) < 0;
            if is_baby {
                entity.send_meta_data(
                    &[Metadata::new(
                        TrackedData::BABY_ID,
                        MetaDataType::BOOLEAN,
                        true,
                    )],
                    None,
                );
            }
            // Vanilla always equips a bow (or sword for wither skeleton).
            self.equip_default_weapon().await;
        })
    }
}
