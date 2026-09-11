use crate::wit::pumpkin::plugin::event::{EntityInteractEventData, Event, EventType};

use super::super::FromIntoEvent;

/// Event triggered when an entity interacts with a block.
pub struct EntityInteractEvent;
impl FromIntoEvent for EntityInteractEvent {
    const EVENT_TYPE: EventType = EventType::EntityInteractEvent;
    type Data = EntityInteractEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::EntityInteractEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::EntityInteractEvent(data)
    }
}
