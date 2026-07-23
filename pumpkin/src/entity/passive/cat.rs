use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, AtomicU8, Ordering},
};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::entity::{EntityStatus, EntityType};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::particle::Particle;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::math::vector3::Vector3;
use uuid::Uuid;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture,
    ai::goal::{
        active_target::ActiveTargetGoal, breed::BreedGoal, escape_danger::EscapeDangerGoal,
        follow_owner::FollowOwnerGoal, follow_parent::FollowParentGoal,
        leap_at_target::LeapAtTargetGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, melee_attack::MeleeAttackGoal, sit::SitGoal,
        swim::SwimGoal, tempt::TemptGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
    player::Player,
};

const TEMPT_ITEMS: &[&Item] = &[&Item::COD, &Item::SALMON];
const SITTING_FLAG: u8 = 0x1;
const TAMED_FLAG: u8 = 0x4;

pub struct CatEntity {
    pub mob_entity: MobEntity,
    pub variant: AtomicU8,
    owner: AtomicCell<Option<Uuid>>,
    sitting: AtomicBool,
    tamed: AtomicBool,
}

impl CatEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let cat = Self {
            mob_entity,
            variant: AtomicU8::new(9),
            owner: AtomicCell::new(None),
            sitting: AtomicBool::new(false),
            tamed: AtomicBool::new(false),
        };
        let mob_arc = Arc::new(cat);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();
            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();

            goal_selector.add_goal(1, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, EscapeDangerGoal::new(1.5));
            goal_selector.add_goal(2, SitGoal::new());
            goal_selector.add_goal(4, Box::new(TemptGoal::new(0.6, TEMPT_ITEMS)));
            goal_selector.add_goal(5, BreedGoal::new(0.8));
            goal_selector.add_goal(5, Box::new(LeapAtTargetGoal::new(0.3)));
            goal_selector.add_goal(6, Box::new(MeleeAttackGoal::new(1.0, true)));
            goal_selector.add_goal(7, FollowOwnerGoal::new(1.0, 10.0, 5.0));
            goal_selector.add_goal(9, Box::new(FollowParentGoal::new(0.8)));
            goal_selector.add_goal(11, Box::new(WanderAroundGoal::new(0.8)));
            goal_selector.add_goal(
                12,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 10.0),
            );
            goal_selector.add_goal(12, Box::new(RandomLookAroundGoal::default()));

            // Hunt rabbits / baby turtles (vanilla cat prey).
            target_selector.add_goal(
                1,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::RABBIT, true),
            );
            target_selector.add_goal(
                1,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::TURTLE, true),
            );
        };

        mob_arc
    }

    fn is_tamed(&self) -> bool {
        self.tamed.load(Ordering::Relaxed)
    }

    fn set_sitting(&self, sitting: bool) {
        self.sitting.store(sitting, Ordering::Relaxed);
        self.sync_tameable_flags();
        if sitting {
            let mut navigator = self.mob_entity.navigator.lock().unwrap();
            navigator.stop();
        }
    }

    fn set_tamed_owner(&self, owner: Uuid) {
        self.tamed.store(true, Ordering::Relaxed);
        self.owner.store(Some(owner));
        self.sitting.store(false, Ordering::Relaxed);
        self.sync_tameable_flags();
    }

    fn tameable_flags(&self) -> u8 {
        let mut flags = 0u8;
        if self.sitting.load(Ordering::Relaxed) {
            flags |= SITTING_FLAG;
        }
        if self.tamed.load(Ordering::Relaxed) {
            flags |= TAMED_FLAG;
        }
        flags
    }

    fn sync_tameable_flags(&self) {
        self.mob_entity.living_entity.entity.send_meta_data(
            &[Metadata::new(
                TrackedData::TAMEABLE_FLAGS,
                MetaDataType::BYTE,
                self.tameable_flags(),
            )],
            None,
        );
    }
}

impl NBTStorage for CatEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.mob_entity.living_entity.write_nbt(nbt).await;
            let variant_str = match self.variant.load(Ordering::Relaxed) {
                0 => "minecraft:all_black",
                1 => "minecraft:black",
                2 => "minecraft:british_shorthair",
                3 => "minecraft:calico",
                4 => "minecraft:jellie",
                5 => "minecraft:persian",
                6 => "minecraft:ragdoll",
                7 => "minecraft:red",
                8 => "minecraft:siamese",
                10 => "minecraft:white",
                _ => "minecraft:tabby",
            };
            nbt.put_string("variant", variant_str.to_string());
            nbt.put_bool("Sitting", self.sitting.load(Ordering::Relaxed));
            if let Some(owner) = self.owner.load() {
                nbt.put_uuid("Owner", owner);
            }
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
                    "all_black" => 0,
                    "black" => 1,
                    "british_shorthair" => 2,
                    "calico" => 3,
                    "jellie" => 4,
                    "persian" => 5,
                    "ragdoll" => 6,
                    "red" => 7,
                    "siamese" => 8,
                    "white" => 10,
                    _ => 9,
                };
                self.variant.store(variant, Ordering::Relaxed);
            }
            self.sitting
                .store(nbt.get_bool("Sitting").unwrap_or(false), Ordering::Relaxed);
            if let Some(owner) = nbt.get_uuid("Owner") {
                self.tamed.store(true, Ordering::Relaxed);
                self.owner.store(Some(owner));
            }
        })
    }
}

impl Mob for CatEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn get_owner_uuid(&self) -> Option<Uuid> {
        self.owner.load()
    }

    fn is_sitting(&self) -> bool {
        self.sitting.load(Ordering::Relaxed)
    }

    fn mob_set_variant_name(&self, name: &str) {
        let variant = match name.strip_prefix("minecraft:").unwrap_or(name) {
            "all_black" => 0,
            "black" => 1,
            "british_shorthair" => 2,
            "calico" => 3,
            "jellie" => 4,
            "persian" => 5,
            "ragdoll" => 6,
            "red" => 7,
            "siamese" => 8,
            "white" => 10,
            _ => 9,
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
                        TrackedData::BABY_ID,
                        MetaDataType::BOOLEAN,
                        true,
                    )],
                    None,
                );
            }
            entity.send_meta_data(
                &[Metadata::new(
                    TrackedData::CAT_VARIANT,
                    MetaDataType::CAT_VARIANT,
                    VarInt(self.variant.load(Ordering::Relaxed) as i32),
                )],
                None,
            );
            self.sync_tameable_flags();
        })
    }

    fn mob_interact<'a>(
        &'a self,
        player: &'a Arc<Player>,
        item_stack: &'a mut ItemStack,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            let entity = &self.mob_entity.living_entity.entity;
            let world = entity.world.load();
            let pos = entity.pos.load();
            let is_fish = TEMPT_ITEMS.iter().any(|i| i.id == item_stack.item.id);

            if !self.is_tamed() {
                if !is_fish {
                    return false;
                }
                item_stack.decrement_unless_creative(player.gamemode.load(), 1);
                // Vanilla cat tame ~1/3
                let success = {
                    use rand::RngExt;
                    rand::rng().random_range(0..3) == 0
                };
                if success {
                    self.set_tamed_owner(player.gameprofile.id);
                    world.send_entity_status(entity, EntityStatus::TamingSucceeded);
                    world.spawn_particle(
                        pos + Vector3::new(0.0, f64::from(entity.height()), 0.0),
                        Vector3::new(0.5, 0.5, 0.5),
                        1.0,
                        7,
                        Particle::Heart,
                    );
                } else {
                    world.send_entity_status(entity, EntityStatus::TamingFailed);
                }
                return true;
            }

            let Some(owner) = self.get_owner_uuid() else {
                return false;
            };
            if owner != player.gameprofile.id {
                return false;
            }
            if !item_stack.is_empty() {
                return false;
            }
            let new_sitting = !self.is_sitting();
            self.set_sitting(new_sitting);
            world.play_sound(Sound::EntityCatAmbient, SoundCategory::Neutral, &pos);
            true
        })
    }
}
