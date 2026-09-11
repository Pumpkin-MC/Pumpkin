use crate::wit::pumpkin::plugin::event::{EntityDyeEventData, Event, EventType};

use super::super::FromIntoEvent;

/// Event triggered when an entity is dyed.
pub struct EntityDyeEvent;
impl FromIntoEvent for EntityDyeEvent {
    const EVENT_TYPE: EventType = EventType::EntityDyeEvent;
    type Data = EntityDyeEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::EntityDyeEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::EntityDyeEvent(data)
    }
}
