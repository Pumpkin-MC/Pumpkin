use crate::wit::pumpkin::plugin::event::{CreeperPowerEventData, Event, EventType};

use super::super::FromIntoEvent;

/// An event that occurs when a creeper is powered.
pub struct CreeperPowerEvent;
impl FromIntoEvent for CreeperPowerEvent {
    const EVENT_TYPE: EventType = EventType::CreeperPowerEvent;
    type Data = CreeperPowerEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::CreeperPowerEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::CreeperPowerEvent(data)
    }
}
