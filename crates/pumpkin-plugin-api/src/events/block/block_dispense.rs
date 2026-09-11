use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{BlockDispenseEventData, Event, EventType};

/// An event that occurs when a block dispenses an item.
pub struct BlockDispenseEvent;
impl FromIntoEvent for BlockDispenseEvent {
    const EVENT_TYPE: EventType = EventType::BlockDispenseEvent;
    type Data = BlockDispenseEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::BlockDispenseEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::BlockDispenseEvent(data)
    }
}
