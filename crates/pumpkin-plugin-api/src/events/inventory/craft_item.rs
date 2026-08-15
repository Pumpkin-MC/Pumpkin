use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{CraftItemEventData, Event, EventType};

/// Event triggered when an item is crafted.
pub struct CraftItemEvent;
impl FromIntoEvent for CraftItemEvent {
    const EVENT_TYPE: EventType = EventType::CraftItemEvent;
    type Data = CraftItemEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::CraftItemEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::CraftItemEvent(data)
    }
}
