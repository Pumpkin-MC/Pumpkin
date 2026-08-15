use crate::wit::pumpkin::plugin::event::{EntityEnterLoveModeEventData, Event, EventType};

use super::super::FromIntoEvent;

/// Event triggered when an entity enters love mode.
pub struct EntityEnterLoveModeEvent;
impl FromIntoEvent for EntityEnterLoveModeEvent {
    const EVENT_TYPE: EventType = EventType::EntityEnterLoveModeEvent;
    type Data = EntityEnterLoveModeEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::EntityEnterLoveModeEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::EntityEnterLoveModeEvent(data)
    }
}
