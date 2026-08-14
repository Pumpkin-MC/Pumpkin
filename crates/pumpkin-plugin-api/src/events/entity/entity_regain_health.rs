use crate::wit::pumpkin::plugin::event::{EntityRegainHealthEventData, Event, EventType};

use super::super::FromIntoEvent;

/// Event triggered when an entity regains health.
pub struct EntityRegainHealthEvent;
impl FromIntoEvent for EntityRegainHealthEvent {
    const EVENT_TYPE: EventType = EventType::EntityRegainHealthEvent;
    type Data = EntityRegainHealthEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::EntityRegainHealthEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::EntityRegainHealthEvent(data)
    }
}
