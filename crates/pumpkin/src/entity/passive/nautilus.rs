use crossbeam::atomic::AtomicCell;
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicI32, Ordering},
};
use tokio::sync::Mutex;
use uuid::Uuid;

use pumpkin_data::{
    effect::StatusEffect,
    entity::EntityStatus,
    item_stack::ItemStack,
    particle::Particle,
    potion::Effect,
    sound::{Sound, SoundCategory},
};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::math::vector3::Vector3;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NbtFuture,
    mob::{Mob, MobEntity},
    passive::animal::Animal,
    player::Player,
};

pub struct NautilusEntity {
    pub mob_entity: MobEntity,
    pub is_tame: AtomicBool,
    pub owner: AtomicCell<Option<Uuid>>,
    pub is_dashing: AtomicBool,
    pub dash_cooldown: AtomicI32,
    pub is_saddled: AtomicBool,
    pub inventory: Mutex<Vec<ItemStack>>,
}

impl NautilusEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let nautilus = Self {
            mob_entity,
            is_tame: AtomicBool::new(false),
            owner: AtomicCell::new(None),
            is_dashing: AtomicBool::new(false),
            dash_cooldown: AtomicI32::new(0),
            is_saddled: AtomicBool::new(false),
            inventory: Mutex::new(vec![ItemStack::new(0, &pumpkin_data::item::Item::AIR); 9]),
        };

        Arc::new(nautilus)
    }

    pub fn is_dashing(&self) -> bool {
        self.is_dashing.load(Ordering::Relaxed)
    }

    pub fn set_dashing(&self, dashing: bool) {
        self.is_dashing.store(dashing, Ordering::Relaxed);
        self.mob_entity.living_entity.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::nautilus::DASH,
                dashing,
            )],
            None,
        );
    }

    pub fn is_tame(&self) -> bool {
        self.is_tame.load(Ordering::Relaxed)
    }

    pub fn set_tame(&self, tame: bool, owner: Option<Uuid>) {
        self.is_tame.store(tame, Ordering::Relaxed);
        self.owner.store(owner);
    }

    pub fn get_ambient_sound(&self) -> Sound {
        let is_baby = self
            .mob_entity
            .living_entity
            .entity
            .age
            .load(Ordering::Relaxed)
            < 0;
        let is_water = self
            .mob_entity
            .living_entity
            .entity
            .touching_water
            .load(Ordering::Relaxed);
        if is_baby {
            if is_water {
                Sound::EntityBabyNautilusAmbient
            } else {
                Sound::EntityBabyNautilusAmbientLand
            }
        } else if is_water {
            Sound::EntityNautilusAmbient
        } else {
            Sound::EntityNautilusAmbientLand
        }
    }

    pub fn get_hurt_sound(&self) -> Sound {
        let is_baby = self
            .mob_entity
            .living_entity
            .entity
            .age
            .load(Ordering::Relaxed)
            < 0;
        let is_water = self
            .mob_entity
            .living_entity
            .entity
            .touching_water
            .load(Ordering::Relaxed);
        if is_baby {
            if is_water {
                Sound::EntityBabyNautilusHurt
            } else {
                Sound::EntityBabyNautilusHurtLand
            }
        } else if is_water {
            Sound::EntityNautilusHurt
        } else {
            Sound::EntityNautilusHurtLand
        }
    }

    pub fn get_death_sound(&self) -> Sound {
        let is_baby = self
            .mob_entity
            .living_entity
            .entity
            .age
            .load(Ordering::Relaxed)
            < 0;
        let is_water = self
            .mob_entity
            .living_entity
            .entity
            .touching_water
            .load(Ordering::Relaxed);
        if is_baby {
            if is_water {
                Sound::EntityBabyNautilusDeath
            } else {
                Sound::EntityBabyNautilusDeathLand
            }
        } else if is_water {
            Sound::EntityNautilusDeath
        } else {
            Sound::EntityNautilusDeathLand
        }
    }

    pub fn get_dash_sound(&self) -> Sound {
        let is_water = self
            .mob_entity
            .living_entity
            .entity
            .touching_water
            .load(Ordering::Relaxed);
        if is_water {
            Sound::EntityNautilusDash
        } else {
            Sound::EntityNautilusDashLand
        }
    }

    pub fn get_dash_ready_sound(&self) -> Sound {
        let is_water = self
            .mob_entity
            .living_entity
            .entity
            .touching_water
            .load(Ordering::Relaxed);
        if is_water {
            Sound::EntityNautilusDashReady
        } else {
            Sound::EntityNautilusDashReadyLand
        }
    }

    pub fn get_eat_sound(&self) -> Sound {
        let is_baby = self
            .mob_entity
            .living_entity
            .entity
            .age
            .load(Ordering::Relaxed)
            < 0;
        if is_baby {
            Sound::EntityBabyNautilusEat
        } else {
            Sound::EntityNautilusEat
        }
    }

    pub fn get_swim_sound(&self) -> Sound {
        let is_baby = self
            .mob_entity
            .living_entity
            .entity
            .age
            .load(Ordering::Relaxed)
            < 0;
        if is_baby {
            Sound::EntityBabyNautilusSwim
        } else {
            Sound::EntityNautilusSwim
        }
    }
}

impl Animal for NautilusEntity {
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        item_stack.item == &pumpkin_data::item::Item::NAUTILUS_SHELL
            || item_stack.item == &pumpkin_data::item::Item::PUFFERFISH
            || item_stack.item == &pumpkin_data::item::Item::COD
            || item_stack.item == &pumpkin_data::item::Item::SALMON
            || item_stack.item == &pumpkin_data::item::Item::TROPICAL_FISH
    }
}

impl Mob for NautilusEntity {
    fn as_animal(&self) -> Option<&dyn Animal> {
        Some(self)
    }

    fn mob_write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            nbt.put_bool("IsTame", self.is_tame.load(Ordering::Relaxed));
            nbt.put_bool("Saddled", self.is_saddled.load(Ordering::Relaxed));
            nbt.put_int("DashCooldown", self.dash_cooldown.load(Ordering::Relaxed));
            if let Some(owner) = self.owner.load() {
                nbt.put_uuid("Owner", owner);
            }
        })
    }

    fn mob_read_nbt<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            if let Some(is_tame) = nbt.get_bool("IsTame") {
                self.is_tame.store(is_tame, Ordering::Relaxed);
            }
            if let Some(saddled) = nbt.get_bool("Saddled") {
                self.is_saddled.store(saddled, Ordering::Relaxed);
            }
            if let Some(dash) = nbt.get_int("DashCooldown") {
                self.dash_cooldown.store(dash, Ordering::Relaxed);
            }
            if let Some(owner) = nbt.get_uuid("Owner") {
                self.owner.store(Some(owner));
            }
        })
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            self.mob_entity.living_entity.entity.send_meta_data(
                &[Metadata::new(
                    pumpkin_data::tracked_data::nautilus::DASH,
                    self.is_dashing(),
                )],
                None,
            );
        })
    }

    fn mob_tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let entity = &self.mob_entity.living_entity.entity;

            let passengers = entity.passengers.lock().await;
            if let Some(passenger) = passengers.first()
                && let Some(player) = passenger.cast_any().downcast_ref::<Player>()
            {
                let world = entity.world.load();
                let game_time = world.level_time.lock().await.world_age;
                if game_time % 40 == 0 {
                    player
                        .living_entity
                        .add_effect(Effect {
                            effect_type: &StatusEffect::BREATH_OF_THE_NAUTILUS,
                            duration: 60,
                            amplifier: 0,
                            ambient: true,
                            show_particles: true,
                            show_icon: true,
                            blend: true,
                        })
                        .await;
                }
            }

            if self.is_dashing() && self.dash_cooldown.load(Ordering::Relaxed) < 35 {
                self.set_dashing(false);
            }

            let cooldown = self.dash_cooldown.load(Ordering::Relaxed);
            if cooldown > 0 {
                let next = cooldown - 1;
                self.dash_cooldown.store(next, Ordering::Relaxed);
                if next == 0 {
                    let world = entity.world.load();
                    world.play_sound(
                        self.get_dash_ready_sound(),
                        SoundCategory::Neutral,
                        &entity.pos.load(),
                    );
                }
            }

            if entity.touching_water.load(Ordering::Relaxed) {
                let velo = entity.velocity.load();
                let speed = velo.length();
                let prob = (speed * 2.0).clamp(0.15, 1.0);
                if rand::random::<f64>() < prob {
                    let world = entity.world.load();
                    let pos = entity.pos.load();
                    world.spawn_particle(
                        pos + Vector3::new(0.0, 0.25, 0.0),
                        Vector3::new(0.4, 0.4, 0.4),
                        0.5,
                        2,
                        Particle::Bubble,
                    );
                }
            }
        })
    }

    fn mob_interact<'a>(
        &'a self,
        player: &'a Arc<Player>,
        item_stack: &'a mut ItemStack,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            let mob_entity = &self.mob_entity;
            let entity = &mob_entity.living_entity.entity;

            if !self.is_tame() && self.is_food(item_stack) {
                item_stack.decrement_unless_creative(player.gamemode.load(), 1);
                if rand::random::<u32>().is_multiple_of(3) {
                    self.set_tame(true, Some(player.gameprofile.id));
                    let world = entity.world.load();
                    world.send_entity_status(entity, EntityStatus::TamingSucceeded, None);
                } else {
                    let world = entity.world.load();
                    world.send_entity_status(entity, EntityStatus::TamingFailed, None);
                }
                let world = entity.world.load();
                world.play_sound(
                    self.get_eat_sound(),
                    SoundCategory::Neutral,
                    &entity.pos.load(),
                );
                return true;
            }

            if self.is_tame() && !player.get_entity().is_sneaking() {
                if !self.is_saddled.load(Ordering::Relaxed)
                    && item_stack.item == &pumpkin_data::item::Item::SADDLE
                {
                    item_stack.decrement_unless_creative(player.gamemode.load(), 1);
                    self.is_saddled.store(true, Ordering::Relaxed);
                    let world = entity.world.load();
                    world.play_sound(
                        Sound::ItemNautilusSaddleEquip,
                        SoundCategory::Neutral,
                        &entity.pos.load(),
                    );
                    return true;
                }

                let world = player.world();
                if let Some(vehicle) = world.get_entity_by_id(entity.entity_id)
                    && let Some(passenger) = world.get_player_by_id(player.entity_id())
                {
                    entity
                        .add_passenger(vehicle, passenger as Arc<dyn EntityBase>)
                        .await;
                    return true;
                }
            }

            self.animal_interact(player, item_stack, self.get_ambient_sound())
                .await
        })
    }

    fn is_saddled(&self) -> bool {
        self.is_saddled.load(Ordering::Relaxed)
    }

    fn can_be_saddled(&self) -> bool {
        self.mob_entity.living_entity.entity.is_alive()
    }

    fn set_saddled(&self, saddled: bool) {
        self.is_saddled.store(saddled, Ordering::Relaxed);
    }
}
