use std::sync::{
    Arc, Weak,
    atomic::{AtomicU8, Ordering},
};

use pumpkin_data::{entity::EntityType, item::Item};
use pumpkin_nbt::compound::NbtCompound;
use rand::RngExt;

use crate::entity::{
    Entity, EntityBase,
    ageable::AgeableMob,
    ai::goal::{
        breed::BreedGoal, eat_grass::EatGrassGoal, escape_danger::EscapeDangerGoal,
        follow_parent::FollowParentGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, swim::SwimGoal, tempt::TemptGoal,
        wander_around::WanderAroundGoal,
    },
    item::ItemEntity,
    mob::{Mob, MobEntity, spawn::SpawnGroupData},
    passive::animal::{Animal, get_dye_color_from_item},
    player::Player,
};
use crate::world::World;
use crate::world::loot::{LootContextParameters, generate_loot_from_handle};

use pumpkin_data::chunk::Biome;
use pumpkin_data::dye_color::DyeColor;
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable, WorldgenBiome};
use pumpkin_inventory::Inventory;
use pumpkin_inventory::crafting::crafting_inventory::CraftingInventory;
use pumpkin_inventory::crafting::crafting_screen_handler::match_crafting_recipe;
use pumpkin_inventory::crafting::recipe_provider::RecipeProvider;

const TEMPT_ITEMS: &[&Item] = &[&Item::WHEAT];

#[derive(Clone, Copy)]
enum SheepColorProvider {
    Single(DyeColor),
    CommonColors(DyeColor),
}

type SheepColorSpawnConfiguration = [(SheepColorProvider, u32); 5];

const TEMPERATE_SPAWN_CONFIGURATION: SheepColorSpawnConfiguration = [
    (SheepColorProvider::Single(DyeColor::Black), 5),
    (SheepColorProvider::Single(DyeColor::Gray), 5),
    (SheepColorProvider::Single(DyeColor::LightGray), 5),
    (SheepColorProvider::Single(DyeColor::Brown), 3),
    (SheepColorProvider::CommonColors(DyeColor::White), 82),
];
const WARM_SPAWN_CONFIGURATION: SheepColorSpawnConfiguration = [
    (SheepColorProvider::Single(DyeColor::Gray), 5),
    (SheepColorProvider::Single(DyeColor::LightGray), 5),
    (SheepColorProvider::Single(DyeColor::White), 5),
    (SheepColorProvider::Single(DyeColor::Black), 3),
    (SheepColorProvider::CommonColors(DyeColor::Brown), 82),
];
const COLD_SPAWN_CONFIGURATION: SheepColorSpawnConfiguration = [
    (SheepColorProvider::Single(DyeColor::LightGray), 5),
    (SheepColorProvider::Single(DyeColor::Gray), 5),
    (SheepColorProvider::Single(DyeColor::White), 5),
    (SheepColorProvider::Single(DyeColor::Brown), 3),
    (SheepColorProvider::CommonColors(DyeColor::Black), 82),
];
const COMMON_COLOR_WEIGHT: u32 = 499;

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
const PINK_WEIGHT: u32 = 1;

pub struct SheepEntity {
    pub mob_entity: MobEntity,
    color_and_sheared: AtomicU8,
    pub ageable_data: crate::entity::ageable::AgeableData,
}

impl SheepEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let sheep = Self {
            mob_entity,
            color_and_sheared: AtomicU8::new(0),
            ageable_data: crate::entity::ageable::AgeableData::default(),
        };
        let mob_arc = Arc::new(sheep);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let goal_selector = &mob_arc.mob_entity.goals_selector;

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, EscapeDangerGoal::new(1.25));
            goal_selector.add_goal(2, BreedGoal::new(1.0));
            goal_selector.add_goal(3, Box::new(TemptGoal::new(1.1, TEMPT_ITEMS, false)));
            goal_selector.add_goal(4, Box::new(FollowParentGoal::new(1.1)));
            goal_selector.add_goal(5, Box::new(EatGrassGoal::default()));
            goal_selector.add_goal(6, Box::new(WanderAroundGoal::water_avoiding(1.0)));
            goal_selector.add_goal(
                7,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(8, Box::new(RandomLookAroundGoal::default()));
        };

        mob_arc
    }

    fn get_packed_byte(&self) -> u8 {
        self.color_and_sheared.load(Ordering::Relaxed)
    }

    pub fn get_color(&self) -> u8 {
        self.get_packed_byte() & 0x0F
    }

    pub fn is_sheared(&self) -> bool {
        (self.get_packed_byte() & 0x10) != 0
    }

    fn set_packed_and_sync(&self, byte: u8) {
        self.color_and_sheared.store(byte, Ordering::Relaxed);
        let entity = &self.mob_entity.living_entity.entity;
        entity.set_synced_data(pumpkin_data::tracked_data::sheep::WOOL_ID, byte as i8);
        if let Some(meta) = self.mob_metadata() {
            entity.send_bedrock_actor_data(&meta.to_bedrock_actor_data());
        }
    }

    pub fn set_color(&self, color: u8) {
        let byte = (self.get_packed_byte() & 0xF0) | (color & 0x0F);
        self.set_packed_and_sync(byte);
    }

    fn get_random_sheep_color(&self, biome: &Biome) -> DyeColor {
        let configuration =
            if biome.has_tag(&WorldgenBiome::MINECRAFT_SPAWNS_WARM_VARIANT_FARM_ANIMALS) {
                WARM_SPAWN_CONFIGURATION
            } else if biome.has_tag(&WorldgenBiome::MINECRAFT_SPAWNS_COLD_VARIANT_FARM_ANIMALS) {
                COLD_SPAWN_CONFIGURATION
            } else {
                TEMPERATE_SPAWN_CONFIGURATION
            };
        let mut rng = self.get_random();
        let mut roll =
            rng.random_range(0..configuration.iter().map(|(_, weight)| weight).sum::<u32>());
        let provider = configuration
            .iter()
            .find(|(_, weight)| {
                let hit = roll < *weight;
                roll = roll.saturating_sub(*weight);
                hit
            })
            .map_or(configuration[0].0, |(provider, _)| *provider);
        match provider {
            SheepColorProvider::Single(color) => color,
            SheepColorProvider::CommonColors(color) => {
                if rng.random_range(0..COMMON_COLOR_WEIGHT + PINK_WEIGHT) < COMMON_COLOR_WEIGHT {
                    color
                } else {
                    DyeColor::Pink
                }
            }
        }
    }

    fn get_mixed_color(&self, world: &World, color1: DyeColor, color2: DyeColor) -> DyeColor {
        find_color_mix_in_recipes(world, color1, color2).unwrap_or_else(|| {
            if self.get_random().random::<bool>() {
                color1
            } else {
                color2
            }
        })
    }

    pub fn ready_for_shearing(&self) -> bool {
        !self.is_sheared() && !self.is_baby()
    }

    pub fn shear(&self, category: SoundCategory, tool: &ItemStack) {
        let entity = self.get_entity();
        let world = entity.world.load();
        let pos = entity.pos.load();
        world.play_sound(Sound::EntitySheepShear, category, &pos);
        let color = DyeColor::from(self.get_color());
        if let Some(table) =
            world.get_loot_table(&format!("minecraft:shearing/sheep/{}", color.name()))
        {
            let params = LootContextParameters {
                world: Some(&world),
                position: Some(pos),
                this_entity: Some(self),
                tool: Some(tool.clone()),
                ..Default::default()
            };
            let mut rng = self.get_random();
            for stack in generate_loot_from_handle(&table, rng.random(), &params) {
                for _ in 0..stack.item_count {
                    let item_entity = ItemEntity::new(
                        Entity::new(world.clone(), pos.add_raw(0.0, 1.0, 0.0), &EntityType::ITEM),
                        stack.copy_with_count(1),
                    );
                    let velocity = &item_entity.get_entity().velocity;
                    velocity.store(velocity.load().add_raw(
                        f64::from((rng.random::<f32>() - rng.random::<f32>()) * 0.1),
                        f64::from(rng.random::<f32>() * 0.05),
                        f64::from((rng.random::<f32>() - rng.random::<f32>()) * 0.1),
                    ));
                    world.spawn_entity(Arc::new(item_entity));
                }
            }
        }
        self.set_sheared(true);
    }

    pub fn set_sheared(&self, sheared: bool) {
        let byte = if sheared {
            self.get_packed_byte() | 0x10
        } else {
            self.get_packed_byte() & !0x10
        };
        self.set_packed_and_sync(byte);
    }
}

impl AgeableMob for SheepEntity {
    fn get_ageable_data(&self) -> &crate::entity::ageable::AgeableData {
        &self.ageable_data
    }
}

impl Animal for SheepEntity {
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        item_stack
            .item
            .has_tag(&pumpkin_data::tag::Item::MINECRAFT_SHEEP_FOOD)
            || TEMPT_ITEMS.iter().any(|i| i.id == item_stack.item.id)
    }

    fn get_breed_offspring(&self, partner: &dyn EntityBase) -> Arc<dyn EntityBase> {
        let entity = self.get_entity();
        let world = entity.world.load_full();
        let baby = Self::new(Entity::new(
            world.clone(),
            entity.pos.load(),
            &EntityType::SHEEP,
        ));
        if let Some(partner) = partner.cast_any().downcast_ref::<Self>() {
            let color = self.get_mixed_color(
                &world,
                DyeColor::from(self.get_color()),
                DyeColor::from(partner.get_color()),
            );
            baby.set_color(color.id());
        }
        baby
    }
}

impl Mob for SheepEntity {
    fn mob_get_component(
        &self,
        component: pumpkin_data::data_component::DataComponent,
    ) -> Option<Box<dyn pumpkin_data::data_component_impl::DataComponentImpl>> {
        use pumpkin_data::{
            data_component::DataComponent, data_component_impl::SheepColorImpl, dye_color::DyeColor,
        };
        match component {
            DataComponent::SheepColor => Some(Box::new(SheepColorImpl {
                value: DyeColor::from(self.get_color()).name().into(),
            })),
            _ => None,
        }
    }

    fn has_audited_tick(&self) -> bool {
        true
    }

    fn as_ageable(&self) -> Option<&dyn AgeableMob> {
        Some(self)
    }

    fn as_animal(&self) -> Option<&dyn Animal> {
        Some(self)
    }

    fn mob_metadata(&self) -> Option<crate::entity::mob::MobMetadata> {
        let mut meta = crate::entity::mob::MobMetadata::new()
            .sheep_wool(DyeColor::from(self.get_color()), self.is_sheared());
        if self.is_baby() {
            meta = meta.baby(true);
        }
        Some(meta)
    }

    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        if self.is_baby() {
            entity.set_synced_data(pumpkin_data::tracked_data::sheep::DATA_BABY_ID, true);
        }
        entity.set_synced_data(
            pumpkin_data::tracked_data::sheep::WOOL_ID,
            self.get_packed_byte() as i8,
        );
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_bool("Sheared", self.is_sheared());
        nbt.put_byte("Color", self.get_color() as i8);
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        let sheared = nbt
            .get_bool("Sheared")
            .or_else(|| nbt.get_byte("Sheared").map(|b| b == 1))
            .unwrap_or(false);
        let color = nbt.get_byte("Color").unwrap_or(0) as u8;
        let byte = (color & 0x0F) | if sheared { 0x10 } else { 0 };
        self.color_and_sheared.store(byte, Ordering::Relaxed);
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn on_eating_grass(&self) {
        self.set_sheared(false);
        if self.can_age_up() {
            self.age_up(60, false);
        }
    }

    fn finalize_spawn(
        &self,
        world: &Arc<World>,
        group_data: Option<SpawnGroupData>,
    ) -> Option<SpawnGroupData> {
        let biome = world.get_biome(&self.get_entity().block_pos.load());
        self.set_color(self.get_random_sheep_color(biome).id());
        self.mob_entity.finalize_spawn_base();
        group_data
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        let item = item_stack.get_item();

        if item == &Item::SHEARS {
            if !self.ready_for_shearing() {
                return true;
            }
            let entity = self.get_entity();
            let world = entity.world.load();
            if let Some(server) = world.server.upgrade() {
                let mut event = crate::plugin::api::events::player::player_shear_entity::PlayerShearEntityEvent {
                    player: player.clone(),
                    entity_id: entity.entity_id,
                    hand: 0,
                    cancelled: false,
                };
                server.plugin_manager.fire_blocking(&server, &mut event);
                if event.cancelled {
                    return false;
                }
            }
            self.shear(SoundCategory::Players, item_stack);
            world.emit_game_event(GameEvent::Shear.name(), entity.pos.load());
            let _ = player.damage_item_stack(item_stack, 1);
            return true;
        }

        self.animal_interact(player, item_stack, Sound::EntitySheepAmbient)
    }
}
