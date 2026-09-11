use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{Event, EventType, InventoryMoveItemEventData};

/// Event triggered when an item is moved between inventories.
pub struct InventoryMoveItemEvent;
impl FromIntoEvent for InventoryMoveItemEvent {
    const EVENT_TYPE: EventType = EventType::InventoryMoveItemEvent;
    type Data = InventoryMoveItemEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::InventoryMoveItemEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::InventoryMoveItemEvent(data)
    }
}
