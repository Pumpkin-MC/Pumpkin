use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerPickupArrowEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a player picks up an arrow.
pub struct PlayerPickupArrowEvent;
impl FromIntoEvent for PlayerPickupArrowEvent {
    const EVENT_TYPE: EventType = EventType::PlayerPickupArrowEvent;
    type Data = PlayerPickupArrowEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerPickupArrowEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerPickupArrowEvent(data)
    }
}
