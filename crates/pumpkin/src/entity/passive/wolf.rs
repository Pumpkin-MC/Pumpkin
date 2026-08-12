use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, AtomicI64, AtomicU8, Ordering},
};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::Metadata;
use rand::RngExt;
use uuid::Uuid;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture,
    ai::goal::{
        active_target::ActiveTargetGoal, beg::BegGoal, breed::BreedGoal,
        escape_danger::EscapeDangerGoal, follow_parent::FollowParentGoal,
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal,
        melee_attack::MeleeAttackGoal, owner_hurt_by_target::OwnerHurtByTargetGoal,
        owner_hurt_target::OwnerHurtTargetGoal, revenge::RevengeGoal, swim::SwimGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

pub struct WolfEntity {
    pub mob_entity: MobEntity,
    pub variant: AtomicU8,
    tamed: AtomicBool,
    anger_target: AtomicCell<Option<Uuid>>,
    anger_end_time: AtomicI64,
}

impl WolfEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let wolf = Self {
            mob_entity,
            variant: AtomicU8::new(3), // Default to pale
            tamed: AtomicBool::new(false),
            anger_target: AtomicCell::new(None),
            anger_end_time: AtomicI64::new(-1),
        };
        let mob_arc = Arc::new(wolf);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc
                .mob_entity
                .goals_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            goal_selector.add_goal(1, Box::new(SwimGoal::default()));
            // goal_selector.add_goal(2, SitGoal::new(mob_arc.clone()));
            goal_selector.add_goal(4, EscapeDangerGoal::new(1.5));
            goal_selector.add_goal(5, Box::new(MeleeAttackGoal::new(1.0, true)));
            goal_selector.add_goal(7, BreedGoal::new(1.0));
            goal_selector.add_goal(8, Box::new(FollowParentGoal::new(1.1)));
            goal_selector.add_goal(9, BegGoal::new(8.0, &[&Item::BONE]));
            goal_selector.add_goal(
                10,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(10, Box::new(RandomLookAroundGoal::default()));
            goal_selector.add_goal(12, Box::new(WanderAroundGoal::new(1.0)));
        };

        {
            let mut target_selector = mob_arc
                .mob_entity
                .target_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            // Matches Wolf.registerGoals target priorities. The owner goals are
            // harmless until taming/ownership data is available on the entity.
            target_selector.add_goal(1, OwnerHurtByTargetGoal::new());
            target_selector.add_goal(2, OwnerHurtTargetGoal::new());
            target_selector.add_goal(3, Box::new(RevengeGoal::new(true)));
            let mut player_goal =
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PLAYER, true);
            player_goal.set_max_distance(10.0);
            let wolf_ref = Arc::downgrade(&mob_arc);
            player_goal.set_predicate(move |target, world| {
                let wolf_ref = wolf_ref.clone();
                async move {
                    let Some(wolf) = wolf_ref.upgrade() else {
                        return false;
                    };
                    let now = world.level_time.lock().await.world_age;
                    wolf.anger_end_time.load(Ordering::Relaxed) > now
                        && wolf.anger_target.load() == Some(target.entity.entity_uuid)
                }
            });
            target_selector.add_goal(4, player_goal);

            for prey_type in [&EntityType::SHEEP, &EntityType::RABBIT, &EntityType::FOX] {
                let mut prey_goal =
                    ActiveTargetGoal::with_default(&mob_arc.mob_entity, prey_type, false);
                prey_goal.set_only_when_untamed(true);
                target_selector.add_goal(5, prey_goal);
            }

            let mut turtle_goal =
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::TURTLE, false);
            turtle_goal.set_only_when_untamed(true);
            turtle_goal.set_predicate(|target, _world| async move {
                target.entity.age.load(Ordering::Relaxed) < 0 && !target.is_in_water()
            });
            target_selector.add_goal(6, turtle_goal);

            let mut skeleton_goal =
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::SKELETON, false);
            skeleton_goal.set_target_types(vec![
                &EntityType::SKELETON,
                &EntityType::WITHER_SKELETON,
                &EntityType::STRAY,
                &EntityType::BOGGED,
                &EntityType::PARCHED,
            ]);
            target_selector.add_goal(7, skeleton_goal);
        };

        mob_arc
    }
}

impl NBTStorage for WolfEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.mob_entity.living_entity.write_nbt(nbt).await;
            let variant_str = match self.variant.load(Ordering::Relaxed) {
                0 => "minecraft:ashen",
                1 => "minecraft:black",
                2 => "minecraft:chestnut",
                4 => "minecraft:rusty",
                5 => "minecraft:snowy",
                6 => "minecraft:spotted",
                7 => "minecraft:striped",
                8 => "minecraft:woods",
                _ => "minecraft:pale",
            };
            nbt.put_string("variant", variant_str.to_string());
            nbt.put_bool("Tame", self.tamed.load(Ordering::Relaxed));
            if let Some(target) = self.anger_target.load() {
                nbt.put_uuid("angry_at", target);
            }
            nbt.put_long(
                "anger_end_time",
                self.anger_end_time.load(Ordering::Relaxed),
            );
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.mob_entity.living_entity.read_nbt_non_mut(nbt).await;
            if let Some(variant_str) = nbt.get_string("variant") {
                let variant = match variant_str
                    .strip_prefix("minecraft:")
                    .unwrap_or(variant_str)
                {
                    "ashen" => 0,
                    "black" => 1,
                    "chestnut" => 2,
                    "rusty" => 4,
                    "snowy" => 5,
                    "spotted" => 6,
                    "striped" => 7,
                    "woods" => 8,
                    _ => 3,
                };
                self.variant.store(variant, Ordering::Relaxed);
            }
            self.tamed
                .store(nbt.get_bool("Tame").unwrap_or(false), Ordering::Relaxed);
            self.anger_target.store(nbt.get_uuid("angry_at"));
            self.anger_end_time.store(
                nbt.get_long("anger_end_time").unwrap_or(-1),
                Ordering::Relaxed,
            );
        })
    }
}

impl Mob for WolfEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_set_variant_name(&self, name: &str) {
        let variant = match name.strip_prefix("minecraft:").unwrap_or(name) {
            "ashen" => 0,
            "black" => 1,
            "chestnut" => 2,
            "rusty" => 4,
            "snowy" => 5,
            "spotted" => 6,
            "striped" => 7,
            "woods" => 8,
            _ => 3,
        };
        self.variant.store(variant, Ordering::Relaxed);
    }

    fn is_tame(&self) -> bool {
        self.tamed.load(Ordering::Relaxed)
    }

    fn on_damage<'a>(
        &'a self,
        _damage_type: DamageType,
        source: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let Some(source) = source else {
                return;
            };
            let world = self.mob_entity.living_entity.entity.world.load();
            let now = world.level_time.lock().await.world_age;
            let anger_ticks = {
                let mut random = self.get_random();
                random.random_range(20..40) * 20
            };
            self.anger_target
                .store(Some(source.get_entity().entity_uuid));
            self.anger_end_time
                .store(now + i64::from(anger_ticks), Ordering::Relaxed);
        })
    }

    fn mob_init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            let entity = self.get_entity();
            let is_baby = entity.age.load(Ordering::Relaxed) < 0;
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
            entity.send_meta_data(
                &[Metadata::new(
                    TrackedData::WOLF_VARIANT_ID,
                    MetaDataType::WOLF_VARIANT,
                    VarInt(self.variant.load(Ordering::Relaxed) as i32),
                )],
                None,
            );
        })
    }
}
