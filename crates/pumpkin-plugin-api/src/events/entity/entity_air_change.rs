use crate::wit::pumpkin::plugin::event::{EntityAirChangeEventData, Event, EventType};

use super::super::FromIntoEvent;

/// Event triggered when an entity's air level changes.
pub struct EntityAirChangeEvent;
impl FromIntoEvent for EntityAirChangeEvent {
    const EVENT_TYPE: EventType = EventType::EntityAirChangeEvent;
    type Data = EntityAirChangeEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::EntityAirChangeEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::EntityAirChangeEvent(data)
    }
}
