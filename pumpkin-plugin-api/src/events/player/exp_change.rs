use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerExpChangeEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a player's experience changes.
///
/// The associated [`PlayerExpChangeEventData`] contains the player and the amount of
/// experience being added (can be negative).
pub struct PlayerExpChangeEvent;
impl FromIntoEvent for PlayerExpChangeEvent {
    const EVENT_TYPE: EventType = EventType::PlayerExpChangeEvent;
    type Data = PlayerExpChangeEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerExpChangeEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerExpChangeEvent(data)
    }
}
