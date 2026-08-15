use crate::wit::pumpkin::plugin::event::{BlockSpreadEventData, Event, EventType};

use super::super::FromIntoEvent;

/// An event that occurs when a block spreads based on world conditions.
pub struct BlockSpreadEvent;
impl FromIntoEvent for BlockSpreadEvent {
    const EVENT_TYPE: EventType = EventType::BlockSpreadEvent;
    type Data = BlockSpreadEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::BlockSpreadEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::BlockSpreadEvent(data)
    }
}
