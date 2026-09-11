use crate::wit::pumpkin::plugin::event::{BlockReceiveGameEventData, Event, EventType};

use super::super::FromIntoEvent;

/// An event that occurs when a block receives a game event.
pub struct BlockReceiveGameEvent;
impl FromIntoEvent for BlockReceiveGameEvent {
    const EVENT_TYPE: EventType = EventType::BlockReceiveGameEvent;
    type Data = BlockReceiveGameEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::BlockReceiveGameEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::BlockReceiveGameEvent(data)
    }
}
