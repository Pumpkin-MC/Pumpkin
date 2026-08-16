use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{Event, EventType, GenericGameEventData};

/// Generic game event.
pub struct GenericGameEvent;
impl FromIntoEvent for GenericGameEvent {
    const EVENT_TYPE: EventType = EventType::GenericGameEvent;
    type Data = GenericGameEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::GenericGameEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::GenericGameEvent(data)
    }
}
