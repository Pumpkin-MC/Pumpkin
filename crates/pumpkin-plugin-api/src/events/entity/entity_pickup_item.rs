use crate::wit::pumpkin::plugin::event::{EntityPickupItemEventData, Event, EventType};

use super::super::FromIntoEvent;

/// Event triggered when an entity picks up an item.
pub struct EntityPickupItemEvent;
impl FromIntoEvent for EntityPickupItemEvent {
    const EVENT_TYPE: EventType = EventType::EntityPickupItemEvent;
    type Data = EntityPickupItemEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::EntityPickupItemEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::EntityPickupItemEvent(data)
    }
}
