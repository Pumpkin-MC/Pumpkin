use std::sync::Arc;
use std::sync::atomic::Ordering;

use pumpkin_data::damage::DamageType;
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::potion::Effect;
use pumpkin_data::sound::{Sound, SoundCategory};

use crate::entity::{
    Entity, EntityBase,
    custom_sound::CustomSound,
    mob::{Mob, MobEntity, equipment::RegionalDifficulty, skeleton::SkeletonEntityBase},
};
use crate::world::World;

pub struct WitherSkeletonEntity {
    entity: Arc<SkeletonEntityBase>,
}

impl WitherSkeletonEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let entity = SkeletonEntityBase::new_wither(entity);
        let skeleton = Self { entity };
        Arc::new(skeleton)
    }
}

impl CustomSound for WitherSkeletonEntity {
    fn death_sound(&self) -> Option<Sound> {
        Some(Sound::EntityWitherSkeletonDeath)
    }

    fn hurt_sound(&self) -> Option<Sound> {
        Some(Sound::EntityWitherSkeletonHurt)
    }
}

impl Mob for WitherSkeletonEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity.mob_entity
    }

    fn as_custom_sound(&self) -> Option<&dyn CustomSound> {
        Some(self)
    }

    fn on_attack(&self, target: &dyn EntityBase) {
        // Vanilla WitherSkeleton inflicts Wither effect for 200 ticks (10 seconds) on hit
        if let Some(living) = target.get_living_entity() {
            living.add_effect(Effect {
                effect_type: &StatusEffect::WITHER,
                duration: 200,
                amplifier: 0,
                ambient: false,
                show_particles: true,
                show_icon: true,
                blend: false,
            });
        }
    }

    fn pre_damage(&self, damage_type: DamageType, _source: Option<&dyn EntityBase>) -> bool {
        // Vanilla WitherSkeleton is immune to Wither damage
        if damage_type == DamageType::WITHER {
            return false;
        }
        true
    }

    fn populate_default_equipment_slots(
        &self,
        _world: &Arc<World>,
        _difficulty: &RegionalDifficulty,
    ) {
        // Wither Skeletons spawn with a Stone Sword and no armor
        let living = &self.entity.mob_entity.living_entity;
        let mut equipment = living
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        equipment.put(&EquipmentSlot::MAIN_HAND, ItemStack::new(1, &Item::STONE_SWORD));
    }

    fn populate_default_equipment_enchantments(&self, _difficulty: &RegionalDifficulty) {
        // Vanilla WitherSkeleton does not generate equipment enchantments
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        let base_entity = &self.entity.mob_entity.living_entity.entity;
        if !base_entity.is_alive() {
            return;
        }

        // Ambient sound
        if rand::random_range(0..100) == 0 && !base_entity.silent.load(Ordering::Relaxed) {
            base_entity.world.load().play_sound_fine(
                Sound::EntityWitherSkeletonAmbient,
                SoundCategory::Hostile,
                &base_entity.pos.load(),
                1.0,
                1.0,
            );
        }
    }
}
