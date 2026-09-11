use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{BlockExplodeEventData, Event, EventType};

/// An event that occurs when a block explodes.
pub struct BlockExplodeEvent;
impl FromIntoEvent for BlockExplodeEvent {
    const EVENT_TYPE: EventType = EventType::BlockExplodeEvent;
    type Data = BlockExplodeEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::BlockExplodeEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::BlockExplodeEvent(data)
    }
}
