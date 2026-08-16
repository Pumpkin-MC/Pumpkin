use std::any::Any;
use std::future::Future;
use std::pin::Pin;

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
    fn normal_use<'a>(
        &'a self,
        _item: &'a Item,
        player: &'a Player,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let vehicle_opt = player.get_entity().vehicle.lock().await.clone();
            if let Some(vehicle) = vehicle_opt
                && let Some(steerable) = vehicle.get_item_steerable()
                && steerable.boost()
            {
                player.damage_held_item(7).await;
            }
        })
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
    fn normal_use<'a>(
        &'a self,
        _item: &'a Item,
        player: &'a Player,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let vehicle_opt = player.get_entity().vehicle.lock().await.clone();
            if let Some(vehicle) = vehicle_opt
                && let Some(steerable) = vehicle.get_item_steerable()
                && steerable.boost()
            {
                player.damage_held_item(7).await;
            }
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
