use crate::wit::pumpkin::plugin::event::{EntityExhaustionEventData, Event, EventType};

use super::super::FromIntoEvent;

/// Event triggered when an entity experiences hunger exhaustion.
pub struct EntityExhaustionEvent;
impl FromIntoEvent for EntityExhaustionEvent {
    const EVENT_TYPE: EventType = EventType::EntityExhaustionEvent;
    type Data = EntityExhaustionEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::EntityExhaustionEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::EntityExhaustionEvent(data)
    }
}
