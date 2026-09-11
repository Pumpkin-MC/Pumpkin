use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerSwapHandsEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a player swaps items between hands.
pub struct PlayerSwapHandsEvent;
impl FromIntoEvent for PlayerSwapHandsEvent {
    const EVENT_TYPE: EventType = EventType::PlayerSwapHandsEvent;
    type Data = PlayerSwapHandsEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerSwapHandsEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerSwapHandsEvent(data)
    }
}
