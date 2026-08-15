use crate::wit::pumpkin::plugin::event::{BellRingEventData, Event, EventType};

use super::super::FromIntoEvent;

/// An event that occurs when a bell is rung.
pub struct BellRingEvent;
impl FromIntoEvent for BellRingEvent {
    const EVENT_TYPE: EventType = EventType::BellRingEvent;
    type Data = BellRingEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::BellRingEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::BellRingEvent(data)
    }
}
