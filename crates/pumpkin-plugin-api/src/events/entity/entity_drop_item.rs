use crate::wit::pumpkin::plugin::event::{EntityDropItemEventData, Event, EventType};

use super::super::FromIntoEvent;

/// Event triggered when an entity drops an item.
pub struct EntityDropItemEvent;
impl FromIntoEvent for EntityDropItemEvent {
    const EVENT_TYPE: EventType = EventType::EntityDropItemEvent;
    type Data = EntityDropItemEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::EntityDropItemEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::EntityDropItemEvent(data)
    }
}
