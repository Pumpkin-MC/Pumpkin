use crate::wit::pumpkin::plugin::event::{BlockExpEventData, Event, EventType};

use super::super::FromIntoEvent;

/// An event that occurs when a block gives experience.
pub struct BlockExpEvent;
impl FromIntoEvent for BlockExpEvent {
    const EVENT_TYPE: EventType = EventType::BlockExpEvent;
    type Data = BlockExpEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::BlockExpEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::BlockExpEvent(data)
    }
}
