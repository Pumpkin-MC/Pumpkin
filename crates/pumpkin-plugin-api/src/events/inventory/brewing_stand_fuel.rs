use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{BrewingStandFuelEventData, Event, EventType};

/// Event triggered when brewing stand fuel is consumed or refilled.
pub struct BrewingStandFuelEvent;
impl FromIntoEvent for BrewingStandFuelEvent {
    const EVENT_TYPE: EventType = EventType::BrewingStandFuelEvent;
    type Data = BrewingStandFuelEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::BrewingStandFuelEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::BrewingStandFuelEvent(data)
    }
}
