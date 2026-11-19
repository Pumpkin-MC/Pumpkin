use crossbeam::atomic::AtomicCell;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicI32, Ordering},
};
use uuid::Uuid;

use pumpkin_data::BlockState;
use pumpkin_data::{
    data_component_impl::EquipmentSlot,
    effect::StatusEffect,
    entity::EntityStatus,
    item_stack::ItemStack,
    particle::Particle,
    potion::Effect,
    sound::{Sound, SoundCategory},
};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::bedrock::server::ActorEventID;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

use crate::entity::{
    Entity, EntityBase,
    custom_sound::CustomSound,
    mob::{Mob, MobEntity, equipment::equip_on_target},
    passive::animal::Animal,
    player::Player,
};

pub struct NautilusEntity {
    pub mob_entity: MobEntity,
    pub ageable_data: crate::entity::ageable::AgeableData,
    pub is_tame: AtomicBool,
    pub owner: AtomicCell<Option<Uuid>>,
    pub is_dashing: AtomicBool,
    pub dash_cooldown: AtomicI32,
    pub inventory: Mutex<Vec<ItemStack>>,
}

impl NautilusEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let nautilus = Self {
            mob_entity,
            ageable_data: crate::entity::ageable::AgeableData::default(),
            is_tame: AtomicBool::new(false),
            owner: AtomicCell::new(None),
            is_dashing: AtomicBool::new(false),
            dash_cooldown: AtomicI32::new(0),
            inventory: Mutex::new(vec![ItemStack::new(0, &pumpkin_data::item::Item::AIR); 9]),
        };

        Arc::new(nautilus)
    }

    pub fn is_dashing(&self) -> bool {
        self.is_dashing.load(Ordering::Relaxed)
    }

    pub fn set_dashing(&self, dashing: bool) {
        self.is_dashing.store(dashing, Ordering::Relaxed);
        self.mob_entity
            .living_entity
            .entity
            .set_synced_data(pumpkin_data::tracked_data::nautilus::DASH, dashing);
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
}

impl Animal for NautilusEntity {
    fn play_eating_sound(&self) {
        self.make_sound(if self.mob_entity.living_entity.is_baby() {
            Sound::EntityBabyNautilusEat
        } else {
            Sound::EntityNautilusEat
        });
    }

    fn is_food(&self, item_stack: &ItemStack) -> bool {
        item_stack.item == &pumpkin_data::item::Item::NAUTILUS_SHELL
            || item_stack.item == &pumpkin_data::item::Item::PUFFERFISH
            || item_stack.item == &pumpkin_data::item::Item::COD
            || item_stack.item == &pumpkin_data::item::Item::SALMON
            || item_stack.item == &pumpkin_data::item::Item::TROPICAL_FISH
    }
}

impl CustomSound for NautilusEntity {
    fn hurt_sound(&self) -> Option<Sound> {
        let entity = self.get_entity();
        let is_baby = entity.age.load(Ordering::Relaxed) < 0;
        let is_water = entity.touching_water.load(Ordering::Relaxed);
        Some(if is_baby {
            if is_water {
                Sound::EntityBabyNautilusHurt
            } else {
                Sound::EntityBabyNautilusHurtLand
            }
        } else if is_water {
            Sound::EntityNautilusHurt
        } else {
            Sound::EntityNautilusHurtLand
        })
    }

    fn death_sound(&self) -> Option<Sound> {
        let entity = self.get_entity();
        let is_baby = entity.age.load(Ordering::Relaxed) < 0;
        let is_water = entity.touching_water.load(Ordering::Relaxed);
        Some(if is_baby {
            if is_water {
                Sound::EntityBabyNautilusDeath
            } else {
                Sound::EntityBabyNautilusDeathLand
            }
        } else if is_water {
            Sound::EntityNautilusDeath
        } else {
            Sound::EntityNautilusDeathLand
        })
    }
}

impl crate::entity::ageable::AgeableMob for NautilusEntity {
    fn get_ageable_data(&self) -> &crate::entity::ageable::AgeableData {
        &self.ageable_data
    }
}

impl Mob for NautilusEntity {
    fn as_ageable(&self) -> Option<&dyn crate::entity::ageable::AgeableMob> {
        Some(self)
    }

    fn can_dispenser_equip_into_slot(&self, slot: &EquipmentSlot) -> bool {
        *slot == EquipmentSlot::BODY
            || *slot == EquipmentSlot::SADDLE
            || self.mob_entity.can_pick_up_loot()
    }

    fn is_tamed(&self) -> bool {
        self.is_tame()
    }

    fn get_owner_uuid(&self) -> Option<Uuid> {
        self.owner.load()
    }

    fn mob_controlling_passenger(&self) -> Option<Arc<dyn EntityBase>> {
        self.saddled_controlling_passenger(None)
    }

    fn mob_get_swim_sound(&self) -> Option<Sound> {
        let is_baby = self
            .mob_entity
            .living_entity
            .entity
            .age
            .load(Ordering::Relaxed)
            < 0;
        Some(if is_baby {
            Sound::EntityBabyNautilusSwim
        } else {
            Sound::EntityNautilusSwim
        })
    }

    /// Vanilla `AbstractNautilus` steps silently.
    fn mob_play_step_sound(&self, _pos: &BlockPos, _state: &BlockState) {}

    fn as_custom_sound(&self) -> Option<&dyn crate::entity::custom_sound::CustomSound> {
        Some(self)
    }

    fn as_animal(&self) -> Option<&dyn Animal> {
        Some(self)
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_bool("IsTame", self.is_tame.load(Ordering::Relaxed));
        nbt.put_int("DashCooldown", self.dash_cooldown.load(Ordering::Relaxed));
        if let Some(owner) = self.owner.load() {
            nbt.put_uuid("Owner", owner);
        }
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.mob_entity
            .read_legacy_saddle(nbt.get_bool("Saddled") == Some(true));
        if let Some(is_tame) = nbt.get_bool("IsTame") {
            self.is_tame.store(is_tame, Ordering::Relaxed);
        }
        if let Some(dash) = nbt.get_int("DashCooldown") {
            self.dash_cooldown.store(dash, Ordering::Relaxed);
        }
        if let Some(owner) = nbt.get_uuid("Owner") {
            self.owner.store(Some(owner));
        }
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_init_data_tracker(&self) {
        self.mob_entity.living_entity.entity.set_synced_data(
            pumpkin_data::tracked_data::nautilus::DASH,
            self.is_dashing(),
        );
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        tick_abstract_nautilus(
            &self.mob_entity,
            &self.dash_cooldown,
            self.is_dashing(),
            |dashing| self.set_dashing(dashing),
            || self.get_dash_ready_sound(),
        );
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        interact_abstract_nautilus(
            self,
            player,
            item_stack,
            self.is_tame(),
            |owner| self.set_tame(true, Some(owner)),
            || self.get_eat_sound(),
        )
    }

    fn can_use_slot(&self, slot: &EquipmentSlot) -> bool {
        nautilus_can_use_slot(self, slot)
    }

    fn get_equip_sound(&self, slot: &EquipmentSlot) -> Option<Sound> {
        nautilus_equip_sound(self.get_entity(), slot)
    }
}

pub(super) fn tick_abstract_nautilus(
    mob_entity: &MobEntity,
    dash_cooldown: &AtomicI32,
    is_dashing: bool,
    set_dashing: impl FnOnce(bool),
    dash_ready_sound: impl FnOnce() -> Sound,
) {
    let entity = &mob_entity.living_entity.entity;

    if let Ok(passengers) = entity.passengers.try_lock()
        && let Some(passenger) = passengers.first()
        && let Some(player) = passenger.cast_any().downcast_ref::<Player>()
    {
        let world = entity.world.load();
        let has_effect = player
            .living_entity
            .has_effect(&StatusEffect::BREATH_OF_THE_NAUTILUS);
        if (!has_effect || world.get_world_age() % 40 == 0)
            && let Some(p) = world.get_player_by_uuid(player.gameprofile.id)
        {
            p.add_effect(Effect {
                effect_type: &StatusEffect::BREATH_OF_THE_NAUTILUS,
                duration: 60,
                amplifier: 0,
                ambient: true,
                show_particles: true,
                show_icon: true,
                blend: true,
            });
        }
    }

    if is_dashing && dash_cooldown.load(Ordering::Relaxed) < 35 {
        set_dashing(false);
    }

    let cooldown = dash_cooldown.load(Ordering::Relaxed);
    if cooldown > 0 {
        let next = cooldown - 1;
        dash_cooldown.store(next, Ordering::Relaxed);
        if next == 0 {
            entity.world.load().play_sound(
                dash_ready_sound(),
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
}

pub(super) fn interact_abstract_nautilus<M: Animal>(
    mob: &M,
    player: &Arc<Player>,
    item_stack: &mut ItemStack,
    is_tame: bool,
    on_tamed: impl FnOnce(Uuid),
    eat_sound: impl FnOnce() -> Sound,
) -> bool {
    use pumpkin_data::tag::{self, Taggable};

    let mob_entity = mob.get_mob_entity();
    let entity = &mob_entity.living_entity.entity;

    let consume_nautilus_food = |item_stack: &mut ItemStack| {
        if item_stack
            .item
            .has_tag(&tag::Item::MINECRAFT_NAUTILUS_BUCKET_FOOD)
        {
            crate::item::items::bucket::fill_bucket_stack(
                item_stack,
                player,
                &pumpkin_data::item::Item::WATER_BUCKET,
            );
        } else {
            item_stack.decrement_unless_creative(player.gamemode.load(), 1);
        }
    };

    if !is_tame && mob.is_food(item_stack) {
        consume_nautilus_food(item_stack);
        let world = entity.world.load();
        if rand::random::<u32>().is_multiple_of(3) && mob.approve_taming(player) {
            on_tamed(player.gameprofile.id);
            mob_entity
                .navigator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .stop();
            world.send_entity_status(
                entity,
                EntityStatus::TamingSucceeded,
                ActorEventID::TamingSucceeded,
            );
        } else {
            world.send_entity_status(
                entity,
                EntityStatus::TamingFailed,
                ActorEventID::TamingFailed,
            );
        }
        world.play_sound(eat_sound(), SoundCategory::Neutral, &entity.pos.load());
        return true;
    }

    if !item_stack.is_empty()
        && mob.is_food(item_stack)
        && mob_entity.living_entity.health.load() < mob_entity.living_entity.get_max_health()
    {
        let heal = item_stack
            .get_data_component::<pumpkin_data::data_component_impl::FoodImpl>()
            .map_or(1.0, |food| 2.0 * food.nutrition as f32);
        consume_nautilus_food(item_stack);
        mob_entity.living_entity.heal(heal);
        entity
            .world
            .load()
            .play_sound(eat_sound(), SoundCategory::Neutral, &entity.pos.load());
        return true;
    }

    if is_tame && !player.get_entity().is_sneaking() {
        if equip_on_target(mob, player, item_stack) {
            return true;
        }

        if !mob.is_food(item_stack) {
            let world = player.world();
            if let Some(vehicle) = world.get_entity_by_id(entity.entity_id)
                && let Some(passenger) = world.get_player_by_id(player.entity_id())
            {
                entity.add_passenger(vehicle, passenger as Arc<dyn EntityBase>);
                return true;
            }
        }
    }

    mob.animal_interact(player, item_stack)
}

pub(super) fn nautilus_can_use_slot(mob: &dyn Mob, slot: &EquipmentSlot) -> bool {
    (*slot != EquipmentSlot::SADDLE && *slot != EquipmentSlot::BODY)
        || (mob.get_entity().is_alive()
            && mob.as_ageable().is_none_or(|a| !a.is_baby())
            && mob.is_tamed())
}

pub(super) fn nautilus_equip_sound(entity: &Entity, slot: &EquipmentSlot) -> Option<Sound> {
    if *slot != EquipmentSlot::SADDLE {
        return None;
    }
    Some(if entity.is_under_water() {
        Sound::ItemNautilusSaddleUnderwaterEquip
    } else {
        Sound::ItemNautilusSaddleEquip
    })
}
