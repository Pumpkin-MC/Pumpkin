use std::any::Any;

use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use pumpkin_data::item::Item;
use pumpkin_util::Hand;

pub struct ShieldItem;

impl ItemMetadata for ShieldItem {
    fn ids() -> Box<[u16]> {
        Box::new([Item::SHIELD.id])
    }
}

impl ItemBehaviour for ShieldItem {
    fn normal_use_with_hand(
        &self,
        _item: &Item,
        player: &Player,
        _yaw: f32,
        _pitch: f32,
        hand: Hand,
    ) {
        let stack = player.inventory.get_stack_in_hand(hand);
        player
            .living_entity
            .set_active_hand(hand, stack, self.get_use_duration());
    }

    fn get_use_duration(&self) -> i32 {
        72000
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
