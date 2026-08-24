use std::sync::atomic::{AtomicBool, Ordering::Relaxed};

use crossbeam::atomic::AtomicCell;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::java::client::play::Metadata;
use uuid::Uuid;

use crate::entity::passive::animal::Animal;

pub const SITTING_FLAG: u8 = 1;
pub const TAME_FLAG: u8 = 4;
pub const TELEPORT_WHEN_DISTANCE_IS_SQ: f64 = 144.0;

pub struct TamableData {
    pub is_tame: AtomicBool,
    pub ordered_to_sit: AtomicBool,
    pub owner: AtomicCell<Option<Uuid>>,
}

impl Default for TamableData {
    fn default() -> Self {
        Self {
            is_tame: AtomicBool::new(false),
            ordered_to_sit: AtomicBool::new(false),
            owner: AtomicCell::new(None),
        }
    }
}

pub trait TamableAnimal: Animal {
    fn get_tamable_data(&self) -> &TamableData;

    fn is_tame(&self) -> bool {
        self.get_tamable_data().is_tame.load(Relaxed)
    }

    fn set_tame(&self, tame: bool) {
        let mob_entity = self.get_mob_entity();
        let entity = &mob_entity.living_entity.entity;
        self.get_tamable_data().is_tame.store(tame, Relaxed);
        let mut flags = if self.is_in_sitting_pose() {
            SITTING_FLAG
        } else {
            0
        };
        if tame {
            flags |= TAME_FLAG;
        }
        entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::tamable_animal::DATA_FLAGS_ID,
                flags as i8,
            )],
            None,
        );
    }

    fn is_in_sitting_pose(&self) -> bool {
        self.get_tamable_data().ordered_to_sit.load(Relaxed)
    }

    fn set_in_sitting_pose(&self, sitting: bool) {
        let mob_entity = self.get_mob_entity();
        let entity = &mob_entity.living_entity.entity;
        self.get_tamable_data()
            .ordered_to_sit
            .store(sitting, Relaxed);
        let mut flags = if sitting { SITTING_FLAG } else { 0 };
        if self.is_tame() {
            flags |= TAME_FLAG;
        }
        entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::tamable_animal::DATA_FLAGS_ID,
                flags as i8,
            )],
            None,
        );
    }

    fn is_ordered_to_sit(&self) -> bool {
        self.get_tamable_data().ordered_to_sit.load(Relaxed)
    }

    fn set_ordered_to_sit(&self, ordered_to_sit: bool) {
        self.set_in_sitting_pose(ordered_to_sit);
    }

    fn get_owner(&self) -> Option<Uuid> {
        self.get_tamable_data().owner.load()
    }

    fn set_owner(&self, owner: Option<Uuid>) {
        let mob_entity = self.get_mob_entity();
        let entity = &mob_entity.living_entity.entity;
        self.get_tamable_data().owner.store(owner);
        entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::tamable_animal::DATA_OWNERUUID_ID,
                owner,
            )],
            None,
        );
    }

    fn is_owned_by(&self, player_uuid: &Uuid) -> bool {
        self.get_owner().is_some_and(|id| id == *player_uuid)
    }

    fn tame(&self, player_id: Uuid) {
        self.set_tame(true);
        self.set_owner(Some(player_id));
    }

    fn spawn_taming_particles(&self, success: bool) {
        use pumpkin_data::particle::Particle;
        use pumpkin_util::math::vector3::Vector3;
        let mob_entity = self.get_mob_entity();
        let entity = &mob_entity.living_entity.entity;
        let world = entity.world.load();
        let pos = entity.pos.load();
        let particle = if success {
            Particle::Heart
        } else {
            Particle::Smoke
        };
        world.spawn_particle(
            pos + Vector3::new(0.0, f64::from(entity.height()) * 0.5, 0.0),
            Vector3::new(0.5, 0.5, 0.5),
            0.02,
            7,
            particle,
        );
    }

    fn write_tamable_nbt(&self, nbt: &mut NbtCompound) {
        if let Some(owner) = self.get_owner() {
            nbt.put_uuid("Owner", owner);
        }
        nbt.put_bool("Sitting", self.is_ordered_to_sit());
    }

    fn read_tamable_nbt(&self, nbt: &NbtCompound) {
        if let Some(owner) = nbt.get_uuid("Owner") {
            self.set_owner(Some(owner));
            self.set_tame(true);
        } else if let Some(is_tame) = nbt.get_bool("IsTame") {
            self.set_tame(is_tame);
        }
        let sitting = nbt
            .get_bool("Sitting")
            .or_else(|| nbt.get_byte("Sitting").map(|b| b != 0))
            .unwrap_or(false);
        self.set_ordered_to_sit(sitting);
    }
}
