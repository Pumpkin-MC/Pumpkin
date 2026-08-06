use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{BlockFromToEventData, Event, EventType};

/// Event triggered when a fluid flows from one block position to another.
pub struct BlockFromToEvent;
impl FromIntoEvent for BlockFromToEvent {
    const EVENT_TYPE: EventType = EventType::BlockFromToEvent;
    type Data = BlockFromToEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::BlockFromToEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::BlockFromToEvent(data)
    }
}
