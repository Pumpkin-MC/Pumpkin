use std::any::Any;

use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;

pub struct CarrotOnAStickItem;
pub struct WarpedFungusOnAStickItem;

impl ItemMetadata for CarrotOnAStickItem {
    fn ids() -> Box<[u16]> {
        Box::new([Item::CARROT_ON_A_STICK.id])
    }
}

fn boost_vehicle(player: &Player, hand: pumpkin_util::Hand, entity_type_id: u16, damage: i32) {
    let vehicle_opt = player
        .get_entity()
        .vehicle
        .try_lock()
        .ok()
        .and_then(|guard| guard.clone());
    if let Some(vehicle) = vehicle_opt
        && vehicle.get_entity().entity_type.id == entity_type_id
        && let Some(steerable) = vehicle.get_item_steerable()
        && steerable.boost()
    {
        if player.gamemode.load() == pumpkin_util::GameMode::Creative {
            return;
        }
        let mut stack = match hand {
            pumpkin_util::Hand::Right => player.inventory.held_item(),
            pumpkin_util::Hand::Left => player.inventory.off_hand_item(),
        };
        if !stack.is_empty() {
            let _ = stack.damage_item(damage);
            if stack.is_empty() {
                stack = ItemStack::new(1, &Item::FISHING_ROD);
            }
            match hand {
                pumpkin_util::Hand::Right => player.inventory.set_held_item(stack),
                pumpkin_util::Hand::Left => player
                    .inventory
                    .set_stack_in_hand(pumpkin_util::Hand::Left, stack),
            }
        }
    }
}

impl ItemBehaviour for CarrotOnAStickItem {
    fn normal_use(&self, item: &Item, player: &Player) {
        let (yaw, pitch) = player.rotation();
        self.normal_use_with_hand(item, player, yaw, pitch, pumpkin_util::Hand::Right);
    }

    fn normal_use_with_hand(
        &self,
        _item: &Item,
        player: &Player,
        _yaw: f32,
        _pitch: f32,
        hand: pumpkin_util::Hand,
    ) {
        boost_vehicle(player, hand, EntityType::PIG.id, 7);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl ItemMetadata for WarpedFungusOnAStickItem {
    fn ids() -> Box<[u16]> {
        Box::new([Item::WARPED_FUNGUS_ON_A_STICK.id])
    }
}

impl ItemBehaviour for WarpedFungusOnAStickItem {
    fn normal_use(&self, item: &Item, player: &Player) {
        let (yaw, pitch) = player.rotation();
        self.normal_use_with_hand(item, player, yaw, pitch, pumpkin_util::Hand::Right);
    }

    fn normal_use_with_hand(
        &self,
        _item: &Item,
        player: &Player,
        _yaw: f32,
        _pitch: f32,
        hand: pumpkin_util::Hand,
    ) {
        boost_vehicle(player, hand, EntityType::STRIDER.id, 1);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
