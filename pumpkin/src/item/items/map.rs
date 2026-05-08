use crate::entity::player::Player;
use crate::item::ItemBehaviour;
use crate::item::ItemMetadata;
use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::DataComponentImpl;
use pumpkin_data::data_component_impl::MapIdImpl;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_util::GameMode;
use std::any::Any;
use std::future::Future;
use std::pin::Pin;

pub struct MapItem;

impl ItemMetadata for MapItem {
    fn ids() -> Box<[u16]> {
        [Item::MAP.id].into()
    }
}

impl ItemBehaviour for MapItem {
    fn normal_use<'a>(
        &'a self,
        _item: &'a Item,
        player: &'a Player,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let server = player.world().server.upgrade().unwrap();

            let inventory = player.inventory();
            let main_hand_item = inventory.held_item();
            let off_hand_item = inventory.off_hand_item().await;
            let mut hand_stack = main_hand_item.lock().await;

            let found = if !hand_stack.is_empty() && hand_stack.item.id == Item::MAP.id {
                true
            } else {
                drop(hand_stack);
                hand_stack = off_hand_item.lock().await;
                !hand_stack.is_empty() && hand_stack.item.id == Item::MAP.id
            };

            if found {
                let map_id = server.next_map_id();
                let _ = server.map_manager.create_map(
                    map_id,
                    player.world().dimension,
                    player.position().x as i32,
                    player.position().z as i32,
                    0, // Default scale
                );

                let mut filled_map = ItemStack::new(1, &Item::FILLED_MAP);
                filled_map.patch.push((
                    DataComponent::MapId,
                    Some(MapIdImpl { id: map_id }.to_dyn()),
                ));

                let gamemode = player.gamemode.load();
                if hand_stack.item_count == 1 && gamemode != GameMode::Creative {
                    *hand_stack = filled_map;
                } else {
                    hand_stack.decrement_unless_creative(gamemode, 1);
                    drop(hand_stack);
                    inventory.offer_or_drop_stack(filled_map, player).await;
                }
            }
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
