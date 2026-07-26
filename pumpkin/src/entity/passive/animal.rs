use std::sync::Arc;

use pumpkin_data::entity::{EntityType, entity_from_egg};
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::particle::Particle;
use pumpkin_data::sound::{Sound, SoundCategory};
use uuid::Uuid;

use crate::entity::{
    EntityBaseFuture, ageable::BABY_START_AGE, mob::Mob, player::Player, r#type::from_type,
};
use crate::item::items::spawn_egg::apply_entity_variant;
use pumpkin_util::math::vector3::Vector3;

/// Whether `item_stack` is the spawn egg for `entity_type`.
///
/// Vanilla `SpawnEggItem.spawnsEntity`: only an egg for the mob's own type spawns a
/// baby from it. Any other spawn egg is left to the normal use-on-block path, which
/// places a new adult next to the mob.
fn is_spawn_egg_for(item_stack: &ItemStack, entity_type: &EntityType) -> bool {
    entity_from_egg(item_stack.item.id).is_some_and(|egg_type| egg_type == entity_type)
}

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

    /// Spawns a baby of this mob's own type at its position and consumes one egg,
    /// mirroring vanilla `SpawnEggItem.spawnOffspringFromSpawnEgg`.
    fn spawn_baby_from_egg<'a>(
        &'a self,
        player: &'a Arc<Player>,
        item_stack: &'a mut ItemStack,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let entity = &self.get_mob_entity().living_entity.entity;
            let world = entity.world.load();
            let baby = from_type(
                entity.entity_type,
                entity.pos.load(),
                &world,
                Uuid::new_v4(),
            );

            // `init_data_tracker` derives the baby flag from a negative age and runs as
            // part of `spawn_entity`, so the age has to be set before spawning.
            baby.get_entity().set_age(BABY_START_AGE);
            // Vanilla `snapTo(pos, 0.0F, 0.0F)`.
            baby.get_entity().set_rotation(0.0, 0.0);
            apply_entity_variant(item_stack, baby.as_ref());

            world.spawn_entity(baby).await;
            item_stack.decrement_unless_creative(player.gamemode.load(), 1);
        })
    }

    fn animal_interact<'a>(
        &'a self,
        player: &'a Arc<Player>,
        item_stack: &'a mut ItemStack,
        ambient_sound: Sound,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            let mob_entity = self.get_mob_entity();

            // Vanilla runs this in `Mob.checkAndHandleImportantInteractions`, ahead of
            // any food handling. No spawn egg is a breeding item, so the order only
            // matters for staying close to vanilla.
            if is_spawn_egg_for(item_stack, mob_entity.living_entity.entity.entity_type) {
                self.spawn_baby_from_egg(player, item_stack).await;
                return true;
            }

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

            mob_entity.mob_interact(player, item_stack).await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::is_spawn_egg_for;
    use pumpkin_data::entity::EntityType;
    use pumpkin_data::item::Item;
    use pumpkin_data::item_stack::ItemStack;

    #[test]
    fn matching_egg_is_recognized() {
        let stack = ItemStack::new(1, &Item::COW_SPAWN_EGG);
        assert!(is_spawn_egg_for(&stack, &EntityType::COW));
    }

    /// The wrong egg has to fall through, otherwise using a cow egg on a pig would
    /// spawn a baby pig instead of placing a cow.
    #[test]
    fn egg_for_another_mob_is_not_recognized() {
        let stack = ItemStack::new(1, &Item::COW_SPAWN_EGG);
        assert!(!is_spawn_egg_for(&stack, &EntityType::PIG));
        assert!(!is_spawn_egg_for(&stack, &EntityType::CHICKEN));
    }

    #[test]
    fn a_non_egg_item_is_not_recognized() {
        let stack = ItemStack::new(1, &Item::WHEAT);
        assert!(!is_spawn_egg_for(&stack, &EntityType::COW));
    }
}
