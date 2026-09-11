use crate::wit::pumpkin::plugin::event::{CampfireStartEventData, Event, EventType};

use super::super::FromIntoEvent;

/// An event that occurs when a campfire starts cooking an item.
pub struct CampfireStartEvent;
impl FromIntoEvent for CampfireStartEvent {
    const EVENT_TYPE: EventType = EventType::CampfireStartEvent;
    type Data = CampfireStartEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::CampfireStartEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::CampfireStartEvent(data)
    }
}
