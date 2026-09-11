use crate::wit::pumpkin::plugin::event::{ArrowBodyCountChangeEventData, Event, EventType};

use super::super::FromIntoEvent;

/// An event that occurs when the number of arrows stuck in an entity changes.
pub struct ArrowBodyCountChangeEvent;
impl FromIntoEvent for ArrowBodyCountChangeEvent {
    const EVENT_TYPE: EventType = EventType::ArrowBodyCountChangeEvent;
    type Data = ArrowBodyCountChangeEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::ArrowBodyCountChangeEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::ArrowBodyCountChangeEvent(data)
    }
}
