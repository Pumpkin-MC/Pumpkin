use crate::wit::pumpkin::plugin::event::{Event, EventType, FireworkExplodeEventData};

use super::super::FromIntoEvent;

/// Event triggered when a firework rocket explodes.
pub struct FireworkExplodeEvent;
impl FromIntoEvent for FireworkExplodeEvent {
    const EVENT_TYPE: EventType = EventType::FireworkExplodeEvent;
    type Data = FireworkExplodeEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::FireworkExplodeEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::FireworkExplodeEvent(data)
    }
}
