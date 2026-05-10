use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerToggleFlightEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a player toggles flight.
pub struct PlayerToggleFlightEvent;
impl FromIntoEvent for PlayerToggleFlightEvent {
    const EVENT_TYPE: EventType = EventType::PlayerToggleFlightEvent;
    type Data = PlayerToggleFlightEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerToggleFlightEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerToggleFlightEvent(data)
    }
}
