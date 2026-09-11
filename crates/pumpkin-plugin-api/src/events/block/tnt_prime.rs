use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{Event, EventType, TntPrimeEventData};

/// An event that occurs when TNT is primed.
pub struct TNTPrimeEvent;
impl FromIntoEvent for TNTPrimeEvent {
    const EVENT_TYPE: EventType = EventType::TntPrimeEvent;
    type Data = TntPrimeEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::TntPrimeEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::TntPrimeEvent(data)
    }
}
