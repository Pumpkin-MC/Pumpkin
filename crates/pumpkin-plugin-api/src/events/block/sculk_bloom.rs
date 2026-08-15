use crate::wit::pumpkin::plugin::event::{Event, EventType, SculkBloomEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a sculk catalyst blooms.
pub struct SculkBloomEvent;
impl FromIntoEvent for SculkBloomEvent {
    const EVENT_TYPE: EventType = EventType::SculkBloomEvent;
    type Data = SculkBloomEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::SculkBloomEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::SculkBloomEvent(data)
    }
}
