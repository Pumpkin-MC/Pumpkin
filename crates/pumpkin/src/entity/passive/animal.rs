use std::sync::Arc;

use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::particle::Particle;
use pumpkin_data::sound::{Sound, SoundCategory};

use crate::entity::{mob::Mob, player::Player};
use pumpkin_protocol::bedrock::server::actor_event::ActorEventID;
use pumpkin_util::math::vector3::Vector3;

pub trait Animal: Mob {
    fn is_food(&self, item_stack: &ItemStack) -> bool;

    fn play_eating_sound(&self, sound: Sound) {
        let mob_entity = self.get_mob_entity();
        let entity = &mob_entity.living_entity.entity;
        let world = entity.world.load();
        world.play_sound(sound, SoundCategory::Neutral, &entity.pos.load());
    }

    fn write_animal_nbt(&self, nbt: &mut pumpkin_nbt::compound::NbtCompound) {
        let mob_entity = self.get_mob_entity();
        let in_love = mob_entity
            .love_ticks
            .load(std::sync::atomic::Ordering::Relaxed);
        nbt.put_int("InLove", in_love);
        if let Some(uuid) = mob_entity.breeder.load() {
            nbt.put_uuid("LoveCause", uuid);
        }
    }

    fn read_animal_nbt(&self, nbt: &pumpkin_nbt::compound::NbtCompound) {
        let mob_entity = self.get_mob_entity();
        let in_love = nbt.get_int("InLove").unwrap_or(0);
        let love_cause = nbt.get_uuid("LoveCause");
        mob_entity.set_love_ticks(in_love, love_cause);
    }

    fn animal_interact(
        &self,
        player: &Arc<Player>,
        item_stack: &mut ItemStack,
        ambient_sound: Sound,
    ) -> bool {
        let mob_entity = self.get_mob_entity();
        if self.is_food(item_stack) {
            let age = mob_entity
                .living_entity
                .entity
                .age
                .load(std::sync::atomic::Ordering::Relaxed);

            if age >= 0 && mob_entity.is_breeding_ready() && !mob_entity.is_in_love() {
                item_stack.decrement_unless_creative(player.gamemode.load(), 1);

                mob_entity.set_love_ticks(600, Some(player.gameprofile.id));
                let entity = &mob_entity.living_entity.entity;
                let world = entity.world.load();
                let pos = entity.pos.load();

                world.send_entity_status(
                    entity,
                    pumpkin_data::entity::EntityStatus::InLoveHearts,
                    Some(ActorEventID::InLoveHearts),
                );

                world.spawn_particle(
                    pos + Vector3::new(0.0, f64::from(entity.height()), 0.0),
                    Vector3::new(0.5, 0.5, 0.5),
                    1.0,
                    7,
                    Particle::Heart,
                );
                world.play_sound(ambient_sound, SoundCategory::Neutral, &entity.pos.load());
                return true;
            }

            if age < 0 {
                item_stack.decrement_unless_creative(player.gamemode.load(), 1);
                let speedup = (-age / 10).max(1);
                mob_entity
                    .living_entity
                    .entity
                    .age
                    .fetch_add(speedup, std::sync::atomic::Ordering::Relaxed);

                let entity = &mob_entity.living_entity.entity;
                let world = entity.world.load();
                let pos = entity.pos.load();

                world.spawn_particle(
                    pos + Vector3::new(0.0, f64::from(entity.height()), 0.0),
                    Vector3::new(0.5, 0.5, 0.5),
                    1.0,
                    7,
                    Particle::HappyVillager,
                );
                self.play_eating_sound(ambient_sound);
                return true;
            }
        }

        mob_entity.mob_interact(player, item_stack)
    }
}

#[must_use]
pub fn get_dye_color_from_item(item: &pumpkin_data::item::Item) -> Option<u8> {
    match item.registry_key {
        "white_dye" => Some(0),
        "orange_dye" => Some(1),
        "magenta_dye" => Some(2),
        "light_blue_dye" => Some(3),
        "yellow_dye" => Some(4),
        "lime_dye" => Some(5),
        "pink_dye" => Some(6),
        "gray_dye" => Some(7),
        "light_gray_dye" => Some(8),
        "cyan_dye" => Some(9),
        "purple_dye" => Some(10),
        "blue_dye" => Some(11),
        "brown_dye" => Some(12),
        "green_dye" => Some(13),
        "red_dye" => Some(14),
        "black_dye" => Some(15),
        _ => None,
    }
}

#[must_use]
pub const fn get_wool_item_for_color(color: u8) -> &'static pumpkin_data::item::Item {
    match color {
        0 => &pumpkin_data::item::Item::WHITE_WOOL,
        1 => &pumpkin_data::item::Item::ORANGE_WOOL,
        2 => &pumpkin_data::item::Item::MAGENTA_WOOL,
        3 => &pumpkin_data::item::Item::LIGHT_BLUE_WOOL,
        4 => &pumpkin_data::item::Item::YELLOW_WOOL,
        5 => &pumpkin_data::item::Item::LIME_WOOL,
        6 => &pumpkin_data::item::Item::PINK_WOOL,
        7 => &pumpkin_data::item::Item::GRAY_WOOL,
        8 => &pumpkin_data::item::Item::LIGHT_GRAY_WOOL,
        9 => &pumpkin_data::item::Item::CYAN_WOOL,
        10 => &pumpkin_data::item::Item::PURPLE_WOOL,
        11 => &pumpkin_data::item::Item::BLUE_WOOL,
        12 => &pumpkin_data::item::Item::BROWN_WOOL,
        13 => &pumpkin_data::item::Item::GREEN_WOOL,
        14 => &pumpkin_data::item::Item::RED_WOOL,
        _ => &pumpkin_data::item::Item::BLACK_WOOL,
    }
}

#[must_use]
pub fn get_carpet_color_from_item(item: &pumpkin_data::item::Item) -> Option<u8> {
    match item.registry_key {
        "white_carpet" => Some(0),
        "orange_carpet" => Some(1),
        "magenta_carpet" => Some(2),
        "light_blue_carpet" => Some(3),
        "yellow_carpet" => Some(4),
        "lime_carpet" => Some(5),
        "pink_carpet" => Some(6),
        "gray_carpet" => Some(7),
        "light_gray_carpet" => Some(8),
        "cyan_carpet" => Some(9),
        "purple_carpet" => Some(10),
        "blue_carpet" => Some(11),
        "brown_carpet" => Some(12),
        "green_carpet" => Some(13),
        "red_carpet" => Some(14),
        "black_carpet" => Some(15),
        _ => None,
    }
}
