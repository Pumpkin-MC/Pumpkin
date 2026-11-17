use pumpkin_data::BlockState;
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::potion::Effect;
use pumpkin_data::sound::Sound;
use pumpkin_data::world::WorldEvent;
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

use crate::entity::mob::equipment::RegionalDifficulty;
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
}

const CAMEL_HUSK_JOCKEY_CHANCE: f32 = 0.1;
const HUNGER_TICKS_PER_DIFFICULTY: i32 = 140;

impl Mob for HuskEntity {
    crate::zombie_mob_hooks!(entity);

    fn finalize_spawn(
        &self,
        world: &Arc<World>,
        reason: EntitySpawnReason,
        group_data: Option<SpawnGroupData>,
    ) -> Option<SpawnGroupData> {
        let group_data = self
            .entity
            .finalize_zombie_spawn(self, world, reason, group_data);
        let mut rng = self.get_random();
        let difficulty_modifier =
            RegionalDifficulty::at(world, self.get_entity().pos.load()).special_multiplier;
        if reason != EntitySpawnReason::Conversion {
            self.get_mob_entity().set_can_pick_up_loot(
                rng.random::<f32>() < MobEntity::MAX_PICKUP_LOOT_CHANCE * difficulty_modifier,
            );
        }
        let entity = self.get_entity();
        let pos = entity.pos.load();
        let block_pos = entity.block_pos.load();
        if group_data.is_some()
            && reason == EntitySpawnReason::Natural
            && world.is_space_empty(EntityType::CAMEL_HUSK.get_spawn_aabb(
                f64::from(block_pos.0.x) + 0.5,
                f64::from(block_pos.0.y),
                f64::from(block_pos.0.z) + 0.5,
            ))
            && rng.random::<f32>() < CAMEL_HUSK_JOCKEY_CHANCE
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

    fn wants_to_pick_up(&self, stack: &ItemStack) -> bool {
        super::wants_to_pick_up(self, stack)
    }

    fn get_ambient_sound(&self) -> Option<Sound> {
        self.get_entity().entity_type.ambient_sound
    }

    fn mob_play_step_sound(&self, _pos: &BlockPos, _state: &BlockState) {
        self.play_entity_sound(Sound::EntityHuskStep, 0.15, 1.0);
    }

    fn do_hurt_target(&self, target: &dyn EntityBase) -> bool {
        if !self.entity.do_hurt_target(self, target) {
            return false;
        }
        let main_hand_empty = self
            .get_mob_entity()
            .living_entity
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&EquipmentSlot::MAIN_HAND)
            .is_empty();
        if main_hand_empty && let Some(living) = target.get_living_entity() {
            let world = self.get_entity().world.load_full();
            let difficulty =
                RegionalDifficulty::at(&world, self.get_entity().pos.load()).effective_difficulty;
            living.add_effect(Effect {
                effect_type: &StatusEffect::HUNGER,
                duration: HUNGER_TICKS_PER_DIFFICULTY * difficulty as i32,
                amplifier: 0,
                ambient: false,
                show_particles: true,
                show_icon: true,
                blend: false,
            });
        }
        true
    }

    fn post_tick(&self) {
        self.entity
            .tick_drowning(self, &EntityType::ZOMBIE, WorldEvent::SoundHuskToZombie);
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
