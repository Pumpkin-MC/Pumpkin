use crate::wit::pumpkin::plugin::event::{Event, EventType, PigZombieAngerEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a zombie pigman becomes angry.
pub struct PigZombieAngerEvent;
impl FromIntoEvent for PigZombieAngerEvent {
    const EVENT_TYPE: EventType = EventType::PigZombieAngerEvent;
    type Data = PigZombieAngerEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PigZombieAngerEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PigZombieAngerEvent(data)
    }
}
