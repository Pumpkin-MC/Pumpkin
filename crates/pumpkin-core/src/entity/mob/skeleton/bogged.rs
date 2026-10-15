use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_nbt::compound::NbtCompound;

use crate::entity::{
    Entity, EntityBase,
    custom_sound::CustomSound,
    item::ItemEntity,
    mob::{Mob, MobEntity, equipment::RegionalDifficulty, skeleton::SkeletonEntityBase},
    player::Player,
};
use crate::world::World;

pub struct BoggedSkeletonEntity {
    entity: Arc<SkeletonEntityBase>,
    sheared: AtomicBool,
}

impl BoggedSkeletonEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        // Bogged attack interval is 70 ticks (50 on Hard difficulty)
        let entity = SkeletonEntityBase::new_with_attack_interval(entity, 70);
        let bogged = Self {
            entity,
            sheared: AtomicBool::new(false),
        };
        let bogged_arc = Arc::new(bogged);
        bogged_arc.mob_init_data_tracker();
        bogged_arc
    }

    pub fn is_sheared(&self) -> bool {
        self.sheared.load(Ordering::Relaxed)
    }

    pub fn set_sheared(&self, sheared: bool) {
        self.sheared.store(sheared, Ordering::Relaxed);
        let entity = &self.entity.mob_entity.living_entity.entity;
        entity.set_synced_data(pumpkin_data::tracked_data::bogged::DATA_SHEARED, sheared);
    }

    fn mob_init_data_tracker(&self) {
        let entity = &self.entity.mob_entity.living_entity.entity;
        entity.set_synced_data(
            pumpkin_data::tracked_data::bogged::DATA_SHEARED,
            self.is_sheared(),
        );
    }
}

impl CustomSound for BoggedSkeletonEntity {
    fn death_sound(&self) -> Option<Sound> {
        Some(Sound::EntityBoggedDeath)
    }

    fn hurt_sound(&self) -> Option<Sound> {
        Some(Sound::EntityBoggedHurt)
    }
}

impl Mob for BoggedSkeletonEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity.mob_entity
    }

    fn as_custom_sound(&self) -> Option<&dyn CustomSound> {
        Some(self)
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        let base_entity = &self.entity.mob_entity.living_entity.entity;
        if !base_entity.is_alive() {
            return;
        }

        // Ambient sound
        if rand::random_range(0..100) == 0 && !base_entity.silent.load(Ordering::Relaxed) {
            base_entity.world.load().play_sound_fine(
                Sound::EntityBoggedAmbient,
                SoundCategory::Hostile,
                &base_entity.pos.load(),
                1.0,
                1.0,
            );
        }
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        let item = item_stack.get_item();
        if item == &Item::SHEARS && !self.is_sheared() {
            self.set_sheared(true);
            let entity = &self.entity.mob_entity.living_entity.entity;
            let world = entity.world.load();
            let pos = entity.pos.load();
            world.play_sound(
                Sound::EntityBoggedShear,
                pumpkin_data::sound::SoundCategory::Players,
                &pos,
            );

            // Vanilla drops from shearing/bogged loot table: 1 brown mushroom and 1 red mushroom
            for mushroom in [&Item::BROWN_MUSHROOM, &Item::RED_MUSHROOM] {
                let item_entity = Arc::new(ItemEntity::new(
                    Entity::new(world.clone(), pos, &EntityType::ITEM),
                    ItemStack::new(1, mushroom),
                ));
                world.spawn_entity(item_entity);
            }
            player.damage_held_item(1);
            return true;
        }

        self.get_mob_entity().mob_interact(player, item_stack)
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_bool("sheared", self.is_sheared());
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        if let Some(sheared) = nbt.get_bool("sheared") {
            self.set_sheared(sheared);
        }
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
