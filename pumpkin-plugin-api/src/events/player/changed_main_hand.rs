use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerChangedMainHandEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a player changes their main hand in the settings.
///
/// The associated [`PlayerChangedMainHandEventData`] contains the player and their
/// newly selected main hand.
pub struct PlayerChangedMainHandEvent;
impl FromIntoEvent for PlayerChangedMainHandEvent {
    const EVENT_TYPE: EventType = EventType::PlayerChangedMainHandEvent;
    type Data = PlayerChangedMainHandEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerChangedMainHandEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerChangedMainHandEvent(data)
    }
}
