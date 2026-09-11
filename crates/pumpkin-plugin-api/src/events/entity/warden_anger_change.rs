use crate::wit::pumpkin::plugin::event::{Event, EventType, WardenAngerChangeEventData};

use super::super::FromIntoEvent;

/// Event triggered when a warden's anger towards an entity changes.
pub struct WardenAngerChangeEvent;
impl FromIntoEvent for WardenAngerChangeEvent {
    const EVENT_TYPE: EventType = EventType::WardenAngerChangeEvent;
    type Data = WardenAngerChangeEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::WardenAngerChangeEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::WardenAngerChangeEvent(data)
    }
}
