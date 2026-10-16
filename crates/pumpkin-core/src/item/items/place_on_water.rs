use std::any::Any;

use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::world::BlockFlags;
use pumpkin_data::Block;
use pumpkin_data::item::Item;
use pumpkin_data::sound::Sound;

pub struct PlaceOnWaterBlockItem;

impl ItemMetadata for PlaceOnWaterBlockItem {
    fn ids() -> Box<[u16]> {
        [Item::LILY_PAD.id, Item::FROGSPAWN.id].into()
    }
}

impl ItemBehaviour for PlaceOnWaterBlockItem {
    fn normal_use(&self, item: &Item, player: &Player) {
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
        if above_state.is_air() {
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
            world.play_sound(
                sound,
                pumpkin_data::sound::SoundCategory::Blocks,
                &above_pos.to_f64(),
            );

            let mut main_hand = player.inventory.held_item();
            let consumed = if !main_hand.is_empty() && main_hand.item.id == item.id {
                main_hand.decrement_unless_creative(player.gamemode.load(), 1);
                player.inventory.set_held_item(main_hand);
                true
            } else {
                false
            };

            if !consumed {
                let mut off_hand = player.inventory.off_hand_item();
                if !off_hand.is_empty() && off_hand.item.id == item.id {
                    off_hand.decrement_unless_creative(player.gamemode.load(), 1);
                    player
                        .inventory
                        .set_stack_in_hand(pumpkin_util::Hand::Left, off_hand);
                }
            }
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
