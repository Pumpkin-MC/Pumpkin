use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{BlockFormEventData, Event, EventType};

/// Event triggered when a block is formed.
pub struct BlockFormEvent;
impl FromIntoEvent for BlockFormEvent {
    const EVENT_TYPE: EventType = EventType::BlockFormEvent;
    type Data = BlockFormEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::BlockFormEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::BlockFormEvent(data)
    }
}
