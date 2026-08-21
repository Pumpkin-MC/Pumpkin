use crate::wit::pumpkin::plugin::event::{EntityTargetEventData, Event, EventType};

use super::super::FromIntoEvent;

/// Event triggered when an entity targets another entity.
pub struct EntityTargetEvent;
impl FromIntoEvent for EntityTargetEvent {
    const EVENT_TYPE: EventType = EventType::EntityTargetEvent;
    type Data = EntityTargetEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::EntityTargetEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::EntityTargetEvent(data)
    }
}
