use crate::wit::pumpkin::plugin::event::{EntityToggleGlideEventData, Event, EventType};

use super::super::FromIntoEvent;

/// Event triggered when an entity toggles gliding.
pub struct EntityToggleGlideEvent;
impl FromIntoEvent for EntityToggleGlideEvent {
    const EVENT_TYPE: EventType = EventType::EntityToggleGlideEvent;
    type Data = EntityToggleGlideEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::EntityToggleGlideEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::EntityToggleGlideEvent(data)
    }
}
