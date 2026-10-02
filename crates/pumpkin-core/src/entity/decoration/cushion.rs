use std::sync::{
    Arc,
    atomic::{AtomicU8, Ordering},
};

use pumpkin_data::damage::DamageType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::entity::player::Player;
use crate::entity::{Entity, EntityBase, living::LivingEntity};
use crate::server::Server;

pub struct CushionEntity {
    pub entity: Entity,
    color: AtomicU8,
}

impl CushionEntity {
    pub const fn new(entity: Entity, color: u8) -> Self {
        Self {
            entity,
            color: AtomicU8::new(color),
        }
    }

    pub fn color(&self) -> u8 {
        self.color.load(Ordering::Relaxed)
    }

    pub fn set_color(&self, color: u8) {
        self.color.store(color, Ordering::Relaxed);
        self.sync_color();
    }

    fn sync_color(&self) {
        self.entity.set_synced_data(
            pumpkin_data::tracked_data::cushion::COLOR,
            VarInt(i32::from(self.color())),
        );
    }

    #[must_use]
    pub const fn item_for_color(color: u8) -> &'static Item {
        match color {
            0 => &Item::WHITE_CUSHION,
            1 => &Item::ORANGE_CUSHION,
            2 => &Item::MAGENTA_CUSHION,
            3 => &Item::LIGHT_BLUE_CUSHION,
            4 => &Item::YELLOW_CUSHION,
            5 => &Item::LIME_CUSHION,
            6 => &Item::PINK_CUSHION,
            7 => &Item::GRAY_CUSHION,
            8 => &Item::LIGHT_GRAY_CUSHION,
            9 => &Item::CYAN_CUSHION,
            10 => &Item::PURPLE_CUSHION,
            11 => &Item::BLUE_CUSHION,
            12 => &Item::BROWN_CUSHION,
            13 => &Item::GREEN_CUSHION,
            14 => &Item::RED_CUSHION,
            _ => &Item::BLACK_CUSHION,
        }
    }

    #[must_use]
    pub const fn color_from_item(item: &Item) -> u8 {
        match item.id {
            id if id == Item::WHITE_CUSHION.id => 0,
            id if id == Item::ORANGE_CUSHION.id => 1,
            id if id == Item::MAGENTA_CUSHION.id => 2,
            id if id == Item::LIGHT_BLUE_CUSHION.id => 3,
            id if id == Item::YELLOW_CUSHION.id => 4,
            id if id == Item::LIME_CUSHION.id => 5,
            id if id == Item::PINK_CUSHION.id => 6,
            id if id == Item::GRAY_CUSHION.id => 7,
            id if id == Item::LIGHT_GRAY_CUSHION.id => 8,
            id if id == Item::CYAN_CUSHION.id => 9,
            id if id == Item::PURPLE_CUSHION.id => 10,
            id if id == Item::BLUE_CUSHION.id => 11,
            id if id == Item::BROWN_CUSHION.id => 12,
            id if id == Item::GREEN_CUSHION.id => 13,
            id if id == Item::RED_CUSHION.id => 14,
            _ => 15,
        }
    }

    fn drop_and_remove(&self) {
        let entity = &self.entity;
        let world = entity.world.load();
        world.play_sound(
            Sound::EntityCushionBreak,
            SoundCategory::Blocks,
            &entity.pos.load(),
        );
        world.drop_stack(
            &entity.block_pos.load(),
            ItemStack::new(1, Self::item_for_color(self.color())),
        );
        entity.remove();
    }
}

impl EntityBase for CushionEntity {
    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    fn init_data_tracker(&self) {
        self.sync_color();
    }

    /// The colour has to ride along with the spawn packet; a client seeing the
    /// cushion for the first time never got the tracked data update.
    fn java_spawn_metadata(&self, version: JavaMinecraftVersion) -> Option<Box<[u8]>> {
        let mut metadata = Vec::new();
        Metadata::new(
            pumpkin_data::tracked_data::cushion::COLOR,
            VarInt(i32::from(self.color())),
        )
        .write(&mut metadata, &version)
        .ok()?;
        metadata.push(255);
        Some(metadata.into_boxed_slice())
    }

    fn tick(&self, _caller: &dyn EntityBase, _server: &Server) {}

    fn can_hit(&self) -> bool {
        self.entity.is_alive()
    }

    fn is_collidable(&self, _entity: Option<Box<dyn EntityBase>>) -> bool {
        true
    }

    fn write_custom_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_byte("Color", self.color() as i8);
    }

    fn read_custom_nbt(&self, nbt: &NbtCompound) {
        if let Some(color) = nbt.get_byte("Color") {
            self.set_color(color as u8);
        }
    }

    fn interact(&self, player: &Arc<Player>, _item_stack: &mut ItemStack) -> bool {
        if player.get_entity().is_sneaking() {
            return false;
        }
        if player.get_entity().has_vehicle() {
            return false;
        }
        if !self
            .entity
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_empty()
        {
            return false;
        }

        let world = self.entity.world.load();
        let Some(vehicle) = world.get_entity_by_id(self.entity.entity_id) else {
            return false;
        };
        let Some(passenger) = world.get_player_by_id(player.entity_id()) else {
            return false;
        };

        world.play_sound(
            Sound::EntityCushionSit,
            SoundCategory::Blocks,
            &self.entity.pos.load(),
        );
        self.entity
            .add_passenger(vehicle, passenger as Arc<dyn EntityBase>);
        true
    }

    fn damage_with_context(
        &self,
        _caller: &dyn EntityBase,
        _amount: f32,
        _damage_type: DamageType,
        _position: Option<Vector3<f64>>,
        _source: Option<&dyn EntityBase>,
        _cause: Option<&dyn EntityBase>,
    ) -> bool {
        if !self.entity.is_alive() {
            return false;
        }
        self.drop_and_remove();
        true
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }
}
