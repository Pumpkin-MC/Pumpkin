use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{Event, EventType, WorldInitEventData};

/// Event triggered when a world initializes.
pub struct WorldInitEvent;
impl FromIntoEvent for WorldInitEvent {
    const EVENT_TYPE: EventType = EventType::WorldInitEvent;
    type Data = WorldInitEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::WorldInitEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::WorldInitEvent(data)
    }
}
