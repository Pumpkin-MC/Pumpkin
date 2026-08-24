use crate::wit::pumpkin::plugin::event::{EntityShootBowEventData, Event, EventType};

use super::super::FromIntoEvent;

/// Event triggered when an entity shoots a bow.
pub struct EntityShootBowEvent;
impl FromIntoEvent for EntityShootBowEvent {
    const EVENT_TYPE: EventType = EventType::EntityShootBowEvent;
    type Data = EntityShootBowEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::EntityShootBowEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::EntityShootBowEvent(data)
    }
}
