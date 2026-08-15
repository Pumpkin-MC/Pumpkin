use crate::wit::pumpkin::plugin::event::{Event, EventType, VaultDisplayItemEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a vault displays an item.
pub struct VaultDisplayItemEvent;
impl FromIntoEvent for VaultDisplayItemEvent {
    const EVENT_TYPE: EventType = EventType::VaultDisplayItemEvent;
    type Data = VaultDisplayItemEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::VaultDisplayItemEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::VaultDisplayItemEvent(data)
    }
}
