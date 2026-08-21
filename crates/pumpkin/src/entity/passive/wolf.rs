use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, AtomicU8, Ordering},
};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::Metadata;
use uuid::Uuid;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture,
    ageable::AgeableMob,
    ai::goal::{
        active_target::ActiveTargetGoal, avoid_entity::AvoidEntityGoal, beg::BegGoal,
        breed::BreedGoal, escape_danger::EscapeDangerGoal, follow_owner::FollowOwnerGoal,
        follow_parent::FollowParentGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, melee_attack::MeleeAttackGoal,
        owner_hurt_by_target::OwnerHurtByTargetGoal, owner_hurt_target::OwnerHurtTargetGoal,
        revenge::RevengeGoal, swim::SwimGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
    passive::animal::Animal,
};

pub struct WolfEntity {
    pub mob_entity: MobEntity,
    pub variant: AtomicU8,
    pub collar_color: AtomicU8,
    pub is_tame: AtomicBool,
    pub is_sitting: AtomicBool,
    pub owner: AtomicCell<Option<Uuid>>,
    pub ageable_data: crate::entity::ageable::AgeableData,
}

impl WolfEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let wolf = Self {
            mob_entity,
            variant: AtomicU8::new(3),       // Default to pale
            collar_color: AtomicU8::new(14), // Default to red
            is_tame: AtomicBool::new(false),
            is_sitting: AtomicBool::new(false),
            owner: AtomicCell::new(None),
            ageable_data: crate::entity::ageable::AgeableData::default(),
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

            // Goal selector (matching Vanilla registerGoals):
            // 1: SwimGoal (FloatGoal)
            goal_selector.add_goal(1, Box::new(SwimGoal::default()));
            // 1: EscapeDangerGoal (TamableAnimalPanicGoal)
            goal_selector.add_goal(1, EscapeDangerGoal::new(1.5));
            // 3: Avoid Llama
            goal_selector.add_goal(
                3,
                Box::new(AvoidEntityGoal::new(&EntityType::LLAMA, 24.0, 1.5, 1.5)),
            );
            // 5: MeleeAttackGoal
            goal_selector.add_goal(5, Box::new(MeleeAttackGoal::new(1.0, true)));
            // 6: FollowOwnerGoal
            goal_selector.add_goal(6, FollowOwnerGoal::new(1.0, 10.0, 2.0));
            // 7: BreedGoal
            goal_selector.add_goal(7, BreedGoal::new(1.0));
            // 8: FollowParentGoal & WanderAroundGoal
            goal_selector.add_goal(8, Box::new(FollowParentGoal::new(1.1)));
            goal_selector.add_goal(8, Box::new(WanderAroundGoal::new(1.0)));
            // 9: BegGoal
            goal_selector.add_goal(9, BegGoal::new(8.0));
            // 10: LookAtPlayer & RandomLookAround
            goal_selector.add_goal(
                10,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(10, Box::new(RandomLookAroundGoal::default()));
        };

        {
            let mut target_selector = mob_arc
                .mob_entity
                .target_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            // Target selector (matching Vanilla registerGoals):
            // 1: OwnerHurtByTargetGoal
            target_selector.add_goal(1, OwnerHurtByTargetGoal::new());
            // 2: OwnerHurtTargetGoal
            target_selector.add_goal(2, OwnerHurtTargetGoal::new());
            // 3: HurtByTargetGoal (RevengeGoal)
            target_selector.add_goal(3, Box::new(RevengeGoal::new(true)));
            // 4: NearestAttackableTarget (Player)
            target_selector.add_goal(
                4,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PLAYER, true),
            );
            // 5: NonTameRandomTarget (Sheep, Rabbit, Fox)
            target_selector.add_goal(
                5,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::SHEEP, false),
            );
            target_selector.add_goal(
                5,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::RABBIT, false),
            );
            target_selector.add_goal(
                5,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::FOX, false),
            );
            // 7: NearestAttackableTarget (Skeleton)
            target_selector.add_goal(
                7,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::SKELETON, false),
            );
        };

        mob_arc
    }

    pub fn get_tame_flags(&self) -> u8 {
        let mut flags = 0u8;
        if self.is_sitting.load(Ordering::Relaxed) {
            flags |= 0x01;
        }
        if self.is_tame.load(Ordering::Relaxed) {
            flags |= 0x04;
        }
        flags
    }
}

impl AgeableMob for WolfEntity {
    fn get_ageable_data(&self) -> &crate::entity::ageable::AgeableData {
        &self.ageable_data
    }
}

impl Animal for WolfEntity {
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        let item = item_stack.get_item();
        item.has_tag(&tag::Item::MINECRAFT_WOLF_FOOD) || item == &Item::BONE
    }
}

impl NBTStorage for WolfEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.mob_entity.living_entity.write_nbt(nbt).await;
            self.write_ageable_nbt(nbt);
            self.write_animal_nbt(nbt);
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
            nbt.put_byte(
                "CollarColor",
                self.collar_color.load(Ordering::Relaxed) as i8,
            );
            nbt.put_bool("IsTame", self.is_tame.load(Ordering::Relaxed));
            nbt.put_bool("Sitting", self.is_sitting.load(Ordering::Relaxed));
            if let Some(owner) = self.owner.load() {
                nbt.put_uuid("Owner", owner);
            }
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.mob_entity.living_entity.read_nbt_non_mut(nbt).await;
            self.read_ageable_nbt(nbt);
            self.read_animal_nbt(nbt);
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
            if let Some(collar) = nbt.get_byte("CollarColor") {
                self.collar_color.store(collar as u8, Ordering::Relaxed);
            } else if let Some(collar_int) = nbt.get_int("CollarColor") {
                self.collar_color.store(collar_int as u8, Ordering::Relaxed);
            }
            if let Some(sitting) = nbt.get_bool("Sitting") {
                self.is_sitting.store(sitting, Ordering::Relaxed);
            }
            if let Some(owner) = nbt.get_uuid("Owner") {
                self.owner.store(Some(owner));
                self.is_tame.store(true, Ordering::Relaxed);
            } else if let Some(is_tame) = nbt.get_bool("IsTame") {
                self.is_tame.store(is_tame, Ordering::Relaxed);
            }
        })
    }
}

impl Mob for WolfEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn get_owner_uuid(&self) -> Option<Uuid> {
        self.owner.load()
    }

    fn is_sitting(&self) -> bool {
        self.is_sitting.load(Ordering::Relaxed)
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

    fn mob_init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            let entity = self.get_entity();
            let is_baby = entity.age.load(Ordering::Relaxed) < 0;
            if is_baby {
                entity.send_meta_data(
                    &[Metadata::new(
                        pumpkin_data::tracked_data::wolf::BABY_ID,
                        true,
                    )],
                    None,
                );
            }
            entity.send_meta_data(
                &[Metadata::new(
                    pumpkin_data::tracked_data::wolf::TAMEABLE_FLAGS,
                    self.get_tame_flags(),
                )],
                None,
            );
            entity.send_meta_data(
                &[Metadata::new(
                    pumpkin_data::tracked_data::wolf::COLLAR_COLOR,
                    VarInt(self.collar_color.load(Ordering::Relaxed) as i32),
                )],
                None,
            );
            entity.send_meta_data(
                &[Metadata::new(
                    pumpkin_data::tracked_data::wolf::WOLF_VARIANT_ID,
                    VarInt(self.variant.load(Ordering::Relaxed) as i32),
                )],
                None,
            );
            entity.send_meta_data(
                &[Metadata::new(
                    pumpkin_data::tracked_data::wolf::OWNER_UUID,
                    self.owner.load(),
                )],
                None,
            );
        })
    }
}
