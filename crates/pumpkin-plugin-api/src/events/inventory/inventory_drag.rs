use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{Event, EventType, InventoryDragEventData};

/// Event triggered when items are dragged in an inventory.
pub struct InventoryDragEvent;
impl FromIntoEvent for InventoryDragEvent {
    const EVENT_TYPE: EventType = EventType::InventoryDragEvent;
    type Data = InventoryDragEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::InventoryDragEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::InventoryDragEvent(data)
    }
}
