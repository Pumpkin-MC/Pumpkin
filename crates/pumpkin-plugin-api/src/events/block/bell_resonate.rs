use crate::wit::pumpkin::plugin::event::{BellResonateEventData, Event, EventType};

use super::super::FromIntoEvent;

/// An event that occurs when a bell resonates.
pub struct BellResonateEvent;
impl FromIntoEvent for BellResonateEvent {
    const EVENT_TYPE: EventType = EventType::BellResonateEvent;
    type Data = BellResonateEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::BellResonateEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::BellResonateEvent(data)
    }
}
