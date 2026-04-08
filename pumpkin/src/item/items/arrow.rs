use std::any::Any;

use crate::item::{ItemBehaviour, ItemMetadata};
use pumpkin_data::item::Item;

pub struct ArrowItem;

impl ItemMetadata for ArrowItem {
    fn ids() -> Box<[u16]> {
        Box::new([Item::ARROW.id])
    }
}

impl ItemBehaviour for ArrowItem {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
