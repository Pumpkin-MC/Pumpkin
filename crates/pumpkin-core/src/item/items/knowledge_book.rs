use std::any::Any;

use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use pumpkin_data::data_component_impl::RecipesImpl;
use pumpkin_data::item::Item;

pub struct KnowledgeBookItem;
pub struct DiscFragmentItem;

impl ItemMetadata for KnowledgeBookItem {
    fn ids() -> Box<[u16]> {
        Box::new([Item::KNOWLEDGE_BOOK.id])
    }
}

impl ItemBehaviour for KnowledgeBookItem {
    fn normal_use(&self, _item: &Item, player: &Player) {
        let mut held = player.inventory().held_item();
        let mut matched_main = true;
        if held.is_empty() || held.item.id != Item::KNOWLEDGE_BOOK.id {
            held = player.inventory().off_hand_item();
            matched_main = false;
            if held.is_empty() || held.item.id != Item::KNOWLEDGE_BOOK.id {
                return;
            }
        }

        let _recipes = held.get_data_component::<RecipesImpl>();

        held.decrement_unless_creative(player.gamemode.load(), 1);
        if matched_main {
            player.inventory().set_held_item(held);
        } else {
            player
                .inventory()
                .set_stack_in_hand(pumpkin_util::Hand::Left, held);
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl ItemMetadata for DiscFragmentItem {
    fn ids() -> Box<[u16]> {
        Box::new([Item::DISC_FRAGMENT_5.id])
    }
}

impl ItemBehaviour for DiscFragmentItem {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
