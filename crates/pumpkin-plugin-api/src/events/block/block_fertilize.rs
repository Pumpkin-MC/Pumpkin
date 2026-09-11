use crate::wit::pumpkin::plugin::event::{BlockFertilizeEventData, Event, EventType};

use super::super::FromIntoEvent;

/// An event that occurs when a block is fertilized.
pub struct BlockFertilizeEvent;
impl FromIntoEvent for BlockFertilizeEvent {
    const EVENT_TYPE: EventType = EventType::BlockFertilizeEvent;
    type Data = BlockFertilizeEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::BlockFertilizeEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::BlockFertilizeEvent(data)
    }
}
