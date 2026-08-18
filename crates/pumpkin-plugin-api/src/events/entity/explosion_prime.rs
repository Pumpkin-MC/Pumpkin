use crate::wit::pumpkin::plugin::event::{Event, EventType, ExplosionPrimeEventData};

use super::super::FromIntoEvent;

/// Event triggered when an entity is primed to explode.
pub struct ExplosionPrimeEvent;
impl FromIntoEvent for ExplosionPrimeEvent {
    const EVENT_TYPE: EventType = EventType::ExplosionPrimeEvent;
    type Data = ExplosionPrimeEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::ExplosionPrimeEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::ExplosionPrimeEvent(data)
    }
}
