use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerJoinEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a player joins the server.
///
/// The associated [`PlayerJoinEventData`] contains the player and a join message
/// that can be modified or suppressed. This event is cancellable.
pub struct PlayerJoinEvent;
impl FromIntoEvent for PlayerJoinEvent {
    const EVENT_TYPE: EventType = EventType::PlayerJoinEvent;
    type Data = PlayerJoinEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerJoinEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerJoinEvent(data)
    }
}
