use crate::wit::pumpkin::plugin::event::{EntityKnockbackByEntityEventData, Event, EventType};

use super::super::FromIntoEvent;

/// An event that occurs when an entity is knocked back by another entity.
pub struct EntityKnockbackByEntityEvent;
impl FromIntoEvent for EntityKnockbackByEntityEvent {
    const EVENT_TYPE: EventType = EventType::EntityKnockbackByEntityEvent;
    type Data = EntityKnockbackByEntityEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::EntityKnockbackByEntityEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::EntityKnockbackByEntityEvent(data)
    }
}
