use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::{AtomicBool, AtomicI32};

use pumpkin_data::entity::EntityType;
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_nbt::compound::NbtCompound;

use crate::entity::mob::Mob;
use crate::entity::r#type::from_type;
use crate::entity::{Entity, EntityBase};

#[derive(Clone, Copy)]
pub struct ConversionParams {
    pub keep_equipment: bool,
    pub preserve_can_pick_up_loot: bool,
}

impl ConversionParams {
    #[must_use]
    pub const fn single(keep_equipment: bool, preserve_can_pick_up_loot: bool) -> Self {
        Self {
            keep_equipment,
            preserve_can_pick_up_loot,
        }
    }
}

pub fn convert_to(
    from: &dyn Mob,
    entity_type: &'static EntityType,
    params: ConversionParams,
    after_conversion: impl FnOnce(&dyn Mob),
) -> Option<Arc<dyn EntityBase>> {
    let from_entity = from.get_entity();
    if from_entity.is_removed() {
        return None;
    }
    let world = from_entity.world.load_full();
    let converted = from_type(
        entity_type,
        from_entity.pos.load(),
        &world,
        uuid::Uuid::new_v4(),
    );
    let to = converted.get_mob()?;
    convert_single(from, to, params);
    after_conversion(to);
    let leash_holder = from_entity.get_leash_holder();
    world.spawn_entity(converted.clone());
    if let Some(holder) = leash_holder {
        converted.get_entity().attach_leash(&holder);
    }
    from_entity.remove();
    Some(converted)
}

/// Spawns one split-off child of a dying mob, run while the parent is removed.
/// Removal already detached its riders and leash, so only the common data is copied.
pub fn split_on_death(
    from: &dyn Mob,
    entity_type: &'static EntityType,
    after_conversion: impl FnOnce(&dyn Mob),
) {
    let from_entity = from.get_entity();
    let world = from_entity.world.load_full();
    let converted = from_type(
        entity_type,
        from_entity.pos.load(),
        &world,
        uuid::Uuid::new_v4(),
    );
    let Some(to) = converted.get_mob() else {
        return;
    };
    convert_common(from, to, ConversionParams::single(false, false));
    after_conversion(to);
    world.spawn_entity(converted);
}

fn convert_single(from: &dyn Mob, to: &dyn Mob, params: ConversionParams) {
    let from_mob = from.get_mob_entity();
    let to_mob = to.get_mob_entity();
    let from_living = &from_mob.living_entity;
    let to_living = &to_mob.living_entity;
    let from_entity = &from_living.entity;
    let to_entity = &to_living.entity;

    to_entity.set_rotation(from_entity.yaw.load(), from_entity.pitch.load());
    to_entity.velocity.store(from_entity.velocity.load());

    if params.keep_equipment {
        let mut from_equipment = from_living
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut to_equipment = to_living
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for (slot, stack) in std::mem::take(&mut from_equipment.equipment) {
            if stack.is_empty() {
                continue;
            }
            to_equipment.put(&slot, stack);
            let chance = from_mob.drop_chance(&slot);
            to_living
                .equipment_drop_chances
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .insert(slot, chance);
        }
    }

    to_living
        .fall_distance
        .store(from_living.fall_distance.load());
    to_entity
        .fall_flying
        .store(from_entity.fall_flying.load(Relaxed), Relaxed);
    to_living
        .last_hurt_by_player
        .store(from_living.last_hurt_by_player.load());
    to_living.last_hurt_by_player_memory_time.store(
        from_living.last_hurt_by_player_memory_time.load(Relaxed),
        Relaxed,
    );
    to_entity.body_yaw.store(from_entity.body_yaw.load());
    to_entity
        .on_ground
        .store(from_entity.on_ground.load(Relaxed), Relaxed);

    convert_common(from, to, params);
}

fn convert_common(from: &dyn Mob, to: &dyn Mob, params: ConversionParams) {
    let from_mob = from.get_mob_entity();
    let to_mob = to.get_mob_entity();
    let from_living = &from_mob.living_entity;
    let to_living = &to_mob.living_entity;
    let from_entity = &from_living.entity;
    let to_entity = &to_living.entity;

    to_living.set_absorption(from_living.absorption.load());
    let effects: Vec<_> = from_living
        .active_effects
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .values()
        .cloned()
        .collect();
    for effect in effects {
        to_living.add_effect(effect);
    }

    if from_living.is_baby() {
        to.spawn_as_baby();
    }
    if let (Some(from_ageable), Some(to_ageable)) = (from.as_ageable(), to.as_ageable()) {
        to_ageable.set_age(from_ageable.get_age());
        let from_data = from_ageable.get_ageable_data();
        let to_data = to_ageable.get_ageable_data();
        to_data
            .forced_age
            .store(from_data.forced_age.load(Relaxed), Relaxed);
        to_data
            .forced_age_timer
            .store(from_data.forced_age_timer.load(Relaxed), Relaxed);
    }

    if params.preserve_can_pick_up_loot {
        to_mob.set_can_pick_up_loot(from_mob.can_pick_up_loot());
    }
    to_mob.set_left_handed(from_mob.is_left_handed());
    to_mob.set_no_ai(from_mob.is_no_ai());
    if from_mob.persistence_required.load(Relaxed) {
        to_mob.persistence_required.store(true, Relaxed);
    }

    to_entity.set_custom_name_visible(from_entity.custom_name_visible.load(Relaxed));
    to_entity.set_on_fire(from_entity.is_on_fire());
    to_entity.set_invulnerable(from_entity.invulnerable.load(Relaxed));
    to_living
        .hurt_cooldown
        .store(from_living.hurt_cooldown.load(Relaxed), Relaxed);
    to_entity.set_has_no_gravity(from_entity.has_no_gravity.load(Relaxed));
    to_entity
        .portal_cooldown
        .store(from_entity.portal_cooldown.load(Relaxed), Relaxed);
    to_entity.set_silent(from_entity.silent.load(Relaxed));
    to_entity
        .scoreboard_tags
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .extend(
            from_entity
                .scoreboard_tags
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .iter()
                .cloned(),
        );

    if let Some(custom_name) = &**from_entity.custom_name.load() {
        to_entity.set_custom_name(custom_name.clone());
    }
    let custom_data = from_entity
        .custom_data
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    *to_entity
        .custom_data
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = custom_data;
}

const NOT_CONVERTING: i32 = -1;

/// Counts how long a mob stays afflicted, then how long it takes to convert.
pub struct ConversionTracker {
    converting_data: TrackedData,
    affliction_time_tag: &'static str,
    total_affliction_time: i32,
    conversion_time_tag: &'static str,
    total_conversion_time: i32,
    affliction_time: AtomicI32,
    conversion_time: AtomicI32,
    converting: AtomicBool,
}

impl ConversionTracker {
    #[must_use]
    pub const fn new(
        converting_data: TrackedData,
        affliction_time_tag: &'static str,
        total_affliction_time: i32,
        conversion_time_tag: &'static str,
        total_conversion_time: i32,
    ) -> Self {
        Self {
            converting_data,
            affliction_time_tag,
            total_affliction_time,
            conversion_time_tag,
            total_conversion_time,
            affliction_time: AtomicI32::new(0),
            conversion_time: AtomicI32::new(0),
            converting: AtomicBool::new(false),
        }
    }

    /// Runs `convert` once the conversion time runs out.
    pub fn tick(&self, mob: &dyn Mob, afflicted: bool, convert: impl FnOnce()) {
        let entity = mob.get_entity();
        if !entity.is_alive() || mob.get_mob_entity().is_no_ai() {
            return;
        }
        if !afflicted {
            self.affliction_time.store(NOT_CONVERTING, Relaxed);
            self.set_converting(entity, false);
        } else if self.is_converting() {
            if self.conversion_time.fetch_sub(1, Relaxed) - 1 < 0 {
                convert();
            }
        } else if self.affliction_time.fetch_add(1, Relaxed) + 1 >= self.total_affliction_time {
            self.start_conversion(entity, self.total_conversion_time);
        }
    }

    #[must_use]
    pub fn is_converting(&self) -> bool {
        self.converting.load(Relaxed)
    }

    fn set_converting(&self, entity: &Entity, converting: bool) {
        if self.converting.swap(converting, Relaxed) != converting {
            entity.set_synced_data(self.converting_data, converting);
        }
    }

    pub fn start_conversion(&self, entity: &Entity, time: i32) {
        self.conversion_time.store(time, Relaxed);
        self.set_converting(entity, true);
    }

    pub fn write_nbt(&self, nbt: &mut NbtCompound, afflicted: bool) {
        nbt.put_int(
            self.conversion_time_tag,
            if self.is_converting() {
                self.conversion_time.load(Relaxed)
            } else {
                NOT_CONVERTING
            },
        );
        nbt.put_int(
            self.affliction_time_tag,
            if afflicted {
                self.affliction_time.load(Relaxed)
            } else {
                NOT_CONVERTING
            },
        );
    }

    pub fn read_nbt(&self, entity: &Entity, nbt: &NbtCompound) {
        self.affliction_time
            .store(nbt.get_int(self.affliction_time_tag).unwrap_or(0), Relaxed);
        match nbt
            .get_int(self.conversion_time_tag)
            .unwrap_or(NOT_CONVERTING)
        {
            NOT_CONVERTING => self.set_converting(entity, false),
            time => self.start_conversion(entity, time),
        }
    }
}
