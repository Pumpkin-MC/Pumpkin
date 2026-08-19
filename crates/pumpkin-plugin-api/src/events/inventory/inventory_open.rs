use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{Event, EventType, InventoryOpenEventData};

/// Event triggered when an inventory container is opened.
pub struct InventoryOpenEvent;
impl FromIntoEvent for InventoryOpenEvent {
    const EVENT_TYPE: EventType = EventType::InventoryOpenEvent;
    type Data = InventoryOpenEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::InventoryOpenEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::InventoryOpenEvent(data)
    }
}
