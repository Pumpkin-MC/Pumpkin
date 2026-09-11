use crate::wit::pumpkin::plugin::event::{BlockDispenseLootEventData, Event, EventType};

use super::super::FromIntoEvent;

/// An event that occurs when a block dispenses loot.
pub struct BlockDispenseLootEvent;
impl FromIntoEvent for BlockDispenseLootEvent {
    const EVENT_TYPE: EventType = EventType::BlockDispenseLootEvent;
    type Data = BlockDispenseLootEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::BlockDispenseLootEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::BlockDispenseLootEvent(data)
    }
}
