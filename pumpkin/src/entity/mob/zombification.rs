//! Vanilla overworld zombification shared by piglins, brutes, and hoglins.
//!
//! Mirrors `AbstractPiglin.customServerAiStep` / `Hoglin` conversion: while
//! `PIGLINS_ZOMBIFY` holds (false only in the nether, `DimensionTypes.java`),
//! a non-immune mob with AI counts `timeInOverworld`; past 300 ticks it plays
//! its converted sound and becomes the zombified type with Nausea 200.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::dimension::Dimension;
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::EntityType;
use pumpkin_data::potion::Effect;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::difficulty::Difficulty;
use uuid::Uuid;

use crate::entity::EntityBase;
use crate::entity::mob::Mob;

/// `AbstractPiglin.CONVERSION_TIME` / `Hoglin.CONVERSION_TIME`.
const CONVERSION_TIME: i32 = 300;

const COPIED_SLOTS: [EquipmentSlot; 6] = [
    EquipmentSlot::MAIN_HAND,
    EquipmentSlot::OFF_HAND,
    EquipmentSlot::FEET,
    EquipmentSlot::LEGS,
    EquipmentSlot::CHEST,
    EquipmentSlot::HEAD,
];

#[derive(Default)]
pub struct ZombificationState {
    /// Vanilla `timeInOverworld`.
    pub time_in_overworld: AtomicI32,
    /// Vanilla `IsImmuneToZombification`.
    pub immune: AtomicBool,
}

impl ZombificationState {
    pub fn write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_int(
            "TimeInOverworld",
            self.time_in_overworld.load(Ordering::Relaxed),
        );
        nbt.put_bool(
            "IsImmuneToZombification",
            self.immune.load(Ordering::Relaxed),
        );
    }

    pub fn read_nbt(&self, nbt: &NbtCompound) {
        self.time_in_overworld.store(
            nbt.get_int("TimeInOverworld").unwrap_or(0),
            Ordering::Relaxed,
        );
        self.immune.store(
            nbt.get_bool("IsImmuneToZombification").unwrap_or(false),
            Ordering::Relaxed,
        );
    }
}

/// One `customServerAiStep` worth of conversion bookkeeping.
///
/// `skip_sound_on_peaceful` is true for piglins/brutes (`AbstractPiglin`
/// gates only the sound on peaceful); hoglins always play theirs.
pub async fn tick_zombification(
    mob: &dyn Mob,
    state: &ZombificationState,
    converted_type: &'static EntityType,
    converted_sound: Sound,
    skip_sound_on_peaceful: bool,
    keep_equipment: bool,
) {
    let entity = mob.get_entity();
    let world = entity.world.load_full();

    // Vanilla isConverting: !immune && !isNoAi && PIGLINS_ZOMBIFY.
    let converting = !state.immune.load(Ordering::Relaxed)
        && !mob.get_mob_entity().is_no_ai()
        && world.dimension != Dimension::THE_NETHER;
    if !converting {
        state.time_in_overworld.store(0, Ordering::Relaxed);
        return;
    }

    let time = state.time_in_overworld.fetch_add(1, Ordering::Relaxed) + 1;
    if time <= CONVERSION_TIME {
        return;
    }

    let Some(source) = world.get_entity_by_id(entity.entity_id) else {
        return;
    };

    let peaceful = world.level_info.load().difficulty == Difficulty::Peaceful;
    if !(skip_sound_on_peaceful && peaceful) {
        world.play_sound(converted_sound, SoundCategory::Hostile, &entity.pos.load());
    }

    let custom_name = entity.custom_name.load().as_ref().clone();
    let custom_name_visible = entity.custom_name_visible.load(Ordering::Relaxed);
    let converted =
        crate::entity::r#type::from_type(converted_type, entity.pos.load(), &world, Uuid::new_v4());
    let converted_entity = converted.get_entity();
    converted_entity.yaw.store(entity.yaw.load());
    converted_entity.pitch.store(entity.pitch.load());
    converted_entity.head_yaw.store(entity.head_yaw.load());
    converted_entity.body_yaw.store(entity.body_yaw.load());
    converted_entity.velocity.store(entity.velocity.load());
    converted_entity
        .on_ground
        .store(entity.on_ground.load(Ordering::Relaxed), Ordering::Relaxed);

    // Vanilla convertTo does not re-run finalizeSpawn, so send metadata only
    // and hand over the existing gear when the conversion keeps equipment.
    world.remove_entity(source.as_ref()).await;
    world.broadcast_entity_spawn(&converted);
    if keep_equipment && let Some(converted_living) = converted.get_living_entity() {
        let source_living = &mob.get_mob_entity().living_entity;
        let mut copied = Vec::with_capacity(COPIED_SLOTS.len());
        {
            let source_equipment = source_living.entity_equipment.lock().await;
            for slot in COPIED_SLOTS {
                let stack = source_equipment.get(&slot).lock().await.clone();
                if !stack.is_empty() {
                    copied.push((slot, stack));
                }
            }
        }
        let mut converted_equipment = converted_living.entity_equipment.lock().await;
        for (slot, stack) in &copied {
            converted_equipment.put(slot, stack.clone()).await;
        }
        drop(converted_equipment);
        converted_living.send_equipment_changes(&copied);
    }
    if let Some(mob_converted) = converted_mob(&converted) {
        mob_converted.mob_init_data_tracker().await;
    }
    world.add_entity_silent(converted.clone()).await;

    if let Some(custom_name) = custom_name {
        converted_entity.set_custom_name(custom_name);
    }
    if custom_name_visible {
        converted_entity.set_custom_name_visible(true);
    }
    if let Some(living) = converted.get_living_entity() {
        living
            .add_effect(Effect {
                effect_type: &StatusEffect::NAUSEA,
                duration: 200,
                amplifier: 0,
                ambient: false,
                show_particles: true,
                show_icon: true,
                blend: false,
            })
            .await;
    }
}

fn converted_mob(entity: &Arc<dyn EntityBase>) -> Option<&dyn Mob> {
    use crate::entity::mob::zoglin::ZoglinEntity;
    use crate::entity::mob::zombified_piglin::ZombifiedPiglinEntity;
    let any = entity.cast_any();
    if let Some(mob) = any.downcast_ref::<ZombifiedPiglinEntity>() {
        return Some(mob);
    }
    if let Some(mob) = any.downcast_ref::<ZoglinEntity>() {
        return Some(mob);
    }
    None
}
