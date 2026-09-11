use crate::wit::pumpkin::plugin::event::{EntityCombustByEntityEventData, Event, EventType};

use super::super::FromIntoEvent;

/// An event that occurs when an entity is set on fire by another entity.
pub struct EntityCombustByEntityEvent;
impl FromIntoEvent for EntityCombustByEntityEvent {
    const EVENT_TYPE: EventType = EventType::EntityCombustByEntityEvent;
    type Data = EntityCombustByEntityEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::EntityCombustByEntityEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::EntityCombustByEntityEvent(data)
    }
}
