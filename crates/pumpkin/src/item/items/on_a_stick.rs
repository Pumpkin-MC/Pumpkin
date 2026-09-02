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

impl ItemBehaviour for CarrotOnAStickItem {
    fn normal_use(&self, _item: &Item, player: &Player) {
        let vehicle_opt = player
            .get_entity()
            .vehicle
            .try_lock()
            .ok()
            .and_then(|guard| guard.clone());
        if let Some(vehicle) = vehicle_opt
            && vehicle.get_entity().entity_type.id == EntityType::PIG.id
            && let Some(steerable) = vehicle.get_item_steerable()
            && steerable.boost()
        {
            let before = player.inventory.held_item();
            player.damage_held_item(7);
            if !before.is_empty() && player.inventory.held_item().is_empty() {
                player
                    .inventory
                    .set_held_item(ItemStack::new(1, &Item::FISHING_ROD));
            }
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
            && vehicle.get_entity().entity_type.id == EntityType::STRIDER.id
            && let Some(steerable) = vehicle.get_item_steerable()
            && steerable.boost()
        {
            let before = player.inventory.held_item();
            player.damage_held_item(1);
            if !before.is_empty() && player.inventory.held_item().is_empty() {
                player
                    .inventory
                    .set_held_item(ItemStack::new(1, &Item::FISHING_ROD));
            }
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
