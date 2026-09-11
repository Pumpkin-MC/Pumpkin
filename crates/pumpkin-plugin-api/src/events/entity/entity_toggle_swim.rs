use crate::wit::pumpkin::plugin::event::{EntityToggleSwimEventData, Event, EventType};

use super::super::FromIntoEvent;

/// Event triggered when an entity starts or stops swimming.
pub struct EntityToggleSwimEvent;
impl FromIntoEvent for EntityToggleSwimEvent {
    const EVENT_TYPE: EventType = EventType::EntityToggleSwimEvent;
    type Data = EntityToggleSwimEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::EntityToggleSwimEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::EntityToggleSwimEvent(data)
    }
}
