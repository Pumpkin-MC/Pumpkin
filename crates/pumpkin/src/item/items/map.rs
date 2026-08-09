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
            let Some(server) = player.world().server.upgrade() else {
                return;
            };

            let inventory = player.inventory();
            let held_stack = inventory.held_item().await;
            let (found, mut hand_stack, hand) =
                if !held_stack.is_empty() && held_stack.item.id == Item::MAP.id {
                    (true, held_stack, pumpkin_util::Hand::Right)
                } else {
                    let off_hand = inventory.off_hand_item().await;
                    if !off_hand.is_empty() && off_hand.item.id == Item::MAP.id {
                        (true, off_hand, pumpkin_util::Hand::Left)
                    } else {
                        (false, held_stack, pumpkin_util::Hand::Right)
                    }
                };

            if found {
                let map_id = server.next_map_id();
                let _ = server.map_manager.create_map(
                    map_id,
                    player.world().dimension.clone(),
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
                    inventory.set_stack_in_hand(hand, filled_map).await;
                } else {
                    hand_stack.decrement_unless_creative(gamemode, 1);
                    inventory.set_stack_in_hand(hand, hand_stack).await;
                    inventory.offer_or_drop_stack(filled_map, player).await;
                }
            }
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
