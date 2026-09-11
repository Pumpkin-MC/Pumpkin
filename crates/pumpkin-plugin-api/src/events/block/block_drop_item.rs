use crate::wit::pumpkin::plugin::event::{BlockDropItemEventData, Event, EventType};

use super::super::FromIntoEvent;

/// An event that occurs when a block drops items into the world.
pub struct BlockDropItemEvent;
impl FromIntoEvent for BlockDropItemEvent {
    const EVENT_TYPE: EventType = EventType::BlockDropItemEvent;
    type Data = BlockDropItemEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::BlockDropItemEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::BlockDropItemEvent(data)
    }
}
