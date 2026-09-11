use crate::wit::pumpkin::plugin::event::{Event, EventType, RaidStopEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a raid stops.
pub struct RaidStopEvent;
impl FromIntoEvent for RaidStopEvent {
    const EVENT_TYPE: EventType = EventType::RaidStopEvent;
    type Data = RaidStopEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::RaidStopEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::RaidStopEvent(data)
    }
}
