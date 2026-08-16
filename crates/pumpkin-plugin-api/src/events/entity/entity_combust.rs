use crate::wit::pumpkin::plugin::event::{EntityCombustEventData, Event, EventType};

use super::super::FromIntoEvent;

/// Event triggered when an entity catches fire.
pub struct EntityCombustEvent;
impl FromIntoEvent for EntityCombustEvent {
    const EVENT_TYPE: EventType = EventType::EntityCombustEvent;
    type Data = EntityCombustEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::EntityCombustEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::EntityCombustEvent(data)
    }
}
