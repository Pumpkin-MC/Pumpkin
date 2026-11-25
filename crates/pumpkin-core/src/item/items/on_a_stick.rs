use std::any::Any;

use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::{DamageResult, ItemStack};
use pumpkin_data::statistic::StatisticCategory;

pub struct CarrotOnAStickItem;
pub struct WarpedFungusOnAStickItem;

impl ItemMetadata for CarrotOnAStickItem {
    fn ids() -> Box<[u16]> {
        Box::new([Item::CARROT_ON_A_STICK.id])
    }
}

fn boost_vehicle(
    player: &Player,
    item: &Item,
    hand: pumpkin_util::Hand,
    entity_type_id: u16,
    damage: i32,
) {
    if let Some(vehicle) = player.get_entity().get_vehicle()
        && vehicle
            .get_controlling_passenger()
            .is_some_and(|controller| controller.get_entity().entity_id == player.entity_id())
        && vehicle.get_entity().entity_type.id == entity_type_id
        && let Some(steerable) = vehicle.get_item_steerable()
        && steerable.boost()
    {
        let mut stack = player.inventory.get_stack_in_hand(hand);
        if !stack.is_empty() {
            if player.damage_held_stack(&mut stack, damage, hand) == DamageResult::Broken {
                stack = ItemStack::new(1, &Item::FISHING_ROD);
            }
            player.inventory.set_stack_in_hand(hand, stack);
        }
        return;
    }
    player.increment_stat(StatisticCategory::Used, i32::from(item.id), 1);
}

impl ItemBehaviour for CarrotOnAStickItem {
    fn normal_use(&self, item: &Item, player: &Player) {
        let (yaw, pitch) = player.rotation();
        self.normal_use_with_hand(item, player, yaw, pitch, pumpkin_util::Hand::Right);
    }

    fn normal_use_with_hand(
        &self,
        item: &Item,
        player: &Player,
        _yaw: f32,
        _pitch: f32,
        hand: pumpkin_util::Hand,
    ) {
        boost_vehicle(player, item, hand, EntityType::PIG.id, 7);
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
        item: &Item,
        player: &Player,
        _yaw: f32,
        _pitch: f32,
        hand: pumpkin_util::Hand,
    ) {
        boost_vehicle(player, item, hand, EntityType::STRIDER.id, 1);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
