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
use pumpkin_util::difficulty::Difficulty;
use rand::RngExt;
use uuid::Uuid;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture,
    ai::goal::{
        active_target::ActiveTargetGoal, beg::BegGoal, breed::BreedGoal,
        escape_danger::EscapeDangerGoal, follow_owner::FollowOwnerGoal,
        leap_at_target::LeapAtTargetGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, melee_attack::MeleeAttackGoal,
        owner_hurt_by_target::OwnerHurtByTargetGoal, owner_hurt_target::OwnerHurtTargetGoal,
        revenge::RevengeGoal, sit::SitGoal, swim::SwimGoal, wander_around::WanderAroundGoal,
    },
    living::LivingEntity,
    mob::{Mob, MobEntity},
};

pub struct WolfEntity {
    pub mob_entity: MobEntity,
    pub variant: AtomicU8,
    tamed: AtomicBool,
    owner_uuid: AtomicCell<Option<Uuid>>,
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
            owner_uuid: AtomicCell::new(None),
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
            goal_selector.add_goal(2, SitGoal::new());
            goal_selector.add_goal(4, EscapeDangerGoal::new(1.5));
            goal_selector.add_goal(4, LeapAtTargetGoal::new(0.4));
            goal_selector.add_goal(5, Box::new(MeleeAttackGoal::new(1.0, true)));
            goal_selector.add_goal(6, FollowOwnerGoal::new(1.0, 10.0, 2.0));
            goal_selector.add_goal(7, BreedGoal::new(1.0));
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
            target_selector.add_goal(3, Box::new(RevengeGoal::with_alert_others(true)));
            let mut player_goal =
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PLAYER, true);
            player_goal.set_max_distance(10.0);
            let wolf_ref = Arc::downgrade(&mob_arc);
            player_goal.set_predicate(move |target, world| {
                let wolf_ref = wolf_ref.clone();
                let target_uuid = target.entity.entity_uuid;
                Box::pin(async move {
                    let Some(wolf) = wolf_ref.upgrade() else {
                        return false;
                    };
                    let now = world.level_time.lock().await.world_age;
                    wolf.anger_end_time.load(Ordering::Relaxed) > now
                        && wolf.anger_target.load() == Some(target_uuid)
                })
            });
            target_selector.add_goal(4, player_goal);

            let mut prey_goal =
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::SHEEP, false);
            prey_goal.set_target_types(vec![
                &EntityType::SHEEP,
                &EntityType::RABBIT,
                &EntityType::FOX,
            ]);
            prey_goal.set_only_when_untamed(true);
            target_selector.add_goal(5, prey_goal);

            let mut turtle_goal =
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::TURTLE, false);
            turtle_goal.set_only_when_untamed(true);
            turtle_goal.set_predicate(|target, _world| {
                let is_baby_on_land =
                    target.entity.age.load(Ordering::Relaxed) < 0 && !target.is_in_water();
                Box::pin(async move { is_baby_on_land })
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

    fn can_attack_entity(&self, target: &dyn EntityBase) -> bool {
        target
            .get_living_entity()
            .is_some_and(|living| self.can_attack(living))
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
            if let Some(owner) = self.owner_uuid.load() {
                nbt.put_uuid("Owner", owner);
            }
            nbt.put_bool("Sitting", self.mob_entity.is_ordered_to_sit());
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
            let owner = nbt.get_uuid("Owner");
            self.owner_uuid.store(owner);
            if owner.is_some() {
                self.tamed.store(true, Ordering::Relaxed);
            }
            self.mob_entity
                .set_ordered_to_sit(nbt.get_bool("Sitting").unwrap_or(false));
            if self.tamed.load(Ordering::Relaxed) {
                self.mob_entity.living_entity.set_max_health(40.0).await;
            }
            self.anger_target.store(nbt.get_uuid("angry_at"));
            let world = self.mob_entity.living_entity.entity.world.load();
            let now = world.level_time.lock().await.world_age;
            let anger_end = nbt.get_long("anger_end_time").or_else(|| {
                nbt.get_int("AngerTime")
                    .map(|remaining| now + i64::from(remaining.max(0)))
            });
            self.anger_end_time
                .store(anger_end.unwrap_or(-1), Ordering::Relaxed);
            if let Some(uuid) = self.anger_target.load()
                && let Some(target) = world.get_entity_by_uuid(uuid)
                && self.can_attack_entity(target.as_ref())
            {
                self.mob_entity.set_target(Some(target)).await;
            }
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

    fn get_owner_uuid(&self) -> Option<Uuid> {
        self.owner_uuid.load()
    }

    fn is_sitting(&self) -> bool {
        self.mob_entity.is_ordered_to_sit()
    }

    fn can_attack(&self, target: &crate::entity::living::LivingEntity) -> bool {
        if self.owner_uuid.load() == Some(target.entity.entity_uuid) {
            return false;
        }
        let world = self.mob_entity.living_entity.entity.world.load();
        target.entity.entity_type != &EntityType::GHAST
            && self
                .mob_entity
                .living_entity
                .can_attack_target(target, &world)
    }

    fn can_attack_with_owner(&self, target: &dyn EntityBase, owner: &dyn EntityBase) -> bool {
        let target_entity = target.get_entity();
        if target_entity.entity_type == &EntityType::CREEPER
            || target_entity.entity_type == &EntityType::GHAST
            || target_entity.entity_type == &EntityType::ARMOR_STAND
        {
            return false;
        }
        if target_entity.entity_type == &EntityType::WOLF {
            return target.get_mob().is_none_or(|target_mob| {
                !target_mob.is_tame() || !target_mob_owner_is(owner, target)
            }) && self.can_attack_entity(target);
        }
        if target.get_mob().is_some_and(Mob::is_tame) {
            return false;
        }
        self.can_attack_entity(target)
    }

    fn mob_interact<'a>(
        &'a self,
        player: &'a Arc<crate::entity::player::Player>,
        item_stack: &'a mut pumpkin_data::item_stack::ItemStack,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            let world = self.mob_entity.living_entity.entity.world.load();
            let now = world.level_time.lock().await.world_age;
            if !self.is_tame()
                && item_stack.item.registry_key == "bone"
                && self.anger_end_time.load(Ordering::Relaxed) <= now
            {
                item_stack.decrement_unless_creative(player.gamemode.load(), 1);
                if self.get_random().random_range(0..3) == 0 {
                    self.tamed.store(true, Ordering::Relaxed);
                    self.owner_uuid.store(Some(player.gameprofile.id));
                    self.mob_entity.living_entity.set_max_health(40.0).await;
                    self.mob_entity.living_entity.set_health(40.0);
                    self.mob_entity.set_target(None).await;
                    self.mob_entity.set_ordered_to_sit(true);
                    self.mob_entity
                        .navigator
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .stop();
                    world.send_entity_status(
                        &self.mob_entity.living_entity.entity,
                        pumpkin_data::entity::EntityStatus::TamingSucceeded,
                        None,
                    );
                } else {
                    world.send_entity_status(
                        &self.mob_entity.living_entity.entity,
                        pumpkin_data::entity::EntityStatus::TamingFailed,
                        None,
                    );
                }
                return true;
            }

            let interacted = self.mob_entity.mob_interact(player, item_stack).await;
            if !interacted
                && self.is_tame()
                && self.owner_uuid.load() == Some(player.gameprofile.id)
            {
                self.mob_entity
                    .set_ordered_to_sit(!self.mob_entity.is_ordered_to_sit());
                self.mob_entity
                    .navigator
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .stop();
                self.mob_entity.set_target(None).await;
                return true;
            }
            interacted
        })
    }

    fn mob_tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let world = self.mob_entity.living_entity.entity.world.load();
            let now = world.level_time.lock().await.world_age;
            let target = self.mob_entity.target.lock().await.clone();
            let anger_target = self.anger_target.load();

            let dead_persistent_mob = target.as_ref().is_some_and(|target| {
                let dead_or_dying = target.get_living_entity().is_some_and(|living| {
                    living.dead.load(Ordering::Relaxed) || living.health.load() <= 0.0
                });
                dead_or_dying
                    && target.get_mob().is_some()
                    && anger_target == Some(target.get_entity().entity_uuid)
            });
            if dead_persistent_mob {
                self.mob_entity.set_target(None).await;
                self.anger_target.store(None);
                self.anger_end_time.store(-1, Ordering::Relaxed);
                return;
            }

            if let Some(target) = target.as_ref() {
                let new_target = anger_target != Some(target.get_entity().entity_uuid);
                if new_target {
                    self.anger_target
                        .store(Some(target.get_entity().entity_uuid));
                }
                let anger_ticks = {
                    let mut random = self.get_random();
                    random.random_range(20..40) * 20
                };
                self.anger_end_time
                    .store(now + i64::from(anger_ticks), Ordering::Relaxed);
            }

            let persistent_target = anger_target
                .and_then(|uuid| world.get_entity_by_uuid(uuid))
                .filter(|target| self.can_attack_entity(target.as_ref()));
            if target.is_none()
                && let Some(persistent_target) = persistent_target.as_ref()
                && persistent_target
                    .get_living_entity()
                    .is_some_and(LivingEntity::is_alive)
            {
                self.mob_entity
                    .set_target(Some(persistent_target.clone()))
                    .await;
            }
            if persistent_target
                .as_ref()
                .and_then(|entity| entity.get_player())
                .is_some_and(|player| {
                    player.is_spectator()
                        || player.is_creative()
                        || world.level_info.load().difficulty == Difficulty::Peaceful
                })
            {
                self.mob_entity.set_target(None).await;
                self.anger_target.store(None);
                self.anger_end_time.store(-1, Ordering::Relaxed);
            }
            if self.anger_end_time.load(Ordering::Relaxed) <= now {
                if anger_target.is_some() {
                    self.mob_entity.set_target(None).await;
                }
                self.anger_target.store(None);
                self.anger_end_time.store(-1, Ordering::Relaxed);
            }
        })
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
            self.mob_entity.set_ordered_to_sit(false);
            self.mob_entity
                .living_entity
                .entity
                .set_pose(pumpkin_data::entity::EntityPose::Standing);
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

fn target_mob_owner_is(owner: &dyn EntityBase, target: &dyn EntityBase) -> bool {
    target
        .get_mob()
        .and_then(Mob::get_owner_uuid)
        .is_some_and(|owner_uuid| owner_uuid == owner.get_entity().entity_uuid)
}
