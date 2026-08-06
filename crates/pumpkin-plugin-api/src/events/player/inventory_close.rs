use crate::wit::pumpkin::plugin::event::{Event, EventType, InventoryCloseEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a player closes an inventory.
pub struct InventoryCloseEvent;

impl FromIntoEvent for InventoryCloseEvent {
    const EVENT_TYPE: EventType = EventType::InventoryCloseEvent;
    type Data = InventoryCloseEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::InventoryCloseEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::InventoryCloseEvent(data)
    }
}
