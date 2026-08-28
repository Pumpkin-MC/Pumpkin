use std::any::Any;
use std::sync::Arc;

use crate::entity::Entity;
use crate::entity::EntityBase;
use crate::entity::item::ItemEntity;
use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::{Block, BlockDirection, BlockStateId};
use pumpkin_util::GameMode;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::world::BlockFlags;

pub struct ShearsItem;

impl ItemMetadata for ShearsItem {
    fn ids() -> Box<[u16]> {
        Box::new([Item::SHEARS.id])
    }
}

const fn get_wool_item_for_color(color: u8) -> &'static Item {
    match color {
        0 => &Item::WHITE_WOOL,
        1 => &Item::ORANGE_WOOL,
        2 => &Item::MAGENTA_WOOL,
        3 => &Item::LIGHT_BLUE_WOOL,
        4 => &Item::YELLOW_WOOL,
        5 => &Item::LIME_WOOL,
        6 => &Item::PINK_WOOL,
        7 => &Item::GRAY_WOOL,
        8 => &Item::LIGHT_GRAY_WOOL,
        9 => &Item::CYAN_WOOL,
        10 => &Item::PURPLE_WOOL,
        11 => &Item::BLUE_WOOL,
        12 => &Item::BROWN_WOOL,
        13 => &Item::GREEN_WOOL,
        14 => &Item::RED_WOOL,
        _ => &Item::BLACK_WOOL,
    }
}

impl ItemBehaviour for ShearsItem {
    fn can_mine(&self, player: &Player) -> bool {
        player.gamemode.load() != GameMode::Creative
    }

    fn use_on_block(
        &self,
        _item: &mut ItemStack,
        player: &Player,
        location: BlockPos,
        _face: BlockDirection,
        _cursor_pos: Vector3<f32>,
        block: &Block,
        _server: &Server,
    ) {
        let world = player.world();
        let state_id = world.get_block_state_id(&location);

        if handle_growing_plant(player, &location, block, state_id) {
            return;
        }

        if handle_beehive(player, &location, block, state_id) {
            return;
        }

        handle_pumpkin(player, &location, block);
    }

    fn use_on_entity(&self, _item: &mut ItemStack, player: &Player, entity: Arc<dyn EntityBase>) {
        if let Some(sheep) = entity
            .cast_any()
            .downcast_ref::<crate::entity::passive::sheep::SheepEntity>()
            && !sheep.is_sheared()
        {
            sheep.set_sheared(true);
            let world = player.world();
            let pos = sheep.mob_entity.living_entity.entity.pos.load();
            world.play_sound(Sound::EntitySheepShear, SoundCategory::Players, &pos);

            let wool_count = (rand::random::<u8>() % 3 + 1) as u8;
            let wool_item = get_wool_item_for_color(sheep.get_color());
            let item_entity = Arc::new(ItemEntity::new(
                Entity::new(world.clone(), pos, &EntityType::ITEM),
                ItemStack::new(wool_count, wool_item),
            ));
            world.spawn_entity(item_entity);
            player.damage_held_item(1);
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

fn handle_growing_plant(
    player: &Player,
    location: &BlockPos,
    block: &Block,
    state_id: BlockStateId,
) -> bool {
    let is_growing_plant = block.id == Block::KELP.id
        || block.id == Block::CAVE_VINES.id
        || block.id == Block::CAVE_VINES_PLANT.id
        || block.id == Block::TWISTING_VINES.id
        || block.id == Block::WEEPING_VINES.id;

    if !is_growing_plant {
        return false;
    }

    let world = player.world();
    let action = block.properties(state_id).and_then(|props| {
        let prop_map = props.to_props();
        prop_map
            .iter()
            .find(|(k, _)| *k == "age")
            .and_then(|(_, age_str)| age_str.parse::<u8>().ok())
            .filter(|&age| age < 25)
            .map(|_| {
                let new_props: Vec<(&str, &str)> = prop_map
                    .iter()
                    .map(|(k, v)| if *k == "age" { (*k, "25") } else { (*k, *v) })
                    .collect();
                block.from_properties(&new_props).to_state_id(block)
            })
    });

    if let Some(new_state_id) = action {
        world.set_block_state(location, new_state_id, BlockFlags::NOTIFY_ALL);
        world.play_sound(
            Sound::BlockGrowingPlantCrop,
            SoundCategory::Blocks,
            &location.to_f64(),
        );
        player.damage_held_item(1);
        return true;
    }

    false
}

fn handle_beehive(
    player: &Player,
    location: &BlockPos,
    block: &Block,
    state_id: BlockStateId,
) -> bool {
    if block.id != Block::BEEHIVE.id && block.id != Block::BEE_NEST.id {
        return false;
    }

    let world = player.world();
    let action = block.properties(state_id).and_then(|props| {
        let prop_map = props.to_props();
        prop_map
            .iter()
            .find(|(k, v)| *k == "honey_level" && *v == "5")
            .map(|_| {
                let new_props: Vec<(&str, &str)> = prop_map
                    .iter()
                    .map(|(k, v)| {
                        if *k == "honey_level" {
                            (*k, "0")
                        } else {
                            (*k, *v)
                        }
                    })
                    .collect();
                block.from_properties(&new_props).to_state_id(block)
            })
    });

    if let Some(new_state_id) = action {
        world.set_block_state(location, new_state_id, BlockFlags::NOTIFY_ALL);
        world.play_sound(
            Sound::BlockBeehiveShear,
            SoundCategory::Blocks,
            &location.to_f64(),
        );

        let drop_pos = Vector3::new(
            f64::from(location.0.x) + 0.5,
            f64::from(location.0.y) + 0.5,
            f64::from(location.0.z) + 0.5,
        );
        let item_entity = Arc::new(ItemEntity::new(
            Entity::new(world.clone(), drop_pos, &EntityType::ITEM),
            ItemStack::new(3, &Item::HONEYCOMB),
        ));
        world.spawn_entity(item_entity);
        player.damage_held_item(1);
        return true;
    }

    false
}

fn handle_pumpkin(player: &Player, location: &BlockPos, block: &Block) {
    if block.id == Block::PUMPKIN.id {
        let world = player.world();
        let carved_state = Block::CARVED_PUMPKIN.default_state.id;
        world.set_block_state(location, carved_state, BlockFlags::NOTIFY_ALL);
        world.play_sound(
            Sound::BlockPumpkinCarve,
            SoundCategory::Blocks,
            &location.to_f64(),
        );

        let drop_pos = Vector3::new(
            f64::from(location.0.x) + 0.5,
            f64::from(location.0.y) + 0.5,
            f64::from(location.0.z) + 0.5,
        );
        let item_entity = Arc::new(ItemEntity::new(
            Entity::new(world.clone(), drop_pos, &EntityType::ITEM),
            ItemStack::new(4, &Item::PUMPKIN_SEEDS),
        ));
        world.spawn_entity(item_entity);
        player.damage_held_item(1);
    }
}
