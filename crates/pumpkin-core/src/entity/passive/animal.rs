use std::sync::Arc;

use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::particle::Particle;

use crate::entity::{EntityBase, mob::Mob, player::Player, r#type::from_type};
use pumpkin_protocol::bedrock::server::actor_event::ActorEventID;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

pub trait Animal: Mob {
    fn is_food(&self, item_stack: &ItemStack) -> bool;

    fn get_breed_offspring(&self, _partner: &dyn EntityBase) -> Arc<dyn EntityBase> {
        let entity = self.get_entity();
        from_type(
            entity.entity_type,
            entity.pos.load(),
            &entity.world.load(),
            uuid::Uuid::new_v4(),
        )
    }

    /// Extra partner checks on top of the type and love checks in `BreedGoal`.
    fn can_mate(&self, _partner: &dyn EntityBase) -> bool {
        true
    }

    /// Returns `false` when a plugin cancels.
    fn set_in_love(&self, player: &Arc<Player>) -> bool {
        let Some(ticks) = self.approve_love(Some(player.entity_id()), 600) else {
            return false;
        };
        let mob_entity = self.get_mob_entity();
        mob_entity.set_love_ticks(ticks, Some(player.gameprofile.id));
        let entity = &mob_entity.living_entity.entity;
        entity.world.load().send_entity_status(
            entity,
            pumpkin_data::entity::EntityStatus::InLoveHearts,
            ActorEventID::InLoveHearts,
        );
        true
    }

    /// Animals prefer grass, then bright spots.
    fn animal_walk_target_value(&self, pos: &pumpkin_util::math::position::BlockPos) -> f32 {
        let world = self.get_mob_entity().living_entity.entity.world.load();
        if world.get_block(&pos.down()).id == pumpkin_data::Block::GRASS_BLOCK.id {
            return 10.0;
        }
        world.get_pathfinding_cost_from_light_levels(pos)
    }

    fn play_eating_sound(&self) {}

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

    fn animal_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        let mob_entity = self.get_mob_entity();
        if self.is_food(item_stack) {
            let age = mob_entity
                .living_entity
                .entity
                .age
                .load(std::sync::atomic::Ordering::Relaxed);

            if age >= 0 && mob_entity.is_breeding_ready() && !mob_entity.is_in_love() {
                item_stack.decrement_unless_creative(player.gamemode.load(), 1);
                if self.set_in_love(player) {
                    self.play_eating_sound();
                }
                return true;
            }

            if let Some(ageable) = self.as_ageable()
                && ageable.can_age_up()
            {
                item_stack.decrement_unless_creative(player.gamemode.load(), 1);
                ageable.age_up(
                    crate::entity::ageable::get_speed_up_seconds_when_feeding(-age),
                    true,
                );

                let entity = &mob_entity.living_entity.entity;
                entity.world.load().spawn_particle(
                    entity.pos.load() + Vector3::new(0.0, f64::from(entity.height()), 0.0),
                    Vector3::new(0.5, 0.5, 0.5),
                    1.0,
                    7,
                    Particle::HappyVillager,
                );
                self.play_eating_sound();
                return true;
            }
        }

        mob_entity.mob_interact(player, item_stack, self.can_be_leashed())
    }
}

#[must_use]
pub fn get_dye_color_from_item(item: &pumpkin_data::item::Item) -> Option<u8> {
    use pumpkin_data::data_component::DataComponent;
    use pumpkin_data::data_component_impl::{DyeImpl, get};

    for (id, component) in item.components {
        if *id == DataComponent::Dye {
            return Some(get::<DyeImpl>(*component).color.id());
        }
    }
    None
}

/// Base experience of animals and water creatures, rolled on every death.
pub fn experience_reward<M: Mob + ?Sized>(mob: &M) -> u32 {
    1 + mob.get_random().random_range(0..3)
}
