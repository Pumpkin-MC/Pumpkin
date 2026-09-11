use crate::wit::pumpkin::plugin::event::{EntityChangeBlockEventData, Event, EventType};

use super::super::FromIntoEvent;

/// Event triggered when an entity changes a block in the world.
pub struct EntityChangeBlockEvent;
impl FromIntoEvent for EntityChangeBlockEvent {
    const EVENT_TYPE: EventType = EventType::EntityChangeBlockEvent;
    type Data = EntityChangeBlockEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::EntityChangeBlockEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::EntityChangeBlockEvent(data)
    }
}
