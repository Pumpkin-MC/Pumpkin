use crate::wit::pumpkin::plugin::event::{EntityDamageByBlockEventData, Event, EventType};

use super::super::FromIntoEvent;

/// Event triggered when an entity takes damage from a block.
pub struct EntityDamageByBlockEvent;
impl FromIntoEvent for EntityDamageByBlockEvent {
    const EVENT_TYPE: EventType = EventType::EntityDamageByBlockEvent;
    type Data = EntityDamageByBlockEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::EntityDamageByBlockEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::EntityDamageByBlockEvent(data)
    }
}
