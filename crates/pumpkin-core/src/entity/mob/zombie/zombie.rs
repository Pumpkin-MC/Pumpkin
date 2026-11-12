use crate::entity::mob::Mob;
use crate::entity::mob::equipment::RegionalDifficulty;
use crate::entity::mob::spawn::{EntitySpawnReason, SpawnGroupData};
use crate::entity::mob::zombie::ZombieEntityBase;
use crate::entity::{Entity, EntityBase};
use crate::world::World;
use pumpkin_data::BlockState;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::Sound;
use pumpkin_data::world::WorldEvent;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
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
}

impl Mob for ZombieEntity {
    crate::zombie_mob_hooks!(entity);

    fn wants_to_pick_up(&self, stack: &ItemStack) -> bool {
        super::wants_to_pick_up(self, stack)
    }

    fn get_ambient_sound(&self) -> Option<Sound> {
        self.get_entity().entity_type.ambient_sound
    }

    fn mob_play_step_sound(&self, _pos: &BlockPos, _state: &BlockState) {
        self.play_entity_sound(Sound::EntityZombieStep, 0.15, 1.0);
    }

    fn finalize_spawn(
        &self,
        world: &Arc<World>,
        reason: EntitySpawnReason,
        group_data: Option<SpawnGroupData>,
    ) -> Option<SpawnGroupData> {
        self.entity
            .finalize_zombie_spawn(self, world, reason, group_data)
    }

    fn do_hurt_target(&self, target: &dyn EntityBase) -> bool {
        self.entity.do_hurt_target(self, target)
    }

    fn post_tick(&self) {
        self.entity
            .tick_drowning(self, &EntityType::DROWNED, WorldEvent::SoundZombieToDrowned);
    }

    fn populate_default_equipment_slots(
        &self,
        world: &Arc<World>,
        difficulty: &RegionalDifficulty,
    ) {
        self.entity
            .populate_default_equipment_slots(world, difficulty);
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        self.entity.write_nbt(nbt);
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.entity.read_nbt(self, nbt);
    }
}

impl ZombieEntity {
    #[must_use]
    pub fn can_break_doors(&self) -> bool {
        self.entity.can_break_doors()
    }

    pub fn set_can_break_doors(&self, can_break: bool) {
        self.entity.set_can_break_doors(can_break, self);
    }

    #[must_use]
    pub fn is_baby(&self) -> bool {
        self.entity.is_baby()
    }

    pub fn set_baby(&self, baby: bool) {
        self.entity.set_baby(baby);
    }
}
