use crate::wit::pumpkin::plugin::event::{BlockShearEntityEventData, Event, EventType};

use super::super::FromIntoEvent;

/// An event that occurs when a dispenser shears an entity.
pub struct BlockShearEntityEvent;
impl FromIntoEvent for BlockShearEntityEvent {
    const EVENT_TYPE: EventType = EventType::BlockShearEntityEvent;
    type Data = BlockShearEntityEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::BlockShearEntityEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::BlockShearEntityEvent(data)
    }
}
