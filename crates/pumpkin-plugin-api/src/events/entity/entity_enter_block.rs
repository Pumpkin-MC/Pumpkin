use crate::wit::pumpkin::plugin::event::{EntityEnterBlockEventData, Event, EventType};

use super::super::FromIntoEvent;

/// Event triggered when an entity enters a block.
pub struct EntityEnterBlockEvent;
impl FromIntoEvent for EntityEnterBlockEvent {
    const EVENT_TYPE: EventType = EventType::EntityEnterBlockEvent;
    type Data = EntityEnterBlockEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::EntityEnterBlockEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::EntityEnterBlockEvent(data)
    }
}
