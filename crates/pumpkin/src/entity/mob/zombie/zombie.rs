use crate::entity::Entity;
use crate::entity::mob::equipment::RegionalDifficulty;
use crate::entity::mob::zombie::ZombieEntityBase;
use crate::entity::mob::{Mob, MobEntity};
use crate::world::World;
use pumpkin_nbt::compound::NbtCompound;
use std::sync::Arc;

pub struct ZombieEntity {
    entity: Arc<ZombieEntityBase>,
}

impl ZombieEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let entity = ZombieEntityBase::new(entity);
        let zombie = Self { entity };
        Arc::new(zombie)
    }

    #[must_use]
    pub fn with_can_break_doors(entity: Entity, can_break_doors: bool) -> Arc<Self> {
        let entity = ZombieEntityBase::with_can_break_doors(entity, can_break_doors);
        let zombie = Self { entity };
        Arc::new(zombie)
    }
}

impl Mob for ZombieEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity.mob_entity
    }

    fn populate_default_equipment_slots(
        &self,
        world: &Arc<World>,
        difficulty: &RegionalDifficulty,
    ) {
        self.entity
            .populate_default_equipment_slots(world, difficulty);
    }

    fn populate_default_equipment_enchantments(&self, difficulty: &RegionalDifficulty) {
        self.entity
            .populate_default_equipment_enchantments(difficulty);
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        self.entity.mob_write_nbt(nbt);
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.entity.mob_read_nbt(nbt);
    }
}

impl ZombieEntity {
    #[must_use]
    pub fn can_break_doors(&self) -> bool {
        self.entity
            .can_break_doors
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn set_can_break_doors(&self, can_break: bool) {
        self.entity
            .can_break_doors
            .store(can_break, std::sync::atomic::Ordering::Relaxed);
    }

    #[must_use]
    pub fn is_baby(&self) -> bool {
        self.entity
            .mob_entity
            .living_entity
            .entity
            .age
            .load(std::sync::atomic::Ordering::Relaxed)
            < 0
    }

    pub fn set_baby(&self, baby: bool) {
        let age = if baby { -24000 } else { 0 };
        self.entity
            .mob_entity
            .living_entity
            .entity
            .age
            .store(age, std::sync::atomic::Ordering::Relaxed);
        self.entity.mob_entity.living_entity.entity.send_meta_data(
            &[pumpkin_protocol::java::client::play::Metadata::new(
                pumpkin_data::tracked_data::zombie::BABY,
                baby,
            )],
            None,
        );
    }
}
