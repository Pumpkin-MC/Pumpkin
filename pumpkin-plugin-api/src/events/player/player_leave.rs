use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerLeaveEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a player leaves the server.
///
/// The associated [`PlayerLeaveEventData`] contains the player and a leave message
/// that can be modified or suppressed. This event is cancellable.
pub struct PlayerLeaveEvent;
impl FromIntoEvent for PlayerLeaveEvent {
    const EVENT_TYPE: EventType = EventType::PlayerLeaveEvent;
    type Data = PlayerLeaveEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerLeaveEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerLeaveEvent(data)
    }
}
