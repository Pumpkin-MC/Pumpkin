use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, AtomicU8, Ordering},
};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::entity::{EntityStatus, EntityType};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::bedrock::client::actor_event::ActorEventType;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::Metadata;
use rand::RngExt;
use uuid::Uuid;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture,
    ai::goal::{
        active_target::ActiveTargetGoal, avoid_entity::AvoidEntityGoal, breed::BreedGoal,
        escape_danger::EscapeDangerGoal, follow_owner::FollowOwnerGoal,
        follow_parent::FollowParentGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, swim::SwimGoal, tempt::TemptGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
    passive::animal::Animal,
    player::Player,
};

const TEMPT_ITEMS: &[&Item] = &[&Item::COD, &Item::SALMON];

fn get_dye_color_from_item(item: &Item) -> Option<u8> {
    let key = item.registry_key;
    if key.contains("white") {
        Some(0)
    } else if key.contains("orange") {
        Some(1)
    } else if key.contains("magenta") {
        Some(2)
    } else if key.contains("light_blue") {
        Some(3)
    } else if key.contains("yellow") {
        Some(4)
    } else if key.contains("lime") {
        Some(5)
    } else if key.contains("pink") {
        Some(6)
    } else if key.contains("light_gray") {
        Some(8)
    } else if key.contains("gray") {
        Some(7)
    } else if key.contains("cyan") {
        Some(9)
    } else if key.contains("purple") {
        Some(10)
    } else if key.contains("blue") {
        Some(11)
    } else if key.contains("brown") {
        Some(12)
    } else if key.contains("green") {
        Some(13)
    } else if key.contains("red") {
        Some(14)
    } else if key.contains("black") {
        Some(15)
    } else {
        None
    }
}

pub struct CatEntity {
    pub mob_entity: MobEntity,
    pub variant: AtomicU8,
    pub sound_variant: AtomicU8,
    pub collar_color: AtomicU8,
    pub is_tame: AtomicBool,
    pub is_sitting: AtomicBool,
    pub is_lying: AtomicBool,
    pub relax_state_one: AtomicBool,
    pub owner: AtomicCell<Option<Uuid>>,
}

impl CatEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let cat = Self {
            mob_entity,
            variant: AtomicU8::new(1),       // Default to black
            sound_variant: AtomicU8::new(0), // Default to classic
            collar_color: AtomicU8::new(14), // Default to red
            is_tame: AtomicBool::new(false),
            is_sitting: AtomicBool::new(false),
            is_lying: AtomicBool::new(false),
            relax_state_one: AtomicBool::new(false),
            owner: AtomicCell::new(None),
        };
        let mob_arc = Arc::new(cat);
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

            // Goal 1: SwimGoal (FloatGoal)
            goal_selector.add_goal(1, Box::new(SwimGoal::default()));
            // Goal 1: TamableAnimalPanicGoal (EscapeDangerGoal)
            goal_selector.add_goal(1, EscapeDangerGoal::new(1.5));
            // Goal 4: CatTemptGoal
            goal_selector.add_goal(4, Box::new(TemptGoal::new(0.6, TEMPT_ITEMS)));
            // Goal 4: CatAvoidEntityGoal (when untamed)
            goal_selector.add_goal(
                4,
                Box::new(AvoidEntityGoal::new(&EntityType::PLAYER, 16.0, 0.8, 1.33)),
            );
            // Goal 5: BreedGoal
            goal_selector.add_goal(5, BreedGoal::new(0.8));
            // Goal 6: FollowOwnerGoal
            goal_selector.add_goal(6, FollowOwnerGoal::new(1.0, 10.0, 5.0));
            // Goal 9: FollowParentGoal
            goal_selector.add_goal(9, Box::new(FollowParentGoal::new(0.8)));
            // Goal 11: WanderAroundGoal
            goal_selector.add_goal(11, Box::new(WanderAroundGoal::new(0.8)));
            // Goal 12: LookAtPlayerGoal
            goal_selector.add_goal(
                12,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 10.0),
            );
            // Goal 12: RandomLookAroundGoal
            goal_selector.add_goal(12, Box::new(RandomLookAroundGoal::default()));
        };

        {
            let mut target_selector = mob_arc
                .mob_entity
                .target_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            // Target Goal 1: NonTameRandomTargetGoal for Rabbit and Turtle
            target_selector.add_goal(
                1,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::RABBIT, false),
            );
            target_selector.add_goal(
                1,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::TURTLE, false),
            );
        };

        mob_arc
    }

    pub fn get_tame_flags(&self) -> u8 {
        let mut flags = 0u8;
        if self.is_sitting() {
            flags |= 0x01;
        }
        if self.is_tame() {
            flags |= 0x04;
        }
        flags
    }

    pub fn is_tame(&self) -> bool {
        self.is_tame.load(Ordering::Relaxed)
    }

    pub fn is_sitting(&self) -> bool {
        self.is_sitting.load(Ordering::Relaxed)
    }

    pub fn is_lying(&self) -> bool {
        self.is_lying.load(Ordering::Relaxed)
    }

    pub fn is_relax_state_one(&self) -> bool {
        self.relax_state_one.load(Ordering::Relaxed)
    }

    pub fn get_collar_color(&self) -> u8 {
        self.collar_color.load(Ordering::Relaxed)
    }

    pub fn set_collar_color(&self, color: u8) {
        self.collar_color.store(color, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.send_meta_data(
            &[Metadata::new(
                TrackedData::COLLAR_COLOR,
                MetaDataType::INT,
                VarInt(color as i32),
            )],
            None,
        );
    }

    pub fn set_sitting(&self, sitting: bool) {
        self.is_sitting.store(sitting, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.send_meta_data(
            &[Metadata::new(
                TrackedData::TAMEABLE_FLAGS,
                MetaDataType::BYTE,
                self.get_tame_flags(),
            )],
            None,
        );
    }

    pub fn set_tame(&self, tame: bool, owner: Option<Uuid>) {
        self.is_tame.store(tame, Ordering::Relaxed);
        self.owner.store(owner);
        let entity = self.get_entity();
        entity.send_meta_data(
            &[Metadata::new(
                TrackedData::TAMEABLE_FLAGS,
                MetaDataType::BYTE,
                self.get_tame_flags(),
            )],
            None,
        );
        entity.send_meta_data(
            &[Metadata::new(
                TrackedData::OWNER_UUID,
                MetaDataType::OPTIONAL_UUID,
                owner,
            )],
            None,
        );
    }

    pub fn set_lying(&self, lying: bool) {
        self.is_lying.store(lying, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.send_meta_data(
            &[Metadata::new(
                TrackedData::IS_LYING,
                MetaDataType::BOOLEAN,
                lying,
            )],
            None,
        );
    }

    pub fn set_relax_state_one(&self, relax: bool) {
        self.relax_state_one.store(relax, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.send_meta_data(
            &[Metadata::new(
                TrackedData::RELAX_STATE_ONE,
                MetaDataType::BOOLEAN,
                relax,
            )],
            None,
        );
    }

    pub fn play_eating_sound(&self) {
        let mob_entity = self.get_mob_entity();
        let entity = &mob_entity.living_entity.entity;
        let world = entity.world.load();
        world.play_sound(
            pumpkin_data::sound::Sound::EntityCatEat,
            pumpkin_data::sound::SoundCategory::Neutral,
            &entity.pos.load(),
        );
    }
}

impl NBTStorage for CatEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.mob_entity.living_entity.write_nbt(nbt).await;
            self.write_animal_nbt(nbt);
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
            nbt.put_string("sound_variant", "minecraft:classic".to_string());
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
            self.read_animal_nbt(nbt);
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

impl Animal for CatEntity {
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        let item = item_stack.get_item();
        item.has_tag(&tag::Item::MINECRAFT_CAT_FOOD) || item == &Item::COD || item == &Item::SALMON
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
        self.is_sitting.load(Ordering::Relaxed)
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
                    TrackedData::TAMEABLE_FLAGS,
                    MetaDataType::BYTE,
                    self.get_tame_flags(),
                )],
                None,
            );
            entity.send_meta_data(
                &[Metadata::new(
                    TrackedData::OWNER_UUID,
                    MetaDataType::OPTIONAL_UUID,
                    self.owner.load(),
                )],
                None,
            );
            entity.send_meta_data(
                &[Metadata::new(
                    TrackedData::CAT_VARIANT,
                    MetaDataType::CAT_VARIANT,
                    VarInt(self.variant.load(Ordering::Relaxed) as i32),
                )],
                None,
            );
            entity.send_meta_data(
                &[Metadata::new(
                    TrackedData::IS_LYING,
                    MetaDataType::BOOLEAN,
                    self.is_lying.load(Ordering::Relaxed),
                )],
                None,
            );
            entity.send_meta_data(
                &[Metadata::new(
                    TrackedData::RELAX_STATE_ONE,
                    MetaDataType::BOOLEAN,
                    self.relax_state_one.load(Ordering::Relaxed),
                )],
                None,
            );
            entity.send_meta_data(
                &[Metadata::new(
                    TrackedData::COLLAR_COLOR,
                    MetaDataType::INT,
                    VarInt(self.collar_color.load(Ordering::Relaxed) as i32),
                )],
                None,
            );
            entity.send_meta_data(
                &[Metadata::new(
                    TrackedData::SOUND_VARIANT,
                    MetaDataType::CAT_SOUND_VARIANT,
                    VarInt(self.sound_variant.load(Ordering::Relaxed) as i32),
                )],
                None,
            );
        })
    }

    fn mob_interact<'a>(
        &'a self,
        player: &'a Arc<Player>,
        item_stack: &'a mut ItemStack,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            let item = item_stack.get_item();
            let is_food = self.is_food(item_stack);

            if self.is_tame() {
                if self.get_owner_uuid() == Some(player.gameprofile.id) {
                    if item.has_tag(&tag::Item::MINECRAFT_CAT_COLLAR_DYES)
                        || item.has_tag(&tag::Item::C_DYES)
                    {
                        if let Some(color) = get_dye_color_from_item(item)
                            && color != self.get_collar_color()
                        {
                            self.set_collar_color(color);
                            item_stack.decrement_unless_creative(player.gamemode.load(), 1);
                            return true;
                        }
                    } else if is_food
                        && self.mob_entity.living_entity.health.load()
                            < self.mob_entity.living_entity.get_max_health()
                    {
                        item_stack.decrement_unless_creative(player.gamemode.load(), 1);
                        self.mob_entity.living_entity.heal(2.0);
                        self.play_eating_sound();
                        return true;
                    }

                    let parent_interaction = self.mob_entity.mob_interact(player, item_stack).await;
                    if !parent_interaction {
                        self.set_sitting(!self.is_sitting());
                        return true;
                    }
                    return parent_interaction;
                }
            } else if is_food {
                item_stack.decrement_unless_creative(player.gamemode.load(), 1);
                self.play_eating_sound();

                let mut rng = rand::rng();
                if rng.random_range(0..3) == 0 {
                    self.set_tame(true, Some(player.gameprofile.id));
                    self.set_sitting(true);
                    self.get_entity().world.load().send_entity_status(
                        self.get_entity(),
                        EntityStatus::TamingSucceeded,
                        Some(ActorEventType::TamingSucceeded),
                    );
                } else {
                    self.get_entity().world.load().send_entity_status(
                        self.get_entity(),
                        EntityStatus::TamingFailed,
                        Some(ActorEventType::TamingFailed),
                    );
                }

                return true;
            }

            self.mob_entity.mob_interact(player, item_stack).await
        })
    }
}
