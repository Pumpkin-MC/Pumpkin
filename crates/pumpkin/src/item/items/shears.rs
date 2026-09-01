use std::any::Any;
use std::sync::Arc;

use crate::entity::Entity;
use crate::entity::EntityBase;
use crate::entity::item::ItemEntity;
use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use pumpkin_data::BlockId;
use pumpkin_data::block_properties::BeeNestLikeProperties;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::block_properties::CaveVinesLikeProperties;
use pumpkin_data::block_properties::KelpLikeProperties;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::{Block, BlockDirection, BlockStateId};
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
    let new_state_id = if KelpLikeProperties::handles_block_id(block.id) {
        let mut props = KelpLikeProperties::from_state_id(state_id, block);
        if props.age >= 25 {
            return false;
        }
        props.age = 25;
        props.to_state_id(block)
    } else if CaveVinesLikeProperties::handles_block_id(block.id) {
        let mut props = CaveVinesLikeProperties::from_state_id(state_id, block);
        if props.age >= 25 {
            return false;
        }
        props.age = 25;
        props.to_state_id(block)
    } else {
        return false;
    };

    let world = player.world();
    world.set_block_state(location, new_state_id, BlockFlags::NOTIFY_ALL);
    world.play_sound(
        Sound::BlockGrowingPlantCrop,
        SoundCategory::Blocks,
        &location.to_f64(),
    );
    player.damage_held_item(1);
    true
}

fn handle_beehive(
    player: &Player,
    location: &BlockPos,
    block: &Block,
    state_id: BlockStateId,
) -> bool {
    if !BeeNestLikeProperties::handles_block_id(block.id) {
        return false;
    }

    let mut props = BeeNestLikeProperties::from_state_id(state_id, block);

    if props.honey_level != 5 {
        return false;
    }

    props.honey_level = 0;
    let new_state_id = props.to_state_id(block);

    let world = player.world();
    world.set_block_state(location, new_state_id, BlockFlags::NOTIFY_ALL);
    world.play_sound(
        Sound::BlockBeehiveShear,
        SoundCategory::Blocks,
        &location.to_f64(),
    );

    let drop_pos = location.to_centered_f64();
    let item_entity = Arc::new(ItemEntity::new(
        Entity::new(world.clone(), drop_pos, &EntityType::ITEM),
        ItemStack::new(3, &Item::HONEYCOMB),
    ));
    world.spawn_entity(item_entity);
    player.damage_held_item(1);
    true
}

fn handle_pumpkin(player: &Player, location: &BlockPos, block: &Block) {
    if block.id == BlockId::PUMPKIN {
        let world = player.world();
        let carved_state = Block::CARVED_PUMPKIN.default_state.id;
        world.set_block_state(location, carved_state, BlockFlags::NOTIFY_ALL);
        world.play_sound(
            Sound::BlockPumpkinCarve,
            SoundCategory::Blocks,
            &location.to_f64(),
        );

        let drop_pos = location.to_centered_f64();
        let item_entity = Arc::new(ItemEntity::new(
            Entity::new(world.clone(), drop_pos, &EntityType::ITEM),
            ItemStack::new(4, &Item::PUMPKIN_SEEDS),
        ));
        world.spawn_entity(item_entity);
        player.damage_held_item(1);
    }
}
