use crate::wit::pumpkin::plugin::event::{EntityTargetBlockEventData, Event, EventType};

use super::super::FromIntoEvent;

/// An event that occurs when an entity targets a block.
pub struct EntityTargetBlockEvent;
impl FromIntoEvent for EntityTargetBlockEvent {
    const EVENT_TYPE: EventType = EventType::EntityTargetBlockEvent;
    type Data = EntityTargetBlockEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::EntityTargetBlockEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::EntityTargetBlockEvent(data)
    }
}
