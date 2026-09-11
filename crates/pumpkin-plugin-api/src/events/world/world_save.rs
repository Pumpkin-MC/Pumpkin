use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{Event, EventType, WorldSaveEventData};

/// Event triggered when a world is saved.
pub struct WorldSaveEvent;
impl FromIntoEvent for WorldSaveEvent {
    const EVENT_TYPE: EventType = EventType::WorldSaveEvent;
    type Data = WorldSaveEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::WorldSaveEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::WorldSaveEvent(data)
    }
}
