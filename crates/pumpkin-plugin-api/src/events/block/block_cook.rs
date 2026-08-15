use crate::wit::pumpkin::plugin::event::{BlockCookEventData, Event, EventType};

use super::super::FromIntoEvent;

/// An event that occurs when a block cooks an item.
pub struct BlockCookEvent;
impl FromIntoEvent for BlockCookEvent {
    const EVENT_TYPE: EventType = EventType::BlockCookEvent;
    type Data = BlockCookEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::BlockCookEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::BlockCookEvent(data)
    }
}
