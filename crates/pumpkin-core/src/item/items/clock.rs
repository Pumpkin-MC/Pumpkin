use std::any::Any;

use crate::item::{ItemBehaviour, ItemMetadata};
use pumpkin_data::item::Item;

pub struct ClockItem;

impl ItemMetadata for ClockItem {
    fn ids() -> Box<[u16]> {
        Box::new([Item::CLOCK.id])
    }
}

impl ItemBehaviour for ClockItem {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
