use std::sync::Arc;

use pumpkin_data::dye_color::DyeColor;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::particle::Particle;
use pumpkin_data::tag;
use pumpkin_inventory::Inventory;
use pumpkin_inventory::crafting::crafting_inventory::CraftingInventory;
use pumpkin_inventory::crafting::crafting_screen_handler::match_crafting_recipe;
use pumpkin_inventory::crafting::recipe_provider::RecipeProvider;

use crate::entity::{EntityBase, mob::Mob, player::Player, r#type::from_type};
use crate::world::World;
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

    /// `false` for animals whose breeding spawns no baby right away, like frogs laying spawn.
    fn spawns_offspring(&self) -> bool {
        true
    }

    /// Runs instead of spawning a baby when [`Self::spawns_offspring`] is `false`.
    /// `brain` is set when the animal's own brain behavior bred it.
    fn on_bred_without_offspring(&self, _brain: Option<&mut crate::entity::ai::brain::BrainState>) {
    }

    fn can_fall_in_love(&self) -> bool {
        !self.get_mob_entity().is_in_love()
    }

    /// Whether breeding broadcasts the love hearts entity event.
    fn shows_breeding_hearts(&self) -> bool {
        true
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

            if age >= 0 && mob_entity.is_breeding_ready() && self.can_fall_in_love() {
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

fn find_color_mix_in_recipes(
    world: &World,
    color1: DyeColor,
    color2: DyeColor,
) -> Option<DyeColor> {
    let dye_items = |color: DyeColor| {
        tag::Item::C_DYES
            .1
            .iter()
            .filter_map(|id| Item::from_id(*id))
            .filter(move |item| get_dye_color_from_item(item) == Some(color.id()))
    };
    let server = world.server.upgrade();
    let provider = server
        .as_ref()
        .map(|server| server.recipe_manager.as_ref() as &dyn RecipeProvider);
    for dye1 in dye_items(color1) {
        for dye2 in dye_items(color2) {
            let input = CraftingInventory::new(2, 1);
            input.set_stack(0, ItemStack::new(1, dye1));
            input.set_stack(1, ItemStack::new(1, dye2));
            if let Some(result) = match_crafting_recipe(&input, provider)
                && let Some(item) = Item::from_registry_key(&result.item_id)
                && let Some(color) = get_dye_color_from_item(item)
            {
                return Some(DyeColor::from(color));
            }
        }
    }
    None
}

#[must_use]
pub fn get_mixed_color(world: &World, color1: DyeColor, color2: DyeColor) -> DyeColor {
    find_color_mix_in_recipes(world, color1, color2).unwrap_or_else(|| {
        if rand::rng().random::<bool>() {
            color1
        } else {
            color2
        }
    })
}

/// Base experience of animals and water creatures, rolled on every death.
pub fn experience_reward<M: Mob + ?Sized>(mob: &M) -> u32 {
    1 + mob.get_random().random_range(0..3)
}
