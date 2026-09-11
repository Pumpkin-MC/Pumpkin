use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{Event, EventType, SignChangeEventData};

/// An event that occurs when a sign's text is changed.
pub struct SignChangeEvent;
impl FromIntoEvent for SignChangeEvent {
    const EVENT_TYPE: EventType = EventType::SignChangeEvent;
    type Data = SignChangeEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::SignChangeEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::SignChangeEvent(data)
    }
}
