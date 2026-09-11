use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{Event, EventType, InventoryPickupItemEventData};

/// Event triggered when a container inventory picks up an item entity.
pub struct InventoryPickupItemEvent;
impl FromIntoEvent for InventoryPickupItemEvent {
    const EVENT_TYPE: EventType = EventType::InventoryPickupItemEvent;
    type Data = InventoryPickupItemEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::InventoryPickupItemEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::InventoryPickupItemEvent(data)
    }
}
