use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::RwLock;

use pumpkin_data::tracked_data;
use pumpkin_util::text::TextComponent;

use crate::entity::living::LivingEntity;
use crate::entity::{Entity, EntityBase};

pub const DEFAULT_DESCRIPTION_KEY: &str = "entity.minecraft.mannequin.label";
pub const ALL_LAYERS: u8 = 0x7F;

pub struct MannequinEntity {
    pub living_entity: LivingEntity,
    immovable: AtomicBool,
    main_arm: AtomicU8,
    model_customization: AtomicU8,
    description: RwLock<Option<TextComponent>>,
    hide_description: AtomicBool,
}

impl MannequinEntity {
    pub fn new(entity: Entity) -> Self {
        let living_entity = LivingEntity::new(entity);
        let default_desc = TextComponent::translate(DEFAULT_DESCRIPTION_KEY, []);

        living_entity.entity.set_synced_data(
            tracked_data::mannequin::DATA_PLAYER_MODE_CUSTOMISATION,
            ALL_LAYERS,
        );
        living_entity.entity.set_synced_data(
            tracked_data::mannequin::DATA_PLAYER_MAIN_HAND,
            1u8, // HumanoidArm::Right
        );
        living_entity.entity.set_synced_data(
            tracked_data::mannequin::DATA_IMMOVABLE,
            false,
        );
        living_entity.entity.set_synced_data(
            tracked_data::mannequin::DATA_DESCRIPTION,
            Some(default_desc.clone()),
        );

        Self {
            living_entity,
            immovable: AtomicBool::new(false),
            main_arm: AtomicU8::new(1),
            model_customization: AtomicU8::new(ALL_LAYERS),
            description: RwLock::new(Some(default_desc)),
            hide_description: AtomicBool::new(false),
        }
    }

    pub fn is_immovable(&self) -> bool {
        self.immovable.load(Ordering::Relaxed)
    }

    pub fn set_immovable(&self, immovable: bool) {
        self.immovable.store(immovable, Ordering::Relaxed);
        self.living_entity.entity.set_synced_data(
            tracked_data::mannequin::DATA_IMMOVABLE,
            immovable,
        );
    }

    pub fn main_arm(&self) -> u8 {
        self.main_arm.load(Ordering::Relaxed)
    }

    pub fn set_main_arm(&self, arm: u8) {
        self.main_arm.store(arm, Ordering::Relaxed);
        self.living_entity.entity.set_synced_data(
            tracked_data::mannequin::DATA_PLAYER_MAIN_HAND,
            arm,
        );
    }

    pub fn model_customization(&self) -> u8 {
        self.model_customization.load(Ordering::Relaxed)
    }

    pub fn set_model_customization(&self, layers: u8) {
        self.model_customization.store(layers, Ordering::Relaxed);
        self.living_entity.entity.set_synced_data(
            tracked_data::mannequin::DATA_PLAYER_MODE_CUSTOMISATION,
            layers,
        );
    }

    pub fn description(&self) -> Option<TextComponent> {
        self.description
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    pub fn set_description(&self, description: Option<TextComponent>) {
        *self.description
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = description.clone();
        self.update_synced_description();
    }

    pub fn hide_description(&self) -> bool {
        self.hide_description.load(Ordering::Relaxed)
    }

    pub fn set_hide_description(&self, hide: bool) {
        self.hide_description.store(hide, Ordering::Relaxed);
        self.update_synced_description();
    }

    fn update_synced_description(&self) {
        let to_sync = if self.hide_description.load(Ordering::Relaxed) {
            None
        } else {
            self.description()
        };
        self.living_entity.entity.set_synced_data(
            tracked_data::mannequin::DATA_DESCRIPTION,
            to_sync,
        );
    }
}

impl EntityBase for MannequinEntity {
    fn get_entity(&self) -> &Entity {
        &self.living_entity.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        Some(&self.living_entity)
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }
}
