use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use pumpkin_data::BlockState;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tracked_data;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

use crate::entity::{
    Entity, EntityBase,
    custom_sound::CustomSound,
    item::ItemEntity,
    mob::{Mob, MobEntity, equipment::RegionalDifficulty, skeleton::SkeletonEntityBase},
    player::Player,
    shearable::{Shearable, shear_by_player, shearing_loot},
};
use crate::world::World;

const BOGGED_SHEAR_LOOT_TABLE: &str = "minecraft:shearing/bogged";

pub struct BoggedSkeletonEntity {
    entity: Arc<SkeletonEntityBase>,
    sheared: AtomicBool,
}

impl BoggedSkeletonEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let entity = SkeletonEntityBase::new(entity);
        Arc::new(Self {
            entity,
            sheared: AtomicBool::new(false),
        })
    }

    #[must_use]
    pub fn is_sheared(&self) -> bool {
        self.sheared.load(Ordering::Relaxed)
    }

    pub fn set_sheared(&self, sheared: bool) {
        self.sheared.store(sheared, Ordering::Relaxed);
        self.get_entity()
            .set_synced_data(tracked_data::bogged::DATA_SHEARED, sheared);
    }
}

impl Shearable for BoggedSkeletonEntity {
    fn shear(&self, sound_category: SoundCategory, tool: &ItemStack) -> bool {
        if self.sheared.swap(true, Ordering::Relaxed) {
            return false;
        }

        let entity = self.get_entity();
        let world = entity.world.load();
        let pos = entity.pos.load();
        world.play_sound(Sound::EntityBoggedShear, sound_category, &pos);

        let drop_pos = Vector3::new(pos.x, pos.y + f64::from(entity.height()), pos.z);
        for drop in shearing_loot(self, BOGGED_SHEAR_LOOT_TABLE, tool) {
            world.spawn_entity(Arc::new(ItemEntity::new(
                Entity::new(world.clone(), drop_pos, &EntityType::ITEM),
                drop,
            )));
        }
        self.set_sheared(true);
        true
    }

    fn ready_for_shearing(&self) -> bool {
        !self.is_sheared()
    }
}

impl CustomSound for BoggedSkeletonEntity {
    fn hurt_sound(&self) -> Option<Sound> {
        Some(Sound::EntityBoggedHurt)
    }

    fn death_sound(&self) -> Option<Sound> {
        Some(Sound::EntityBoggedDeath)
    }
}

impl Mob for BoggedSkeletonEntity {
    fn wants_to_pick_up(&self, stack: &ItemStack) -> bool {
        super::wants_to_pick_up(self, stack)
    }

    fn mob_play_step_sound(&self, _pos: &BlockPos, _state: &BlockState) {
        self.play_entity_sound(Sound::EntityBoggedStep, 0.15, 1.0);
    }

    fn as_custom_sound(&self) -> Option<&dyn CustomSound> {
        Some(self)
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity.mob_entity
    }

    fn as_shearable(&self) -> Option<&dyn Shearable> {
        Some(self)
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_bool("sheared", self.is_sheared());
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.set_sheared(nbt.get_bool("sheared").unwrap_or(false));
    }

    fn mob_init_data_tracker(&self) {
        self.get_entity()
            .set_synced_data(tracked_data::bogged::DATA_SHEARED, self.is_sheared());
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        if item_stack.get_item() == &Item::SHEARS && self.ready_for_shearing() {
            return shear_by_player(self, player, item_stack);
        }
        self.entity
            .mob_entity
            .mob_interact(player, item_stack, self.can_be_leashed())
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
}
