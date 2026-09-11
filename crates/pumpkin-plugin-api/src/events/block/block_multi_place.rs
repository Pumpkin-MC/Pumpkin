use crate::wit::pumpkin::plugin::event::{BlockMultiPlaceEventData, Event, EventType};

use super::super::FromIntoEvent;

/// An event that occurs when multiple blocks are placed at once.
pub struct BlockMultiPlaceEvent;
impl FromIntoEvent for BlockMultiPlaceEvent {
    const EVENT_TYPE: EventType = EventType::BlockMultiPlaceEvent;
    type Data = BlockMultiPlaceEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::BlockMultiPlaceEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::BlockMultiPlaceEvent(data)
    }
}
