use std::any::Any;

use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use pumpkin_data::item::Item;

pub struct CarrotOnAStickItem;
pub struct WarpedFungusOnAStickItem;

impl ItemMetadata for CarrotOnAStickItem {
    fn ids() -> Box<[u16]> {
        Box::new([Item::CARROT_ON_A_STICK.id])
    }
}

impl ItemBehaviour for CarrotOnAStickItem {
    fn normal_use(&self, _item: &Item, player: &Player) {
        let vehicle_opt = player
            .get_entity()
            .vehicle
            .try_lock()
            .ok()
            .and_then(|guard| guard.clone());
        if let Some(vehicle) = vehicle_opt
            && let Some(steerable) = vehicle.get_item_steerable()
            && steerable.boost()
        {
            player.damage_held_item(7);
        }
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
    fn normal_use(&self, _item: &Item, player: &Player) {
        let vehicle_opt = player
            .get_entity()
            .vehicle
            .try_lock()
            .ok()
            .and_then(|guard| guard.clone());
        if let Some(vehicle) = vehicle_opt
            && let Some(steerable) = vehicle.get_item_steerable()
            && steerable.boost()
        {
            player.damage_held_item(7);
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
