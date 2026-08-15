use crate::wit::pumpkin::plugin::event::{Event, EventType, MapInitializeEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a map is initialized.
pub struct MapInitializeEvent;
impl FromIntoEvent for MapInitializeEvent {
    const EVENT_TYPE: EventType = EventType::MapInitializeEvent;
    type Data = MapInitializeEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::MapInitializeEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::MapInitializeEvent(data)
    }
}
