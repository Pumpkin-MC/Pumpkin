use std::any::Any;

use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use pumpkin_data::block_properties::{BlockProperties, WaterCauldronLikeProperties};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::{Block, BlockDirection, BlockId};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::world::BlockFlags;

pub struct GlassBottleItem;

impl ItemMetadata for GlassBottleItem {
    fn ids() -> Box<[u16]> {
        Box::new([Item::GLASS_BOTTLE.id])
    }
}

impl ItemBehaviour for GlassBottleItem {
    fn use_on_block(
        &self,
        item: &mut ItemStack,
        player: &Player,
        location: BlockPos,
        face: BlockDirection,
        _cursor_pos: Vector3<f32>,
        block: &Block,
        _server: &Server,
    ) {
        let world = player.world();

        let is_water_target = block.id == Block::WATER.id || block.id == Block::WATER_CAULDRON.id;

        let check_pos = if is_water_target {
            location
        } else {
            location.offset(face.to_offset())
        };

        let (check_block, check_state_id) = world.get_block_and_state_id(&check_pos);

        if !matches!(check_block.id, BlockId::WATER | BlockId::WATER_CAULDRON) {
            return;
        }

        if check_block.id == BlockId::WATER_CAULDRON {
            let mut props = WaterCauldronLikeProperties::from_state_id(check_state_id, check_block);
            let new_cauldron = if props.level > 1 {
                props.level -= 1;
                props.to_state_id(check_block)
            } else {
                Block::CAULDRON.default_state.id
            };
            world.set_block_state(&check_pos, new_cauldron, BlockFlags::NOTIFY_ALL);
        }

        world.play_sound(
            Sound::ItemBottleFill,
            SoundCategory::Players,
            &check_pos.to_f64(),
        );

        let mut water_bottle = ItemStack::new(1, &Item::POTION);
        item.decrement_unless_creative(player.gamemode.load(), 1);
        let was_added = player.inventory.insert_stack_anywhere(&mut water_bottle);
        if !was_added && !water_bottle.is_empty() {
            world.drop_stack(&player.position().to_block_pos(), water_bottle);
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
