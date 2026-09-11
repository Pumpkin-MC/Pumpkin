use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{Event, EventType, TimeSkipEventData};

/// Event triggered when world time is skipped.
pub struct TimeSkipEvent;
impl FromIntoEvent for TimeSkipEvent {
    const EVENT_TYPE: EventType = EventType::TimeSkipEvent;
    type Data = TimeSkipEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::TimeSkipEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::TimeSkipEvent(data)
    }
}
