use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{BlockIgniteEventData, Event, EventType};

/// Event triggered when a block is ignited.
pub struct BlockIgniteEvent;
impl FromIntoEvent for BlockIgniteEvent {
    const EVENT_TYPE: EventType = EventType::BlockIgniteEvent;
    type Data = BlockIgniteEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::BlockIgniteEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::BlockIgniteEvent(data)
    }
}
