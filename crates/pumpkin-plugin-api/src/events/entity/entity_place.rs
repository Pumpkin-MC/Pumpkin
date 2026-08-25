use crate::wit::pumpkin::plugin::event::{EntityPlaceEventData, Event, EventType};

use super::super::FromIntoEvent;

/// Event triggered when an entity places a block.
pub struct EntityPlaceEvent;
impl FromIntoEvent for EntityPlaceEvent {
    const EVENT_TYPE: EventType = EventType::EntityPlaceEvent;
    type Data = EntityPlaceEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::EntityPlaceEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::EntityPlaceEvent(data)
    }
}
