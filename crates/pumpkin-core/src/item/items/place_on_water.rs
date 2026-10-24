use std::any::Any;

use crate::block::registry::BlockActionResult;
use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use crate::world::BlockFlags;
use pumpkin_data::fluid::Fluid;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::{Block, BlockDirection};
use pumpkin_util::Hand;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

pub struct PlaceOnWaterBlockItem;

impl ItemMetadata for PlaceOnWaterBlockItem {
    fn ids() -> Box<[u16]> {
        [Item::LILY_PAD.id, Item::FROGSPAWN.id].into()
    }
}

impl ItemBehaviour for PlaceOnWaterBlockItem {
    fn use_on_block(
        &self,
        _item: &mut ItemStack,
        _player: &Player,
        _location: BlockPos,
        _face: BlockDirection,
        _cursor_pos: Vector3<f32>,
        _block: &Block,
        _server: &Server,
    ) -> BlockActionResult {
        // Vanilla PlaceOnWaterBlockItem.useOn returns PASS without calling BlockItem.useOn,
        // and instead places via use() after raycasting Fluid.SOURCE_ONLY.
        BlockActionResult::Fail
    }

    fn normal_use(&self, item: &Item, player: &Player) {
        self.normal_use_with_hand(item, player, 0.0, 0.0, Hand::Right);
    }

    fn normal_use_with_hand(
        &self,
        item: &Item,
        player: &Player,
        _yaw: f32,
        _pitch: f32,
        hand: Hand,
    ) {
        let world = player.world();
        let (start_pos, end_pos) = self.get_start_and_end_pos(player);
        let hit = world.clip(&pumpkin_world::clip::ClipContext::new(
            start_pos,
            end_pos,
            pumpkin_world::clip::ClipBlock::Outline,
            pumpkin_world::clip::ClipFluid::SourceOnly,
        ));
        if hit.is_miss {
            return;
        }

        let hit_pos = hit.block_pos;
        let above_pos = hit_pos.up();
        let above_state = world.get_block_state(&above_pos);
        if !above_state.is_air() || *world.get_fluid(&above_pos) != Fluid::EMPTY {
            return;
        }

        let hit_block = world.get_block(&hit_pos);
        let hit_fluid = world.get_fluid(&hit_pos);
        let can_support = if item.id == Item::LILY_PAD.id {
            hit_fluid.has_tag(&tag::Fluid::MINECRAFT_SUPPORTS_LILY_PAD)
                || hit_block.has_tag(&tag::Fluid::MINECRAFT_SUPPORTS_LILY_PAD)
                || hit_block.has_tag(&tag::Block::MINECRAFT_SUPPORTS_LILY_PAD)
        } else {
            *hit_fluid == Fluid::WATER || hit_block == &Block::WATER
        };
        if !can_support {
            return;
        }

        let (placed_block, sound) = if item.id == Item::LILY_PAD.id {
            (&Block::LILY_PAD, Sound::BlockLilyPadPlace)
        } else {
            (&Block::FROGSPAWN, Sound::BlockFrogspawnPlace)
        };

        world.set_block_state(
            &above_pos,
            placed_block.default_state.id,
            BlockFlags::NOTIFY_ALL,
        );
        world.play_sound(sound, SoundCategory::Blocks, &above_pos.to_f64());

        let mut stack = player.inventory.get_stack_in_hand(hand);
        if !stack.is_empty() && stack.item.id == item.id {
            stack.decrement_unless_creative(player.gamemode.load(), 1);
            player.inventory.set_stack_in_hand(hand, stack);
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
