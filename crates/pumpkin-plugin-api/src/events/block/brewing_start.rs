use crate::wit::pumpkin::plugin::event::{BrewingStartEventData, Event, EventType};

use super::super::FromIntoEvent;

/// An event that occurs when a brewing stand starts brewing.
pub struct BrewingStartEvent;
impl FromIntoEvent for BrewingStartEvent {
    const EVENT_TYPE: EventType = EventType::BrewingStartEvent;
    type Data = BrewingStartEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::BrewingStartEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::BrewingStartEvent(data)
    }
}
