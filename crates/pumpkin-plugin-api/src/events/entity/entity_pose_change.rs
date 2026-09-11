use crate::wit::pumpkin::plugin::event::{EntityPoseChangeEventData, Event, EventType};

use super::super::FromIntoEvent;

/// Event triggered when an entity's pose changes.
pub struct EntityPoseChangeEvent;
impl FromIntoEvent for EntityPoseChangeEvent {
    const EVENT_TYPE: EventType = EventType::EntityPoseChangeEvent;
    type Data = EntityPoseChangeEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::EntityPoseChangeEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::EntityPoseChangeEvent(data)
    }
}
