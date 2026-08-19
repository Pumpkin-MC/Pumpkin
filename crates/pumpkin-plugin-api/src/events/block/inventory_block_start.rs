use crate::wit::pumpkin::plugin::event::{Event, EventType, InventoryBlockStartEventData};

use super::super::FromIntoEvent;

/// An event that occurs when an inventory block starts an operation.
pub struct InventoryBlockStartEvent;
impl FromIntoEvent for InventoryBlockStartEvent {
    const EVENT_TYPE: EventType = EventType::InventoryBlockStartEvent;
    type Data = InventoryBlockStartEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::InventoryBlockStartEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::InventoryBlockStartEvent(data)
    }
}
