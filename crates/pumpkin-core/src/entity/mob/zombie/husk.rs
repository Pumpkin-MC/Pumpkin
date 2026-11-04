use pumpkin_data::BlockState;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::Sound;
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

use crate::entity::mob::zombie::ZombieEntityBase;
use crate::entity::{
    Entity, EntityBase,
    mob::{
        Mob, MobEntity,
        spawn::{EntitySpawnReason, SpawnGroupData, finalize_spawn},
    },
    r#type::from_type,
};
use crate::world::World;
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_nbt::compound::NbtCompound;
use rand::RngExt;
use uuid::Uuid;

pub struct HuskEntity {
    entity: Arc<ZombieEntityBase>,
}

impl HuskEntity {
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

const CAMEL_HUSK_JOCKEY_CHANCE: f32 = 0.1;

impl Mob for HuskEntity {
    fn finalize_spawn(
        &self,
        world: &Arc<World>,
        reason: EntitySpawnReason,
        group_data: Option<SpawnGroupData>,
    ) -> Option<SpawnGroupData> {
        let group_data = self.default_finalize_spawn(world, group_data);
        let entity = self.get_entity();
        let pos = entity.pos.load();
        let block_pos = entity.block_pos.load();
        if reason == EntitySpawnReason::Natural
            && world.is_space_empty(EntityType::CAMEL_HUSK.get_spawn_aabb(
                f64::from(block_pos.0.x) + 0.5,
                f64::from(block_pos.0.y),
                f64::from(block_pos.0.z) + 0.5,
            ))
            && self.get_random().random::<f32>() < CAMEL_HUSK_JOCKEY_CHANCE
        {
            self.get_mob_entity().set_item_slot(
                &EquipmentSlot::MAIN_HAND,
                ItemStack::new(1, &Item::IRON_SPEAR),
            );
            let camel_husk = from_type(&EntityType::CAMEL_HUSK, pos, world, Uuid::new_v4());
            finalize_spawn(&camel_husk, world, reason, None);
            let parched = from_type(&EntityType::PARCHED, pos, world, Uuid::new_v4());
            parched.get_entity().set_rotation(entity.yaw.load(), 0.0);
            finalize_spawn(&parched, world, reason, None);
            if let Some(camel_mob) = camel_husk.get_mob() {
                camel_mob.get_mob_entity().add_pending_rider(parched);
            }
            self.get_mob_entity().set_pending_vehicle(camel_husk);
        }
        group_data
    }

    fn xp_reward(&self) -> u32 {
        self.entity.xp_reward()
    }

    fn can_hold_item(&self, stack: &ItemStack) -> bool {
        self.entity.can_hold_item(stack)
    }

    fn wants_to_pick_up(&self, stack: &ItemStack) -> bool {
        super::wants_to_pick_up(self, stack)
    }

    fn mob_play_step_sound(&self, _pos: &BlockPos, _state: &BlockState) {
        self.play_entity_sound(Sound::EntityHuskStep, 0.15, 1.0);
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity.mob_entity
    }

    fn spawn_as_baby(&self) -> bool {
        self.entity.spawn_as_baby()
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        self.entity.mob_write_nbt(nbt);
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.entity.mob_read_nbt(nbt);
    }
}
