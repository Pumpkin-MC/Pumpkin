use crate::wit::pumpkin::plugin::event::{Event, EventType, ExpBottleEventData};

use super::super::FromIntoEvent;

/// An event that occurs when an experience bottle breaks.
pub struct ExpBottleEvent;
impl FromIntoEvent for ExpBottleEvent {
    const EVENT_TYPE: EventType = EventType::ExpBottleEvent;
    type Data = ExpBottleEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::ExpBottleEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::ExpBottleEvent(data)
    }
}
