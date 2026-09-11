use crate::wit::pumpkin::plugin::event::{EntityBreakDoorEventData, Event, EventType};

use super::super::FromIntoEvent;

/// Event triggered when an entity breaks a door.
pub struct EntityBreakDoorEvent;
impl FromIntoEvent for EntityBreakDoorEvent {
    const EVENT_TYPE: EventType = EventType::EntityBreakDoorEvent;
    type Data = EntityBreakDoorEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::EntityBreakDoorEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::EntityBreakDoorEvent(data)
    }
}
