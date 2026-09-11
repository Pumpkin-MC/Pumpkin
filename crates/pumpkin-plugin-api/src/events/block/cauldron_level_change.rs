use crate::wit::pumpkin::plugin::event::{CauldronLevelChangeEventData, Event, EventType};

use super::super::FromIntoEvent;

/// An event that occurs when a cauldron fluid level changes.
pub struct CauldronLevelChangeEvent;
impl FromIntoEvent for CauldronLevelChangeEvent {
    const EVENT_TYPE: EventType = EventType::CauldronLevelChangeEvent;
    type Data = CauldronLevelChangeEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::CauldronLevelChangeEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::CauldronLevelChangeEvent(data)
    }
}
