use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{Event, EventType, InventoryInteractEventData};

/// Event triggered when a player interacts with an inventory.
pub struct InventoryInteractEvent;
impl FromIntoEvent for InventoryInteractEvent {
    const EVENT_TYPE: EventType = EventType::InventoryInteractEvent;
    type Data = InventoryInteractEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::InventoryInteractEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::InventoryInteractEvent(data)
    }
}
