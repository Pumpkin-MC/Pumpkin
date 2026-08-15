use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{BrewEventData, Event, EventType};

/// Event triggered when potion(s) finish brewing in a brewing stand.
pub struct BrewEvent;
impl FromIntoEvent for BrewEvent {
    const EVENT_TYPE: EventType = EventType::BrewEvent;
    type Data = BrewEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::BrewEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::BrewEvent(data)
    }
}
