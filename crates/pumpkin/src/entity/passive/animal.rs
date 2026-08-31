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
