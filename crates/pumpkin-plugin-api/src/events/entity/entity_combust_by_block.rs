use crate::wit::pumpkin::plugin::event::{EntityCombustByBlockEventData, Event, EventType};

use super::super::FromIntoEvent;

/// An event that occurs when an entity is set on fire by a block.
pub struct EntityCombustByBlockEvent;
impl FromIntoEvent for EntityCombustByBlockEvent {
    const EVENT_TYPE: EventType = EventType::EntityCombustByBlockEvent;
    type Data = EntityCombustByBlockEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::EntityCombustByBlockEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::EntityCombustByBlockEvent(data)
    }
}
