use crate::wit::pumpkin::plugin::event::{EntityMountEventData, Event, EventType};

use super::super::FromIntoEvent;

/// Event triggered when an entity mounts another entity.
pub struct EntityMountEvent;
impl FromIntoEvent for EntityMountEvent {
    const EVENT_TYPE: EventType = EventType::EntityMountEvent;
    type Data = EntityMountEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::EntityMountEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::EntityMountEvent(data)
    }
}
