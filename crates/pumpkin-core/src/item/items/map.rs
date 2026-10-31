use crate::entity::player::Player;
use crate::item::ItemBehaviour;
use crate::item::ItemMetadata;
use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::DataComponentImpl;
use pumpkin_data::data_component_impl::MapIdImpl;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::statistic::StatisticCategory;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_util::{GameMode, Hand};
use std::any::Any;

pub struct MapItem;

impl ItemMetadata for MapItem {
    fn ids() -> Box<[u16]> {
        [Item::MAP.id].into()
    }
}

impl ItemBehaviour for MapItem {
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
        let Some(server) = player.world().server.upgrade() else {
            return;
        };

        let inventory = player.inventory();
        let mut hand_stack = inventory.get_stack_in_hand(hand);
        if hand_stack.is_empty() || hand_stack.item.id != item.id {
            return;
        }

        let pos = player.position();
        player.world().play_sound(
            Sound::UiCartographyTableTakeResult,
            SoundCategory::Players,
            &pos,
        );

        let map_id = server.next_map_id();
        let _ = server.map_manager.create_map(
            map_id,
            player.world().dimension.clone(),
            pos.x.floor() as i32,
            pos.z.floor() as i32,
            0, // Default scale
        );

        let mut filled_map = ItemStack::new(1, &Item::FILLED_MAP);
        filled_map.patch.push((
            DataComponent::MapId,
            Some(MapIdImpl { id: map_id }.to_dyn()),
        ));

        let slot_index = match hand {
            Hand::Right => inventory.get_selected_slot() as usize,
            Hand::Left => PlayerInventory::OFF_HAND_SLOT,
        };
        let gamemode = player.gamemode.load();
        if hand_stack.item_count == 1 && gamemode != GameMode::Creative {
            inventory.set_stack_in_hand(hand, filled_map.clone());
            player.sync_hand_slot(slot_index, filled_map);
        } else {
            hand_stack.decrement_unless_creative(gamemode, 1);
            inventory.set_stack_in_hand(hand, hand_stack.clone());
            player.sync_hand_slot(slot_index, hand_stack);
            let mut stack_to_give = filled_map;
            let was_added = inventory.insert_stack_anywhere(&mut stack_to_give);
            if !was_added && !stack_to_give.is_empty() {
                player
                    .world()
                    .drop_stack(&pos.to_block_pos(), stack_to_give);
            }
        }
        player.increment_stat(StatisticCategory::Used, i32::from(item.id), 1);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
